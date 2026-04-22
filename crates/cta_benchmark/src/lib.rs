//! `cta_benchmark` — benchmark loader, linter, and manifest builder.
//!
//! Responsibilities:
//!
//! - scan `benchmark/<version>/instances/**`
//! - ensure instance ids are unique and well-formed
//! - enforce that each instance carries semantic units, reference obligations,
//!   and a harness
//! - compute a stable manifest hash over the benchmark contents
//!
//! # Examples
//!
//! ```
//! use cta_benchmark::load_benchmark;
//! use cta_core::BenchmarkVersion;
//!
//! let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
//!     .parent()
//!     .unwrap()
//!     .parent()
//!     .unwrap();
//! let version = BenchmarkVersion::new("v0.1").unwrap();
//! let version_root = workspace.join("benchmark").join(version.as_str());
//! let bench = load_benchmark(&version_root, &version).unwrap();
//!
//! // The pilot benchmark ships 12 instances, each with a non-empty
//! // reference implementation and at least one semantic unit.
//! assert!(bench.instances.len() >= 12);
//! for view in bench.instances.values() {
//!     assert!(!view.record.informal_statement.text.is_empty());
//!     assert!(view.semantic_units.exists());
//! }
//! ```

#![deny(missing_docs)]

pub mod lint;
pub mod loader;
pub mod manifest;
pub mod model;

pub use lint::{lint_benchmark, LintIssue, LintReport, LintSeverity};
pub use loader::{load_benchmark, LoadedBenchmark};
pub use manifest::{build_manifest, BenchmarkManifest};
pub use model::{InstanceRecord, InstanceView};
