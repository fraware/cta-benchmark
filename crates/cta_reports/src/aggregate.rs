//! Cross-run aggregation for paper-grade reporting.
//!
//! A single [`ResultsBundle`] captures metrics for one
//! `(system, provider, seed, split)` quadruple. The paper needs
//! cross-run statistics:
//!
//! - per-system mean across seeds and providers,
//! - provider-wise breakdown (`stub`, `openai`, `anthropic`, ...),
//! - per-domain breakdown (`arrays`, `dp`, `graph`, `greedy`, `sorting`,
//!   `trees`),
//! - paired instance-level deltas between two systems on a shared split,
//! - bootstrap confidence intervals for each scalar metric.
//!
//! This module is pure: every function takes fully-materialised run
//! summaries and returns typed aggregates. The CLI
//! (`cta reports aggregate`) is responsible for discovering run bundles
//! on disk and writing the outputs. Determinism is preserved by a
//! fixed-seed bootstrap resampler (see [`BootstrapConfig`]).

use std::collections::BTreeMap;
use std::fmt::Write as _;

use cta_metrics::{InstanceResult, PrimaryMetrics, ResultsBundle};
use serde::{Deserialize, Serialize};

/// Single-run summary used as the aggregation input. Each summary pairs a
/// results bundle with the identifying metadata that would otherwise have
/// to be scraped from a filename.
#[derive(Debug, Clone)]
pub struct RunSummary {
    /// Canonical run id (`run_<date>_<system>_<split>_<nnn>`).
    pub run_id: String,
    /// System identifier (e.g. `text_only_v1`).
    pub system_id: String,
    /// Provider name (e.g. `stub`, `openai`, `anthropic`).
    pub provider: String,
    /// Benchmark split (`dev`, `eval`, ...).
    pub split: String,
    /// Seed the run was generated under.
    pub seed: u64,
    /// The run's results bundle.
    pub bundle: ResultsBundle,
}

impl RunSummary {
    /// Extract the provider block from a run_manifest.json `Value`, falling
    /// back to `"unknown"` when the block is missing or malformed.
    #[must_use]
    pub fn provider_from_manifest(manifest: &serde_json::Value) -> String {
        manifest
            .get("provider")
            .and_then(|p| p.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Extract the system id from a run_manifest.json `Value`, falling
    /// back to the one declared inside the results bundle when absent.
    #[must_use]
    pub fn system_from_manifest(manifest: &serde_json::Value) -> Option<String> {
        manifest
            .get("system_id")
            .and_then(|v| v.as_str())
            .map(str::to_string)
    }

    /// Extract the seed from a run_manifest.json `Value`.
    #[must_use]
    pub fn seed_from_manifest(manifest: &serde_json::Value) -> u64 {
        manifest
            .get("seed")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
    }
}

/// Per-system mean across seeds and providers, with optional bootstrap CIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAggregate {
    /// System identifier.
    pub system_id: String,
    /// Number of runs folded into this aggregate.
    pub run_count: usize,
    /// Mean of the six primary scalars across runs.
    pub mean_primary: PrimaryMetrics,
    /// Bootstrap 95% confidence intervals, keyed by metric name. Empty when
    /// `BootstrapConfig::resamples == 0`.
    pub ci: BTreeMap<String, (f64, f64)>,
}

/// Paired instance-level delta between two systems on a shared split.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedDelta {
    /// Instance id the comparison is computed on.
    pub instance_id: String,
    /// Delta in `faithfulness_score / num_obligations` (b - a), where a
    /// and b are the [`PairedDeltas`] system identifiers. Zero if either
    /// side produced zero obligations.
    pub delta_faithfulness_fraction: f64,
    /// Delta in `num_consistent / (num_consistent + num_inconsistent)`
    /// (b - a); zero if either side's denominator is zero.
    pub delta_consistency_fraction: f64,
    /// Delta in `critical_units_covered / critical_units_total` (b - a);
    /// zero if either side's total is zero.
    pub delta_coverage_fraction: f64,
}

