//! `cta_metrics` — pure, deterministic CTA benchmark metrics.
//!
//! The implementation is kept intentionally boring: every function takes
//! fully-materialized inputs and returns a scalar in `[0, 1]` (or a ratio).
//! This means metrics are trivially testable and reproducible.
//!
//! At M6 this crate also materialises the [`InstanceResult`] and
//! [`ResultsBundle`] records consumed by `cta_reports`. All aggregation is
//! a pure function of `(annotations, generated_outputs, lean_diagnostics,
//! behavior_reports)`; no IO happens in the `compute_results_bundle`
//! entrypoint itself.

#![deny(missing_docs)]

use cta_annotations::{Annotation, AnnotationPack};
use serde::{Deserialize, Serialize};

pub mod agreement;
pub use agreement::InterAnnotatorAgreement;

/// Canonical metrics version emitted into `results_bundle.aggregate_metrics`.
pub const METRICS_VERSION: &str = "metrics_v1";

/// A single instance's per-instance tallies as consumed by the metrics layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceTally {
    /// Whether the generated Lean file elaborated.
    pub elaborated: bool,
    /// Total obligations generated.
    pub num_obligations: u32,
    /// Obligations the annotator marked `faithful`.
    pub num_faithful: u32,
    /// Obligations flagged vacuous.
    pub num_vacuous: u32,
    /// Obligations flagged as contradicting the reference implementation.
    pub num_inconsistent: u32,
    /// Critical SUs covered out of total.
    pub critical_units_covered: u32,
    /// Total critical SUs.
    pub critical_units_total: u32,
    /// Whether this instance had at least one obligation used in a proof.
    pub proof_used: bool,
}

/// Per-instance signal consumed by [`tally_from_annotation`].
#[derive(Debug, Clone, Default)]
pub struct InstanceSignal {
    /// Whether the generated Lean file elaborated cleanly.
    pub elaborated: bool,
    /// Whether any obligation was closed by a downstream proof.
    pub proof_used: bool,
    /// Total critical semantic units known for the instance (from
    /// `semantic_units.json` + `reference_obligations.json`).
    pub critical_units_total: u32,
}

/// Per-instance result row matching `results_bundle.schema.json > InstanceResult`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InstanceResult {
    /// Instance id (canonical form).
    pub instance_id: String,
    /// Whether the generated Lean elaborated cleanly.
    pub elaborated: bool,
    /// Number of generated obligations.
    pub num_obligations: u32,
    /// Obligations annotated `faithful`.
    pub num_faithful: u32,
    /// Obligations flagged vacuous.
    pub num_vacuous: u32,
    /// Obligations flagged inconsistent with the Rust reference.
    pub num_inconsistent: u32,
    /// Critical semantic units covered.
    pub critical_units_covered: u32,
    /// Total critical semantic units.
    pub critical_units_total: u32,
    /// Path (run-local) to the Lean diagnostics JSON, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lean_diagnostics_path: Option<String>,
    /// Path (run-local) to the behavior report JSON, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub behavior_report_path: Option<String>,
}

/// Aggregated primary metrics (matches `results_bundle.schema.json > primary`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrimaryMetrics {
    /// Fraction of instances whose generated Lean file elaborated.
    pub elaboration_rate: f64,
    /// Mean semantic faithfulness across instances with >0 obligations.
    pub semantic_faithfulness_mean: f64,
    /// Fraction of critical SUs covered across the benchmark.
    pub critical_unit_coverage: f64,
    /// Fraction of obligations consistent with the Rust reference.
    pub rust_consistency_rate: f64,
    /// Fraction of obligations flagged vacuous.
    pub vacuity_rate: f64,
    /// Fraction of instances where at least one obligation was used in proof.
    pub proof_utility: f64,
}

