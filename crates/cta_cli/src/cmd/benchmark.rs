use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use clap::Args;
use cta_benchmark::{
    build_manifest, check_authoring, lint_benchmark, load_benchmark, load_experiment_summaries,
    load_manifest, load_splits, validate_release, LintIssue, LintReport, LintSeverity,
    ReleaseCheckContext,
};
use cta_core::{BenchmarkVersion, Domain, MetricsVersion, RubricVersion};

use super::benchmark_dir;

#[derive(Debug, Args)]
pub struct LintArgs {
    #[arg(long, default_value = "v0.1", value_parser = crate::parse_bench_version)]
    pub version: BenchmarkVersion,

    /// Emit report as JSON on stdout.
    #[arg(long)]
    pub json: bool,

    /// Additionally run cross-artifact release-coherence checks
    /// (splits, manifest, experiment configs).
    #[arg(long, default_value_t = false)]
    pub release: bool,

    /// Rubric version used when recomputing the manifest for hash comparison
    /// during `--release`. Defaults to `rubric_v1`.
    #[arg(long, default_value = "rubric_v1")]
    pub rubric: String,

    /// Metrics version used when recomputing the manifest for hash comparison
    /// during `--release`. Defaults to the current metrics contract.
    #[arg(long, default_value = cta_metrics::METRICS_VERSION)]
    pub metrics: String,

    /// Enable the authoring-heuristic lints (vacuous termination,
    /// unconditional preconditions, uncovered critical SUs, orphan
    /// obligations). These are warnings; use `--strict-authoring` to
    /// promote them to errors.
    #[arg(long, default_value_t = false)]
    pub authoring: bool,

    /// Promote `AUTHORING_*` warnings to errors (implies `--authoring`).
    #[arg(long, default_value_t = false)]
    pub strict_authoring: bool,
}

pub fn lint(workspace: &Path, args: LintArgs) -> Result<()> {
    let root = benchmark_dir(workspace, args.version.as_str());
    let bench = load_benchmark(&root, &args.version)
        .with_context(|| format!("loading benchmark at {}", root.display()))?;
    let mut report = lint_benchmark(&bench);

    if args.authoring || args.strict_authoring {
        check_authoring(&bench, &mut report.issues);
        if args.strict_authoring {
            for issue in report.issues.iter_mut() {
                if issue.code.starts_with("AUTHORING_") {
                    issue.severity = LintSeverity::Error;
                }
            }
        }
    }

    if args.release {
        let splits = load_splits(&root, &args.version)
            .with_context(|| format!("loading splits under {}", root.display()))?;
        let manifest =
            load_manifest(&root).with_context(|| format!("loading manifest under {}", root.display()))?;
        let (experiments, mut parse_issues) = load_experiment_summaries(workspace)
            .with_context(|| "loading experiment configs under configs/experiments/")?;
        let rubric = RubricVersion::new(args.rubric.clone())
            .map_err(|e| anyhow::anyhow!("invalid rubric version: {e}"))?;
        let metrics = MetricsVersion::new(args.metrics.clone())
            .map_err(|e| anyhow::anyhow!("invalid metrics version: {e}"))?;
        let ctx = ReleaseCheckContext {
            workspace_root: workspace,
            benchmark: &bench,
            splits: &splits,
            manifest: manifest.as_ref(),
            experiments: &experiments,
            rubric_version: &rubric,
            metrics_version: &metrics,
        };
        let release_report = validate_release(&ctx);
        report.issues.append(&mut parse_issues);
        report.issues.extend(release_report.issues.into_iter());
    }

    emit_report(&report, bench.len(), args.json)?;

    if report.has_errors() {
        anyhow::bail!(
            "benchmark lint failed with {} error(s)",
            report
                .issues
                .iter()
                .filter(|i| i.severity == LintSeverity::Error)
                .count()
        );
    }
    Ok(())
}

fn emit_report(report: &LintReport, bench_len: usize, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for issue in &report.issues {
            print_issue(issue);
        }
        println!(
            "\nsummary: {} error(s), {} warning(s) across {} instance(s)",
            report.error_count(),
            report.warning_count(),
            bench_len
        );
    }
    Ok(())
}

fn print_issue(issue: &LintIssue) {
    println!(
        "[{}] {} {}: {}",
        issue.severity, issue.code, issue.instance_id, issue.message
    );
}

#[derive(Debug, Args)]
pub struct StatsArgs {
    #[arg(long, default_value = "v0.1", value_parser = crate::parse_bench_version)]
    pub version: BenchmarkVersion,
}

pub fn stats(workspace: &Path, args: StatsArgs) -> Result<()> {
    let root = benchmark_dir(workspace, args.version.as_str());
    let bench = load_benchmark(&root, &args.version)?;

    let mut by_domain: BTreeMap<Domain, u32> = BTreeMap::new();
    for (_, v) in bench.iter() {
        *by_domain.entry(v.record.domain).or_insert(0) += 1;
    }
    println!("benchmark: {}", args.version);
    println!("root:      {}", root.display());
    println!("instances: {}", bench.len());
    println!("by domain:");
    for (d, n) in by_domain {
        println!("  {:<8} {n}", d.as_str());
    }
    Ok(())
}

#[derive(Debug, Args)]
pub struct ManifestArgs {
    #[arg(long, default_value = "v0.1", value_parser = crate::parse_bench_version)]
    pub version: BenchmarkVersion,

    /// Rubric version to pin into the manifest.
    #[arg(long, default_value = "rubric_v1")]
    pub rubric: String,

    /// Metrics version to pin into the manifest. Defaults to the current
    /// canonical metrics contract in `cta_metrics::METRICS_VERSION`.
    #[arg(long, default_value = cta_metrics::METRICS_VERSION)]
    pub metrics: String,

    /// Output path; defaults to `<bench>/manifests/benchmark_manifest.json`.
    #[arg(long)]
    pub out: Option<std::path::PathBuf>,
}

pub fn manifest(workspace: &Path, args: ManifestArgs) -> Result<()> {
    let root = benchmark_dir(workspace, args.version.as_str());
    let bench = load_benchmark(&root, &args.version)?;
    let rubric = RubricVersion::new(args.rubric.clone())
        .map_err(|e| anyhow::anyhow!("invalid rubric version: {e}"))?;
    let metrics = MetricsVersion::new(args.metrics.clone())
        .map_err(|e| anyhow::anyhow!("invalid metrics version: {e}"))?;

    let now = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());

    let m = build_manifest(&bench, &rubric, &metrics, &now)?;
    let out = args
        .out
        .unwrap_or_else(|| root.join("manifests").join("benchmark_manifest.json"));
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out, serde_json::to_vec_pretty(&m)?)?;
    println!("wrote {}", out.display());
    println!("content_hash: {}", m.content_hash);
    Ok(())
}
