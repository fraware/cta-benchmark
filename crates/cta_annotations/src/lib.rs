//! `cta_annotations` — human annotation ingest and adjudication.
//!
//! Annotation storage layout lives under `benchmark/<version>/annotation/` and
//! is documented in `docs/annotation_manual.md`. This crate parses and
//! validates annotations, computes critical-unit coverage, and produces
//! adjudicated merged records downstream consumers can score against.

#![deny(missing_docs)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use cta_core::{InstanceId, RubricVersion, SystemId};
use cta_schema::{SchemaName, SchemaRegistry};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors produced by the annotations layer.
#[derive(Debug, Error)]
pub enum AnnotationError {
    /// IO failure.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON parse error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// Schema validation failure on an individual annotation file.
    #[error("annotation schema validation failed for {path}: {error}")]
    SchemaInvalid {
        /// File that failed.
        path: PathBuf,
        /// Human-readable error message.
        error: String,
    },
    /// Adjudication required but no adjudicator record was provided.
    #[error("missing adjudicator record for instance {0}, system {1}")]
    MissingAdjudicator(String, String),
    /// Could not find any annotation files in the given directory.
    #[error("no annotations found under {0}")]
    Empty(PathBuf),
    /// Core identifier parsing error.
    #[error("core id error: {0}")]
    Core(#[from] cta_core::CoreError),
    /// Schema loading error.
    #[error("schema error: {0}")]
    Schema(#[from] cta_schema::SchemaError),
}

/// Result alias.
pub type Result<T> = std::result::Result<T, AnnotationError>;

/// Faithfulness label.
///
/// Ordering: `Unfaithful < Partial < Ambiguous < Faithful`. This ordering
/// is used for ordinal statistics (weighted kappa on agreement) and mirrors
/// the numeric weights defined in [`FaithfulnessLabel::weight`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FaithfulnessLabel {
    /// The obligation misrepresents the semantics (weight `0.0`).
    Unfaithful,
    /// The annotator could not decide (weight `0.0`).
    Ambiguous,
    /// The obligation captures semantics only partially (weight `0.5`).
    Partial,
    /// The obligation faithfully captures the semantics (weight `1.0`).
    Faithful,
}

impl FaithfulnessLabel {
    /// Parse from the canonical string form.
    ///
    /// # Errors
    /// Returns `None` if the string does not match a known label.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "faithful" => Some(Self::Faithful),
            "partial" => Some(Self::Partial),
            "unfaithful" => Some(Self::Unfaithful),
            "ambiguous" => Some(Self::Ambiguous),
            _ => None,
        }
    }

    /// Canonical string form.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Faithful => "faithful",
            Self::Partial => "partial",
            Self::Unfaithful => "unfaithful",
            Self::Ambiguous => "ambiguous",
        }
    }

    /// Contribution of this label to `semantic_faithfulness_mean` under the
    /// `metrics_v2` contract.
    ///
    /// The weights are documented in `docs/evaluation_contract.md` and
    /// mirrored in `docs/annotation_manual.md`:
    ///
    /// | label       | weight |
    /// |-------------|--------|
    /// | faithful    | 1.0    |
    /// | partial     | 0.5    |
    /// | ambiguous   | 0.0    |
    /// | unfaithful  | 0.0    |
    ///
    /// `ambiguous` is treated as 0.0 (not 0.5) because the label indicates
    /// the annotator could not decide, not that the obligation is half-right.
    #[must_use]
    pub const fn weight(self) -> f64 {
        match self {
            Self::Faithful => 1.0,
            Self::Partial => 0.5,
            Self::Ambiguous | Self::Unfaithful => 0.0,
        }
    }

    /// Ordinal rank in `[0, 4)`, matching the weighted-kappa ordering.
    #[must_use]
    pub const fn ord(self) -> u8 {
        match self {
            Self::Unfaithful => 0,
            Self::Ambiguous => 1,
            Self::Partial => 2,
            Self::Faithful => 3,
        }
    }
}

/// Consistency label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsistencyLabel {
    /// The obligation is consistent with the reference Rust implementation.
    Consistent,
    /// The obligation contradicts the reference.
    Inconsistent,
    /// The obligation is structural and does not apply.
    NotApplicable,
}

impl ConsistencyLabel {
    /// Parse from the canonical string form.
    ///
    /// # Errors
    /// Returns `None` if the string does not match a known label.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "consistent" => Some(Self::Consistent),
            "inconsistent" => Some(Self::Inconsistent),
            "not_applicable" => Some(Self::NotApplicable),
            _ => None,
        }
    }

    /// Canonical string form.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Consistent => "consistent",
            Self::Inconsistent => "inconsistent",
            Self::NotApplicable => "not_applicable",
        }
    }
}