/// Secondary metrics bundle (matches `results_bundle.schema.json > secondary`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecondaryMetrics {
    /// Average obligation count per instance (`[0, +inf)`).
    pub avg_obligations_per_instance: f64,
    /// Density of `faithful` obligations over total obligations.
    pub faithful_obligation_density: f64,
    /// Contradiction rate on obligations linked to critical SUs.
    pub contradiction_rate_on_critical_units: f64,
    /// Fraction of text-faithful obligations that were code-inconsistent.
    pub text_faithful_code_inconsistent_rate: f64,
    /// Fraction of code-faithful obligations whose NL gloss was incomplete.
    /// (Approximated as 0.0 without NL semantics; kept for schema parity.)
    pub code_faithful_text_incomplete_rate: f64,
    /// Optional inter-annotator agreement; emitted only if the run had more
    /// than one annotator per `(instance, system)` pair.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inter_annotator_agreement: Option<InterAnnotatorAgreement>,
}

/// Full metrics bundle emitted into the results bundle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AggregateMetrics {
    /// Metrics version identifier.
    pub metrics_version: String,
    /// Primary metrics.
    pub primary: PrimaryMetrics,
    /// Secondary metrics.
    pub secondary: SecondaryMetrics,
}

/// Results bundle matching `results_bundle.schema.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResultsBundle {
    /// Schema version.
    pub schema_version: String,
    /// Run manifest, embedded as-is (opaque to metrics).
    pub run_manifest: serde_json::Value,
    /// Per-instance result rows.
    pub instance_results: Vec<InstanceResult>,
    /// Aggregate metrics.
    pub aggregate_metrics: AggregateMetrics,
}

/// Convert one adjudicated [`Annotation`] plus its external signal into an
/// [`InstanceTally`].
///
/// The critical-unit denominator is the sum of covered + missed SUs from the
/// annotation, unless `signal.critical_units_total` is strictly larger (which
/// happens when the annotator omitted SUs they did not review).
#[must_use]
pub fn tally_from_annotation(a: &Annotation, signal: &InstanceSignal) -> InstanceTally {
    let num_obligations = u32::try_from(a.generated_obligations.len()).unwrap_or(u32::MAX);
    let num_faithful = u32::try_from(
        a.generated_obligations
            .iter()
            .filter(|o| o.faithfulness_label == "faithful")
            .count(),
    )
    .unwrap_or(u32::MAX);
    let num_vacuous = u32::try_from(
        a.generated_obligations
            .iter()
            .filter(|o| o.is_vacuous)
            .count(),
    )
    .unwrap_or(u32::MAX);
    let num_inconsistent = u32::try_from(
        a.generated_obligations
            .iter()
            .filter(|o| o.consistency_label == "inconsistent")
            .count(),
    )
    .unwrap_or(u32::MAX);
    let cov = u32::try_from(a.critical_unit_coverage.covered.len()).unwrap_or(u32::MAX);
    let missed = u32::try_from(a.critical_unit_coverage.missed.len()).unwrap_or(u32::MAX);
    let annotated_total = cov.saturating_add(missed);
    let critical_units_total = signal.critical_units_total.max(annotated_total);
    InstanceTally {
        elaborated: signal.elaborated,
        num_obligations,
        num_faithful,
        num_vacuous,
        num_inconsistent,
        critical_units_covered: cov,
        critical_units_total,
        proof_used: signal.proof_used,
    }
}

/// Convert an adjudicated [`Annotation`] + signal into an [`InstanceResult`].
#[must_use]
pub fn instance_result_from_annotation(
    a: &Annotation,
    signal: &InstanceSignal,
    lean_diagnostics_path: Option<String>,
    behavior_report_path: Option<String>,
) -> InstanceResult {
    let t = tally_from_annotation(a, signal);
    InstanceResult {
        instance_id: a.instance_id.as_str().to_string(),
        elaborated: t.elaborated,
        num_obligations: t.num_obligations,
        num_faithful: t.num_faithful,
        num_vacuous: t.num_vacuous,
        num_inconsistent: t.num_inconsistent,
        critical_units_covered: t.critical_units_covered,
        critical_units_total: t.critical_units_total,
        lean_diagnostics_path,
        behavior_report_path,
    }
}