/// Deterministic bootstrap-resampling configuration. `seed == 0` and
/// `resamples == 0` together disable CI computation.
#[derive(Debug, Clone, Copy)]
pub struct BootstrapConfig {
    /// Number of bootstrap resamples. Set to 0 to disable CI computation.
    pub resamples: usize,
    /// RNG seed for reproducibility. The implementation uses a
    /// linear-congruential generator seeded here; results are stable
    /// across platforms.
    pub seed: u64,
    /// Confidence level in `(0, 1)`. Typical: `0.95`.
    pub confidence: f64,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            resamples: 1000,
            seed: 0x5EED_CAFE_B00F_F117,
            confidence: 0.95,
        }
    }
}

/// Aggregate per-system across runs. Runs are grouped by `system_id`.
#[must_use]
pub fn aggregate_by_system(runs: &[RunSummary], cfg: BootstrapConfig) -> Vec<SystemAggregate> {
    let mut groups: BTreeMap<&str, Vec<&RunSummary>> = BTreeMap::new();
    for r in runs {
        groups.entry(&r.system_id).or_default().push(r);
    }
    groups
        .into_iter()
        .map(|(system_id, rs)| aggregate_runs(system_id.to_string(), &rs, cfg))
        .collect()
}

/// Aggregate by `(system_id, provider)` pair. Useful for the per-provider
/// breakdown table in the paper.
#[must_use]
pub fn provider_breakdown(
    runs: &[RunSummary],
    cfg: BootstrapConfig,
) -> Vec<(String, SystemAggregate)> {
    let mut groups: BTreeMap<(String, String), Vec<&RunSummary>> = BTreeMap::new();
    for r in runs {
        groups
            .entry((r.system_id.clone(), r.provider.clone()))
            .or_default()
            .push(r);
    }
    groups
        .into_iter()
        .map(|((sys, prov), rs)| (prov, aggregate_runs(sys, &rs, cfg)))
        .collect()
}

/// Aggregate a per-domain slice of one system's runs. For each domain we
/// restrict each bundle's `instance_results` to the matching domain prefix
/// (e.g. `arrays_*`) before computing the scalar means. Returns a
/// deterministic order `(domain_prefix, SystemAggregate)`.
#[must_use]
pub fn domain_breakdown(
    runs: &[RunSummary],
    cfg: BootstrapConfig,
) -> Vec<(String, SystemAggregate)> {
    // Walk all instance ids and derive a distinct domain set.
    let mut domains: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for r in runs {
        for ir in &r.bundle.instance_results {
            if let Some((domain, _)) = ir.instance_id.split_once('_') {
                domains.insert(domain.to_string());
            }
        }
    }
    let mut out = Vec::new();
    for domain in domains {
        let filtered: Vec<RunSummary> = runs
            .iter()
            .map(|r| {
                let mut clone = r.clone();
                clone.bundle.instance_results.retain(|ir| {
                    ir.instance_id
                        .split_once('_')
                        .map(|(d, _)| d == domain)
                        .unwrap_or(false)
                });
                clone
            })
            .filter(|r| !r.bundle.instance_results.is_empty())
            .collect();
        if filtered.is_empty() {
            continue;
        }
        let grouped: BTreeMap<&str, Vec<&RunSummary>> = {
            let mut g = BTreeMap::new();
            for r in &filtered {
                g.entry(r.system_id.as_str())
                    .or_insert_with(Vec::new)
                    .push(r);
            }
            g
        };
        for (sys, rs) in grouped {
            out.push((domain.clone(), aggregate_runs(sys.to_string(), &rs, cfg)));
        }
    }
    out
}

