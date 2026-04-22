//! `cta_generate` — candidate obligation generation.
//!
//! Defines the [`Provider`] trait that all generation systems implement, a
//! deterministic offline provider ([`StubProvider`]) for tests and CI, and
//! wire-level providers for OpenAI and Anthropic ([`OpenAiProvider`],
//! [`AnthropicProvider`]). Downstream crates consume the normalized
//! [`GeneratedOutputBundle`].

#![deny(missing_docs)]

use cta_core::{InstanceId, RunId, SystemId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub mod normalize;
pub mod pipeline;
pub mod prompts;
pub mod providers;

pub use normalize::normalize_response;
pub use pipeline::{build_context, generate, generate_bundle, GenerateParams, GenerationOutcome};
pub use prompts::{PromptContext, PromptKind, PromptTemplate};
pub use providers::{
    build_from_config, AnthropicProvider, OpenAiProvider, Provider, ProviderConfig,
    ProviderRequest, ProviderResponse, StubProvider,
};

/// Errors produced during generation.
#[derive(Debug, Error)]
pub enum GenerateError {
    /// Network or provider error.
    #[error("provider error: {0}")]
    Provider(String),
    /// Output parsing failed.
    #[error("parse error: {0}")]
    Parse(String),
    /// IO error while persisting output.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON serialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// Prompt template still contained unresolved `{{placeholders}}` after context bind.
    #[error(transparent)]
    Prompt(#[from] crate::prompts::PromptError),
    /// `reference.rs` missing or empty for a system that requires Rust source.
    #[error(
        "reference implementation is missing or empty at {} (required for this prompt kind)",
        path.display()
    )]
    MissingReferenceRust {
        /// Expected path to `reference.rs` under the instance directory.
        path: std::path::PathBuf,
    },
}

/// Result alias.
pub type Result<T> = std::result::Result<T, GenerateError>;

/// Normalized single-obligation record in a generated bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedObligation {
    /// Kind as classified (falls back to `unknown`).
    pub kind: String,
    /// Lean theorem / lemma body.
    pub lean_statement: String,
    /// Natural-language gloss.
    pub nl_gloss: String,
    /// Linked semantic units, if the generator predicted them.
    #[serde(default)]
    pub linked_semantic_units: Vec<String>,
    /// Self-reported confidence, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

/// Parse status for a generated output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseStatus {
    /// Whether normalization fully succeeded.
    pub ok: bool,
    /// Error class if `!ok`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_class: Option<String>,
    /// Error message if `!ok`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl ParseStatus {
    /// Construct an ok status.
    #[must_use]
    pub fn ok() -> Self {
        Self {
            ok: true,
            error_class: None,
            error_message: None,
        }
    }

    /// Construct an error status.
    #[must_use]
    pub fn err(class: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            ok: false,
            error_class: Some(class.into()),
            error_message: Some(message.into()),
        }
    }
}

/// Normalized generation output matching `generated_output.schema.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedOutputBundle {
    /// Schema version.
    pub schema_version: String,
    /// Run id.
    pub run_id: RunId,
    /// System id.
    pub system_id: SystemId,
    /// Instance id.
    pub instance_id: InstanceId,
    /// Provider metadata.
    pub provider_metadata: ProviderMetadata,
    /// Path to the raw (unparsed) output; always preserved.
    pub raw_output_path: String,
    /// Parse status.
    pub parse_status: ParseStatus,
    /// Normalized obligations; possibly empty on parse failure.
    pub normalized_obligations: Vec<GeneratedObligation>,
    /// Deterministic sha256 of the prompt template used.
    pub prompt_hash: String,
    /// Seed for provider-side determinism if supported.
    pub seed: u64,
}

/// Provider metadata embedded in every output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    /// Provider name (e.g. `openai`, `anthropic`, `stub`).
    pub name: String,
    /// Model family (e.g. `gpt-5.4-medium`).
    pub model: String,
    /// Exact version string the provider returned.
    pub model_version: String,
    /// End-to-end latency in milliseconds, if measured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    /// Prompt tokens consumed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_tokens: Option<u64>,
    /// Completion tokens consumed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_tokens: Option<u64>,
}

/// Hash a prompt string into the canonical `sha256:<hex>` form.
///
/// The output is `sha256:` followed by the 64-character lowercase hex
/// encoding of the SHA-256 digest of the UTF-8 bytes of `prompt`. This is the
/// exact form that appears in every `run_manifest.json` under
/// `prompt_hashes[<instance_id>]`, so two runs can be compared for prompt
/// equivalence by byte-comparing their manifest entries.
///
/// # Examples
///
/// ```
/// use cta_generate::hash_prompt;
///
/// let h = hash_prompt("Hello, world!");
/// assert_eq!(
///     h,
///     "sha256:315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3"
/// );
///
/// // The canonical prefix is deterministic and distinguishes the hash from
/// // other digest schemes that might be introduced later.
/// assert!(h.starts_with("sha256:"));
/// assert_eq!(h.len(), "sha256:".len() + 64);
/// ```
#[must_use]
pub fn hash_prompt(prompt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prompt.as_bytes());
    format!("sha256:{}", hex::encode(hasher.finalize()))
}