/// Deserialized single annotation file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    /// Schema version constant.
    pub schema_version: String,
    /// Rubric version this annotation was produced under.
    pub rubric_version: RubricVersion,
    /// Instance id.
    pub instance_id: InstanceId,
    /// System id.
    pub system_id: SystemId,
    /// Annotator id (e.g. `ann_01`, `adjudicator`).
    pub annotator_id: String,
    /// Set-level scalar scores.
    pub set_level_scores: SetLevelScores,
    /// Critical unit coverage (covered + missed).
    pub critical_unit_coverage: CriticalUnitCoverage,
    /// Per-obligation labels.
    pub generated_obligations: Vec<AnnotatedObligation>,
    /// Free-form annotator notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotator_notes: Option<String>,
}

/// Set-level scalar scores in [0, 1].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetLevelScores {
    /// Mean semantic faithfulness.
    pub semantic_faithfulness: f64,
    /// Fraction consistent with the reference Rust implementation.
    pub code_consistency: f64,
    /// Fraction of vacuous obligations.
    pub vacuity_rate: f64,
    /// Proof utility of the set.
    pub proof_utility: f64,
}

/// Critical-unit coverage record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalUnitCoverage {
    /// SUs covered by the generated set.
    pub covered: Vec<String>,
    /// SUs missed by the generated set.
    pub missed: Vec<String>,
}

/// Per-obligation annotator labels.
///
/// Labels are stored as typed enums internally; the on-wire JSON form is
/// still the canonical lowercase string (enforced by `annotation.schema.json`
/// and by the `#[serde(rename_all = ...)]` attributes on the enums). This
/// eliminates the risk of silent drift between the annotation rubric and
/// the metrics implementation: string comparisons on label fields are
/// impossible from Rust.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedObligation {
    /// Index into the generated obligation list.
    pub obligation_index: u32,
    /// Faithfulness label (typed).
    pub faithfulness_label: FaithfulnessLabel,
    /// Consistency label (typed).
    pub consistency_label: ConsistencyLabel,
    /// Vacuity flag.
    pub is_vacuous: bool,
    /// Linked SUs.
    #[serde(default)]
    pub linked_semantic_units: Vec<String>,
    /// Free-form notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Bundle of annotations produced by several annotators for a single
/// (instance, system) pair.
#[derive(Debug, Clone)]
pub struct AnnotatorGroup {
    /// Non-adjudicator annotators.
    pub annotators: Vec<Annotation>,
    /// Adjudicator record, if any.
    pub adjudicator: Option<Annotation>,
}

impl AnnotatorGroup {
    /// Total number of annotation records in the group.
    #[must_use]
    pub fn len(&self) -> usize {
        self.annotators.len() + usize::from(self.adjudicator.is_some())
    }

    /// Whether the group has no annotations at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Result of loading a directory of annotations.
#[derive(Debug, Clone)]
pub struct AnnotationSet {
    /// Annotations keyed by `(instance_id, system_id)`.
    pub groups: BTreeMap<(String, String), AnnotatorGroup>,
}

impl AnnotationSet {
    /// Return all unique `(instance_id, system_id)` pairs in deterministic order.
    #[must_use]
    pub fn keys(&self) -> Vec<(String, String)> {
        self.groups.keys().cloned().collect()
    }

    /// Total number of annotation records ingested.
    #[must_use]
    pub fn total_records(&self) -> usize {
        self.groups.values().map(AnnotatorGroup::len).sum()
    }
}

/// Walk a directory recursively and load every `*.json` as an `Annotation`.
///
/// Files are validated against `annotation.schema.json` via the supplied
/// [`SchemaRegistry`]. Sub-directory structure is ignored; any nesting is
/// acceptable so long as each JSON file parses as an `Annotation`.
///
/// # Errors
/// Returns an error if:
/// - the directory does not exist, is empty, or contains no `*.json` files;
/// - any file fails schema validation or JSON deserialization.
pub fn load_dir(root: &Path, registry: &SchemaRegistry) -> Result<AnnotationSet> {
    let mut files: Vec<PathBuf> = Vec::new();
    collect_json_files(root, &mut files)?;
    if files.is_empty() {
        return Err(AnnotationError::Empty(root.to_path_buf()));
    }
    files.sort();

    let mut groups: BTreeMap<(String, String), AnnotatorGroup> = BTreeMap::new();
    for path in files {
        let bytes = std::fs::read(&path)?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)?;
        registry
            .validate(SchemaName::Annotation, &value)
            .map_err(|e| AnnotationError::SchemaInvalid {
                path: path.clone(),
                error: format!("{e}"),
            })?;
        let annotation: Annotation = serde_json::from_value(value)?;
        let key = (
            annotation.instance_id.as_str().to_string(),
            annotation.system_id.as_str().to_string(),
        );
        let entry = groups.entry(key).or_insert_with(|| AnnotatorGroup {
            annotators: Vec::new(),
            adjudicator: None,
        });
        if annotation.annotator_id == "adjudicator" {
            entry.adjudicator = Some(annotation);
        } else {
            entry.annotators.push(annotation);
        }
    }
    Ok(AnnotationSet { groups })
}

