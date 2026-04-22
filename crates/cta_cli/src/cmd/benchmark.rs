use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use clap::Args;
use cta_benchmark::{build_manifest, lint_benchmark, load_benchmark, LintSeverity};
use cta_core::{BenchmarkVersion, Domain, MetricsVersion, RubricVersion};

use super::benchmark_dir;

#[derive(Debug, Args)]
pub struct LintArgs {
    #[arg(long, default_value = "v0.1", value_parser = crate::parse_bench_version)]
    pub version: BenchmarkVersion,

    /// Emit report as JSON on stdout.
    #[arg(long)]
    pub json: bool,
}

pub fn lint(workspace: &Path, args: LintArgs) -> Result<()> {
    let root = benchmark_dir(workspace, args.version.as_str());
    let bench = load_benchmark(&root, &args.version)
        .with_context(|| format!("loading benchmark at {}", root.display()))?;
    let report = lint_benchmark(&bench);

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for issue in &report.issues {
            println!(
                "[{}] {} {}: {}",
                issue.severity, issue.code, issue.instance_id, issue.message
            );
        }
        println!(
            "\nsummary: {} error(s), {} warning(s) across {} instance(s)",
            report.error_count(),
            report.warning_count(),
            bench.len()
        );
    }

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

    /// Metrics version to pin into the manifest.
    #[arg(long, default_value = "metrics_v1")]
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