/// Paired instance-level deltas between two systems. Both systems must
/// have been scored on the same set of instances; pairs where either side
/// is missing are skipped. When a system has multiple runs, the
/// instance-level scalars are averaged across its runs before differencing.
#[must_use]
pub fn paired_deltas(
    runs: &[RunSummary],
    system_a: &str,
    system_b: &str,
) -> Vec<PairedDelta> {
    let a = instance_means(runs, system_a);
    let b = instance_means(runs, system_b);
    let mut out = Vec::new();
    for (id, ma) in &a {
        if let Some(mb) = b.get(id) {
            out.push(PairedDelta {
                instance_id: id.clone(),
                delta_faithfulness_fraction: fraction(mb.faithfulness_score, mb.num_obligations)
                    - fraction(ma.faithfulness_score, ma.num_obligations),
                delta_consistency_fraction: consistency_fraction(mb)
                    - consistency_fraction(ma),
                delta_coverage_fraction: coverage_fraction(mb) - coverage_fraction(ma),
            });
        }
    }
    out.sort_by(|l, r| l.instance_id.cmp(&r.instance_id));
    out
}

/// Render a LaTeX `tabular` of per-system primary means. Column order:
/// system, runs, elab, faith, cov, cons, vac, proof.
#[must_use]
pub fn summary_primary_latex(aggregates: &[SystemAggregate]) -> String {
    let mut s = String::new();
    s.push_str("\\begin{tabular}{lrrrrrrr}\n\\toprule\n");
    s.push_str("system & runs & elab & faith & cov & cons & vac & proof \\\\\n\\midrule\n");
    for a in aggregates {
        let _ = writeln!(
            s,
            "{sys} & {runs} & {elab:.3} & {faith:.3} & {cov:.3} & {cons:.3} & {vac:.3} & {proof:.3} \\\\",
            sys = a.system_id,
            runs = a.run_count,
            elab = a.mean_primary.elaboration_rate,
            faith = a.mean_primary.semantic_faithfulness_mean,
            cov = a.mean_primary.critical_unit_coverage,
            cons = a.mean_primary.rust_consistency_rate,
            vac = a.mean_primary.vacuity_rate,
            proof = a.mean_primary.proof_utility,
        );
    }
    s.push_str("\\bottomrule\n\\end{tabular}\n");
    s
}

/// Render a LaTeX `tabular` of per-system per-domain means.
#[must_use]
pub fn domain_breakdown_latex(rows: &[(String, SystemAggregate)]) -> String {
    let mut s = String::new();
    s.push_str("\\begin{tabular}{llrrrrrr}\n\\toprule\n");
    s.push_str(
        "domain & system & elab & faith & cov & cons & vac & proof \\\\\n\\midrule\n",
    );
    for (dom, a) in rows {
        let _ = writeln!(
            s,
            "{dom} & {sys} & {elab:.3} & {faith:.3} & {cov:.3} & {cons:.3} & {vac:.3} & {proof:.3} \\\\",
            dom = dom,
            sys = a.system_id,
            elab = a.mean_primary.elaboration_rate,
            faith = a.mean_primary.semantic_faithfulness_mean,
            cov = a.mean_primary.critical_unit_coverage,
            cons = a.mean_primary.rust_consistency_rate,
            vac = a.mean_primary.vacuity_rate,
            proof = a.mean_primary.proof_utility,
        );
    }
    s.push_str("\\bottomrule\n\\end{tabular}\n");
    s
}

/// Render a LaTeX `tabular` of per-system per-provider means.
#[must_use]
pub fn provider_breakdown_latex(rows: &[(String, SystemAggregate)]) -> String {
    let mut s = String::new();
    s.push_str("\\begin{tabular}{llrrrrrr}\n\\toprule\n");
    s.push_str(
        "provider & system & elab & faith & cov & cons & vac & proof \\\\\n\\midrule\n",
    );
    for (prov, a) in rows {
        let _ = writeln!(
            s,
            "{prov} & {sys} & {elab:.3} & {faith:.3} & {cov:.3} & {cons:.3} & {vac:.3} & {proof:.3} \\\\",
            prov = prov,
            sys = a.system_id,
            elab = a.mean_primary.elaboration_rate,
            faith = a.mean_primary.semantic_faithfulness_mean,
            cov = a.mean_primary.critical_unit_coverage,
            cons = a.mean_primary.rust_consistency_rate,
            vac = a.mean_primary.vacuity_rate,
            proof = a.mean_primary.proof_utility,
        );
    }
    s.push_str("\\bottomrule\n\\end{tabular}\n");
    s
}

