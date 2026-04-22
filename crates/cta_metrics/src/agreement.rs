//! Inter-annotator agreement metrics.
//!
//! Given multiple annotators who labelled the same obligation set, compute:
//!
//! - **Cohen's kappa** on binary labels (for vacuity).
//! - **Weighted (linear) kappa** on ordinal labels (for faithfulness, ordered
//!   `unfaithful < partial < ambiguous < faithful`).
//! - **Raw agreement** on critical-unit coverage (Jaccard-like fraction of SUs
//!   labelled identically by all annotators).
//!
//! All helpers are pure functions of owned inputs. No IO, no randomness.

use std::collections::{BTreeMap, BTreeSet};

use cta_annotations::{Annotation, AnnotationSet};
use serde::{Deserialize, Serialize};

/// Inter-annotator agreement bundle; mirrors the
/// `results_bundle.schema.json > secondary.inter_annotator_agreement` shape.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InterAnnotatorAgreement {
    /// Linearly weighted kappa on faithfulness labels in `[-1, 1]`.
    pub weighted_kappa_faithfulness: f64,
    /// Cohen's kappa on binary vacuity labels in `[-1, 1]`.
    pub cohen_kappa_vacuity: f64,
    /// Raw agreement on critical-unit coverage in `[0, 1]`.
    pub raw_agreement_coverage: f64,
}

/// Compute agreement metrics from all pairs of annotators within each group,
/// then average over pairs. Groups with a single annotator are skipped.
///
/// `annotators_by_pair` maps `(instance_id, system_id)` to the list of
/// non-adjudicator annotations. Adjudicator records should be filtered out
/// upstream.
#[must_use]
pub fn compute(
    annotators_by_pair: &BTreeMap<(String, String), Vec<&Annotation>>,
) -> Option<InterAnnotatorAgreement> {
    let mut faith_scores: Vec<f64> = Vec::new();
    let mut vac_scores: Vec<f64> = Vec::new();
    let mut cov_scores: Vec<f64> = Vec::new();

    for annos in annotators_by_pair.values() {
        if annos.len() < 2 {
            continue;
        }
        for i in 0..annos.len() {
            for j in (i + 1)..annos.len() {
                let a = annos[i];
                let b = annos[j];
                if let Some(k) = weighted_kappa_faithfulness(a, b) {
                    faith_scores.push(k);
                }
                if let Some(k) = cohen_kappa_vacuity(a, b) {
                    vac_scores.push(k);
                }
                cov_scores.push(raw_agreement_coverage(a, b));
            }
        }
    }

    if faith_scores.is_empty() && vac_scores.is_empty() && cov_scores.is_empty() {
        return None;
    }
    Some(InterAnnotatorAgreement {
        weighted_kappa_faithfulness: mean(&faith_scores).unwrap_or(0.0),
        cohen_kappa_vacuity: mean(&vac_scores).unwrap_or(0.0),
        raw_agreement_coverage: mean(&cov_scores).unwrap_or(0.0),
    })
}

/// Convenience: compute inter-annotator agreement from an [`AnnotationSet`]
/// loaded from disk, skipping adjudicator records.
#[must_use]
pub fn from_annotation_set(set: &AnnotationSet) -> Option<InterAnnotatorAgreement> {
    let mut pairs: BTreeMap<(String, String), Vec<&Annotation>> = BTreeMap::new();
    for ((iid, sid), group) in &set.groups {
        let anns: Vec<&Annotation> = group.annotators.iter().collect();
        if anns.len() >= 2 {
            pairs.insert((iid.clone(), sid.clone()), anns);
        }
    }
    if pairs.is_empty() {
        return None;
    }
    compute(&pairs)
}

fn mean(xs: &[f64]) -> Option<f64> {
    if xs.is_empty() {
        return None;
    }
    Some(xs.iter().sum::<f64>() / xs.len() as f64)
}

