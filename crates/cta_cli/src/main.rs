//! `cta` — single CLI entry point for the CTA benchmark toolchain.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use cta_core::BenchmarkVersion;
use tracing_subscriber::EnvFilter;

mod cmd;

#[derive(Debug, Parser)]
#[command(name = "cta", version, about = "CTA benchmark toolchain", long_about = None)]
struct Cli {
    /// Path to workspace root. Defaults to the current working directory.
    #[arg(long, global = true, env = "CTA_WORKSPACE")]
    workspace: Option<PathBuf>,

    /// Verbosity (repeat for more).
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate schemas and schema-governed artifacts.
    #[command(subcommand)]
    Validate(ValidateCmd),

    /// Benchmark-level operations.
    #[command(subcommand)]
    Benchmark(BenchmarkCmd),

    /// Rust semantic extraction.
    #[command(subcommand)]
    Extract(ExtractCmd),

    /// Run a single generation system for a split.
    Generate(cmd::generate::GenerateArgs),

    /// Lean elaboration operations.
    #[command(subcommand)]
    Lean(LeanCmd),

    /// Behavioral harness operations.
    #[command(subcommand)]
    Behavior(BehaviorCmd),

    /// Annotation ingest and packaging.
    #[command(subcommand)]
    Annotate(AnnotateCmd),

    /// Metric computation.
    #[command(subcommand)]
    Metrics(MetricsCmd),

    /// Report generation.
    #[command(subcommand)]
    Reports(ReportsCmd),

    /// Orchestrate a full experiment from a config.
    Experiment(cmd::experiment::ExperimentArgs),
}

#[derive(Debug, Subcommand)]
enum ValidateCmd {
    /// Load all canonical JSON schemas and report any compile errors.
    Schemas(cmd::validate::SchemasArgs),
    /// Validate a loaded benchmark's artifacts against their schemas.
    Benchmark(cmd::validate::BenchmarkArgs),
    /// Validate a single JSON artifact against a named schema.
    File(cmd::validate::FileArgs),
}

#[derive(Debug, Subcommand)]
enum BenchmarkCmd {
    /// Run the benchmark linter.
    Lint(cmd::benchmark::LintArgs),
    /// Print benchmark statistics.
    Stats(cmd::benchmark::StatsArgs),
    /// Compute and write the benchmark manifest.
    Manifest(cmd::benchmark::ManifestArgs),
    /// Run paper-track fail-fast orchestration pipeline.
    PaperOrchestrate(cmd::benchmark::PaperOrchestrateArgs),
    /// Generate gold-audit workbook CSVs for eval split.
    AuditWorkbook(cmd::benchmark::AuditWorkbookArgs),
}

#[derive(Debug, Subcommand)]
enum ExtractCmd {
    /// Extract a Rust summary for a single instance.
    RustSummary(cmd::extract::RustSummaryArgs),
}

#[derive(Debug, Subcommand)]
enum LeanCmd {
    /// Elaborate a Lean file via `lake env lean` and report diagnostics.
    Check(cmd::lean::CheckArgs),
}

#[derive(Debug, Subcommand)]
enum BehaviorCmd {
    /// Run the behavioral harness for a single instance.
    Check(cmd::behavior::CheckArgs),
}

#[derive(Debug, Subcommand)]
enum AnnotateCmd {
    /// Adjudicate and pack annotations into a single canonical pack.
    Pack(cmd::annotate::PackArgs),
    /// Initialize task-board artifacts for annotation closure.
    Plan(cmd::annotate::PlanArgs),
    /// Compute annotation coverage and write summary artifacts.
    Coverage(cmd::annotate::CoverageArgs),
    /// Materialize strict per-system annotation batches.
    Batches(cmd::annotate::BatchesArgs),
    /// Sync adjudicator records from review packets into adjudicated subset.
    SyncReviewPackets(cmd::annotate::SyncReviewPacketsArgs),
    /// Build self-contained review packets for pair list.
    BuildReviewPackets(cmd::annotate::BuildReviewPacketsArgs),
    /// Ingest assistant draft annotations as non-human raw material.
    IngestDraft(cmd::annotate::IngestDraftArgs),
    /// Validate all review packet packet.json files and emit signed summary.
    VerifyReviewPackets(cmd::annotate::VerifyReviewPacketsArgs),
    /// Recompute lean_check metadata and emit proof dashboard.
    RefreshLeanCheck(cmd::annotate::RefreshLeanCheckArgs),
}

