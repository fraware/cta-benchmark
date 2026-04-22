//! Provider abstraction: deterministic offline provider plus live HTTP
//! providers for OpenAI and Anthropic.
//!
//! `cta_generate` is **build-pure** (no network activity during `cargo
//! build`), but the live providers perform real HTTP calls at runtime when
//! their credential environment variable is set. Calls are synchronous and
//! use a small blocking `reqwest` client; no async runtime is required.
//!
//! The offline [`StubProvider`] is the default for CI and smoke tests: it
//! returns a deterministic, schema-valid response encoding a minimum viable
//! obligation bundle so the rest of the pipeline can be exercised
//! end-to-end without any secrets.

use crate::normalize::normalize_response;
use crate::{GeneratedObligation, ParseStatus, Result};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

const DEFAULT_OPENAI_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";
const DEFAULT_ANTHROPIC_ENDPOINT: &str = "https://api.anthropic.com/v1/messages";
const DEFAULT_ANTHROPIC_VERSION: &str = "2023-06-01";
const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 120;

fn http_client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS))
        .user_agent(concat!("cta-benchmark/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| crate::GenerateError::Provider(format!("http client build failed: {e}")))
}

/// Request sent to a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRequest {
    /// Rendered prompt string.
    pub prompt: String,
    /// Seed, if the provider supports deterministic sampling.
    pub seed: u64,
    /// Maximum completion tokens.
    pub max_tokens: u32,
    /// Sampling temperature.
    pub temperature: f64,
}

/// Response returned by a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponse {
    /// Raw text response.
    pub raw: String,
    /// Parsed obligations (empty if parse failed).
    pub obligations: Vec<GeneratedObligation>,
    /// Parse status.
    pub parse_status: ParseStatus,
    /// Latency in milliseconds.
    pub latency_ms: u64,
    /// Tokens consumed (prompt, completion), if known.
    pub tokens: Option<(u64, u64)>,
    /// Provider-reported model version.
    pub model_version: String,
}

/// Trait implemented by every generation provider.
pub trait Provider: std::fmt::Debug + Send + Sync {
    /// Provider name (e.g. `openai`, `anthropic`, `stub`).
    fn name(&self) -> &str;
    /// Model family being used.
    fn model(&self) -> &str;
    /// Invoke the provider.
    ///
    /// # Errors
    /// Returns [`crate::GenerateError::Provider`] for provider-level failures
    /// (misconfiguration, network, rate limits).
    fn generate(&self, req: &ProviderRequest) -> Result<ProviderResponse>;
}

/// Offline stub provider: emits a deterministic, schema-valid bundle whose
/// obligations are suggestive but always parse cleanly. Useful as the
/// baseline for CI smoke tests.
#[derive(Debug, Clone)]
pub struct StubProvider {
    model_version: String,
}

impl StubProvider {
    /// Construct with a specific model version string.
    #[must_use]
    pub fn new(model_version: impl Into<String>) -> Self {
        Self {
            model_version: model_version.into(),
        }
    }
}

impl Default for StubProvider {
    fn default() -> Self {
        Self::new("stub-0.0.1")
    }
}

impl Provider for StubProvider {
    fn name(&self) -> &str {
        "stub"
    }

    fn model(&self) -> &str {
        "stub-local"
    }

    fn generate(&self, req: &ProviderRequest) -> Result<ProviderResponse> {
        // The stub emits a minimum viable obligation bundle that:
        // - parses cleanly (tests the normalizer happy path),
        // - is deterministic under a fixed seed,
        // - does not pretend to be faithful (metrics will score it low).
        let raw = format!(
            "{{\"obligations\": [{{\"kind\": \"structural\", \"lean_statement\": \"True\", \"nl_gloss\": \"stub obligation for seed {}\"}}]}}",
            req.seed
        );
        let (obligations, parse_status) = normalize_response(&raw);
        Ok(ProviderResponse {
            raw,
            obligations,
            parse_status,
            latency_ms: 0,
            tokens: Some((0, 0)),
            model_version: self.model_version.clone(),
        })
    }
}

/// Provider config loaded from `configs/providers/*.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider name (e.g. `openai`).
    pub name: String,
    /// Model family (e.g. `gpt-5.4-medium`).
    pub model: String,
    /// HTTP endpoint, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    /// Environment variable holding the credential.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_env: Option<String>,
    /// Default request parameters.
    #[serde(default)]
    pub request_defaults: serde_json::Value,
    /// Rate-limit hints.
    #[serde(default)]
    pub rate_limit: serde_json::Value,
}

