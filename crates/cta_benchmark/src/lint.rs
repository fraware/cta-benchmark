//! Benchmark lint pass.
//!
//! Detects the hard-spec failure modes:
//!
//! - missing semantic units
//! - missing reference obligations
//! - bad ids
//! - duplicate tasks
//! - empty edge cases
//! - malformed file layouts

use std::fmt;
use std::fs;
use std::path::PathBuf;

use serde::Serialize;
use serde_json::Value;

use crate::loader::LoadedBenchmark;
use crate::model::InstanceView;

/// Severity of a lint issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LintSeverity {
    /// Blocking; prevents promotion to a released benchmark version.
    Error,
    /// Non-blocking; flagged for review.
    Warning,
}

impl fmt::Display for LintSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LintSeverity::Error => f.write_str("error"),
            LintSeverity::Warning => f.write_str("warn"),
        }
    }
}

/// A single lint finding attached to an instance.
#[derive(Debug, Clone, Serialize)]
pub struct LintIssue {
    /// Id of the affected instance (or "<global>" for benchmark-level issues).
    pub instance_id: String,
    /// Severity.
    pub severity: LintSeverity,
    /// Short machine-readable code.
    pub code: &'static str,
    /// Human-readable message.
    pub message: String,
    /// Path the issue applies to, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

/// Aggregate lint report.
#[derive(Debug, Clone, Serialize)]
pub struct LintReport {
    /// All issues in encounter order.
    pub issues: Vec<LintIssue>,
}

impl LintReport {
    /// True if any issue is an error.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == LintSeverity::Error)
    }

    /// Count of errors.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == LintSeverity::Error)
            .count()
    }

    /// Count of warnings.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == LintSeverity::Warning)
            .count()
    }
}

/// Lint a loaded benchmark and return a structured report.
pub fn lint_benchmark(b: &LoadedBenchmark) -> LintReport {
    let mut issues = Vec::new();

    // Per-instance checks.
    for (id, view) in b.iter() {
        lint_instance(id.as_str(), view, &mut issues);
    }

    // Global: domain coverage at least > 0 once benchmark is non-empty.
    // (Pilots are allowed to have sparse domain coverage; we only warn.)
    if !b.is_empty() {
        let used_domains = b
            .iter()
            .map(|(_, v)| v.record.domain)
            .collect::<std::collections::BTreeSet<_>>();
        if used_domains.len() < 2 {
            issues.push(LintIssue {
                instance_id: "<global>".to_string(),
                severity: LintSeverity::Warning,
                code: "BENCH_DOMAIN_COVERAGE",
                message: format!(
                    "benchmark covers only {} domain(s); consider broader coverage",
                    used_domains.len()
                ),
                path: None,
            });
        }
    }

    LintReport { issues }
}

/// Resolve the canonical Lean module file for a given instance namespace.
///
/// The convention is: namespace `CTA.Benchmark.<Domain>.<Family>NNN` maps to
/// file `<workspace>/lean/CTA/Benchmark/<Domain>/<Family>NNN.lean`, where
/// `<workspace>` is the parent of the benchmark-version directory's parent.
fn canonical_lean_path(namespace: &str, workspace_root: &std::path::Path) -> Option<PathBuf> {
    let prefix = "CTA.Benchmark.";
    let suffix = namespace.strip_prefix(prefix)?;
    let mut p = workspace_root.join("lean").join("CTA").join("Benchmark");
    let parts: Vec<&str> = suffix.split('.').collect();
    if parts.len() < 2 {
        return None;
    }
    for part in &parts[..parts.len() - 1] {
        p = p.join(part);
    }
    let last = parts.last()?;
    p = p.join(format!("{last}.lean"));
    Some(p)
}

