//! `cta_reports` — paper-ready tables and figures.
//!
//! Emits CSV, JSON summary, LaTeX table, and Markdown text for a given
//! results bundle. Every helper is a pure function of the input types —
//! no IO happens in this crate; the CLI orchestrates writing the outputs
//! to disk.

#![deny(missing_docs)]

use std::fmt::Write as _;

use cta_metrics::{AggregateMetrics, PrimaryMetrics, ResultsBundle};

pub mod aggregate;
pub use aggregate::{
    aggregate_by_system, domain_breakdown, domain_breakdown_latex, paired_deltas,
    paired_deltas_csv, provider_breakdown, provider_breakdown_latex, summary_primary_latex,
    BootstrapConfig, PairedDelta, RunSummary, SystemAggregate, PAIRED_DELTAS_CSV_HEADER,
};

/// Canonical system id column header used in tables.
pub const SYSTEM_ID_COL: &str = "system_id";

/// Canonical per-instance CSV header.
pub const INSTANCE_CSV_HEADER: &str = "instance_id,elaborated,num_obligations,faithfulness_score,num_faithful_full,num_partial,num_vacuous,num_consistent,num_inconsistent,num_not_applicable,critical_units_covered,critical_units_total\n";

/// Render the primary metrics as a LaTeX table body (tabular-free, caller
/// supplies the `\begin{tabular}{...}` environment).
#[must_use]
pub fn primary_metrics_latex(system_id: &str, m: &PrimaryMetrics) -> String {
    let mut s = String::new();
    let _ = writeln!(
        s,
        "{system} & {elab:.3} & {faith:.3} & {cov:.3} & {cons:.3} & {vac:.3} & {proof:.3} \\\\",
        system = system_id,
        elab = m.elaboration_rate,
        faith = m.semantic_faithfulness_mean,
        cov = m.critical_unit_coverage,
        cons = m.rust_consistency_rate,
        vac = m.vacuity_rate,
        proof = m.proof_utility,
    );
    s
}

/// Render the primary metrics as a Markdown table row.
#[must_use]
pub fn primary_metrics_markdown(system_id: &str, m: &PrimaryMetrics) -> String {
    format!(
        "| {system} | {elab:.3} | {faith:.3} | {cov:.3} | {cons:.3} | {vac:.3} | {proof:.3} |\n",
        system = system_id,
        elab = m.elaboration_rate,
        faith = m.semantic_faithfulness_mean,
        cov = m.critical_unit_coverage,
        cons = m.rust_consistency_rate,
        vac = m.vacuity_rate,
        proof = m.proof_utility,
    )
}

/// CSV header row matching `primary_metrics_csv_row`.
pub const PRIMARY_METRICS_CSV_HEADER: &str =
    "system_id,elaboration_rate,semantic_faithfulness_mean,critical_unit_coverage,rust_consistency_rate,vacuity_rate,proof_utility\n";

/// CSV row matching [`PRIMARY_METRICS_CSV_HEADER`].
#[must_use]
pub fn primary_metrics_csv_row(system_id: &str, m: &PrimaryMetrics) -> String {
    format!(
        "{system},{elab},{faith},{cov},{cons},{vac},{proof}\n",
        system = system_id,
        elab = m.elaboration_rate,
        faith = m.semantic_faithfulness_mean,
        cov = m.critical_unit_coverage,
        cons = m.rust_consistency_rate,
        vac = m.vacuity_rate,
        proof = m.proof_utility,
    )
}

/// Render a full per-instance CSV (header + rows) for a [`ResultsBundle`].
#[must_use]
pub fn instance_results_csv(bundle: &ResultsBundle) -> String {
    let mut s = String::with_capacity(256 + bundle.instance_results.len() * 64);
    s.push_str(INSTANCE_CSV_HEADER);
    for r in &bundle.instance_results {
        let _ = writeln!(
            s,
            "{id},{elab},{n},{fs:.6},{ff},{fp},{v},{cons},{inc},{na},{c},{t}",
            id = r.instance_id,
            elab = r.elaborated,
            n = r.num_obligations,
            fs = r.faithfulness_score,
            ff = r.num_faithful_full,
            fp = r.num_partial,
            v = r.num_vacuous,
            cons = r.num_consistent,
            inc = r.num_inconsistent,
            na = r.num_not_applicable,
            c = r.critical_units_covered,
            t = r.critical_units_total,
        );
    }
    s
}