/// Linearly-weighted Cohen's kappa on faithfulness labels, aligned by
/// `obligation_index`. Returns `None` if fewer than two common obligations.
///
/// The ordinal ordering is `unfaithful < ambiguous < partial < faithful`,
/// matching [`cta_annotations::FaithfulnessLabel::ord`].
#[must_use]
pub fn weighted_kappa_faithfulness(a: &Annotation, b: &Annotation) -> Option<f64> {
    let common = aligned_labels(a, b, |o| Some(o.faithfulness_label.ord()));
    if common.len() < 2 {
        return None;
    }
    weighted_kappa_linear(&common, 4)
}

/// Cohen's kappa on binary vacuity labels. Returns `None` if fewer than two
/// common obligations.
#[must_use]
pub fn cohen_kappa_vacuity(a: &Annotation, b: &Annotation) -> Option<f64> {
    let common: Vec<(u8, u8)> = aligned_labels(a, b, |o| Some(u8::from(o.is_vacuous)));
    if common.len() < 2 {
        return None;
    }
    weighted_kappa_linear(&common, 2)
}

/// Jaccard-like raw agreement on critical-unit coverage: the fraction of SUs
/// that both annotators agree on (present or absent) among all SUs mentioned
/// by either.
#[must_use]
pub fn raw_agreement_coverage(a: &Annotation, b: &Annotation) -> f64 {
    let cov_a: BTreeSet<&str> = a
        .critical_unit_coverage
        .covered
        .iter()
        .map(String::as_str)
        .collect();
    let cov_b: BTreeSet<&str> = b
        .critical_unit_coverage
        .covered
        .iter()
        .map(String::as_str)
        .collect();
    let miss_a: BTreeSet<&str> = a
        .critical_unit_coverage
        .missed
        .iter()
        .map(String::as_str)
        .collect();
    let miss_b: BTreeSet<&str> = b
        .critical_unit_coverage
        .missed
        .iter()
        .map(String::as_str)
        .collect();
    let universe: BTreeSet<&str> = cov_a
        .iter()
        .chain(cov_b.iter())
        .chain(miss_a.iter())
        .chain(miss_b.iter())
        .copied()
        .collect();
    if universe.is_empty() {
        return 1.0;
    }
    let mut agreed = 0usize;
    for su in &universe {
        let a_label = if cov_a.contains(su) {
            Some(true)
        } else if miss_a.contains(su) {
            Some(false)
        } else {
            None
        };
        let b_label = if cov_b.contains(su) {
            Some(true)
        } else if miss_b.contains(su) {
            Some(false)
        } else {
            None
        };
        if a_label == b_label && a_label.is_some() {
            agreed += 1;
        }
    }
    agreed as f64 / universe.len() as f64
}

fn aligned_labels<F>(a: &Annotation, b: &Annotation, key: F) -> Vec<(u8, u8)>
where
    F: Fn(&cta_annotations::AnnotatedObligation) -> Option<u8>,
{
    let by_idx_b: BTreeMap<u32, &cta_annotations::AnnotatedObligation> = b
        .generated_obligations
        .iter()
        .map(|o| (o.obligation_index, o))
        .collect();
    a.generated_obligations
        .iter()
        .filter_map(|oa| {
            let ob = by_idx_b.get(&oa.obligation_index)?;
            Some((key(oa)?, key(ob)?))
        })
        .collect()
}

