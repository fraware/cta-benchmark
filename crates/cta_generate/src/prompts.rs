//! Prompt templates and rendering for generation systems.
//!
//! Each system (`text_only`, `code_only`, `naive_concat`, `full_method`)
//! consumes a different subset of per-instance context. A template body uses
//! `{{placeholder}}` markers which are replaced by [`render`]; unknown
//! placeholders are left untouched so CI can flag missing context.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors produced while loading or rendering templates.
#[derive(Debug, Error)]
pub enum PromptError {
    /// IO failure.
    #[error("io error at {path}: {source}")]
    Io {
        /// Offending path.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// JSON parse failure.
    #[error("failed to parse prompt config at {path}: {source}")]
    Json {
        /// Offending path.
        path: PathBuf,
        /// Underlying serde_json error.
        #[source]
        source: serde_json::Error,
    },
    /// Declared `system_id` inside the config does not match the expected
    /// identifier derived from the file stem.
    #[error("prompt config at {path} declares system_id '{declared}', expected '{expected}'")]
    SystemIdMismatch {
        /// Offending path.
        path: PathBuf,
        /// Declared system id.
        declared: String,
        /// Expected system id.
        expected: String,
    },
    /// Rendering left one or more unresolved placeholders.
    #[error("unresolved placeholders: {0:?}")]
    UnresolvedPlaceholders(Vec<String>),
}

/// Result alias.
pub type Result<T> = std::result::Result<T, PromptError>;

/// Mandatory generation systems from the spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptKind {
    /// Informal statement only.
    TextOnly,
    /// Reference Rust code only.
    CodeOnly,
    /// Text + code concatenated without structure.
    NaiveConcat,
    /// Full structured method with extracted Rust summary.
    FullMethod,
}

impl PromptKind {
    /// Canonical snake_case name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            PromptKind::TextOnly => "text_only",
            PromptKind::CodeOnly => "code_only",
            PromptKind::NaiveConcat => "naive_concat",
            PromptKind::FullMethod => "full_method",
        }
    }

    /// Parse from the snake_case name.
    ///
    /// # Errors
    /// Returns `None` if `s` is not one of the four canonical system names.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "text_only" => Some(PromptKind::TextOnly),
            "code_only" => Some(PromptKind::CodeOnly),
            "naive_concat" => Some(PromptKind::NaiveConcat),
            "full_method" => Some(PromptKind::FullMethod),
            _ => None,
        }
    }
}

/// Placeholders every template may reference. Unknown placeholders in the
/// body are flagged by [`render_strict`] as an error.
#[derive(Debug, Clone, Default)]
pub struct PromptContext {
    /// Key → replacement mapping.
    values: BTreeMap<String, String>,
}

impl PromptContext {
    /// Construct an empty context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    /// Insert a placeholder value.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.values.insert(key.into(), value.into());
        self
    }

    /// Iterate the (key, value) pairs in deterministic order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.values.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// A loaded prompt template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// Schema version constant.
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    /// Canonical `system_id` (e.g. `full_method_v1`).
    pub system_id: String,
    /// Template kind.
    pub kind: PromptKind,
    /// Template version string (e.g. `v1`).
    pub version: String,
    /// Raw template body with `{{placeholder}}` markers.
    pub body: String,
}

fn default_schema_version() -> String {
    "schema_v1".to_string()
}