/// Render a full Markdown summary: title, primary metrics row, then per-instance table.
#[must_use]
pub fn results_markdown(system_id: &str, bundle: &ResultsBundle) -> String {
    let m = &bundle.aggregate_metrics.primary;
    let mut s = String::new();
    let _ = writeln!(s, "# CTA benchmark results — {system_id}\n");
    let _ = writeln!(
        s,
        "- metrics version: `{}`",
        bundle.aggregate_metrics.metrics_version
    );
    let _ = writeln!(s, "- instances scored: {}", bundle.instance_results.len());
    let _ = writeln!(s);
    s.push_str("## Primary metrics\n\n");
    s.push_str("| system | elab | faith | cov | cons | vac | proof |\n");
    s.push_str("|---|---|---|---|---|---|---|\n");
    s.push_str(&primary_metrics_markdown(system_id, m));
    s.push('\n');
    s.push_str("## Secondary metrics\n\n");
    s.push_str("| metric | value |\n|---|---|\n");
    let sec = &bundle.aggregate_metrics.secondary;
    let _ = writeln!(
        s,
        "| avg_obligations_per_instance | {:.3} |",
        sec.avg_obligations_per_instance
    );
    let _ = writeln!(
        s,
        "| faithful_obligation_density | {:.3} |",
        sec.faithful_obligation_density
    );
    let _ = writeln!(
        s,
        "| contradiction_rate_on_critical_units | {:.3} |",
        sec.contradiction_rate_on_critical_units
    );
    let _ = writeln!(
        s,
        "| text_faithful_code_inconsistent_rate | {:.3} |",
        sec.text_faithful_code_inconsistent_rate
    );
    let _ = writeln!(
        s,
        "| code_faithful_text_incomplete_rate | {:.3} |",
        sec.code_faithful_text_incomplete_rate
    );
    s.push('\n');
    s.push_str("## Per-instance\n\n");
    s.push_str(
        "| instance | elab | n | faith-score | ff | fp | vac | cons | inc | na | cov/tot |\n",
    );
    s.push_str("|---|---|---|---|---|---|---|---|---|---|---|\n");
    for r in &bundle.instance_results {
        let _ = writeln!(
            s,
            "| `{id}` | {elab} | {n} | {fs:.3} | {ff} | {fp} | {v} | {cons} | {inc} | {na} | {c}/{t} |",
            id = r.instance_id,
            elab = r.elaborated,
            n = r.num_obligations,
            fs = r.faithfulness_score,
            ff = r.num_faithful_full,
            fp = r.num_partial,
            v = r.num_vacuous,
            cons = r.num_consistent,
            inc = r.num_inconsistent,
            na = r.num_not_applicable,
            c = r.critical_units_covered,
            t = r.critical_units_total,
        );
    }
    s
}

/// Render the primary-metrics LaTeX table (standalone, ready for \input).
#[must_use]
pub fn results_latex(system_id: &str, agg: &AggregateMetrics) -> String {
    let mut s = String::new();
    s.push_str("\\begin{tabular}{lrrrrrr}\n");
    s.push_str("\\toprule\n");
    s.push_str("system & elab & faith & cov & cons & vac & proof \\\\\n");
    s.push_str("\\midrule\n");
    s.push_str(&primary_metrics_latex(system_id, &agg.primary));
    s.push_str("\\bottomrule\n");
    s.push_str("\\end{tabular}\n");
    s
}

/// Collected report artifact paths, relative to the caller-provided output dir.
#[derive(Debug, Clone, Default)]
pub struct ReportArtifacts {
    /// CSV outputs produced.
    pub csv: Vec<String>,
    /// LaTeX outputs produced.
    pub latex: Vec<String>,
    /// Figure outputs produced (reserved for future use).
    pub figures: Vec<String>,
}

