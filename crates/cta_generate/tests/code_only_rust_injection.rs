//! Regression: `code_only_v1` / `naive_concat_v1` templates must receive verbatim
//! `reference.rs` under both `{{reference_rs}}` and `{{rust_reference}}`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use cta_core::{InstanceId, RunId, SystemId};
use cta_generate::{
    build_context, generate, GenerateParams, PromptKind, PromptTemplate, Provider, ProviderRequest,
    ProviderResponse, Result, StubProvider,
};
use serde_json::json;

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf)
        .unwrap_or(manifest)
}

fn load_instance_informal(instance_dir: &std::path::Path) -> String {
    let raw = std::fs::read_to_string(instance_dir.join("instance.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    v["informal_statement"]["text"]
        .as_str()
        .unwrap()
        .to_string()
}

/// Test provider that fails closed if the model prompt still carries template
/// placeholders or an empty Rust fence, and otherwise emits obligations that
/// cite identifiers from the injected reference (so we are not only measuring
/// `True` stubs).
#[derive(Debug)]
struct CodeInjectionAuditProvider;

impl Provider for CodeInjectionAuditProvider {
    fn name(&self) -> &str {
        "code_injection_audit"
    }

    fn model(&self) -> &str {
        "test-harness"
    }

    fn generate(&self, req: &ProviderRequest) -> Result<ProviderResponse> {
        assert!(
            !req.prompt.contains("{{rust_reference}}"),
            "prompt must not contain unresolved {{rust_reference}}; got substring around placeholder"
        );
        assert!(
            !req.prompt.contains("{{reference_rs}}"),
            "prompt must not contain unresolved {{reference_rs}}"
        );
        let code = extract_first_rust_fence(&req.prompt).expect("rust code fence");
        assert!(
            code.len() > 40,
            "expected substantive Rust body in prompt, got len {}",
            code.len()
        );
        assert!(
            !code.contains("{{"),
            "rust fence must not contain template markers: {code:?}"
        );
        let token = first_pub_fn_name(&code).expect("pub fn name in reference");
        assert!(
            token.len() > 2,
            "expected extracted Rust symbol from reference.rs"
        );
        let raw = json!({
            "obligations": [{
                "kind": "postcondition",
                "lean_statement": format!("∀ args, relates_to_{token}"),
                "nl_gloss": format!("Uses Rust symbol `{token}` from the injected reference implementation.")
            }]
        })
        .to_string();
        let (obligations, parse_status) = cta_generate::normalize_response(&raw);
        Ok(ProviderResponse {
            raw,
            obligations,
            parse_status,
            latency_ms: 0,
            tokens: None,
            model_version: "test".into(),
        })
    }
}

fn extract_first_rust_fence(prompt: &str) -> Option<String> {
    let start = prompt
        .find("```rust\n")
        .or_else(|| prompt.find("```rust\r\n"))?;
    let after = if prompt[start..].starts_with("```rust\n") {
        start + "```rust\n".len()
    } else {
        start + "```rust\r\n".len()
    };
    let end = prompt[after..].find("```").map(|i| after + i)?;
    Some(prompt[after..end].to_string())
}

fn first_pub_fn_name(rust: &str) -> Option<String> {
    for line in rust.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("pub fn ") {
            let name = rest
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .next()
                .unwrap_or("");
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

#[test]
fn code_only_v1_prompt_contains_verbatim_reference_rs() {
    let workspace = workspace_root();
    let instance_dir = workspace.join("benchmark/v0.2/instances/arrays/arrays_binary_search_002");
    let informal = load_instance_informal(&instance_dir);
    let ctx = build_context(
        PromptKind::CodeOnly,
        &instance_dir,
        &informal,
        &instance_dir.join("scaffold.lean"),
        &instance_dir.join("semantic_units.json"),
    )
    .unwrap();
    let template =
        PromptTemplate::load(workspace.join("configs/prompts/code_only_v1.json")).unwrap();
    let rendered = template.render(&ctx);
    assert!(!rendered.contains("{{rust_reference}}"));
    assert!(!rendered.contains("{{reference_rs}}"));
    assert!(rendered.contains("pub fn binary_search"));
    for needle in ["```rust", "while lo < hi", "arr[mid]"] {
        assert!(
            rendered.contains(needle),
            "prompt should include Rust from reference.rs; missing {needle:?}"
        );
    }
}

#[test]
fn naive_concat_v1_prompt_resolves_rust_reference_placeholder() {
    let workspace = workspace_root();
    let instance_dir = workspace.join("benchmark/v0.2/instances/graph/graph_dijkstra_001");
    let informal = load_instance_informal(&instance_dir);
    let ctx = build_context(
        PromptKind::NaiveConcat,
        &instance_dir,
        &informal,
        &instance_dir.join("scaffold.lean"),
        &instance_dir.join("semantic_units.json"),
    )
    .unwrap();
    let template =
        PromptTemplate::load(workspace.join("configs/prompts/naive_concat_v1.json")).unwrap();
    let rendered = template.render(&ctx);
    assert!(!rendered.contains("{{rust_reference}}"));
    assert!(rendered.contains("pub fn"));
}

#[test]
fn code_only_generation_rejects_placeholder_and_requires_code_derived_output() {
    let workspace = workspace_root();
    let instance_dir = workspace.join("benchmark/v0.2/instances/dp/dp_knapsack_01_001");
    let informal = load_instance_informal(&instance_dir);
    let ctx = build_context(
        PromptKind::CodeOnly,
        &instance_dir,
        &informal,
        &instance_dir.join("scaffold.lean"),
        &instance_dir.join("semantic_units.json"),
    )
    .unwrap();
    let template =
        PromptTemplate::load(workspace.join("configs/prompts/code_only_v1.json")).unwrap();
    let provider = CodeInjectionAuditProvider;
    let params = GenerateParams {
        run_id: RunId::new("run_2026_04_22_code_only_v1_eval_099").unwrap(),
        system_id: SystemId::new("code_only_v1").unwrap(),
        instance_id: InstanceId::new("dp_knapsack_01_001").unwrap(),
        seed: 1,
        max_tokens: 256,
        temperature: 0.0,
        raw_output_path: "generated/code_only_v1/raw/dp_knapsack_01_001.txt".into(),
    };
    let outcome = generate(&provider, &template, &ctx, &params).unwrap();
    let obl = outcome.bundle.normalized_obligations.first().unwrap();
    assert!(
        obl.lean_statement.contains("relates_to_"),
        "expected code-derived token in lean_statement, got {:?}",
        obl.lean_statement
    );
    assert!(
        obl.nl_gloss.contains("Rust symbol"),
        "expected nl_gloss to cite Rust symbol, got {:?}",
        obl.nl_gloss
    );
}

#[test]
fn stub_code_only_prompt_has_no_placeholders_after_context_build() {
    // Stub provider still emits True-only bundles; this test only guards the
    // prompt path used by real providers.
    let workspace = workspace_root();
    let instance_dir =
        workspace.join("benchmark/v0.2/instances/greedy/greedy_interval_scheduling_001");
    let informal = load_instance_informal(&instance_dir);
    let ctx = build_context(
        PromptKind::CodeOnly,
        &instance_dir,
        &informal,
        &instance_dir.join("scaffold.lean"),
        &instance_dir.join("semantic_units.json"),
    )
    .unwrap();
    let template =
        PromptTemplate::load(workspace.join("configs/prompts/code_only_v1.json")).unwrap();
    let stub = StubProvider::default();
    let params = GenerateParams {
        run_id: RunId::new("run_2026_04_22_code_only_v1_eval_098").unwrap(),
        system_id: SystemId::new("code_only_v1").unwrap(),
        instance_id: InstanceId::new("greedy_interval_scheduling_001").unwrap(),
        seed: 0,
        max_tokens: 64,
        temperature: 0.0,
        raw_output_path: "generated/code_only_v1/raw/greedy_interval_scheduling_001.txt".into(),
    };
    let outcome = generate(&stub, &template, &ctx, &params).unwrap();
    assert!(!outcome.bundle.normalized_obligations.is_empty());
}
