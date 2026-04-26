use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;
use cta_benchmark::{
    load_benchmark, load_experiment_summaries, load_manifest, load_splits, validate_release,
};
use cta_core::{BenchmarkVersion, MetricsVersion, RubricVersion};
use cta_schema::{SchemaName, SchemaRegistry};
use serde_json::json;
use walkdir::WalkDir;

use super::{benchmark_dir, schemas_dir};

/// Validate a single JSON artifact against a named schema.
#[derive(Debug, Args)]
pub struct FileArgs {
    /// Schema name (e.g. `instance`, `run_manifest`, `results_bundle`, `experiment`).
    #[arg(long)]
    pub schema: String,
    /// Path to the JSON artifact.
    #[arg(long)]
    pub path: PathBuf,
    /// Optional schemas root override.
    #[arg(long)]
    pub schemas: Option<PathBuf>,
}

pub fn file(workspace: &Path, args: FileArgs) -> Result<()> {
    let schemas_root = args.schemas.unwrap_or_else(|| schemas_dir(workspace));
    let reg = SchemaRegistry::load(&schemas_root)
        .with_context(|| format!("loading schemas from {}", schemas_root.display()))?;
    let schema_name = SchemaName::parse(&args.schema)
        .ok_or_else(|| anyhow::anyhow!("unknown schema name: {}", args.schema))?;
    reg.validate_file(schema_name, &args.path)
        .with_context(|| format!("validating {}", args.path.display()))?;
    println!(
        "ok: {} validates against {:?}",
        args.path.display(),
        schema_name
    );
    Ok(())
}

#[derive(Debug, Args)]
pub struct SchemasArgs {
    /// Path to schemas directory (defaults to `<workspace>/schemas`).
    #[arg(long)]
    pub schemas: Option<PathBuf>,
}

pub fn schemas(workspace: &Path, args: SchemasArgs) -> Result<()> {
    let dir = args.schemas.unwrap_or_else(|| schemas_dir(workspace));
    let reg = SchemaRegistry::load(&dir)
        .with_context(|| format!("failed to load schemas from {}", dir.display()))?;
    println!(
        "loaded {} canonical schemas from {}",
        SchemaName::ALL.len(),
        reg.root().display()
    );
    for name in SchemaName::ALL {
        println!("  - {:?} ({})", name, name.file_name());
    }
    Ok(())
}

#[derive(Debug, Args)]
pub struct BenchmarkArgs {
    /// Benchmark version, e.g. `v0.1`.
    #[arg(long, default_value = "v0.1", value_parser = crate::parse_bench_version)]
    pub version: BenchmarkVersion,
    /// Also run release-coherence checks (splits/manifest/experiments).
    #[arg(long, default_value_t = false)]
    pub release: bool,
    /// Rubric version used when recomputing the manifest during `--release`.
    #[arg(long, default_value = "rubric_v1")]
    pub rubric: String,
    /// Metrics version used when recomputing the manifest during `--release`.
    #[arg(long, default_value = cta_metrics::METRICS_VERSION)]
    pub metrics: String,
}