#[derive(Debug, Subcommand)]
enum MetricsCmd {
    /// Compute aggregate metrics over a run directory + annotation pack.
    Compute(cmd::metrics::ComputeArgs),
}

#[derive(Debug, Subcommand)]
enum ReportsCmd {
    /// Build CSV/LaTeX/Markdown reports from a results_bundle.json.
    Build(cmd::reports::BuildArgs),
    /// Aggregate all results bundles under `runs/` into paper-ready
    /// cross-run tables (summary, provider breakdown, domain breakdown,
    /// paired deltas).
    Aggregate(cmd::reports::AggregateArgs),
    /// Build paper artifact bundle from canonical run ids.
    Package(cmd::reports::PackageArgs),
}

fn init_tracing(verbose: u8) {
    let level = match verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("cta={level}")));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let workspace = cli
        .workspace
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    load_workspace_env(&workspace);

    let result: anyhow::Result<()> = match cli.cmd {
        Command::Validate(ValidateCmd::Schemas(a)) => cmd::validate::schemas(&workspace, a),
        Command::Validate(ValidateCmd::Benchmark(a)) => cmd::validate::benchmark(&workspace, a),
        Command::Validate(ValidateCmd::File(a)) => cmd::validate::file(&workspace, a),
        Command::Benchmark(BenchmarkCmd::Lint(a)) => cmd::benchmark::lint(&workspace, a),
        Command::Benchmark(BenchmarkCmd::Stats(a)) => cmd::benchmark::stats(&workspace, a),
        Command::Benchmark(BenchmarkCmd::Manifest(a)) => cmd::benchmark::manifest(&workspace, a),
        Command::Benchmark(BenchmarkCmd::PaperOrchestrate(a)) => {
            cmd::benchmark::paper_orchestrate(&workspace, a)
        }
        Command::Benchmark(BenchmarkCmd::AuditWorkbook(a)) => {
            cmd::benchmark::audit_workbook(&workspace, a)
        }
        Command::Extract(ExtractCmd::RustSummary(a)) => cmd::extract::rust_summary(&workspace, a),
        Command::Generate(a) => cmd::generate::run(&workspace, a),
        Command::Lean(LeanCmd::Check(a)) => cmd::lean::check(&workspace, a),
        Command::Behavior(BehaviorCmd::Check(a)) => cmd::behavior::check(&workspace, a),
        Command::Annotate(AnnotateCmd::Pack(a)) => cmd::annotate::pack(&workspace, a),
        Command::Annotate(AnnotateCmd::Plan(a)) => cmd::annotate::plan(&workspace, a),
        Command::Annotate(AnnotateCmd::Coverage(a)) => cmd::annotate::coverage(&workspace, a),
        Command::Annotate(AnnotateCmd::Batches(a)) => cmd::annotate::batches(&workspace, a),
        Command::Annotate(AnnotateCmd::SyncReviewPackets(a)) => {
            cmd::annotate::sync_review_packets(&workspace, a)
        }
        Command::Annotate(AnnotateCmd::BuildReviewPackets(a)) => {
            cmd::annotate::build_review_packets(&workspace, a)
        }
        Command::Annotate(AnnotateCmd::IngestDraft(a)) => {
            cmd::annotate::ingest_draft(&workspace, a)
        }
        Command::Annotate(AnnotateCmd::VerifyReviewPackets(a)) => {
            cmd::annotate::verify_review_packets(&workspace, a)
        }
        Command::Annotate(AnnotateCmd::RefreshLeanCheck(a)) => {
            cmd::annotate::refresh_lean_check(&workspace, a)
        }
        Command::Metrics(MetricsCmd::Compute(a)) => cmd::metrics::compute(&workspace, a),
        Command::Reports(ReportsCmd::Build(a)) => cmd::reports::build(&workspace, a),
        Command::Reports(ReportsCmd::Aggregate(a)) => cmd::reports::aggregate(&workspace, a),
        Command::Reports(ReportsCmd::Package(a)) => cmd::reports::package(&workspace, a),
        Command::Experiment(a) => cmd::experiment::run(&workspace, a),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn load_workspace_env(workspace: &std::path::Path) {
    let env_path = workspace.join(".env");
    if env_path.is_file() {
        let _ = dotenvy::from_path(&env_path);
    }
}

/// Parse a `BenchmarkVersion` from a CLI argument.
fn parse_bench_version(s: &str) -> Result<BenchmarkVersion, String> {
    BenchmarkVersion::new(s).map_err(|e| e.to_string())
}
