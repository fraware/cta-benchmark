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
use super::{annotate, reports, validate};
use serde::Deserialize;

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
        let manifest = load_manifest(&root)
            .with_context(|| format!("loading manifest under {}", root.display()))?;
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
        report.issues.extend(release_report.issues);
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

#[derive(Debug, Args)]
pub struct PaperOrchestrateArgs {
    /// Benchmark version for the paper-track workflow.
    #[arg(long, default_value = "v0.2", value_parser = crate::parse_bench_version)]
    pub benchmark_version: BenchmarkVersion,
    /// Experiment config used by annotation planning/coverage and packaging metadata.
    #[arg(long, default_value = "configs/experiments/benchmark_v1.json")]
    pub experiment_config: std::path::PathBuf,
    /// Canonical run IDs to package into paper artifacts.
    #[arg(long, value_delimiter = ',', required = true)]
    pub canonical_run_ids: Vec<String>,
    /// Optional task-board output override.
    #[arg(long)]
    pub task_board_out: Option<std::path::PathBuf>,
    /// Optional adjudicated subset output override.
    #[arg(long)]
    pub adjudicated_out: Option<std::path::PathBuf>,
    /// Optional path override for adjudicated pack.
    #[arg(long)]
    pub pack: Option<std::path::PathBuf>,
    /// Optional final paper package output override.
    #[arg(long)]
    pub paper_out: Option<std::path::PathBuf>,
    /// Optional figures source directory for package step.
    #[arg(long)]
    pub figures_source: Option<std::path::PathBuf>,
    /// Optional review-packets root override for verification gate.
    #[arg(long)]
    pub review_packets_root: Option<std::path::PathBuf>,
    /// Optional schema override for review packet verification.
    #[arg(long)]
    pub review_packet_schema: Option<std::path::PathBuf>,
    /// Optional output override for signed verification summary.
    #[arg(long)]
    pub review_packets_verification_out: Option<std::path::PathBuf>,
}

