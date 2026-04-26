//! Authoring-heuristic lints for gold obligations.
//!
//! These checks go beyond existence/shape: they inspect the *content* of
//! `reference_obligations.json` and `semantic_units.json` to catch
//! authoring mistakes that produced the v0.1 audit findings — vacuous
//! existential termination statements, unconditional preconditions,
//! critical SUs with no linked obligation, and obligations with no linked
//! SU. Every check emits a stable code prefixed `AUTHORING_*`.
//!
//! These are all warnings by default; the benchmark freezer promotes
//! them to errors via `cta benchmark lint --release --strict-authoring`
//! (not yet exposed as a CLI flag; when added, the knob is simply
//! `issues.iter().any(|i| i.code.starts_with("AUTHORING_"))`).

use std::collections::BTreeSet;
use std::path::Path;

use regex::Regex;
use serde_json::Value;

use crate::lint::{LintIssue, LintSeverity};
use crate::loader::LoadedBenchmark;

/// Run every authoring heuristic over a loaded benchmark and append
/// findings to `issues`.
#[allow(clippy::expect_used)] // fixed authoring patterns; failure is a programmer error
pub fn check_authoring(b: &LoadedBenchmark, issues: &mut Vec<LintIssue>) {
    let vacuous_exists = Regex::new(r"∃\s*[^,]+,[^=]*=").expect("valid regex");
    let unconditional_precond = Regex::new(r"^∀[^→]*$").expect("valid regex");

    for (id, view) in b.iter() {
        let obls_path = &view.reference_obligations;
        let sus_path = &view.semantic_units;

        let obls = load_json_array(obls_path, "obligations");
        let sus = load_json_array(sus_path, "units");

        let mut linked_sus: BTreeSet<String> = BTreeSet::new();
        if let Some(arr) = obls.as_ref() {
            for obl in arr {
                let obl_id = obl
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unknown>");
                let kind = obl.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                let lean = obl
                    .get("lean_statement")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let sus_for_obl = obl
                    .get("linked_semantic_units")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(str::to_string))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                if kind == "termination" && vacuous_exists.is_match(lean) {
                    issues.push(LintIssue {
                        instance_id: id.as_str().to_string(),
                        severity: LintSeverity::Warning,
                        code: "AUTHORING_VACUOUS_TERMINATION",
                        message: format!(
                            "obligation {obl_id} is a vacuous existential termination statement; \
                             prefer a well-founded measure or drop the obligation"
                        ),
                        path: Some(obls_path.clone()),
                    });
                }

                if kind == "precondition" && unconditional_precond.is_match(lean.trim()) {
                    issues.push(LintIssue {
                        instance_id: id.as_str().to_string(),
                        severity: LintSeverity::Warning,
                        code: "AUTHORING_UNCONDITIONAL_PRECONDITION",
                        message: format!(
                            "obligation {obl_id} is a bare universal precondition with no \
                             downstream obligation; thread the precondition into the \
                             postcondition that uses it instead"
                        ),
                        path: Some(obls_path.clone()),
                    });
                }

                if sus_for_obl.is_empty() {
                    issues.push(LintIssue {
                        instance_id: id.as_str().to_string(),
                        severity: LintSeverity::Warning,
                        code: "AUTHORING_OBLIGATION_NO_SEMANTIC_UNITS",
                        message: format!("obligation {obl_id} is not linked to any semantic unit"),
                        path: Some(obls_path.clone()),
                    });
                }

                linked_sus.extend(sus_for_obl);
            }
        }

        if let Some(units) = sus.as_ref() {
            for su in units {
                let Some(su_id) = su.get("id").and_then(|v| v.as_str()) else {
                    continue;
                };
                let criticality = su
                    .get("criticality")
                    .and_then(|v| v.as_str())
                    .unwrap_or("optional");
                if criticality == "critical" && !linked_sus.contains(su_id) {
                    issues.push(LintIssue {
                        instance_id: id.as_str().to_string(),
                        severity: LintSeverity::Warning,
                        code: "AUTHORING_CRITICAL_SU_UNCOVERED",
                        message: format!(
                            "critical semantic unit {su_id} is not linked to any gold obligation; \
                             either add a faithful obligation for it or demote it to supporting"
                        ),
                        path: Some(sus_path.clone()),
                    });
                }
            }
        }

        if let Some(arr) = obls.as_ref() {
            let all_postconditions = !arr.is_empty()
                && arr.iter().all(|o| {
                    o.get("kind")
                        .and_then(|v| v.as_str())
                        .map(|k| k == "postcondition")
                        .unwrap_or(false)
                });
            let mentions_invariant_hint = view
                .record
                .informal_statement
                .required_properties
                .iter()
                .any(|p| {
                    let p = p.to_ascii_lowercase();
                    p.contains("invariant") || p.contains("loop") || p.contains("recursion")
                });
            if all_postconditions && mentions_invariant_hint {
                issues.push(LintIssue {
                    instance_id: id.as_str().to_string(),
                    severity: LintSeverity::Warning,
                    code: "AUTHORING_NO_INVARIANT_STRUCTURE",
                    message: "informal_statement hints at loops or invariants but the \
                              gold obligation set contains only postconditions"
                        .to_string(),
                    path: Some(obls_path.clone()),
                });
            }
        }
    }
}

fn load_json_array(path: &Path, key: &str) -> Option<Vec<Value>> {
    let raw = std::fs::read_to_string(path).ok()?;
    let value: Value = serde_json::from_str(&raw).ok()?;
    value.get(key).and_then(|v| v.as_array()).cloned()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn regex_matches_bare_existential() {
        let re = Regex::new(r"∃\s*[^,]+,[^=]*=").unwrap();
        assert!(re.is_match("∀ (xs : Arr), ∃ ys : Arr, insertionSort xs = ys"));
        assert!(re.is_match("∃ y : Int, f x = y"));
        assert!(!re.is_match("∀ xs, SortedLE (sort xs)"));
    }

    #[test]
    fn regex_matches_unconditional_universal() {
        let re = Regex::new(r"^∀[^→]*$").unwrap();
        assert!(re.is_match("∀ (t : Tree) (k : Int), IsBst t"));
        assert!(!re.is_match("∀ (arr : Arr), arr ≠ [] → ∃ v, max arr = v"));
    }
}
