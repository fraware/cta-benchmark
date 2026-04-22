//! Snapshot tests pinning the stable-by-design shape of every rendered
//! artifact: primary metrics CSV, per-instance CSV, Markdown summary, and
//! LaTeX table. These outputs feed paper tables directly, so any change to
//! their surface must be accompanied by a deliberate snapshot update.

use cta_metrics::{
    AggregateMetrics, InstanceResult, PrimaryMetrics, ResultsBundle, SecondaryMetrics,
};
use cta_reports::{
    instance_results_csv, primary_metrics_csv_row, render_all, results_latex, results_markdown,
    PRIMARY_METRICS_CSV_HEADER,
};

fn canonical_bundle() -> ResultsBundle {
    ResultsBundle {
        schema_version: "schema_v1".into(),
        run_manifest: serde_json::json!({
            "schema_version": "schema_v1",
            "run_id": "run_2026_04_21_full_method_v1_dev_001",
            "system_id": "full_method_v1",
            "benchmark_version": "v0.1",
        }),
        instance_results: vec![
            InstanceResult {
                instance_id: "arrays_binary_search_001".into(),
                elaborated: true,
                num_obligations: 3,
                faithfulness_score: 2.5,
                num_faithful_full: 2,
                num_partial: 1,
                num_vacuous: 0,
                num_consistent: 3,
                num_inconsistent: 0,
                num_not_applicable: 0,
                critical_units_covered: 2,
                critical_units_total: 3,
                lean_diagnostics_path: None,
                behavior_report_path: None,
            },
            InstanceResult {
                instance_id: "sorting_insertion_sort_001".into(),
                elaborated: true,
                num_obligations: 4,
                faithfulness_score: 3.0,
                num_faithful_full: 3,
                num_partial: 0,
                num_vacuous: 1,
                num_consistent: 3,
                num_inconsistent: 0,
                num_not_applicable: 1,
                critical_units_covered: 2,
                critical_units_total: 2,
                lean_diagnostics_path: None,
                behavior_report_path: None,
            },
            InstanceResult {
                instance_id: "graph_dijkstra_001".into(),
                elaborated: false,
                num_obligations: 0,
                faithfulness_score: 0.0,
                num_faithful_full: 0,
                num_partial: 0,
                num_vacuous: 0,
                num_consistent: 0,
                num_inconsistent: 0,
                num_not_applicable: 0,
                critical_units_covered: 0,
                critical_units_total: 3,
                lean_diagnostics_path: None,
                behavior_report_path: None,
            },
        ],
        aggregate_metrics: AggregateMetrics {
            metrics_version: "metrics_v2".into(),
            primary: PrimaryMetrics {
                elaboration_rate: 0.667,
                semantic_faithfulness_mean: 0.792,
                critical_unit_coverage: 0.500,
                rust_consistency_rate: 1.000,
                vacuity_rate: 0.143,
                proof_utility: 0.333,
            },
            secondary: SecondaryMetrics {
                avg_obligations_per_instance: 2.333,
                faithful_obligation_density: 0.786,
                contradiction_rate_on_critical_units: 0.0,
                text_faithful_code_inconsistent_rate: 0.0,
                code_faithful_text_incomplete_rate: 0.0,
                inter_annotator_agreement: None,
            },
        },
    }
}

#[test]
fn snapshot_primary_metrics_csv() {
    let bundle = canonical_bundle();
    let csv = format!(
        "{PRIMARY_METRICS_CSV_HEADER}{}",
        primary_metrics_csv_row("full_method_v1", &bundle.aggregate_metrics.primary)
    );
    insta::assert_snapshot!("primary_metrics_csv", csv);
}

#[test]
fn snapshot_instance_results_csv() {
    insta::assert_snapshot!(
        "instance_results_csv",
        instance_results_csv(&canonical_bundle())
    );
}

#[test]
fn snapshot_results_markdown() {
    insta::assert_snapshot!(
        "results_markdown",
        results_markdown("full_method_v1", &canonical_bundle())
    );
}

#[test]
fn snapshot_results_latex() {
    insta::assert_snapshot!(
        "results_latex",
        results_latex("full_method_v1", &canonical_bundle().aggregate_metrics)
    );
}

#[test]
fn snapshot_render_all_preserves_blob_ordering() {
    let r = render_all("full_method_v1", &canonical_bundle());
    let concat = format!(
        "=== primary_csv ===\n{}\n=== instance_csv ===\n{}\n=== markdown ===\n{}\n=== latex ===\n{}",
        r.primary_csv, r.instance_csv, r.markdown, r.latex
    );
    insta::assert_snapshot!("render_all", concat);
}