impl PromptTemplate {
    /// Construct a template directly.
    #[must_use]
    pub fn new(
        system_id: impl Into<String>,
        kind: PromptKind,
        version: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: default_schema_version(),
            system_id: system_id.into(),
            kind,
            version: version.into(),
            body: body.into(),
        }
    }

    /// Render the template by replacing every `{{placeholder}}` with its
    /// value from `ctx`. Missing placeholders are left as-is.
    #[must_use]
    pub fn render(&self, ctx: &PromptContext) -> String {
        render(&self.body, ctx)
    }

    /// Render strictly: any unresolved placeholder yields an error.
    ///
    /// # Errors
    /// Returns [`PromptError::UnresolvedPlaceholders`] if any
    /// `{{placeholder}}` marker remains after substitution.
    pub fn render_strict(&self, ctx: &PromptContext) -> Result<String> {
        render_strict(&self.body, ctx)
    }

    /// Load a prompt template from a JSON config file (e.g.
    /// `configs/prompts/full_method_v1.json`).
    ///
    /// # Errors
    /// Returns IO or JSON errors; returns [`PromptError::SystemIdMismatch`]
    /// if the declared `system_id` doesn't match the file stem.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let raw = std::fs::read_to_string(path).map_err(|source| PromptError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let template: PromptTemplate =
            serde_json::from_str(&raw).map_err(|source| PromptError::Json {
                path: path.to_path_buf(),
                source,
            })?;
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if stem != template.system_id {
                return Err(PromptError::SystemIdMismatch {
                    path: path.to_path_buf(),
                    declared: template.system_id.clone(),
                    expected: stem.to_string(),
                });
            }
        }
        Ok(template)
    }
}

/// Substitute every `{{key}}` with `ctx[key]`. Missing keys are left as-is.
#[must_use]
pub fn render(body: &str, ctx: &PromptContext) -> String {
    let mut out = String::with_capacity(body.len());
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            if let Some(end) = find_close(bytes, i + 2) {
                let key = &body[i + 2..end];
                if let Some(v) = ctx.values.get(key) {
                    out.push_str(v);
                    i = end + 2;
                    continue;
                }
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Render strictly: fail if any `{{...}}` placeholder remains unresolved.
pub fn render_strict(body: &str, ctx: &PromptContext) -> Result<String> {
    let out = render(body, ctx);
    let unresolved = find_unresolved(&out);
    if unresolved.is_empty() {
        Ok(out)
    } else {
        Err(PromptError::UnresolvedPlaceholders(unresolved))
    }
}

fn find_close(bytes: &[u8], from: usize) -> Option<usize> {
    let mut i = from;
    while i + 1 < bytes.len() {
        if bytes[i] == b'}' && bytes[i + 1] == b'}' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn find_unresolved(s: &str) -> Vec<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            if let Some(end) = find_close(bytes, i + 2) {
                out.push(s[i + 2..end].to_string());
                i = end + 2;
                continue;
            }
        }
        i += 1;
    }
    out.sort();
    out.dedup();
    out
}

/// Load every `.json` prompt config from a directory.
///
/// # Errors
/// Returns IO or JSON errors encountered while scanning.
pub fn load_all_from_dir(dir: impl AsRef<Path>) -> Result<Vec<PromptTemplate>> {
    let dir = dir.as_ref();
    let mut out = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|source| PromptError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| PromptError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) == Some("json") {
            out.push(PromptTemplate::load(&p)?);
        }
    }
    out.sort_by(|a, b| a.system_id.cmp(&b.system_id));
    Ok(out)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn render_substitutes_simple() {
        let mut ctx = PromptContext::new();
        ctx.insert("a", "1").insert("b", "2");
        assert_eq!(render("x {{a}} y {{b}}", &ctx), "x 1 y 2");
    }

    #[test]
    fn render_leaves_unknown_placeholders() {
        let ctx = PromptContext::new();
        assert_eq!(render("x {{unknown}} y", &ctx), "x {{unknown}} y");
    }

    #[test]
    fn render_strict_flags_missing() {
        let ctx = PromptContext::new();
        let err = render_strict("x {{foo}} {{bar}}", &ctx).unwrap_err();
        match err {
            PromptError::UnresolvedPlaceholders(v) => {
                assert_eq!(v, vec!["bar".to_string(), "foo".to_string()]);
            }
            _ => panic!("wrong err kind"),
        }
    }

    #[test]
    fn kind_round_trip() {
        for k in [
            PromptKind::TextOnly,
            PromptKind::CodeOnly,
            PromptKind::NaiveConcat,
            PromptKind::FullMethod,
        ] {
            assert_eq!(PromptKind::parse(k.as_str()), Some(k));
        }
        assert_eq!(PromptKind::parse("nope"), None);
    }
}