pub fn paper_orchestrate(workspace: &Path, args: PaperOrchestrateArgs) -> Result<()> {
    let task_board_out = args.task_board_out.unwrap_or_else(|| {
        workspace
            .join("benchmark")
            .join(args.benchmark_version.as_str())
            .join("annotation")
            .join("task_board")
    });
    let adjudicated_out = args.adjudicated_out.unwrap_or_else(|| {
        workspace
            .join("benchmark")
            .join(args.benchmark_version.as_str())
            .join("annotation")
            .join("adjudicated_subset")
    });
    let pack_path = args
        .pack
        .unwrap_or_else(|| adjudicated_out.join("pack.json"));
    let missing_pairs_path = task_board_out.join("missing_pairs.json");
    let batches_out = task_board_out.join("batches");
    let review_packets_root = args.review_packets_root.unwrap_or_else(|| {
        workspace
            .join("benchmark")
            .join(args.benchmark_version.as_str())
            .join("annotation")
            .join("review_packets")
    });
    let review_packet_schema = args
        .review_packet_schema
        .unwrap_or_else(|| workspace.join("schemas").join("review_packet.schema.json"));
    let review_packets_verification_out = args
        .review_packets_verification_out
        .unwrap_or_else(|| review_packets_root.join("verification_summary.signed.json"));

    println!("orchestrate: phase 1/6 annotate plan");
    annotate::plan(
        workspace,
        annotate::PlanArgs {
            benchmark_version: args.benchmark_version.clone(),
            experiment_config: args.experiment_config.clone(),
            out: task_board_out.clone(),
            pack: Some(pack_path.clone()),
        },
    )?;

    println!("orchestrate: phase 2/6 annotate batches");
    annotate::batches(
        workspace,
        annotate::BatchesArgs {
            benchmark_version: args.benchmark_version.clone(),
            missing_pairs: missing_pairs_path,
            out: batches_out,
        },
    )?;

    println!("orchestrate: phase 3/6 annotate coverage");
    annotate::coverage(
        workspace,
        annotate::CoverageArgs {
            benchmark_version: args.benchmark_version.clone(),
            experiment_config: args.experiment_config.clone(),
            pack: pack_path,
            out: adjudicated_out,
        },
    )?;

    println!("orchestrate: phase 4/7 validate benchmark --release");
    validate::benchmark(
        workspace,
        validate::BenchmarkArgs {
            version: args.benchmark_version.clone(),
            release: true,
            rubric: "rubric_v1".to_string(),
            metrics: cta_metrics::METRICS_VERSION.to_string(),
        },
    )?;

    println!("orchestrate: phase 5/7 annotate refresh-lean-check (strict M1)");
    let proof_dashboard_json = review_packets_root.join("proof_completion_dashboard.json");
    let proof_dashboard_csv = review_packets_root.join("proof_completion_dashboard.csv");
    let wave1_worklist_json = review_packets_root.join("wave1_proof_worklist.json");
    let wave1_worklist_csv = review_packets_root.join("wave1_proof_worklist.csv");
    let global_worklist_json = review_packets_root.join("global_proof_worklist.json");
    let global_worklist_csv = review_packets_root.join("global_proof_worklist.csv");
    let execution_plan_json = review_packets_root.join("proof_execution_plan.json");
    annotate::refresh_lean_check(
        workspace,
        annotate::RefreshLeanCheckArgs {
            benchmark_version: args.benchmark_version.clone(),
            packets_root: review_packets_root.clone(),
            dashboard_json: Some(proof_dashboard_json.clone()),
            dashboard_csv: Some(proof_dashboard_csv.clone()),
            wave1_worklist_json: Some(wave1_worklist_json.clone()),
            wave1_worklist_csv: Some(wave1_worklist_csv.clone()),
            global_worklist_json: Some(global_worklist_json.clone()),
            global_worklist_csv: Some(global_worklist_csv.clone()),
            execution_plan_json: Some(execution_plan_json.clone()),
            axiomize_wave1_admits: false,
            strict_m1: true,
        },
    )?;
    if let Ok(raw) = std::fs::read_to_string(&proof_dashboard_json) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            let elaborated = v
                .get("elaborated_packets")
                .and_then(|x| x.as_u64())
                .unwrap_or(0);
            let total = v.get("total_packets").and_then(|x| x.as_u64()).unwrap_or(0);
            let violations = v
                .get("strict_m1_violations")
                .and_then(|x| x.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            println!(
                "orchestrate: proof readiness elaborated={}/{} strict_m1_violations={}",
                elaborated, total, violations
            );
            println!(
                "orchestrate: proof dashboard json={} csv={}",
                proof_dashboard_json.display(),
                proof_dashboard_csv.display()
            );
            if let Some(batch) = v.get("wave1_next_batch").and_then(|x| x.as_array()) {
                println!(
                    "orchestrate: wave1 next proving batch={} json={} csv={}",
                    batch.len(),
                    wave1_worklist_json.display(),
                    wave1_worklist_csv.display()
                );
            }
            if let Some(batch) = v.get("all_next_batch").and_then(|x| x.as_array()) {
                println!(
                    "orchestrate: global next proving batch={} json={} csv={}",
                    batch.len(),
                    global_worklist_json.display(),
                    global_worklist_csv.display()
                );
            }
            println!(
                "orchestrate: grouped execution plan={}",
                execution_plan_json.display()
            );
        }
    }

    println!("orchestrate: phase 6/7 annotate verify-review-packets");
    annotate::verify_review_packets(
        workspace,
        annotate::VerifyReviewPacketsArgs {
            benchmark_version: args.benchmark_version.clone(),
            packets_root: review_packets_root,
            schema: review_packet_schema,
            out: review_packets_verification_out,
        },
    )?;

    println!("orchestrate: phase 7/7 reports package");
    reports::package(
        workspace,
        reports::PackageArgs {
            benchmark_version: args.benchmark_version,
            canonical_run_ids: args.canonical_run_ids,
            experiment_config: args.experiment_config,
            runs_root: None,
            out: args.paper_out,
            figures_source: args.figures_source,
        },
    )?;

    println!("orchestrate: complete");
    Ok(())
}