fn collect_json_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Err(AnnotationError::Empty(dir.to_path_buf()));
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let path = entry.path();
        if ty.is_dir() {
            collect_json_files(&path, out)?;
        } else if ty.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
            // Skip `pack.json` files by convention: they are
            // `AnnotationPack` artifacts (derived from adjudicated
            // annotations), not raw annotations.  Loading them through
            // `load_dir` would fail the annotation schema because the
            // pack has a `records` envelope instead of the
            // per-annotation fields.
            if path.file_name().and_then(|n| n.to_str()) == Some("pack.json") {
                continue;
            }
            out.push(path);
        }
    }
    Ok(())
}

/// Policy used when picking a canonical record out of a multi-annotator group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjudicationPolicy {
    /// Prefer the adjudicator record if present, otherwise fall back to the
    /// majority vote across annotators (and the average of set-level scalars).
    PreferAdjudicator,
    /// Ignore the adjudicator record and always reduce via majority vote.
    AlwaysMajority,
}

/// Output of [`adjudicate_group`]: one canonical record per `(instance, system)`.
#[derive(Debug, Clone)]
pub struct AdjudicatedRecord {
    /// Adjudicated annotation (synthesised if no explicit adjudicator).
    pub annotation: Annotation,
    /// Whether an explicit adjudicator record was used (true) or the record
    /// was synthesised from majority voting (false).
    pub from_adjudicator: bool,
    /// Per-obligation disagreement count across annotators (informative).
    pub per_obligation_disagreements: Vec<u32>,
}

/// Adjudicate a group of annotations for a single `(instance, system)` pair.
///
/// # Errors
/// Returns an error if the group is empty or (with `PreferAdjudicator` and
/// more than one annotator) no adjudicator record is supplied.
pub fn adjudicate_group(
    group: &AnnotatorGroup,
    policy: AdjudicationPolicy,
) -> Result<AdjudicatedRecord> {
    if group.is_empty() {
        return Err(AnnotationError::Empty(PathBuf::new()));
    }

    let per_obligation_disagreements = count_disagreements(group);

    if matches!(policy, AdjudicationPolicy::PreferAdjudicator) {
        if let Some(a) = group.adjudicator.clone() {
            return Ok(AdjudicatedRecord {
                annotation: a,
                from_adjudicator: true,
                per_obligation_disagreements,
            });
        }
        if group.annotators.len() > 1 {
            let iid = group
                .annotators
                .first()
                .map_or_else(String::new, |a| a.instance_id.as_str().to_string());
            let sid = group
                .annotators
                .first()
                .map_or_else(String::new, |a| a.system_id.as_str().to_string());
            return Err(AnnotationError::MissingAdjudicator(iid, sid));
        }
    }

    if group.annotators.len() == 1 {
        return Ok(AdjudicatedRecord {
            annotation: group.annotators[0].clone(),
            from_adjudicator: false,
            per_obligation_disagreements,
        });
    }

    let synthesised = synthesize_majority(&group.annotators)?;
    Ok(AdjudicatedRecord {
        annotation: synthesised,
        from_adjudicator: false,
        per_obligation_disagreements,
    })
}