pub fn benchmark(workspace: &Path, args: BenchmarkArgs) -> Result<()> {
    let schemas_root = schemas_dir(workspace);
    let reg = SchemaRegistry::load(&schemas_root)
        .with_context(|| format!("loading schemas from {}", schemas_root.display()))?;
    let bench_root = benchmark_dir(workspace, args.version.as_str());
    let bench = load_benchmark(&bench_root, &args.version)?;

    let mut failures = 0usize;
    for (id, view) in bench.iter() {
        if let Err(e) = reg.validate_file(SchemaName::Instance, &view.instance_json) {
            failures += 1;
            eprintln!("[fail] {id}: {e}");
            continue;
        }
        if view.reference_obligations.is_file() {
            if let Err(e) = reg.validate_file(SchemaName::Obligation, &view.reference_obligations) {
                failures += 1;
                eprintln!("[fail] {id} obligations: {e}");
            }
        }
        if view.semantic_units.is_file() {
            if let Err(e) = reg.validate_file(SchemaName::SemanticUnits, &view.semantic_units) {
                failures += 1;
                eprintln!("[fail] {id} semantic_units: {e}");
            }
        }
        if view.harness.is_file() {
            if let Err(e) = reg.validate_file(SchemaName::Harness, &view.harness) {
                failures += 1;
                eprintln!("[fail] {id} harness: {e}");
            }
        }
    }

    // Additionally, validate only the canonical benchmark manifest.
    let manifests = bench_root.join("manifests");
    if manifests.is_dir() {
        for entry in WalkDir::new(&manifests).min_depth(1).max_depth(1) {
            let entry = entry?;
            let path = entry.path();
            if path.file_name().and_then(|n| n.to_str()) == Some("benchmark_manifest.json") {
                if let Err(e) = reg.validate_file(SchemaName::BenchmarkManifest, path) {
                    failures += 1;
                    eprintln!("[fail] manifest {}: {e}", path.display());
                }
            }
        }
    }

    if failures > 0 {
        anyhow::bail!("{failures} validation failure(s)");
    }

    if args.release {
        let splits = load_splits(&bench_root, &args.version)
            .with_context(|| format!("loading splits under {}", bench_root.display()))?;
        let manifest = load_manifest(&bench_root)
            .with_context(|| format!("loading manifest under {}", bench_root.display()))?;
        let (experiments, parse_issues) = load_experiment_summaries(workspace)
            .with_context(|| "loading experiment configs under configs/experiments/")?;
        let rubric = RubricVersion::new(args.rubric.clone())
            .map_err(|e| anyhow::anyhow!("invalid rubric version: {e}"))?;
        let metrics = MetricsVersion::new(args.metrics.clone())
            .map_err(|e| anyhow::anyhow!("invalid metrics version: {e}"))?;
        let ctx = cta_benchmark::ReleaseCheckContext {
            workspace_root: workspace,
            benchmark: &bench,
            splits: &splits,
            manifest: manifest.as_ref(),
            experiments: &experiments,
            rubric_version: &rubric,
            metrics_version: &metrics,
        };
        let release_report = validate_release(&ctx);
        let mut release_issues = parse_issues;
        release_issues.extend(release_report.issues);
        if !release_issues.is_empty() {
            for issue in &release_issues {
                eprintln!(
                    "[{}] {} {}: {}",
                    issue.severity, issue.code, issue.instance_id, issue.message
                );
            }
        }
        let release_errors = release_issues
            .iter()
            .filter(|i| i.severity == cta_benchmark::LintSeverity::Error)
            .count();
        print_release_status(&release_issues);
        write_release_summary(
            &bench_root,
            &args.version,
            workspace,
            manifest.as_ref(),
            &splits,
            &experiments,
            &release_issues,
        )?;
        if release_errors > 0 {
            anyhow::bail!("{release_errors} release validation error(s)");
        }
    }

    println!(
        "ok: validated {} instance(s) under {}",
        bench.len(),
        bench_root.display()
    );
    Ok(())
}

fn print_release_status(issues: &[cta_benchmark::LintIssue]) {
    let has_split = has_error_with_prefix(issues, "SPLIT_");
    let has_annotation = has_error_with_prefix(issues, "EXPERIMENT_ANNOTATION_");
    let has_signoff = has_error_with_prefix(issues, "GOLD_AUDIT_SIGNOFF_");
    let has_manifest = has_error_with_prefix(issues, "MANIFEST_");
    println!(
        "release status: split={} annotation_coverage={} signoff={} manifest={}",
        pass_fail(!has_split),
        pass_fail(!has_annotation),
        pass_fail(!has_signoff),
        pass_fail(!has_manifest)
    );
}

fn write_release_summary(
    bench_root: &Path,
    version: &BenchmarkVersion,
    workspace: &Path,
    manifest: Option<&cta_benchmark::BenchmarkManifest>,
    splits: &std::collections::BTreeMap<cta_benchmark::SplitName, cta_benchmark::Split>,
    experiments: &[cta_benchmark::ExperimentConfigSummary],
    issues: &[cta_benchmark::LintIssue],
) -> Result<()> {
    let eval_size = splits
        .get(&cta_benchmark::SplitName::Eval)
        .map_or(0usize, |s| s.instance_ids.len());
    let dev_size = splits
        .get(&cta_benchmark::SplitName::Dev)
        .map_or(0usize, |s| s.instance_ids.len());
    let annotation_summary =
        compute_annotation_coverage(workspace, bench_root, experiments, splits)?;
    let signoff = read_signoff_state(bench_root)?;
    let release_ok = !issues
        .iter()
        .any(|i| i.severity == cta_benchmark::LintSeverity::Error);
    let generated_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());

    let summary = json!({
        "benchmark_version": version.as_str(),
        "manifest_hash": manifest.map(|m| m.content_hash.as_str()),
        "eval_size": eval_size,
        "dev_size": dev_size,
        "annotation_coverage": annotation_summary,
        "gold_signoff": signoff,
        "release_validation": {
            "status": if release_ok { "pass" } else { "fail" }
        },
        "timestamp": generated_at,
    });
    let out = bench_root.join("manifests").join("release_summary.json");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out, serde_json::to_vec_pretty(&summary)?)?;
    println!("release summary: wrote {}", out.display());
    Ok(())
}