/// Convenience record produced by [`render_all`] — just the rendered text
/// artifacts, to be written by the CLI.
#[derive(Debug, Clone)]
pub struct RenderedReports {
    /// Aggregate primary-metrics CSV (single row for this system).
    pub primary_csv: String,
    /// Per-instance CSV.
    pub instance_csv: String,
    /// Markdown summary.
    pub markdown: String,
    /// Standalone LaTeX tabular.
    pub latex: String,
}

/// Render every text artifact for a single-system [`ResultsBundle`].
#[must_use]
pub fn render_all(system_id: &str, bundle: &ResultsBundle) -> RenderedReports {
    let primary_csv = {
        let mut s = String::from(PRIMARY_METRICS_CSV_HEADER);
        s.push_str(&primary_metrics_csv_row(
            system_id,
            &bundle.aggregate_metrics.primary,
        ));
        s
    };
    RenderedReports {
        primary_csv,
        instance_csv: instance_results_csv(bundle),
        markdown: results_markdown(system_id, bundle),
        latex: results_latex(system_id, &bundle.aggregate_metrics),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cta_metrics::{AggregateMetrics, InstanceResult, PrimaryMetrics, SecondaryMetrics};

    fn demo_metrics() -> PrimaryMetrics {
        PrimaryMetrics {
            elaboration_rate: 0.8,
            semantic_faithfulness_mean: 0.7,
            critical_unit_coverage: 0.6,
            rust_consistency_rate: 0.9,
            vacuity_rate: 0.1,
            proof_utility: 0.5,
        }
    }

    fn demo_bundle() -> ResultsBundle {
        ResultsBundle {
            schema_version: "schema_v1".into(),
            run_manifest: serde_json::json!({}),
            instance_results: vec![InstanceResult {
                instance_id: "arrays_binary_search_001".into(),
                elaborated: true,
                num_obligations: 3,
                faithfulness_score: 2.0,
                num_faithful_full: 2,
                num_partial: 0,
                num_vacuous: 0,
                num_consistent: 3,
                num_inconsistent: 0,
                num_not_applicable: 0,
                critical_units_covered: 2,
                critical_units_total: 3,
                lean_diagnostics_path: None,
                behavior_report_path: None,
            }],
            aggregate_metrics: AggregateMetrics {
                metrics_version: "metrics_v2".into(),
                primary: demo_metrics(),
                secondary: SecondaryMetrics {
                    avg_obligations_per_instance: 3.0,
                    faithful_obligation_density: 0.667,
                    contradiction_rate_on_critical_units: 0.0,
                    text_faithful_code_inconsistent_rate: 0.0,
                    code_faithful_text_incomplete_rate: 0.0,
                    inter_annotator_agreement: None,
                },
            },
        }
    }

    #[test]
    fn csv_row_round_trips_header_column_count() {
        let header_cols = PRIMARY_METRICS_CSV_HEADER.trim_end().split(',').count();
        let row_cols = primary_metrics_csv_row("text_only_v1", &demo_metrics())
            .trim_end()
            .split(',')
            .count();
        assert_eq!(header_cols, row_cols);
    }

    #[test]
    fn latex_row_contains_system() {
        let s = primary_metrics_latex("full_method_v1", &demo_metrics());
        assert!(s.contains("full_method_v1"));
        assert!(s.ends_with("\\\\\n"));
    }

    #[test]
    fn instance_csv_has_header_and_row() {
        let csv = instance_results_csv(&demo_bundle());
        assert!(csv.starts_with(INSTANCE_CSV_HEADER));
        assert!(csv.contains("arrays_binary_search_001"));
    }

    #[test]
    fn markdown_contains_system_and_table() {
        let md = results_markdown("text_only_v1", &demo_bundle());
        assert!(md.contains("text_only_v1"));
        assert!(md.contains("| system |"));
        assert!(md.contains("arrays_binary_search_001"));
    }

    #[test]
    fn render_all_emits_four_blobs() {
        let r = render_all("text_only_v1", &demo_bundle());
        assert!(!r.primary_csv.is_empty());
        assert!(!r.instance_csv.is_empty());
        assert!(!r.markdown.is_empty());
        assert!(r.latex.contains("\\begin{tabular}"));
    }
}