fn synthesize_majority(annotators: &[Annotation]) -> Result<Annotation> {
    debug_assert!(annotators.len() >= 2);
    let first = annotators.first().ok_or_else(|| {
        AnnotationError::MissingAdjudicator("(empty group)".into(), "(empty group)".into())
    })?;

    let mean_scores = SetLevelScores {
        semantic_faithfulness: mean(
            annotators
                .iter()
                .map(|a| a.set_level_scores.semantic_faithfulness),
        ),
        code_consistency: mean(
            annotators
                .iter()
                .map(|a| a.set_level_scores.code_consistency),
        ),
        vacuity_rate: mean(annotators.iter().map(|a| a.set_level_scores.vacuity_rate)),
        proof_utility: mean(annotators.iter().map(|a| a.set_level_scores.proof_utility)),
    };

    let mut covered: Vec<String> = annotators
        .iter()
        .flat_map(|a| a.critical_unit_coverage.covered.iter().cloned())
        .collect();
    covered.sort();
    covered.dedup();
    let mut missed: Vec<String> = annotators
        .iter()
        .flat_map(|a| a.critical_unit_coverage.missed.iter().cloned())
        .collect();
    missed.sort();
    missed.dedup();
    missed.retain(|m| !covered.contains(m));

    let n = annotators
        .iter()
        .map(|a| a.generated_obligations.len())
        .max()
        .unwrap_or(0);

    let mut merged_obligations: Vec<AnnotatedObligation> = Vec::with_capacity(n);
    for idx in 0..n {
        let at: Vec<&AnnotatedObligation> = annotators
            .iter()
            .filter_map(|a| a.generated_obligations.get(idx))
            .collect();
        if at.is_empty() {
            continue;
        }
        let faith = mode_faith(at.iter().map(|o| o.faithfulness_label));
        let cons = mode_cons(at.iter().map(|o| o.consistency_label));
        let vac_votes = at.iter().filter(|o| o.is_vacuous).count();
        let is_vacuous = vac_votes * 2 > at.len();
        let mut linked: Vec<String> = at
            .iter()
            .flat_map(|o| o.linked_semantic_units.iter().cloned())
            .collect();
        linked.sort();
        linked.dedup();
        merged_obligations.push(AnnotatedObligation {
            obligation_index: u32::try_from(idx).unwrap_or(u32::MAX),
            faithfulness_label: faith,
            consistency_label: cons,
            is_vacuous,
            linked_semantic_units: linked,
            notes: None,
        });
    }

    Ok(Annotation {
        schema_version: first.schema_version.clone(),
        rubric_version: first.rubric_version.clone(),
        instance_id: first.instance_id.clone(),
        system_id: first.system_id.clone(),
        annotator_id: "adjudicator".to_string(),
        set_level_scores: mean_scores,
        critical_unit_coverage: CriticalUnitCoverage { covered, missed },
        generated_obligations: merged_obligations,
        annotator_notes: Some(format!(
            "synthesised via majority vote from {} annotators",
            annotators.len()
        )),
    })
}

fn mean<I: Iterator<Item = f64>>(iter: I) -> f64 {
    let v: Vec<f64> = iter.collect();
    if v.is_empty() {
        return 0.0;
    }
    let len = v.len() as f64;
    v.into_iter().sum::<f64>() / len
}

fn mode_faith<I: Iterator<Item = FaithfulnessLabel>>(iter: I) -> FaithfulnessLabel {
    let mut counts: BTreeMap<FaithfulnessLabel, u32> = BTreeMap::new();
    for s in iter {
        *counts.entry(s).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)))
        .map(|(k, _)| k)
        .unwrap_or(FaithfulnessLabel::Ambiguous)
}

fn mode_cons<I: Iterator<Item = ConsistencyLabel>>(iter: I) -> ConsistencyLabel {
    let mut counts: BTreeMap<ConsistencyLabel, u32> = BTreeMap::new();
    for s in iter {
        *counts.entry(s).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)))
        .map(|(k, _)| k)
        .unwrap_or(ConsistencyLabel::NotApplicable)
}

fn count_disagreements(group: &AnnotatorGroup) -> Vec<u32> {
    let annotators = &group.annotators;
    if annotators.len() < 2 {
        return Vec::new();
    }
    let n = annotators
        .iter()
        .map(|a| a.generated_obligations.len())
        .max()
        .unwrap_or(0);
    let mut out = Vec::with_capacity(n);
    for idx in 0..n {
        let labels: Vec<FaithfulnessLabel> = annotators
            .iter()
            .filter_map(|a| a.generated_obligations.get(idx))
            .map(|o| o.faithfulness_label)
            .collect();
        if labels.is_empty() {
            out.push(0);
            continue;
        }
        let mut unique: Vec<FaithfulnessLabel> = labels.clone();
        unique.sort();
        unique.dedup();
        out.push(u32::try_from(unique.len().saturating_sub(1)).unwrap_or(0));
    }
    out
}