/// CSV header for paired-deltas output.
pub const PAIRED_DELTAS_CSV_HEADER: &str =
    "instance_id,delta_faithfulness_fraction,delta_consistency_fraction,delta_coverage_fraction\n";

/// Render a CSV of paired instance-level deltas between two systems.
#[must_use]
pub fn paired_deltas_csv(deltas: &[PairedDelta]) -> String {
    let mut s = String::from(PAIRED_DELTAS_CSV_HEADER);
    for d in deltas {
        let _ = writeln!(
            s,
            "{},{:.6},{:.6},{:.6}",
            d.instance_id,
            d.delta_faithfulness_fraction,
            d.delta_consistency_fraction,
            d.delta_coverage_fraction,
        );
    }
    s
}

fn aggregate_runs(system_id: String, runs: &[&RunSummary], cfg: BootstrapConfig) -> SystemAggregate {
    let mut primaries: Vec<&PrimaryMetrics> =
        runs.iter().map(|r| &r.bundle.aggregate_metrics.primary).collect();
    // Deterministic order for CI computation.
    primaries.sort_by(|a, b| {
        a.elaboration_rate
            .partial_cmp(&b.elaboration_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mean = mean_primary(&primaries);
    let ci = if cfg.resamples == 0 || primaries.len() < 2 {
        BTreeMap::new()
    } else {
        bootstrap_ci(&primaries, cfg)
    };
    SystemAggregate {
        system_id,
        run_count: runs.len(),
        mean_primary: mean,
        ci,
    }
}

fn mean_primary(xs: &[&PrimaryMetrics]) -> PrimaryMetrics {
    let n = xs.len().max(1) as f64;
    let mut m = PrimaryMetrics {
        elaboration_rate: 0.0,
        semantic_faithfulness_mean: 0.0,
        critical_unit_coverage: 0.0,
        rust_consistency_rate: 0.0,
        vacuity_rate: 0.0,
        proof_utility: 0.0,
    };
    for p in xs {
        m.elaboration_rate += p.elaboration_rate;
        m.semantic_faithfulness_mean += p.semantic_faithfulness_mean;
        m.critical_unit_coverage += p.critical_unit_coverage;
        m.rust_consistency_rate += p.rust_consistency_rate;
        m.vacuity_rate += p.vacuity_rate;
        m.proof_utility += p.proof_utility;
    }
    m.elaboration_rate /= n;
    m.semantic_faithfulness_mean /= n;
    m.critical_unit_coverage /= n;
    m.rust_consistency_rate /= n;
    m.vacuity_rate /= n;
    m.proof_utility /= n;
    m
}

fn bootstrap_ci(xs: &[&PrimaryMetrics], cfg: BootstrapConfig) -> BTreeMap<String, (f64, f64)> {
    let metrics: [(&str, fn(&PrimaryMetrics) -> f64); 6] = [
        ("elaboration_rate", |p| p.elaboration_rate),
        ("semantic_faithfulness_mean", |p| p.semantic_faithfulness_mean),
        ("critical_unit_coverage", |p| p.critical_unit_coverage),
        ("rust_consistency_rate", |p| p.rust_consistency_rate),
        ("vacuity_rate", |p| p.vacuity_rate),
        ("proof_utility", |p| p.proof_utility),
    ];
    let mut out = BTreeMap::new();
    for (name, extract) in metrics {
        let values: Vec<f64> = xs.iter().map(|p| extract(p)).collect();
        let ci = percentile_bootstrap(&values, cfg);
        out.insert(name.to_string(), ci);
    }
    out
}

fn percentile_bootstrap(values: &[f64], cfg: BootstrapConfig) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let n = values.len();
    let mut rng = LcgRng::new(cfg.seed);
    let mut means: Vec<f64> = Vec::with_capacity(cfg.resamples);
    for _ in 0..cfg.resamples {
        let mut sum = 0.0;
        for _ in 0..n {
            let idx = (rng.next() as usize) % n;
            sum += values[idx];
        }
        means.push(sum / n as f64);
    }
    means.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let alpha = (1.0 - cfg.confidence).max(0.0).min(1.0);
    let lo_idx = ((alpha / 2.0) * means.len() as f64).floor() as usize;
    let hi_idx = (((1.0 - alpha / 2.0) * means.len() as f64).ceil() as usize)
        .saturating_sub(1)
        .min(means.len().saturating_sub(1));
    (means[lo_idx], means[hi_idx])
}

fn instance_means(runs: &[RunSummary], system_id: &str) -> BTreeMap<String, InstanceAverages> {
    let mut buckets: BTreeMap<String, (InstanceAverages, usize)> = BTreeMap::new();
    for r in runs {
        if r.system_id != system_id {
            continue;
        }
        for ir in &r.bundle.instance_results {
            let entry = buckets
                .entry(ir.instance_id.clone())
                .or_insert_with(|| (InstanceAverages::default(), 0));
            entry.0.accumulate(ir);
            entry.1 += 1;
        }
    }
    buckets
        .into_iter()
        .map(|(id, (mut avg, count))| {
            avg.divide(count as f64);
            (id, avg)
        })
        .collect()
}

#[derive(Debug, Clone, Default)]
struct InstanceAverages {
    faithfulness_score: f64,
    num_obligations: f64,
    num_consistent: f64,
    num_inconsistent: f64,
    critical_units_covered: f64,
    critical_units_total: f64,
}

impl InstanceAverages {
    fn accumulate(&mut self, ir: &InstanceResult) {
        self.faithfulness_score += ir.faithfulness_score;
        self.num_obligations += f64::from(ir.num_obligations);
        self.num_consistent += f64::from(ir.num_consistent);
        self.num_inconsistent += f64::from(ir.num_inconsistent);
        self.critical_units_covered += f64::from(ir.critical_units_covered);
        self.critical_units_total += f64::from(ir.critical_units_total);
    }

    fn divide(&mut self, n: f64) {
        if n == 0.0 {
            return;
        }
        self.faithfulness_score /= n;
        self.num_obligations /= n;
        self.num_consistent /= n;
        self.num_inconsistent /= n;
        self.critical_units_covered /= n;
        self.critical_units_total /= n;
    }
}

fn fraction(num: f64, denom: f64) -> f64 {
    if denom == 0.0 {
        0.0
    } else {
        num / denom
    }
}

fn consistency_fraction(a: &InstanceAverages) -> f64 {
    fraction(a.num_consistent, a.num_consistent + a.num_inconsistent)
}

fn coverage_fraction(a: &InstanceAverages) -> f64 {
    fraction(a.critical_units_covered, a.critical_units_total)
}

/// Minimal linear-congruential RNG used for reproducible bootstrap
/// resampling. Parameters are Numerical Recipes' values; good enough for
/// a percentile-bootstrap and trivial to reproduce in any other language.
#[derive(Debug, Clone)]
struct LcgRng {
    state: u64,
}

impl LcgRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0xA5A5_A5A5_A5A5_A5A5),
        }
    }

    fn next(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cta_metrics::{AggregateMetrics, InstanceResult, SecondaryMetrics};

    fn make_run(run_id: &str, system: &str, provider: &str, seed: u64, bundle: ResultsBundle) -> RunSummary {
        RunSummary {
            run_id: run_id.to_string(),
            system_id: system.to_string(),
            provider: provider.to_string(),
            split: "dev".to_string(),
            seed,
            bundle,
        }
    }

    fn bundle(elab: f64, faith: f64) -> ResultsBundle {
        ResultsBundle {
            schema_version: "schema_v1".into(),
            run_manifest: serde_json::json!({}),
            instance_results: vec![InstanceResult {
                instance_id: "arrays_binary_search_001".into(),
                elaborated: true,
                num_obligations: 2,
                faithfulness_score: faith * 2.0,
                num_faithful_full: 1,
                num_partial: 0,
                num_vacuous: 0,
                num_consistent: 2,
                num_inconsistent: 0,
                num_not_applicable: 0,
                critical_units_covered: 1,
                critical_units_total: 2,
                lean_diagnostics_path: None,
                behavior_report_path: None,
            }],
            aggregate_metrics: AggregateMetrics {
                metrics_version: "metrics_v2".into(),
                primary: PrimaryMetrics {
                    elaboration_rate: elab,
                    semantic_faithfulness_mean: faith,
                    critical_unit_coverage: 0.5,
                    rust_consistency_rate: 1.0,
                    vacuity_rate: 0.0,
                    proof_utility: 0.0,
                },
                secondary: SecondaryMetrics {
                    avg_obligations_per_instance: 2.0,
                    faithful_obligation_density: faith,
                    contradiction_rate_on_critical_units: 0.0,
                    text_faithful_code_inconsistent_rate: 0.0,
                    code_faithful_text_incomplete_rate: 0.0,
                    inter_annotator_agreement: None,
                },
            },
        }
    }

    #[test]
    fn mean_of_two_runs_is_midpoint() {
        let runs = vec![
            make_run("r1", "full_method_v1", "stub", 1, bundle(1.0, 0.6)),
            make_run("r2", "full_method_v1", "stub", 2, bundle(1.0, 0.8)),
        ];
        let agg = aggregate_by_system(&runs, BootstrapConfig { resamples: 0, ..Default::default() });
        assert_eq!(agg.len(), 1);
        assert_eq!(agg[0].run_count, 2);
        assert!((agg[0].mean_primary.semantic_faithfulness_mean - 0.7).abs() < 1e-9);
    }

    #[test]
    fn bootstrap_ci_is_deterministic_with_fixed_seed() {
        let runs = vec![
            make_run("r1", "a", "stub", 1, bundle(1.0, 0.5)),
            make_run("r2", "a", "stub", 2, bundle(1.0, 0.6)),
            make_run("r3", "a", "stub", 3, bundle(1.0, 0.7)),
            make_run("r4", "a", "stub", 4, bundle(1.0, 0.8)),
        ];
        let cfg = BootstrapConfig { resamples: 256, seed: 42, confidence: 0.95 };
        let a = aggregate_by_system(&runs, cfg);
        let b = aggregate_by_system(&runs, cfg);
        assert_eq!(
            a[0].ci.get("semantic_faithfulness_mean"),
            b[0].ci.get("semantic_faithfulness_mean"),
        );
    }

    #[test]
    fn paired_deltas_are_symmetric() {
        let runs = vec![
            make_run("r1", "text_only_v1", "stub", 1, bundle(1.0, 0.5)),
            make_run("r2", "full_method_v1", "stub", 1, bundle(1.0, 0.9)),
        ];
        let d_ab = paired_deltas(&runs, "text_only_v1", "full_method_v1");
        let d_ba = paired_deltas(&runs, "full_method_v1", "text_only_v1");
        assert_eq!(d_ab.len(), 1);
        assert_eq!(d_ba.len(), 1);
        assert!((d_ab[0].delta_faithfulness_fraction + d_ba[0].delta_faithfulness_fraction).abs() < 1e-9);
    }

    #[test]
    fn paired_deltas_csv_shape() {
        let deltas = vec![PairedDelta {
            instance_id: "arrays_binary_search_001".into(),
            delta_faithfulness_fraction: 0.1,
            delta_consistency_fraction: 0.0,
            delta_coverage_fraction: 0.2,
        }];
        let csv = paired_deltas_csv(&deltas);
        assert!(csv.starts_with(PAIRED_DELTAS_CSV_HEADER));
        assert!(csv.contains("arrays_binary_search_001"));
    }
}
