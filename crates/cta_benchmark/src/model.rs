//! Typed model of a benchmark instance as loaded from disk.
//!
//! These types mirror the JSON schema in `schemas/instance.schema.json`.

use std::path::PathBuf;

use cta_core::{BenchmarkVersion, Difficulty, Domain, InstanceId, RubricVersion};
use serde::{Deserialize, Serialize};

/// Deserialized content of `instance.json` for a single instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstanceRecord {
    /// Schema version constant (`schema_v1`).
    pub schema_version: String,
    /// Canonical instance id.
    pub instance_id: InstanceId,
    /// Human-readable title.
    pub title: String,
    /// Algorithmic domain.
    pub domain: Domain,
    /// Difficulty level.
    pub difficulty: Difficulty,
    /// Benchmark version (e.g. `v0.1`).
    pub benchmark_version: BenchmarkVersion,
    /// Informal natural-language statement and properties.
    pub informal_statement: InformalStatement,
    /// Reference Rust implementation pointer.
    pub rust_reference: RustReference,
    /// Lean target pointer.
    pub lean_target: LeanTarget,
    /// Annotation metadata.
    pub annotation: AnnotationMeta,
    /// Behavioral oracle metadata.
    pub behavioral_oracle: BehavioralOracleMeta,
}

/// Informal statement bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InformalStatement {
    /// Human-readable problem statement.
    pub text: String,
    /// Preconditions on inputs.
    pub preconditions: Vec<String>,
    /// Properties that obligations should collectively capture.
    pub required_properties: Vec<String>,
    /// Edge cases that must be considered.
    pub edge_cases: Vec<String>,
}

/// Pointer to the Rust reference implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RustReference {
    /// Repo-relative path to the `.rs` file.
    pub path: String,
    /// Name of the entry function.
    pub entry_fn: String,
}

/// Pointer to Lean scaffold and gold artifacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeanTarget {
    /// Path to the Lean scaffold file.
    pub scaffold_path: String,
    /// Lean namespace for this instance.
    pub namespace: String,
    /// Path to reference obligations JSON.
    pub reference_obligations_path: String,
    /// Path to semantic units JSON.
    pub semantic_units_path: String,
}

/// Annotation metadata pinned to the instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnnotationMeta {
    /// Rubric version the instance was designed for.
    pub rubric_version: RubricVersion,
    /// Optional free-form notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Behavioral oracle metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BehavioralOracleMeta {
    /// Path to the harness JSON.
    pub harness_path: String,
}

/// Loaded instance + resolved filesystem paths.
#[derive(Debug, Clone)]
pub struct InstanceView {
    /// Parsed instance record.
    pub record: InstanceRecord,
    /// Absolute path to the instance directory.
    pub dir: PathBuf,
    /// Absolute path to `instance.json`.
    pub instance_json: PathBuf,
    /// Absolute path to the Rust reference source.
    pub reference_rs: PathBuf,
    /// Absolute path to the Lean scaffold.
    pub scaffold_lean: PathBuf,
    /// Absolute path to reference obligations JSON.
    pub reference_obligations: PathBuf,
    /// Absolute path to semantic units JSON.
    pub semantic_units: PathBuf,
    /// Absolute path to harness JSON.
    pub harness: PathBuf,
}