/// Adjudicate every group in the set, producing one canonical record per
/// `(instance_id, system_id)` pair.
///
/// # Errors
/// Returns the first adjudication error encountered.
pub fn adjudicate_set(
    set: &AnnotationSet,
    policy: AdjudicationPolicy,
) -> Result<BTreeMap<(String, String), AdjudicatedRecord>> {
    let mut out = BTreeMap::new();
    for (k, g) in &set.groups {
        out.insert(k.clone(), adjudicate_group(g, policy)?);
    }
    Ok(out)
}

/// Serialized annotation pack — the canonical adjudicated artifact consumed
/// by metrics and reports downstream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationPack {
    /// Schema version constant.
    pub schema_version: String,
    /// Rubric version the records share.
    pub rubric_version: String,
    /// Adjudicated annotations, one per `(instance_id, system_id)`.
    pub records: Vec<Annotation>,
}

impl AnnotationPack {
    /// Build a pack from a [`BTreeMap`] of adjudicated records.
    ///
    /// # Errors
    /// Returns an error if the input is empty (we cannot infer the rubric version).
    pub fn from_adjudicated(
        records: &BTreeMap<(String, String), AdjudicatedRecord>,
    ) -> Result<Self> {
        let rubric_version = records
            .values()
            .next()
            .map(|r| r.annotation.rubric_version.as_str().to_string())
            .ok_or_else(|| AnnotationError::Empty(PathBuf::new()))?;
        let mut out: Vec<Annotation> = records.values().map(|r| r.annotation.clone()).collect();
        out.sort_by(|a, b| {
            a.instance_id
                .as_str()
                .cmp(b.instance_id.as_str())
                .then_with(|| a.system_id.as_str().cmp(b.system_id.as_str()))
        });
        Ok(Self {
            schema_version: "schema_v1".to_string(),
            rubric_version,
            records: out,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    fn demo(annotator_id: &str, faith_label: &str) -> Annotation {
        Annotation {
            schema_version: "schema_v1".into(),
            rubric_version: RubricVersion::new("rubric_v1").unwrap(),
            instance_id: InstanceId::new("arrays_binary_search_001").unwrap(),
            system_id: SystemId::new("text_only_v1").unwrap(),
            annotator_id: annotator_id.into(),
            set_level_scores: SetLevelScores {
                semantic_faithfulness: 0.8,
                code_consistency: 0.9,
                vacuity_rate: 0.1,
                proof_utility: 0.5,
            },
            critical_unit_coverage: CriticalUnitCoverage {
                covered: vec!["SU1".into()],
                missed: vec!["SU2".into()],
            },
            generated_obligations: vec![AnnotatedObligation {
                obligation_index: 0,
                faithfulness_label: FaithfulnessLabel::parse(faith_label).unwrap(),
                consistency_label: ConsistencyLabel::Consistent,
                is_vacuous: false,
                linked_semantic_units: vec!["SU1".into()],
                notes: None,
            }],
            annotator_notes: None,
        }
    }

    #[test]
    fn prefer_adjudicator_policy_returns_adjudicator() {
        let group = AnnotatorGroup {
            annotators: vec![demo("ann_01", "partial")],
            adjudicator: Some(demo("adjudicator", "faithful")),
        };
        let r = adjudicate_group(&group, AdjudicationPolicy::PreferAdjudicator).unwrap();
        assert!(r.from_adjudicator);
        assert_eq!(
            r.annotation.generated_obligations[0].faithfulness_label,
            FaithfulnessLabel::Faithful
        );
    }

    #[test]
    fn majority_merges_two_annotators() {
        let group = AnnotatorGroup {
            annotators: vec![
                demo("ann_01", "faithful"),
                demo("ann_02", "faithful"),
                demo("ann_03", "partial"),
            ],
            adjudicator: None,
        };
        let r = adjudicate_group(&group, AdjudicationPolicy::AlwaysMajority).unwrap();
        assert!(!r.from_adjudicator);
        assert_eq!(
            r.annotation.generated_obligations[0].faithfulness_label,
            FaithfulnessLabel::Faithful
        );
    }

    #[test]
    fn prefer_adjudicator_errors_without_adjudicator_when_multi() {
        let group = AnnotatorGroup {
            annotators: vec![demo("ann_01", "faithful"), demo("ann_02", "unfaithful")],
            adjudicator: None,
        };
        let e = adjudicate_group(&group, AdjudicationPolicy::PreferAdjudicator).unwrap_err();
        assert!(matches!(e, AnnotationError::MissingAdjudicator(..)));
    }
}
