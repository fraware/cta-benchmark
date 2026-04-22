//! End-to-end per-instance generation pipeline.
//!
//! Given a benchmark instance, a provider, and a prompt template, produce a
//! [`GeneratedOutputBundle`] that records the prompt hash, raw response,
//! normalized obligations, and provider metadata.

use std::path::Path;

use crate::prompts::{PromptContext, PromptKind, PromptTemplate};
use crate::providers::{Provider, ProviderRequest};
use crate::{hash_prompt, GenerateError, GeneratedOutputBundle, ProviderMetadata, Result};
use cta_core::{InstanceId, RunId, SystemId};

/// Parameters for a single-instance generation.
#[derive(Debug, Clone)]
pub struct GenerateParams {
    /// Run id.
    pub run_id: RunId,
    /// System id (e.g. `full_method_v1`).
    pub system_id: SystemId,
    /// Instance id.
    pub instance_id: InstanceId,
    /// Seed forwarded to the provider.
    pub seed: u64,
    /// Max completion tokens for the provider request.
    pub max_tokens: u32,
    /// Sampling temperature.
    pub temperature: f64,
    /// Run-local path at which the raw response will be stored (relative to
    /// the run directory).
    pub raw_output_path: String,
}

/// Build the template context for a single instance by reading the
/// instance's artifacts from disk.
///
/// The specific placeholders produced depend on [`PromptKind`]:
/// - `text_only`: `{{informal_statement}}`
/// - `code_only`: `{{reference_rs}}`
/// - `naive_concat`: both above
/// - `full_method`: `{{problem_summary}}`, `{{semantic_units}}`,
///   `{{rust_summary}}`, `{{lean_scaffold}}`
///
/// Values missing from the instance (e.g. extraction failure) default to an
/// empty string rather than failing, so renders always proceed; the caller
/// may use [`PromptTemplate::render_strict`] to fail hard.
///
/// # Errors
/// Returns an IO error if any of the required instance files cannot be read.
pub fn build_context(
    kind: PromptKind,
    instance_dir: &Path,
    informal_statement: &str,
    lean_scaffold_path: &Path,
    semantic_units_path: &Path,
) -> Result<PromptContext> {
    let mut ctx = PromptContext::new();
    let reference_rs_path = instance_dir.join("reference.rs");
    match kind {
        PromptKind::TextOnly => {
            ctx.insert("informal_statement", informal_statement);
        }
        PromptKind::CodeOnly => {
            let code = read_if_exists(&reference_rs_path)?;
            ctx.insert("reference_rs", code);
        }
        PromptKind::NaiveConcat => {
            let code = read_if_exists(&reference_rs_path)?;
            ctx.insert("informal_statement", informal_statement)
                .insert("reference_rs", code);
        }
        PromptKind::FullMethod => {
            let code = read_if_exists(&reference_rs_path)?;
            let lean_scaffold = read_if_exists(lean_scaffold_path)?;
            let semantic_units = read_if_exists(semantic_units_path)?;
            let rust_summary = extract_rust_summary_json(&code)?;
            ctx.insert("problem_summary", informal_statement)
                .insert("reference_rs", code)
                .insert("semantic_units", semantic_units)
                .insert("rust_summary", rust_summary)
                .insert("lean_scaffold", lean_scaffold);
        }
    }
    Ok(ctx)
}

fn read_if_exists(path: &Path) -> Result<String> {
    if path.is_file() {
        std::fs::read_to_string(path).map_err(GenerateError::Io)
    } else {
        Ok(String::new())
    }
}

fn extract_rust_summary_json(code: &str) -> Result<String> {
    if code.trim().is_empty() {
        return Ok(String::new());
    }
    // The entry function name is not known here; use a best-effort heuristic:
    // try each `pub fn <name>` in the file in order, returning the first
    // successful summary. Deterministic since we preserve source order.
    let mut found = None;
    for line in code.lines() {
        if let Some(rest) = line.trim().strip_prefix("pub fn ") {
            if let Some(name) = rest
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .next()
            {
                if !name.is_empty() {
                    if let Ok(summary) = cta_rust_extract::extract_from_source(code, name) {
                        found = Some(summary);
                        break;
                    }
                }
            }
        }
    }
    match found {
        Some(summary) => serde_json::to_string_pretty(&summary).map_err(GenerateError::Json),
        None => Ok(String::new()),
    }
}