fn compute_annotation_coverage(
    workspace: &Path,
    _bench_root: &Path,
    experiments: &[cta_benchmark::ExperimentConfigSummary],
    splits: &std::collections::BTreeMap<cta_benchmark::SplitName, cta_benchmark::Split>,
) -> Result<serde_json::Value> {
    let Some(exp) = experiments
        .iter()
        .find(|e| e.require_full_annotation_coverage && e.annotation_pack.is_some())
    else {
        return Ok(json!({
            "required_pairs": 0,
            "covered_pairs": 0,
            "missing_pairs": 0,
            "missing_examples": []
        }));
    };
    let Some(split_name) = cta_benchmark::SplitName::parse(&exp.split) else {
        return Ok(json!({
            "required_pairs": 0,
            "covered_pairs": 0,
            "missing_pairs": 0,
            "missing_examples": []
        }));
    };
    let Some(split) = splits.get(&split_name) else {
        return Ok(json!({
            "required_pairs": 0,
            "covered_pairs": 0,
            "missing_pairs": 0,
            "missing_examples": []
        }));
    };

    let mut required = std::collections::BTreeSet::new();
    for iid in &split.instance_ids {
        for sid in &exp.systems {
            required.insert((iid.as_str().to_string(), sid.to_string()));
        }
    }

    let pack_abs = workspace.join(exp.annotation_pack.as_deref().unwrap_or_default());
    let available = if pack_abs.is_file() {
        let raw = std::fs::read_to_string(&pack_abs)?;
        let value: serde_json::Value = serde_json::from_str(&raw)?;
        let mut set = std::collections::BTreeSet::new();
        if let Some(records) = value.get("records").and_then(|v| v.as_array()) {
            for rec in records {
                if let (Some(iid), Some(sid)) = (
                    rec.get("instance_id").and_then(|v| v.as_str()),
                    rec.get("system_id").and_then(|v| v.as_str()),
                ) {
                    set.insert((iid.to_string(), sid.to_string()));
                }
            }
        }
        set
    } else {
        std::collections::BTreeSet::new()
    };
    let missing: Vec<_> = required.difference(&available).take(20).cloned().collect();
    let covered = required.iter().filter(|p| available.contains(*p)).count();
    Ok(json!({
        "required_pairs": required.len(),
        "covered_pairs": covered,
        "missing_pairs": required.len().saturating_sub(covered),
        "missing_examples": missing
            .into_iter()
            .map(|(iid, sid)| json!({"instance_id": iid, "system_id": sid}))
            .collect::<Vec<_>>(),
    }))
}

fn read_signoff_state(bench_root: &Path) -> Result<serde_json::Value> {
    let path = bench_root.join("audit").join("gold_signoff.json");
    if !path.is_file() {
        return Ok(json!({
            "primary_reviewer": "",
            "secondary_reviewer": "",
            "approved": false
        }));
    }
    let raw = std::fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    Ok(json!({
        "primary_reviewer": value.get("primary_reviewer").and_then(|v| v.as_str()).unwrap_or(""),
        "secondary_reviewer": value.get("secondary_reviewer").and_then(|v| v.as_str()).unwrap_or(""),
        "approved": value.get("approved").and_then(|v| v.as_bool()).unwrap_or(false),
        "release_gold_audit_status": value.get("release_gold_audit_status").cloned().unwrap_or(json!(null))
    }))
}

fn has_error_with_prefix(issues: &[cta_benchmark::LintIssue], prefix: &str) -> bool {
    issues
        .iter()
        .any(|i| i.severity == cta_benchmark::LintSeverity::Error && i.code.starts_with(prefix))
}

fn pass_fail(ok: bool) -> &'static str {
    if ok {
        "pass"
    } else {
        "fail"
    }
}