/// Live OpenAI chat-completions provider. Performs a real HTTP POST when
/// its credential env var is set and returns an authentication error
/// otherwise. The request body is produced by
/// [`OpenAiProvider::build_request_body`] and is also exposed for
/// external test harnesses.
#[derive(Debug, Clone)]
pub struct OpenAiProvider {
    config: ProviderConfig,
}

impl OpenAiProvider {
    /// Construct from a provider config.
    #[must_use]
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }

    /// Shape the provider's request body. Exposed for tests and for future
    /// HTTP integration.
    #[must_use]
    pub fn build_request_body(&self, req: &ProviderRequest) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                { "role": "user", "content": req.prompt }
            ],
            "temperature": req.temperature,
            "seed": req.seed,
            "response_format": { "type": "json_object" }
        });
        // Newer GPT-5-family chat models expect `max_completion_tokens`
        // rather than `max_tokens`.
        if self.config.model.starts_with("gpt-5") {
            body["max_completion_tokens"] = serde_json::json!(req.max_tokens);
        } else {
            body["max_tokens"] = serde_json::json!(req.max_tokens);
        }
        body
    }
}

impl Provider for OpenAiProvider {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    fn generate(&self, req: &ProviderRequest) -> Result<ProviderResponse> {
        let env_name = self.config.auth_env.as_deref().unwrap_or("OPENAI_API_KEY");
        let key = std::env::var(env_name).map_err(|_| {
            crate::GenerateError::Provider(format!(
                "openai credentials not configured ({env_name} not set); set CTA_PROVIDER=stub for offline runs"
            ))
        })?;
        let endpoint = self
            .config
            .endpoint
            .as_deref()
            .unwrap_or(DEFAULT_OPENAI_ENDPOINT);
        let body = self.build_request_body(req);
        let client = http_client()?;
        let started = Instant::now();
        let response = client
            .post(endpoint)
            .bearer_auth(key)
            .json(&body)
            .send()
            .map_err(|e| crate::GenerateError::Provider(format!("openai request failed: {e}")))?;
        let status = response.status();
        let latency_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        let text = response.text().map_err(|e| {
            crate::GenerateError::Provider(format!("openai response body read failed: {e}"))
        })?;
        if !status.is_success() {
            return Err(crate::GenerateError::Provider(format!(
                "openai http {status}: {}",
                text.chars().take(512).collect::<String>()
            )));
        }
        parse_openai_response(&text, latency_ms, &self.config.model)
    }
}

fn parse_openai_response(
    text: &str,
    latency_ms: u64,
    fallback_model: &str,
) -> Result<ProviderResponse> {
    let value: serde_json::Value = serde_json::from_str(text)?;
    let raw = value
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();
    let model_version = value
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or(fallback_model)
        .to_string();
    let tokens = value.get("usage").map(|u| {
        (
            u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
            u.get("completion_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        )
    });
    let (obligations, parse_status) = normalize_response(&raw);
    Ok(ProviderResponse {
        raw,
        obligations,
        parse_status,
        latency_ms,
        tokens,
        model_version,
    })
}

/// Live Anthropic messages provider. Performs a real HTTP POST when its
/// credential env var is set; mirrors the OpenAI provider's behaviour
/// otherwise.
#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    config: ProviderConfig,
}

impl AnthropicProvider {
    /// Construct from a provider config.
    #[must_use]
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }

    /// Shape the provider's request body.
    #[must_use]
    pub fn build_request_body(&self, req: &ProviderRequest) -> serde_json::Value {
        serde_json::json!({
            "model": self.config.model,
            "max_tokens": req.max_tokens,
            "temperature": req.temperature,
            "messages": [
                { "role": "user", "content": req.prompt }
            ]
        })
    }
}

impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    fn generate(&self, req: &ProviderRequest) -> Result<ProviderResponse> {
        let env_name = self
            .config
            .auth_env
            .as_deref()
            .unwrap_or("ANTHROPIC_API_KEY");
        let key = std::env::var(env_name).map_err(|_| {
            crate::GenerateError::Provider(format!(
                "anthropic credentials not configured ({env_name} not set); set CTA_PROVIDER=stub for offline runs"
            ))
        })?;
        let endpoint = self
            .config
            .endpoint
            .as_deref()
            .unwrap_or(DEFAULT_ANTHROPIC_ENDPOINT);
        let body = self.build_request_body(req);
        let client = http_client()?;
        let started = Instant::now();
        let response = client
            .post(endpoint)
            .header("x-api-key", key)
            .header("anthropic-version", DEFAULT_ANTHROPIC_VERSION)
            .json(&body)
            .send()
            .map_err(|e| {
                crate::GenerateError::Provider(format!("anthropic request failed: {e}"))
            })?;
        let status = response.status();
        let latency_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        let text = response.text().map_err(|e| {
            crate::GenerateError::Provider(format!("anthropic response body read failed: {e}"))
        })?;
        if !status.is_success() {
            return Err(crate::GenerateError::Provider(format!(
                "anthropic http {status}: {}",
                text.chars().take(512).collect::<String>()
            )));
        }
        parse_anthropic_response(&text, latency_ms, &self.config.model)
    }
}