/// Outcome of a single-instance generation: the schema-valid bundle plus the
/// raw provider text that was normalized into it. The raw text is preserved
/// so callers can persist it verbatim alongside the bundle without ever
/// re-invoking the provider.
#[derive(Debug, Clone)]
pub struct GenerationOutcome {
    /// Canonical, schema-valid bundle.
    pub bundle: GeneratedOutputBundle,
    /// Raw provider response, exactly as returned.
    pub raw: String,
}

/// Render a prompt, call a provider, normalize the response, and package a
/// bundle suitable for writing to disk. The returned [`GenerationOutcome`]
/// carries both the bundle and the raw provider text so the provider is
/// invoked exactly once.
///
/// # Errors
/// Propagates provider errors and IO/JSON failures from normalization.
pub fn generate(
    provider: &dyn Provider,
    template: &PromptTemplate,
    ctx: &PromptContext,
    params: &GenerateParams,
) -> Result<GenerationOutcome> {
    let prompt = template.render(ctx);
    let prompt_hash = hash_prompt(&prompt);
    let response = provider.generate(&ProviderRequest {
        prompt,
        seed: params.seed,
        max_tokens: params.max_tokens,
        temperature: params.temperature,
    })?;
    let (prompt_tokens, completion_tokens) = response.tokens.unwrap_or((0, 0));
    let provider_metadata = ProviderMetadata {
        name: provider.name().to_string(),
        model: provider.model().to_string(),
        model_version: response.model_version.clone(),
        latency_ms: Some(response.latency_ms),
        prompt_tokens: Some(prompt_tokens),
        completion_tokens: Some(completion_tokens),
    };
    let bundle = GeneratedOutputBundle {
        schema_version: "schema_v1".to_string(),
        run_id: params.run_id.clone(),
        system_id: params.system_id.clone(),
        instance_id: params.instance_id.clone(),
        provider_metadata,
        raw_output_path: params.raw_output_path.clone(),
        parse_status: response.parse_status,
        normalized_obligations: response.obligations,
        prompt_hash,
        seed: params.seed,
    };
    Ok(GenerationOutcome {
        bundle,
        raw: response.raw,
    })
}

/// Backwards-compatible wrapper returning only the bundle. Prefer [`generate`]
/// which also returns the raw response exactly once.
///
/// # Errors
/// Same as [`generate`].
pub fn generate_bundle(
    provider: &dyn Provider,
    template: &PromptTemplate,
    ctx: &PromptContext,
    params: &GenerateParams,
) -> Result<GeneratedOutputBundle> {
    generate(provider, template, ctx, params).map(|o| o.bundle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StubProvider;

    #[test]
    fn stub_bundle_roundtrips() {
        let provider = StubProvider::default();
        let template = PromptTemplate::new(
            "text_only_v1",
            PromptKind::TextOnly,
            "v1",
            "prompt: {{informal_statement}}",
        );
        let mut ctx = PromptContext::new();
        ctx.insert("informal_statement", "sort an array");
        let params = GenerateParams {
            run_id: RunId::new("run_2026_04_21_stub_local_000").unwrap(),
            system_id: SystemId::new("text_only_v1").unwrap(),
            instance_id: InstanceId::new("arrays_binary_search_001").unwrap(),
            seed: 42,
            max_tokens: 128,
            temperature: 0.0,
            raw_output_path: "raw.txt".into(),
        };
        let bundle = generate_bundle(&provider, &template, &ctx, &params).unwrap();
        assert!(bundle.parse_status.ok);
        assert!(bundle.prompt_hash.starts_with("sha256:"));
        assert_eq!(bundle.normalized_obligations.len(), 1);
    }
}
