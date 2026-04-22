//! `cta_behavior` — behavioral harness and falsification engine.
//!
//! The harness executes a reference implementation (statically linked into
//! this crate, byte-identical with the benchmark's `reference.rs`) over a
//! stream of deterministic inputs and applies a set of oracle checks. The
//! first falsifying input for each oracle is recorded. The harness is purely
//! native Rust: no subprocesses, no file I/O beyond inputs/outputs.
//!
//! Philosophy (per spec):
//! - a falsification is strong negative evidence;
//! - clean trials are only weak positive evidence.

#![deny(missing_docs)]

use std::collections::HashMap;

use cta_core::InstanceId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod generators;
pub mod pilot;

/// Errors produced by the behavioral harness.
#[derive(Debug, Error)]
pub enum BehaviorError {
    /// The harness config references an input generator that is unknown to
    /// the built-in registry.
    #[error("unknown input generator kind: {0}")]
    UnknownGenerator(String),
    /// The harness config references an oracle check that is unknown to the
    /// built-in registry.
    #[error("unknown oracle check: {0}")]
    UnknownOracleCheck(String),
    /// No adapter has been registered for the given instance id.
    #[error("no harness adapter registered for instance: {0}")]
    UnknownInstance(String),
    /// Input generator failed to produce a valid input.
    #[error("input generator error: {0}")]
    Generator(String),
    /// JSON ser/de failure.
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Result alias.
pub type Result<T> = std::result::Result<T, BehaviorError>;

/// Harness kind corresponding to the JSON schema enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessKind {
    /// Exact per-input output comparison.
    ExactOutput,
    /// Randomized property-based checks.
    PropertyBased,
    /// Compare against a trusted reference on small inputs.
    ReferenceRelational,
    /// Focused edge-case battery.
    EdgeCase,
}

/// Parsed harness config matching `harness.schema.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessConfig {
    /// Schema version constant.
    pub schema_version: String,
    /// Harness kind.
    pub harness_type: HarnessKind,
    /// Deterministic seed.
    pub seed: u64,
    /// Number of trials.
    pub num_trials: u32,
    /// Input generator spec.
    pub input_generator: serde_json::Value,
    /// Oracle check names.
    pub oracle_checks: Vec<String>,
    /// Per-trial timeout in milliseconds (advisory only for the native
    /// runner; inputs are bounded by generator params to stay under the
    /// budget).
    pub timeout_ms: u64,
}

/// A single falsification record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Falsification {
    /// Oracle check that failed.
    pub oracle_check: String,
    /// Compact input representation.
    pub input_repr: String,
    /// Observed output (compact).
    pub observed: String,
    /// Expected behavior description.
    pub expected: String,
    /// Trial index (0-based) at which the falsification was first observed.
    pub trial: u32,
}

/// Aggregate stats for a single oracle check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleCheckStats {
    /// Check name.
    pub name: String,
    /// How many trials this check was evaluated against.
    pub trials_evaluated: u32,
    /// How many trials this check rejected (full count, not just first).
    pub violations: u32,
}

/// Report summarizing a harness run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessReport {
    /// Instance id the run targeted.
    pub instance_id: String,
    /// Config seed (propagated for reproducibility).
    pub seed: u64,
    /// Total trials executed.
    pub trials_run: u32,
    /// Per-oracle stats.
    pub oracle_stats: Vec<OracleCheckStats>,
    /// First falsification seen for each oracle (at most one per oracle).
    pub falsifications: Vec<Falsification>,
}

impl HarnessReport {
    /// Whether any falsification was observed.
    #[must_use]
    pub fn any_falsified(&self) -> bool {
        !self.falsifications.is_empty()
    }
}

/// Trait implemented by per-instance harness adapters.
pub trait HarnessAdapter: Send + Sync {
    /// Canonical instance identifier this adapter targets.
    fn instance_id(&self) -> &str;
    /// Run the harness with the given config. Must be deterministic: same
    /// config → same report (modulo trial order).
    fn run(&self, config: &HarnessConfig) -> Result<HarnessReport>;
}

/// Registry of harness adapters keyed by instance id.
#[derive(Default)]
pub struct AdapterRegistry {
    adapters: HashMap<String, Box<dyn HarnessAdapter>>,
}

impl std::fmt::Debug for AdapterRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdapterRegistry")
            .field("keys", &self.adapters.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl AdapterRegistry {
    /// Empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an adapter. Panics in debug builds on duplicate keys.
    pub fn register(&mut self, adapter: Box<dyn HarnessAdapter>) {
        let id = adapter.instance_id().to_string();
        debug_assert!(
            !self.adapters.contains_key(&id),
            "duplicate adapter registration: {id}"
        );
        self.adapters.insert(id, adapter);
    }

    /// Return the set of registered instance ids.
    #[must_use]
    pub fn keys(&self) -> Vec<&str> {
        let mut out: Vec<&str> = self.adapters.keys().map(String::as_str).collect();
        out.sort_unstable();
        out
    }

    /// Look up an adapter for the given instance id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&dyn HarnessAdapter> {
        self.adapters.get(id).map(AsRef::as_ref)
    }

    /// Construct a registry pre-populated with every pilot adapter in the
    /// v0.1 benchmark.
    #[must_use]
    pub fn with_pilot() -> Self {
        pilot::register_all()
    }
}

/// Convenience: run the harness against a (registry, instance_id, config)
/// triple.
pub fn run(
    registry: &AdapterRegistry,
    instance: &InstanceId,
    config: &HarnessConfig,
) -> Result<HarnessReport> {
    let adapter = registry
        .get(instance.as_str())
        .ok_or_else(|| BehaviorError::UnknownInstance(instance.as_str().to_string()))?;
    adapter.run(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_reports_pilot_instances() {
        let r = AdapterRegistry::with_pilot();
        let keys = r.keys();
        assert!(keys.contains(&"arrays_binary_search_001"));
        assert!(keys.contains(&"sorting_insertion_sort_001"));
        assert!(keys.contains(&"graph_dijkstra_001"));
        assert_eq!(keys.len(), 12);
    }
}