fn parse_anthropic_response(
    text: &str,
    latency_ms: u64,
    fallback_model: &str,
) -> Result<ProviderResponse> {
    let value: serde_json::Value = serde_json::from_str(text)?;
    let raw = value
        .get("content")
        .and_then(|c| c.as_array())
        .map(|blocks| {
            blocks
                .iter()
                .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();
    let model_version = value
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or(fallback_model)
        .to_string();
    let tokens = value.get("usage").map(|u| {
        (
            u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
            u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
        )
    });
    let (obligations, parse_status) = normalize_response(&raw);
    Ok(ProviderResponse {
        raw,
        obligations,
        parse_status,
        latency_ms,
        tokens,
        model_version,
    })
}

/// Build a boxed provider from a `ProviderConfig`. Unknown provider names
/// fall back to the offline stub with their declared model version.
#[must_use]
pub fn build_from_config(config: ProviderConfig) -> Box<dyn Provider> {
    match config.name.as_str() {
        "openai" => Box::new(OpenAiProvider::new(config)),
        "anthropic" => Box::new(AnthropicProvider::new(config)),
        _ => Box::new(StubProvider::new(config.model.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_provider_returns_parseable_bundle() {
        let p = StubProvider::default();
        let resp = p
            .generate(&ProviderRequest {
                prompt: "hello".into(),
                seed: 1,
                max_tokens: 128,
                temperature: 0.0,
            })
            .expect("stub generate");
        assert!(resp.parse_status.ok);
        assert_eq!(resp.obligations.len(), 1);
        assert_eq!(resp.obligations[0].kind, "structural");
    }

    #[test]
    fn openai_request_body_shape() {
        let cfg = ProviderConfig {
            name: "openai".into(),
            model: "gpt-x".into(),
            endpoint: None,
            auth_env: Some("OPENAI_API_KEY".into()),
            request_defaults: serde_json::Value::Null,
            rate_limit: serde_json::Value::Null,
        };
        let p = OpenAiProvider::new(cfg);
        let body = p.build_request_body(&ProviderRequest {
            prompt: "hi".into(),
            seed: 7,
            max_tokens: 16,
            temperature: 0.0,
        });
        assert_eq!(body["model"], "gpt-x");
        assert_eq!(body["seed"], 7);
        assert_eq!(body["messages"][0]["content"], "hi");
    }

    #[test]
    fn openai_response_parser_extracts_content_model_and_usage() {
        let body = serde_json::json!({
            "model": "gpt-x-2026-01",
            "choices": [{
                "message": {"content": "{\"obligations\": []}"}
            }],
            "usage": {"prompt_tokens": 12, "completion_tokens": 34}
        });
        let resp = parse_openai_response(&body.to_string(), 77, "fallback").unwrap();
        assert_eq!(resp.model_version, "gpt-x-2026-01");
        assert_eq!(resp.tokens, Some((12, 34)));
        assert_eq!(resp.latency_ms, 77);
        assert!(resp.raw.contains("obligations"));
    }

    #[test]
    fn anthropic_response_parser_extracts_blocks_and_usage() {
        let body = serde_json::json!({
            "model": "claude-x-2026-01",
            "content": [
                {"type": "text", "text": "partial "},
                {"type": "text", "text": "reply"}
            ],
            "usage": {"input_tokens": 5, "output_tokens": 9}
        });
        let resp = parse_anthropic_response(&body.to_string(), 42, "fallback").unwrap();
        assert_eq!(resp.model_version, "claude-x-2026-01");
        assert_eq!(resp.raw, "partial reply");
        assert_eq!(resp.tokens, Some((5, 9)));
        assert_eq!(resp.latency_ms, 42);
    }

    #[test]
    fn live_providers_refuse_without_credentials() {
        let cfg = ProviderConfig {
            name: "openai".into(),
            model: "gpt-x".into(),
            endpoint: None,
            auth_env: Some("CTA_TEST_NEVER_SET_OPENAI".into()),
            request_defaults: serde_json::Value::Null,
            rate_limit: serde_json::Value::Null,
        };
        let p = OpenAiProvider::new(cfg);
        let err = p
            .generate(&ProviderRequest {
                prompt: "x".into(),
                seed: 0,
                max_tokens: 1,
                temperature: 0.0,
            })
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("credentials not configured"), "got: {msg}");
    }
}
