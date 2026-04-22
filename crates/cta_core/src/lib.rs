//! `cta_core` — canonical domain types, identifiers, versions, and enums for the
//! CTA benchmark.
//!
//! This crate is intentionally **logic-free** apart from canonical parsing,
//! formatting, and validation. Business logic lives elsewhere.

#![deny(missing_docs)]

pub mod enums;
pub mod error;
pub mod ids;
pub mod versions;

pub use enums::{
    ConsistencyLabel, Difficulty, Domain, FaithfulnessLabel, Importance, ObligationKind,
    ProofRelevance,
};
pub use error::{CoreError, Result};
pub use ids::{InstanceId, ObligationId, RunId, SemanticUnitId, SystemId};
pub use versions::{BenchmarkVersion, MetricsVersion, RubricVersion, SchemaVersion};

/// Current schema version constant used across the workspace.
pub const SCHEMA_VERSION: &str = "schema_v1";
/// Current metrics contract version.
pub const METRICS_VERSION: &str = "metrics_v1";
/// Current annotation rubric version.
pub const RUBRIC_VERSION: &str = "rubric_v1";
