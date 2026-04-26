//! End-to-end M6 pipeline test: annotations -> adjudicated pack -> results bundle.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use cta_annotations::{adjudicate_set, load_dir, AdjudicationPolicy, AnnotationPack};
use cta_metrics::{compute_results_bundle, InstanceInputs, InstanceSignal};
use cta_schema::{SchemaName, SchemaRegistry};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn end_to_end_pipeline_produces_schema_valid_bundle() {
    let workspace = workspace_root();
    let schemas = workspace.join("schemas");
    let registry = SchemaRegistry::load(&schemas).expect("load schemas");

    let annotations_dir = workspace
        .join("benchmark")
        .join("v0.1")
        .join("annotation")
        .join("adjudicated_subset");
    let set = load_dir(&annotations_dir, &registry).expect("load annotations");
    assert!(
        set.total_records() >= 3,
        "expected at least 3 sample annotations"
    );

    let adjudicated =
        adjudicate_set(&set, AdjudicationPolicy::PreferAdjudicator).expect("adjudicate");
    let pack = AnnotationPack::from_adjudicated(&adjudicated).expect("pack");
    assert_eq!(pack.records.len(), adjudicated.len());

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
        "provider": {
            "name": "stub",
            "model": "stub",
            "model_version": "v1"
        },
        "prompt_template_hash":
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        "seed": 0,
        "generation_parameters": {
            "temperature": 0.0
        },
        "toolchains": {
            "rust": "1.88.0",
            "lean": "unknown"
        },
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
                critical_units_total: 5,
            },
            lean_diagnostics_path: None,
            behavior_report_path: None,
        },
    );

    let bundle = compute_results_bundle(run_manifest, &pack, &inputs);
    let value = serde_json::to_value(&bundle).expect("to_value");
    registry
        .validate(SchemaName::ResultsBundle, &value)
        .expect("results_bundle validates");

    let p = &bundle.aggregate_metrics.primary;
    assert!((0.0..=1.0).contains(&p.elaboration_rate));
    assert!((0.0..=1.0).contains(&p.semantic_faithfulness_mean));
    assert!((0.0..=1.0).contains(&p.critical_unit_coverage));
    assert!((0.0..=1.0).contains(&p.rust_consistency_rate));
    assert!((0.0..=1.0).contains(&p.vacuity_rate));

    let arrays = bundle
        .instance_results
        .iter()
        .find(|r| r.instance_id == "arrays_binary_search_001")
        .expect("arrays instance present");
    assert!(arrays.elaborated);
    assert_eq!(arrays.num_obligations, 3);
    assert_eq!(arrays.critical_units_covered, 2);
    assert_eq!(arrays.critical_units_total, 5);
}