/// Compute primary metrics from per-instance tallies.
///
/// Metrics are undefined on zero-sized denominators; we return `0.0` for each
/// such case. The spec's acceptance criterion forbids promoting a run with
/// empty instance tallies anyway.
#[must_use]
pub fn primary_metrics(tallies: &[InstanceTally]) -> PrimaryMetrics {
    let n = tallies.len() as f64;
    if n == 0.0 {
        return PrimaryMetrics {
            elaboration_rate: 0.0,
            semantic_faithfulness_mean: 0.0,
            critical_unit_coverage: 0.0,
            rust_consistency_rate: 0.0,
            vacuity_rate: 0.0,
            proof_utility: 0.0,
        };
    }

    let elaboration_rate = tallies.iter().filter(|t| t.elaborated).count() as f64 / n;
    let proof_utility = tallies.iter().filter(|t| t.proof_used).count() as f64 / n;

    let (total_obligations, total_vacuous, total_inconsistent) =
        tallies.iter().fold((0u64, 0u64, 0u64), |acc, t| {
            (
                acc.0 + u64::from(t.num_obligations),
                acc.1 + u64::from(t.num_vacuous),
                acc.2 + u64::from(t.num_inconsistent),
            )
        });

    let rust_consistency_rate = if total_obligations == 0 {
        0.0
    } else {
        1.0 - (total_inconsistent as f64 / total_obligations as f64)
    };
    let vacuity_rate = if total_obligations == 0 {
        0.0
    } else {
        total_vacuous as f64 / total_obligations as f64
    };

    let instances_with_output: Vec<&InstanceTally> =
        tallies.iter().filter(|t| t.num_obligations > 0).collect();
    let semantic_faithfulness_mean = if instances_with_output.is_empty() {
        0.0
    } else {
        instances_with_output
            .iter()
            .map(|t| f64::from(t.num_faithful) / f64::from(t.num_obligations))
            .sum::<f64>()
            / instances_with_output.len() as f64
    };

    let (cov_num, cov_den) = tallies.iter().fold((0u64, 0u64), |acc, t| {
        (
            acc.0 + u64::from(t.critical_units_covered),
            acc.1 + u64::from(t.critical_units_total),
        )
    });
    let critical_unit_coverage = if cov_den == 0 {
        0.0
    } else {
        cov_num as f64 / cov_den as f64
    };

    PrimaryMetrics {
        elaboration_rate,
        semantic_faithfulness_mean,
        critical_unit_coverage,
        rust_consistency_rate,
        vacuity_rate,
        proof_utility,
    }
}

/// Compute secondary metrics from per-instance tallies + annotations.
///
/// The annotations are used to compute contradiction-on-critical-units and
/// text-faithful/code-inconsistent cross-rates, which require per-obligation
/// access. `text_faithful_code_inconsistent_rate` counts obligations where
/// `faithfulness_label == "faithful"` but `consistency_label == "inconsistent"`.
#[must_use]
pub fn secondary_metrics(
    tallies: &[InstanceTally],
    annotations: &[Annotation],
) -> SecondaryMetrics {
    let n = tallies.len() as f64;
    let total_obligations: u64 = tallies.iter().map(|t| u64::from(t.num_obligations)).sum();
    let total_faithful: u64 = tallies.iter().map(|t| u64::from(t.num_faithful)).sum();

    let avg_obligations_per_instance = if n == 0.0 {
        0.0
    } else {
        total_obligations as f64 / n
    };

    let faithful_obligation_density = if total_obligations == 0 {
        0.0
    } else {
        total_faithful as f64 / total_obligations as f64
    };

    let mut critical_obligations = 0u64;
    let mut critical_inconsistent = 0u64;
    let mut faithful_but_inconsistent = 0u64;
    let mut total_faithful_obl = 0u64;
    for a in annotations {
        for o in &a.generated_obligations {
            let is_critical = !o.linked_semantic_units.is_empty();
            if is_critical {
                critical_obligations += 1;
                if o.consistency_label == "inconsistent" {
                    critical_inconsistent += 1;
                }
            }
            if o.faithfulness_label == "faithful" {
                total_faithful_obl += 1;
                if o.consistency_label == "inconsistent" {
                    faithful_but_inconsistent += 1;
                }
            }
        }
    }
    let contradiction_rate_on_critical_units = if critical_obligations == 0 {
        0.0
    } else {
        critical_inconsistent as f64 / critical_obligations as f64
    };
    let text_faithful_code_inconsistent_rate = if total_faithful_obl == 0 {
        0.0
    } else {
        faithful_but_inconsistent as f64 / total_faithful_obl as f64
    };

    SecondaryMetrics {
        avg_obligations_per_instance,
        faithful_obligation_density,
        contradiction_rate_on_critical_units,
        text_faithful_code_inconsistent_rate,
        code_faithful_text_incomplete_rate: 0.0,
        inter_annotator_agreement: None,
    }
}

