//! `cta_schema` — JSON schema loading and validation for CTA artifacts.
//!
//! Responsibilities (per architecture spec):
//!
//! - load schema files from a schema root directory
//! - validate instances, annotations, manifests, and outputs
//! - expose a uniform `SchemaRegistry` to the rest of the workspace
//!
//! Rule: every persisted artifact must validate against a schema.
//!
//! # Examples
//!
//! ```
//! use cta_schema::{SchemaName, SchemaRegistry};
//!
//! let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
//!     .parent()
//!     .unwrap()
//!     .parent()
//!     .unwrap();
//! let registry = SchemaRegistry::load(workspace.join("schemas")).unwrap();
//!
//! let ok = serde_json::json!({
//!     "schema_version": "schema_v1",
//!     "run_id": "run_2026_04_21_full_method_v1_dev_001",
//!     "system_id": "full_method_v1",
//!     "benchmark_version": "v0.1",
//!     "split": "dev",
//!     "instances": ["arrays_binary_search_001"],
//!     "provider": { "name": "stub", "model": "stub-1" },
//!     "prompts_used": {"arrays_binary_search_001": "sha256:0"},
//!     "prompt_hashes": {"arrays_binary_search_001": "sha256:0"},
//!     "temperature": 0.0,
//!     "max_tokens": 64,
//!     "seed": 0,
//!     "benchmark_artifacts_hash": "sha256:0",
//!     "generated_at": "2026-04-21T00:00:00Z",
//!     "metrics_version": "metrics_v1",
//!     "annotation_rubric_version": "rubric_v1"
//! });
//! // Only the _shape_ is validated here; most required fields are present.
//! // This demonstrates that the registry returns structured errors when
//! // validation fails, rather than panicking.
//! let _ = registry.validate(SchemaName::RunManifest, &ok);
//! ```

#![deny(missing_docs)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use jsonschema::{Draft, JSONSchema};
use serde_json::Value;
use thiserror::Error;

/// Canonical name of each schema the benchmark knows about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SchemaName {
    /// `instance.schema.json`
    Instance,
    /// `obligation.schema.json` (reference obligations)
    Obligation,
    /// `annotation.schema.json`
    Annotation,
    /// `run_manifest.schema.json`
    RunManifest,
    /// `generated_output.schema.json`
    GeneratedOutput,
    /// `results_bundle.schema.json`
    ResultsBundle,
    /// `semantic_units.schema.json`
    SemanticUnits,
    /// `harness.schema.json`
    Harness,
    /// `benchmark_manifest.schema.json`
    BenchmarkManifest,
    /// `experiment.schema.json`
    Experiment,
}

impl SchemaName {
    /// Canonical filename for this schema.
    #[must_use]
    pub const fn file_name(self) -> &'static str {
        match self {
            SchemaName::Instance => "instance.schema.json",
            SchemaName::Obligation => "obligation.schema.json",
            SchemaName::Annotation => "annotation.schema.json",
            SchemaName::RunManifest => "run_manifest.schema.json",
            SchemaName::GeneratedOutput => "generated_output.schema.json",
            SchemaName::ResultsBundle => "results_bundle.schema.json",
            SchemaName::SemanticUnits => "semantic_units.schema.json",
            SchemaName::Harness => "harness.schema.json",
            SchemaName::BenchmarkManifest => "benchmark_manifest.schema.json",
            SchemaName::Experiment => "experiment.schema.json",
        }
    }

    /// All schema names in declaration order.
    pub const ALL: &'static [SchemaName] = &[
        SchemaName::Instance,
        SchemaName::Obligation,
        SchemaName::Annotation,
        SchemaName::RunManifest,
        SchemaName::GeneratedOutput,
        SchemaName::ResultsBundle,
        SchemaName::SemanticUnits,
        SchemaName::Harness,
        SchemaName::BenchmarkManifest,
        SchemaName::Experiment,
    ];

    /// Parse a [`SchemaName`] from its CLI-friendly lowercase identifier.
    /// Accepts both short (e.g. `instance`) and kebab-ish (e.g. `run-manifest`)
    /// forms.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "instance" => Some(Self::Instance),
            "obligation" => Some(Self::Obligation),
            "annotation" => Some(Self::Annotation),
            "run_manifest" | "run-manifest" => Some(Self::RunManifest),
            "generated_output" | "generated-output" => Some(Self::GeneratedOutput),
            "results_bundle" | "results-bundle" => Some(Self::ResultsBundle),
            "semantic_units" | "semantic-units" => Some(Self::SemanticUnits),
            "harness" => Some(Self::Harness),
            "benchmark_manifest" | "benchmark-manifest" => Some(Self::BenchmarkManifest),
            "experiment" => Some(Self::Experiment),
            _ => None,
        }
    }
}

/// Errors produced while loading or validating schemas.
#[derive(Debug, Error)]
pub enum SchemaError {
    /// The schema root directory does not exist.
    #[error("schema root not found: {0}")]
    RootMissing(PathBuf),