#[derive(Debug, Args)]
pub struct AuditWorkbookArgs {
    /// Benchmark version (e.g. `v0.2`).
    #[arg(long, default_value = "v0.2", value_parser = crate::parse_bench_version)]
    pub version: BenchmarkVersion,
    /// Output directory for audit evidence CSVs.
    #[arg(long)]
    pub out: Option<std::path::PathBuf>,
}

#[derive(Debug, Deserialize)]
struct SplitJson {
    instance_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct InstanceJson {
    domain: String,
}

#[derive(Debug, Deserialize)]
struct ObligationSetJson {
    obligations: Vec<ObligationJson>,
}

#[derive(Debug, Deserialize)]
struct ObligationJson {
    id: String,
    kind: String,
    linked_semantic_units: Vec<String>,
}

pub fn audit_workbook(workspace: &Path, args: AuditWorkbookArgs) -> Result<()> {
    let bench_root = benchmark_dir(workspace, args.version.as_str());
    let out_dir = args
        .out
        .unwrap_or_else(|| bench_root.join("audit").join("evidence"));
    std::fs::create_dir_all(&out_dir)?;

    let eval_path = bench_root.join("splits").join("eval.json");
    let eval_raw = std::fs::read_to_string(&eval_path)
        .with_context(|| format!("reading {}", eval_path.display()))?;
    let eval: SplitJson = serde_json::from_str(&eval_raw)
        .with_context(|| format!("parsing {}", eval_path.display()))?;

    let mut per_instance_lines = vec![
        "instance_id,domain,reviewed_by_primary,reviewed_by_secondary,status,critical_units_ok,gold_obligations_ok,harness_ok,notes".to_string(),
    ];
    let mut obligation_lines = vec![
        "instance_id,obligation_id,kind,linked_semantic_units,primary_review,secondary_review,disposition,notes".to_string(),
    ];
    for iid in &eval.instance_ids {
        let inst_path = find_instance_json(&bench_root, iid)?;
        let inst_raw = std::fs::read_to_string(&inst_path)
            .with_context(|| format!("reading {}", inst_path.display()))?;
        let inst: InstanceJson = serde_json::from_str(&inst_raw)
            .with_context(|| format!("parsing {}", inst_path.display()))?;
        per_instance_lines.push(format!(
            "{iid},{domain},,,pending,,,,",
            domain = inst.domain
        ));

        let obl_path = inst_path
            .parent()
            .unwrap_or(&bench_root)
            .join("reference_obligations.json");
        let obl_raw = std::fs::read_to_string(&obl_path)
            .with_context(|| format!("reading {}", obl_path.display()))?;
        let obligations: ObligationSetJson = serde_json::from_str(&obl_raw)
            .with_context(|| format!("parsing {}", obl_path.display()))?;
        for obl in obligations.obligations {
            obligation_lines.push(format!(
                "{iid},{oid},{kind},\"{links}\",,,,",
                oid = obl.id,
                kind = obl.kind,
                links = obl.linked_semantic_units.join(";")
            ));
        }
    }

    std::fs::write(
        out_dir.join("per_instance_audit.csv"),
        per_instance_lines.join("\n"),
    )?;
    std::fs::write(
        out_dir.join("obligation_audit.csv"),
        obligation_lines.join("\n"),
    )?;
    println!(
        "benchmark audit-workbook: wrote {} and {}",
        out_dir.join("per_instance_audit.csv").display(),
        out_dir.join("obligation_audit.csv").display()
    );
    Ok(())
}

fn find_instance_json(bench_root: &Path, instance_id: &str) -> Result<std::path::PathBuf> {
    let instances_root = bench_root.join("instances");
    for entry in walkdir::WalkDir::new(&instances_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        if entry.file_name() == "instance.json" {
            let path = entry.path();
            if path
                .parent()
                .and_then(std::path::Path::file_name)
                .and_then(std::ffi::OsStr::to_str)
                == Some(instance_id)
            {
                return Ok(path.to_path_buf());
            }
        }
    }
    anyhow::bail!(
        "instance {} not found under {}",
        instance_id,
        instances_root.display()
    )
}