/// Input signal for a single instance, keyed by instance_id, used by
/// [`compute_results_bundle`].
#[derive(Debug, Clone, Default)]
pub struct InstanceInputs {
    /// External signal (elaboration, proof usage, total critical units).
    pub signal: InstanceSignal,
    /// Optional path to Lean diagnostics, embedded as-is in the result row.
    pub lean_diagnostics_path: Option<String>,
    /// Optional path to behavior report, embedded as-is in the result row.
    pub behavior_report_path: Option<String>,
}

/// Compute a full [`ResultsBundle`] from a run manifest, an annotation pack,
/// and per-instance signals.
///
/// Instances absent from `inputs` use [`InstanceSignal::default`].
///
/// # Panics
/// This function does not panic on empty input; it just produces an empty
/// bundle with zero metrics. Validating the produced bundle against
/// `results_bundle.schema.json` is the caller's responsibility.
#[must_use]
pub fn compute_results_bundle(
    run_manifest: serde_json::Value,
    pack: &AnnotationPack,
    inputs: &std::collections::BTreeMap<String, InstanceInputs>,
) -> ResultsBundle {
    compute_results_bundle_with_agreement(run_manifest, pack, inputs, None)
}

/// Extended [`compute_results_bundle`] accepting a pre-computed inter-annotator
/// agreement block. Used when the caller has access to the raw annotator set
/// (i.e., prior to adjudication).
#[must_use]
pub fn compute_results_bundle_with_agreement(
    run_manifest: serde_json::Value,
    pack: &AnnotationPack,
    inputs: &std::collections::BTreeMap<String, InstanceInputs>,
    agreement: Option<InterAnnotatorAgreement>,
) -> ResultsBundle {
    let mut tallies: Vec<InstanceTally> = Vec::with_capacity(pack.records.len());
    let mut rows: Vec<InstanceResult> = Vec::with_capacity(pack.records.len());
    let default_inputs = InstanceInputs::default();
    for a in &pack.records {
        let key = a.instance_id.as_str();
        let inp = inputs.get(key).unwrap_or(&default_inputs);
        tallies.push(tally_from_annotation(a, &inp.signal));
        rows.push(instance_result_from_annotation(
            a,
            &inp.signal,
            inp.lean_diagnostics_path.clone(),
            inp.behavior_report_path.clone(),
        ));
    }

    let primary = primary_metrics(&tallies);
    let mut secondary = secondary_metrics(&tallies, &pack.records);
    secondary.inter_annotator_agreement = agreement;

    tracing::info!(
        metrics_version = METRICS_VERSION,
        instances = tallies.len(),
        elaboration_rate = primary.elaboration_rate,
        faithfulness_mean = primary.semantic_faithfulness_mean,
        critical_unit_coverage = primary.critical_unit_coverage,
        "computed results bundle"
    );

    ResultsBundle {
        schema_version: "schema_v1".to_string(),
        run_manifest,
        instance_results: rows,
        aggregate_metrics: AggregateMetrics {
            metrics_version: METRICS_VERSION.to_string(),
            primary,
            secondary,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cta_annotations::{AnnotatedObligation, Annotation, CriticalUnitCoverage, SetLevelScores};
    use cta_core::{InstanceId, RubricVersion, SystemId};

    #[allow(clippy::too_many_arguments)]
    fn t(
        elaborated: bool,
        num: u32,
        faithful: u32,
        vacuous: u32,
        inconsistent: u32,
        cov: u32,
        tot: u32,
        proof: bool,
    ) -> InstanceTally {
        InstanceTally {
            elaborated,
            num_obligations: num,
            num_faithful: faithful,
            num_vacuous: vacuous,
            num_inconsistent: inconsistent,
            critical_units_covered: cov,
            critical_units_total: tot,
            proof_used: proof,
        }
    }

    fn ann(inst: &str, entries: &[(&str, &str, bool, &[&str])]) -> Annotation {
        Annotation {
            schema_version: "schema_v1".into(),
            rubric_version: RubricVersion::new("rubric_v1").unwrap(),
            instance_id: InstanceId::new(inst).unwrap(),
            system_id: SystemId::new("text_only_v1").unwrap(),
            annotator_id: "adjudicator".into(),
            set_level_scores: SetLevelScores {
                semantic_faithfulness: 0.0,
                code_consistency: 0.0,
                vacuity_rate: 0.0,
                proof_utility: 0.0,
            },
            critical_unit_coverage: CriticalUnitCoverage {
                covered: vec!["SU1".into()],
                missed: vec!["SU2".into()],
            },
            generated_obligations: entries
                .iter()
                .enumerate()
                .map(|(i, (f, c, v, sus))| AnnotatedObligation {
                    obligation_index: u32::try_from(i).unwrap(),
                    faithfulness_label: (*f).to_string(),
                    consistency_label: (*c).to_string(),
                    is_vacuous: *v,
                    linked_semantic_units: sus.iter().map(|s| (*s).to_string()).collect(),
                    notes: None,
                })
                .collect(),
            annotator_notes: None,
        }
    }

    #[test]
    fn empty_input_returns_zeros() {
        let m = primary_metrics(&[]);
        assert_eq!(m.elaboration_rate, 0.0);
    }

    #[test]
    fn elaboration_rate_matches() {
        let tallies = vec![
            t(true, 1, 1, 0, 0, 1, 1, true),
            t(false, 0, 0, 0, 0, 0, 1, false),
        ];
        let m = primary_metrics(&tallies);
        assert!((m.elaboration_rate - 0.5).abs() < 1e-9);
    }

    #[test]
    fn vacuity_and_consistency_compute() {
        let tallies = vec![
            t(true, 10, 8, 2, 1, 3, 5, true),
            t(true, 5, 4, 0, 0, 2, 5, false),
        ];
        let m = primary_metrics(&tallies);
        assert!((m.vacuity_rate - 2.0 / 15.0).abs() < 1e-9);
        assert!((m.rust_consistency_rate - 14.0 / 15.0).abs() < 1e-9);
        assert!((m.critical_unit_coverage - 5.0 / 10.0).abs() < 1e-9);
    }

    #[test]
    fn tally_counts_labels_correctly() {
        let a = ann(
            "arrays_binary_search_001",
            &[
                ("faithful", "consistent", false, &["SU1"]),
                ("partial", "inconsistent", false, &["SU2"]),
                ("faithful", "consistent", true, &[]),
            ],
        );
        let sig = InstanceSignal {
            elaborated: true,
            proof_used: false,
            critical_units_total: 4,
        };
        let tally = tally_from_annotation(&a, &sig);
        assert_eq!(tally.num_obligations, 3);
        assert_eq!(tally.num_faithful, 2);
        assert_eq!(tally.num_vacuous, 1);
        assert_eq!(tally.num_inconsistent, 1);
        assert_eq!(tally.critical_units_covered, 1);
        assert_eq!(tally.critical_units_total, 4);
        assert!(tally.elaborated);
    }

    #[test]
    fn secondary_contradiction_on_critical_units() {
        let a = ann(
            "arrays_binary_search_001",
            &[
                ("faithful", "consistent", false, &["SU1"]),
                ("faithful", "inconsistent", false, &["SU2"]),
                ("unfaithful", "inconsistent", false, &[]),
            ],
        );
        let sig = InstanceSignal::default();
        let tally = tally_from_annotation(&a, &sig);
        let s = secondary_metrics(&[tally], &[a]);
        assert!((s.contradiction_rate_on_critical_units - 0.5).abs() < 1e-9);
        assert!((s.text_faithful_code_inconsistent_rate - 0.5).abs() < 1e-9);
        assert!((s.avg_obligations_per_instance - 3.0).abs() < 1e-9);
    }
}