    /// A schema file was missing from the root.
    #[error("schema file missing: {0}")]
    SchemaFileMissing(PathBuf),

    /// A schema failed to parse as JSON.
    #[error("failed to read schema {path}: {source}")]
    ReadFailed {
        /// Path to the file we attempted to read.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// JSON deserialization failed.
    #[error("failed to parse JSON at {path}: {source}")]
    JsonParse {
        /// Path to the offending file.
        path: PathBuf,
        /// Underlying serde_json error.
        #[source]
        source: serde_json::Error,
    },

    /// The schema itself is malformed.
    #[error("schema compilation failed for {path}: {message}")]
    SchemaCompile {
        /// Path to the schema file.
        path: PathBuf,
        /// Compiled error message.
        message: String,
    },

    /// Instance validation failed.
    #[error("validation failed for {path} against {schema:?}:\n  - {errors}")]
    Validation {
        /// Path to the artifact.
        path: PathBuf,
        /// Which schema was used.
        schema: SchemaName,
        /// Joined, one-per-line error messages.
        errors: String,
    },
}

/// Result alias for this crate.
pub type Result<T> = std::result::Result<T, SchemaError>;

/// In-memory registry of compiled JSON schemas keyed by [`SchemaName`].
#[derive(Debug)]
pub struct SchemaRegistry {
    root: PathBuf,
    compiled: BTreeMap<SchemaName, JSONSchema>,
}

impl SchemaRegistry {
    /// Load all canonical schemas from `root`.
    ///
    /// Every schema's JSON document is also registered as a resolvable
    /// document (keyed by its `$id`) so that cross-schema `$ref`s like
    /// `"results_bundle.schema.json" -> "run_manifest.schema.json"` resolve
    /// against local files rather than attempting a network fetch.
    pub fn load(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        if !root.is_dir() {
            return Err(SchemaError::RootMissing(root));
        }

        let mut raw_docs: Vec<(SchemaName, PathBuf, Value)> =
            Vec::with_capacity(SchemaName::ALL.len());
        for name in SchemaName::ALL {
            let path = root.join(name.file_name());
            if !path.exists() {
                return Err(SchemaError::SchemaFileMissing(path));
            }
            let raw = fs::read_to_string(&path).map_err(|source| SchemaError::ReadFailed {
                path: path.clone(),
                source,
            })?;
            let value: Value =
                serde_json::from_str(&raw).map_err(|source| SchemaError::JsonParse {
                    path: path.clone(),
                    source,
                })?;
            raw_docs.push((*name, path, value));
        }

        let mut compiled = BTreeMap::new();
        for (name, path, value) in &raw_docs {
            let mut options = JSONSchema::options();
            options.with_draft(Draft::Draft7);
            for (_, _, other) in &raw_docs {
                if let Some(id) = other.get("$id").and_then(Value::as_str) {
                    options.with_document(id.to_string(), other.clone());
                }
            }
            let schema = options
                .compile(value)
                .map_err(|e| SchemaError::SchemaCompile {
                    path: path.clone(),
                    message: e.to_string(),
                })?;
            compiled.insert(*name, schema);
        }

        Ok(Self { root, compiled })
    }

    /// Root directory the schemas were loaded from.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Validate a JSON value against a named schema.
    pub fn validate(&self, name: SchemaName, value: &Value) -> Result<()> {
        self.validate_with_path(name, value, Path::new("<memory>"))
    }

    /// Validate a JSON file on disk against a named schema.
    pub fn validate_file(&self, name: SchemaName, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path).map_err(|source| SchemaError::ReadFailed {
            path: path.to_path_buf(),
            source,
        })?;
        let value: Value = serde_json::from_str(&raw).map_err(|source| SchemaError::JsonParse {
            path: path.to_path_buf(),
            source,
        })?;
        self.validate_with_path(name, &value, path)
    }

    fn validate_with_path(&self, name: SchemaName, value: &Value, path: &Path) -> Result<()> {
        let schema = self
            .compiled
            .get(&name)
            .expect("all canonical schemas are loaded in `SchemaRegistry::load`");

        let result = schema.validate(value);
        if let Err(errors) = result {
            let joined = errors
                .map(|e| format!("{} at {}", e, e.instance_path))
                .collect::<Vec<_>>()
                .join("\n  - ");
            return Err(SchemaError::Validation {
                path: path.to_path_buf(),
                schema: name,
                errors: joined,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema_root() -> PathBuf {
        // Walk up from this file to the workspace root and return schemas/.
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest
            .parent()
            .expect("crates dir")
            .parent()
            .expect("workspace root")
            .join("schemas")
    }

    #[test]
    fn loads_all_canonical_schemas() {
        let reg = SchemaRegistry::load(schema_root()).expect("load schemas");
        for name in SchemaName::ALL {
            assert!(reg.compiled.contains_key(name), "missing {:?}", name);
        }
    }

    #[test]
    fn rejects_missing_root() {
        let err = SchemaRegistry::load("/definitely/does/not/exist").unwrap_err();
        assert!(matches!(err, SchemaError::RootMissing(_)));
    }
}