fn lint_instance(id: &str, view: &InstanceView, issues: &mut Vec<LintIssue>) {
    let r = &view.record;

    if r.informal_statement.edge_cases.is_empty() {
        issues.push(LintIssue {
            instance_id: id.to_string(),
            severity: LintSeverity::Error,
            code: "INST_EDGE_CASES_EMPTY",
            message: "informal_statement.edge_cases must not be empty".to_string(),
            path: Some(view.instance_json.clone()),
        });
    }

    if r.informal_statement.required_properties.is_empty() {
        issues.push(LintIssue {
            instance_id: id.to_string(),
            severity: LintSeverity::Error,
            code: "INST_PROPERTIES_EMPTY",
            message: "informal_statement.required_properties must not be empty".to_string(),
            path: Some(view.instance_json.clone()),
        });
    }

    // Existence of companion files.
    for (path, code) in [
        (&view.reference_rs, "INST_MISSING_RUST_REFERENCE"),
        (&view.scaffold_lean, "INST_MISSING_LEAN_SCAFFOLD"),
        (&view.reference_obligations, "INST_MISSING_OBLIGATIONS"),
        (&view.semantic_units, "INST_MISSING_SEMANTIC_UNITS"),
        (&view.harness, "INST_MISSING_HARNESS"),
    ] {
        if !path.is_file() {
            issues.push(LintIssue {
                instance_id: id.to_string(),
                severity: LintSeverity::Error,
                code,
                message: format!("required file missing: {}", path.display()),
                path: Some(path.clone()),
            });
        }
    }

    // Semantic unit / obligation content checks.
    if view.semantic_units.is_file() {
        if let Some(json) = read_json(&view.semantic_units, issues, id) {
            let units = json.get("units").and_then(|v| v.as_array());
            match units {
                Some(arr) if !arr.is_empty() => {}
                _ => {
                    issues.push(LintIssue {
                        instance_id: id.to_string(),
                        severity: LintSeverity::Error,
                        code: "INST_SEMANTIC_UNITS_EMPTY",
                        message: "semantic_units.json must contain a non-empty `units` array"
                            .to_string(),
                        path: Some(view.semantic_units.clone()),
                    });
                }
            }
        }
    }

    if view.reference_obligations.is_file() {
        if let Some(json) = read_json(&view.reference_obligations, issues, id) {
            let obligs = json.get("obligations").and_then(|v| v.as_array());
            match obligs {
                Some(arr) if !arr.is_empty() => {}
                _ => {
                    issues.push(LintIssue {
                        instance_id: id.to_string(),
                        severity: LintSeverity::Error,
                        code: "INST_OBLIGATIONS_EMPTY",
                        message: "reference_obligations.json must contain a non-empty `obligations` array"
                            .to_string(),
                        path: Some(view.reference_obligations.clone()),
                    });
                }
            }
        }
    }

    // Canonical Lean scaffold byte-identity: the instance-local `scaffold.lean`
    // and the file under `lean/CTA/Benchmark/<Domain>/<Family>NNN.lean` must be
    // identical byte-for-byte. This is what lets Lake build the instance while
    // preserving the instance directory as the human-facing source of truth.
    if view.scaffold_lean.is_file() {
        // Workspace root is the grandparent-of-the-grandparent of the instance
        // directory:
        //   view.dir = benchmark/v0.1/instances/<domain>/<id>
        //   view.dir.ancestors() yields, in order, <id>, <domain>, instances,
        //   v0.1, benchmark, <workspace-root>, ... so we want index 5.
        if let Some(workspace_root) = view.dir.ancestors().nth(5) {
            if let Some(canonical) = canonical_lean_path(&r.lean_target.namespace, workspace_root) {
                match (fs::read(&view.scaffold_lean), fs::read(&canonical)) {
                    (Ok(a), Ok(b)) => {
                        if a != b {
                            issues.push(LintIssue {
                                instance_id: id.to_string(),
                                severity: LintSeverity::Error,
                                code: "INST_LEAN_SCAFFOLD_DIVERGENCE",
                                message: format!(
                                    "instance scaffold.lean and canonical Lean module differ: {}",
                                    canonical.display()
                                ),
                                path: Some(view.scaffold_lean.clone()),
                            });
                        }
                    }
                    (_, Err(_)) => {
                        issues.push(LintIssue {
                            instance_id: id.to_string(),
                            severity: LintSeverity::Error,
                            code: "INST_CANONICAL_LEAN_MISSING",
                            message: format!(
                                "canonical Lean module not found: {}",
                                canonical.display()
                            ),
                            path: Some(canonical),
                        });
                    }
                    (Err(_), _) => {
                        // already reported by the missing-files pass above
                    }
                }
            } else {
                issues.push(LintIssue {
                    instance_id: id.to_string(),
                    severity: LintSeverity::Warning,
                    code: "INST_NAMESPACE_NON_CANONICAL",
                    message: format!(
                        "instance namespace `{}` does not map to a canonical CTA.Benchmark.* path",
                        r.lean_target.namespace
                    ),
                    path: Some(view.instance_json.clone()),
                });
            }
        }
    }
}

fn read_json(path: &std::path::Path, issues: &mut Vec<LintIssue>, id: &str) -> Option<Value> {
    match fs::read_to_string(path) {
        Ok(s) => match serde_json::from_str::<Value>(&s) {
            Ok(v) => Some(v),
            Err(e) => {
                issues.push(LintIssue {
                    instance_id: id.to_string(),
                    severity: LintSeverity::Error,
                    code: "INST_JSON_PARSE",
                    message: format!("failed to parse JSON: {e}"),
                    path: Some(path.to_path_buf()),
                });
                None
            }
        },
        Err(e) => {
            issues.push(LintIssue {
                instance_id: id.to_string(),
                severity: LintSeverity::Error,
                code: "INST_JSON_READ",
                message: format!("failed to read file: {e}"),
                path: Some(path.to_path_buf()),
            });
            None
        }
    }
}