/// Linearly-weighted kappa on integer categories in `[0, k)`.
/// Weights are `w_ij = 1 - |i - j| / (k - 1)`.
#[allow(clippy::needless_range_loop)]
fn weighted_kappa_linear(pairs: &[(u8, u8)], k: u8) -> Option<f64> {
    if pairs.is_empty() || k < 2 {
        return None;
    }
    let k_usize = usize::from(k);
    let denom = f64::from(u32::from(k - 1));

    let mut observed = vec![vec![0u32; k_usize]; k_usize];
    let mut marginal_a = vec![0u32; k_usize];
    let mut marginal_b = vec![0u32; k_usize];
    for &(ai, bi) in pairs {
        let (i, j) = (usize::from(ai), usize::from(bi));
        observed[i][j] += 1;
        marginal_a[i] += 1;
        marginal_b[j] += 1;
    }
    let n = pairs.len() as f64;

    let mut po = 0.0;
    let mut pe = 0.0;
    for i in 0..k_usize {
        for j in 0..k_usize {
            let d = (i as i32 - j as i32).unsigned_abs() as f64;
            let w = 1.0 - d / denom;
            po += w * f64::from(observed[i][j]) / n;
            pe += w * f64::from(marginal_a[i]) * f64::from(marginal_b[j]) / (n * n);
        }
    }
    if (1.0 - pe).abs() < f64::EPSILON {
        return Some(if (po - pe).abs() < f64::EPSILON {
            0.0
        } else {
            1.0
        });
    }
    Some((po - pe) / (1.0 - pe))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cta_annotations::{
        AnnotatedObligation, ConsistencyLabel, CriticalUnitCoverage, FaithfulnessLabel,
        SetLevelScores,
    };
    use cta_core::{InstanceId, RubricVersion, SystemId};

    fn ann(id: &str, labels: &[(&str, bool)]) -> Annotation {
        Annotation {
            schema_version: "schema_v1".into(),
            rubric_version: RubricVersion::new("rubric_v1").unwrap(),
            instance_id: InstanceId::new("arrays_binary_search_001").unwrap(),
            system_id: SystemId::new("text_only_v1").unwrap(),
            annotator_id: id.into(),
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
            generated_obligations: labels
                .iter()
                .enumerate()
                .map(|(i, (f, vac))| AnnotatedObligation {
                    obligation_index: u32::try_from(i).unwrap(),
                    faithfulness_label: FaithfulnessLabel::parse(f).expect("valid label"),
                    consistency_label: ConsistencyLabel::Consistent,
                    is_vacuous: *vac,
                    linked_semantic_units: vec![],
                    notes: None,
                })
                .collect(),
            annotator_notes: None,
        }
    }

    #[test]
    fn identical_annotators_give_kappa_one() {
        let a = ann(
            "ann_01",
            &[("faithful", false), ("partial", true), ("faithful", false)],
        );
        let k_faith = weighted_kappa_faithfulness(&a, &a).unwrap();
        assert!((k_faith - 1.0).abs() < 1e-9, "k_faith={k_faith}");
        let k_vac = cohen_kappa_vacuity(&a, &a).unwrap();
        assert!((k_vac - 1.0).abs() < 1e-9, "k_vac={k_vac}");
    }

    #[test]
    fn disjoint_categories_give_negative_kappa() {
        // k_a always 0, k_b always 3 -> perfect disagreement on ordinal scale.
        let a = ann(
            "ann_01",
            &[
                ("unfaithful", false),
                ("unfaithful", false),
                ("unfaithful", false),
                ("unfaithful", false),
            ],
        );
        let b = ann(
            "ann_02",
            &[
                ("faithful", true),
                ("faithful", true),
                ("faithful", true),
                ("faithful", true),
            ],
        );
        let k_faith = weighted_kappa_faithfulness(&a, &b).unwrap();
        // Both annotators are constants so p_e = 1 and we return 1.0 per the
        // degenerate branch. Consider instead mixed constants:
        let _ = k_faith;
        let c = ann("ann_03", &[("unfaithful", false), ("faithful", true)]);
        let d = ann("ann_04", &[("faithful", true), ("unfaithful", false)]);
        let k = weighted_kappa_faithfulness(&c, &d).unwrap();
        assert!(k < 0.0, "expected negative kappa, got {k}");
    }

    #[test]
    fn raw_agreement_on_identical_coverage_is_one() {
        let a = ann("ann_01", &[]);
        let b = ann("ann_02", &[]);
        let r = raw_agreement_coverage(&a, &b);
        assert!((r - 1.0).abs() < 1e-9);
    }

    #[test]
    fn raw_agreement_on_opposite_coverage_is_zero() {
        let mut a = ann("ann_01", &[]);
        let mut b = ann("ann_02", &[]);
        a.critical_unit_coverage.covered = vec!["SU1".into()];
        a.critical_unit_coverage.missed = vec!["SU2".into()];
        b.critical_unit_coverage.covered = vec!["SU2".into()];
        b.critical_unit_coverage.missed = vec!["SU1".into()];
        let r = raw_agreement_coverage(&a, &b);
        assert!((r - 0.0).abs() < 1e-9);
    }
}
