//! Exercises the multi-annotator adjudication + agreement pathway end-to-end.
//!
//! Uses the checked-in `benchmark/v0.1/annotation/multi_annotator_fixture`
//! directory, which has two non-adjudicator annotators plus an explicit
//! adjudicator for the same (instance, system) pair.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use cta_annotations::{adjudicate_set, load_dir, AdjudicationPolicy, AnnotationPack};
use cta_metrics::{
    agreement, compute_results_bundle_with_agreement, InstanceInputs, InstanceSignal,
};
use cta_schema::{SchemaName, SchemaRegistry};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn fixture_dir(ws: &std::path::Path) -> PathBuf {
    ws.join("benchmark")
        .join("v0.1")
        .join("annotation")
        .join("multi_annotator_fixture")
}

#[test]
fn prefer_adjudicator_policy_selects_adjudicator_record() {
    let ws = workspace_root();
    let registry = SchemaRegistry::load(ws.join("schemas")).expect("schemas");
    let set = load_dir(&fixture_dir(&ws), &registry).expect("load");
    assert_eq!(set.total_records(), 3);
    let adjudicated =
        adjudicate_set(&set, AdjudicationPolicy::PreferAdjudicator).expect("adjudicate");
    assert_eq!(adjudicated.len(), 1);
    let record = adjudicated.values().next().expect("one record");
    assert!(record.from_adjudicator);
    // Adjudicator said SU1+SU2 covered, SU3 missed.
    assert_eq!(
        record.annotation.critical_unit_coverage.covered,
        vec!["SU1".to_string(), "SU2".to_string()]
    );
}

#[test]
fn majority_policy_synthesises_from_non_adjudicator_annotators() {
    let ws = workspace_root();
    let registry = SchemaRegistry::load(ws.join("schemas")).expect("schemas");
    let set = load_dir(&fixture_dir(&ws), &registry).expect("load");
    let adjudicated = adjudicate_set(&set, AdjudicationPolicy::AlwaysMajority).expect("adjudicate");
    let record = adjudicated.values().next().expect("one record");
    assert!(
        !record.from_adjudicator,
        "majority policy must ignore the adjudicator record"
    );
    // Disagreement on obligation_index 1: ann_01 says `faithful`,
    // ann_02 says `partial` -> tie broken by first-seen in majority
    // synthesis, which is deterministic.
    assert_eq!(record.per_obligation_disagreements.len(), 3);
}

#[test]
fn agreement_metrics_flow_through_results_bundle() {
    let ws = workspace_root();
    let registry = SchemaRegistry::load(ws.join("schemas")).expect("schemas");
    let set = load_dir(&fixture_dir(&ws), &registry).expect("load");
    let agreement_block = agreement::from_annotation_set(&set).expect("agreement");
    assert!(
        agreement_block.weighted_kappa_faithfulness >= -1.0
            && agreement_block.weighted_kappa_faithfulness <= 1.0,
        "faith kappa out of range: {}",
        agreement_block.weighted_kappa_faithfulness
    );
    assert!(
        (0.0..=1.0).contains(&agreement_block.raw_agreement_coverage),
        "coverage out of range: {}",
        agreement_block.raw_agreement_coverage
    );

    let adjudicated =
        adjudicate_set(&set, AdjudicationPolicy::PreferAdjudicator).expect("adjudicate");
    let pack = AnnotationPack::from_adjudicated(&adjudicated).expect("pack");

    let run_manifest = serde_json::json!({
        "schema_version": "schema_v1",
        "run_id": "run_2026_04_21_text_only_v1_dev_001",
        "repo_commit": "0000000",
        "benchmark_version": "v0.1",
        "schema_versions": {
            "instance": "schema_v1",
            "obligation": "schema_v1",
            "annotation": "schema_v1",
            "generated_output": "schema_v1",
            "results_bundle": "schema_v1",
            "metrics": "metrics_v2",
            "rubric": "rubric_v1"
        },
        "system_id": "text_only_v1",
        "provider": { "name": "stub", "model": "stub", "model_version": "v1" },
        "prompt_template_hash":
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "seed": 0,
        "generation_parameters": { "temperature": 0.0 },
        "toolchains": { "rust": "1.88.0", "lean": "unknown" },
        "created_at": "2026-04-21T00:00:00Z",
        "runner": { "hostname": "test-runner" }
    });
    let mut inputs: BTreeMap<String, InstanceInputs> = BTreeMap::new();
    inputs.insert(
        "arrays_binary_search_001".to_string(),
        InstanceInputs {
            signal: InstanceSignal {
                elaborated: true,
                proof_used: false,
                critical_units_total: 3,
            },
            lean_diagnostics_path: None,
            behavior_report_path: None,
        },
    );

    let bundle = compute_results_bundle_with_agreement(
        run_manifest,
        &pack,
        &inputs,
        Some(agreement_block.clone()),
    );
    let value = serde_json::to_value(&bundle).expect("to_value");
    registry
        .validate(SchemaName::ResultsBundle, &value)
        .expect("results_bundle validates with agreement block");

    let emitted = bundle
        .aggregate_metrics
        .secondary
        .inter_annotator_agreement
        .expect("agreement present in bundle");
    assert_eq!(emitted, agreement_block);
}
