use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;
use cta_metrics::ResultsBundle;
use cta_reports::{
    aggregate_by_system, domain_breakdown, domain_breakdown_latex, paired_deltas,
    paired_deltas_csv, provider_breakdown, provider_breakdown_latex, render_all,
    summary_primary_latex, BootstrapConfig, RunSummary,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Run id to build reports for. Equivalent to `--bundle runs/<run>/results_bundle.json`.
    #[arg(long, conflicts_with = "bundle")]
    pub run: Option<String>,
    /// Explicit path to a `results_bundle.json`.
    #[arg(long)]
    pub bundle: Option<PathBuf>,
    /// Directory to write report artifacts into. Defaults to `runs/<run>/reports/`.
    #[arg(long)]
    pub out: Option<PathBuf>,
}

pub fn build(workspace: &Path, args: BuildArgs) -> Result<()> {
    let bundle_path = if let Some(p) = args.bundle.clone() {
        p
    } else if let Some(run) = args.run.as_ref() {
        workspace.join("runs").join(run).join("results_bundle.json")
    } else {
        anyhow::bail!("either --run or --bundle must be supplied");
    };
    let out_dir = args.out.unwrap_or_else(|| {
        bundle_path
            .parent()
            .map_or_else(|| PathBuf::from("reports"), |p| p.join("reports"))
    });

    let raw = std::fs::read(&bundle_path)
        .with_context(|| format!("reading bundle at {}", bundle_path.display()))?;
    let bundle: ResultsBundle = serde_json::from_slice(&raw)
        .with_context(|| format!("parsing bundle at {}", bundle_path.display()))?;
    let system_id = bundle
        .run_manifest
        .get("system_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("system_unknown_v0")
        .to_string();

    let rendered = render_all(&system_id, &bundle);
    std::fs::create_dir_all(&out_dir)?;
    let primary_csv = out_dir.join("primary_metrics.csv");
    let instance_csv = out_dir.join("instance_results.csv");
    let markdown = out_dir.join("results.md");
    let latex = out_dir.join("results.tex");
    std::fs::write(&primary_csv, &rendered.primary_csv)?;
    std::fs::write(&instance_csv, &rendered.instance_csv)?;
    std::fs::write(&markdown, &rendered.markdown)?;
    std::fs::write(&latex, &rendered.latex)?;

    println!(
        "reports build: wrote\n  {a}\n  {b}\n  {c}\n  {d}",
        a = primary_csv.display(),
        b = instance_csv.display(),
        c = markdown.display(),
        d = latex.display()
    );
    Ok(())
}

#[derive(Debug, Args)]
pub struct AggregateArgs {
    /// Root directory holding `run_*/` subdirectories. Defaults to `runs/`.
    #[arg(long)]
    pub runs_root: Option<PathBuf>,
    /// Output directory for aggregate artifacts. Defaults to
    /// `reports/aggregate/`.
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Bootstrap resamples for CIs. `0` disables CI computation.
    #[arg(long, default_value_t = 1000)]
    pub bootstrap_resamples: usize,
    /// Bootstrap seed for reproducibility.
    #[arg(long, default_value_t = 0x5EED_CAFE_B00F_F117u64)]
    pub bootstrap_seed: u64,
    /// Paired-delta system pairs, comma-separated. Each entry is
    /// `a=<system_a>,b=<system_b>` and produces a dedicated CSV at
    /// `<out>/paired_deltas__<a>__<b>.csv`.
    #[arg(long, value_delimiter = ';')]
    pub paired: Vec<String>,
}

#[derive(Debug, Args)]
pub struct PackageArgs {
    /// Benchmark version to package, e.g. v0.2.
    #[arg(long, default_value = "v0.2", value_parser = crate::parse_bench_version)]
    pub benchmark_version: cta_core::BenchmarkVersion,
    /// Canonical run ids to include (comma-separated).
    #[arg(long, value_delimiter = ',', required = true)]
    pub canonical_run_ids: Vec<String>,
    /// Optional experiment config (used for systems/providers/seeds metadata).
    #[arg(long, default_value = "configs/experiments/benchmark_v1.json")]
    pub experiment_config: PathBuf,
    /// Optional runs root override.
    #[arg(long)]
    pub runs_root: Option<PathBuf>,
    /// Output directory for paper artifacts.
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Optional source directory for figure PDFs to copy into figures/.
    #[arg(long)]
    pub figures_source: Option<PathBuf>,
}

pub fn aggregate(workspace: &Path, args: AggregateArgs) -> Result<()> {
    let runs_root = args.runs_root.unwrap_or_else(|| workspace.join("runs"));
    let out_dir = args
        .out
        .unwrap_or_else(|| workspace.join("reports").join("aggregate"));

    let summaries = discover_run_summaries(&runs_root)
        .with_context(|| format!("discovering runs under {}", runs_root.display()))?;
    if summaries.is_empty() {
        anyhow::bail!(
            "reports aggregate: no run summaries found under {}; did you run any experiments?",
            runs_root.display()
        );
    }

    let cfg = BootstrapConfig {
        resamples: args.bootstrap_resamples,
        seed: args.bootstrap_seed,
        confidence: 0.95,
    };

    std::fs::create_dir_all(&out_dir)?;

    let per_system = aggregate_by_system(&summaries, cfg);
    std::fs::write(
        out_dir.join("summary_primary.tex"),
        summary_primary_latex(&per_system),
    )?;
    std::fs::write(
        out_dir.join("summary_primary.json"),
        serde_json::to_string_pretty(&per_system)?,
    )?;

    let providers = provider_breakdown(&summaries, cfg);
    std::fs::write(
        out_dir.join("provider_breakdown.tex"),
        provider_breakdown_latex(&providers),
    )?;

    let domains = domain_breakdown(&summaries, cfg);
    std::fs::write(
        out_dir.join("domain_breakdown.tex"),
        domain_breakdown_latex(&domains),
    )?;

    for pair in &args.paired {
        let Some((a, b)) = parse_paired_spec(pair) else {
            anyhow::bail!(
                "reports aggregate: invalid --paired entry {pair:?}; expected a=<sys_a>,b=<sys_b>"
            );
        };
        let deltas = paired_deltas(&summaries, &a, &b);
        let csv = paired_deltas_csv(&deltas);
        let fname = format!("paired_deltas__{a}__{b}.csv");
        std::fs::write(out_dir.join(&fname), csv)?;
        println!("reports aggregate: wrote paired deltas for ({a}, {b}) -> {fname}");
    }

    println!(
        "reports aggregate: wrote {n} systems, {p} provider rows, {d} domain rows into {dir}",
        n = per_system.len(),
        p = providers.len(),
        d = domains.len(),
        dir = out_dir.display()
    );
    Ok(())
}

pub fn package(workspace: &Path, args: PackageArgs) -> Result<()> {
    let runs_root = args.runs_root.unwrap_or_else(|| workspace.join("runs"));
    let out_dir = args
        .out
        .unwrap_or_else(|| workspace.join("reports").join("paper_v0.2"));
    let tables_dir = out_dir.join("tables");
    let figures_dir = out_dir.join("figures");
    let appendices_dir = out_dir.join("appendices");
    std::fs::create_dir_all(&tables_dir)?;
    std::fs::create_dir_all(&figures_dir)?;
    std::fs::create_dir_all(&appendices_dir)?;

    let all_runs = discover_run_summaries(&runs_root)
        .with_context(|| format!("discovering runs under {}", runs_root.display()))?;
    let wanted: HashSet<&str> = args.canonical_run_ids.iter().map(String::as_str).collect();
    let selected: Vec<RunSummary> = all_runs
        .into_iter()
        .filter(|r| wanted.contains(r.run_id.as_str()))
        .collect();
    if selected.is_empty() {
        anyhow::bail!("reports package: no canonical run ids resolved under runs/");
    }
    let found: BTreeSet<&str> = selected.iter().map(|r| r.run_id.as_str()).collect();
    let missing: Vec<&str> = wanted
        .iter()
        .copied()
        .filter(|run_id| !found.contains(run_id))
        .collect();
    if !missing.is_empty() {
        anyhow::bail!("reports package: missing canonical runs: {missing:?}");
    }

    let cfg = BootstrapConfig {
        resamples: 1000,
        seed: 0x5EED_CAFE_B00F_F117,
        confidence: 0.95,
    };
    let primary = aggregate_by_system(&selected, cfg);
    let providers = provider_breakdown(&selected, cfg);
    let domains = domain_breakdown(&selected, cfg);
    let (pair_specs, pair_rows) = paired_tables(&selected);

    std::fs::write(
        tables_dir.join("primary_metrics.tex"),
        summary_primary_latex(&primary),
    )?;
    std::fs::write(
        tables_dir.join("provider_breakdown.tex"),
        provider_breakdown_latex(&providers),
    )?;
    std::fs::write(
        tables_dir.join("domain_breakdown.tex"),
        domain_breakdown_latex(&domains),
    )?;
    std::fs::write(tables_dir.join("paired_deltas.tex"), pair_rows)?;

    let coverage = load_json(
        &workspace
            .join("benchmark")
            .join(args.benchmark_version.as_str())
            .join("annotation")
            .join("adjudicated_subset")
            .join("coverage_summary.json"),
    )?;
    std::fs::write(
        tables_dir.join("annotation_coverage.tex"),
        annotation_coverage_table(&coverage),
    )?;

    let release_summary = load_json(
        &workspace
            .join("benchmark")
            .join(args.benchmark_version.as_str())
            .join("manifests")
            .join("release_summary.json"),
    )?;
    let signoff = load_json(
        &workspace
            .join("benchmark")
            .join(args.benchmark_version.as_str())
            .join("audit")
            .join("gold_signoff.json"),
    )?;
    let pack = load_json(
        &workspace
            .join("benchmark")
            .join(args.benchmark_version.as_str())
            .join("annotation")
            .join("adjudicated_subset")
            .join("pack.json"),
    )?;
    let exp = load_experiment_config(&args.experiment_config)?;

    std::fs::write(
        appendices_dir.join("adjudicated_examples.md"),
        adjudicated_examples_md(&pack),
    )?;
    std::fs::write(
        appendices_dir.join("gold_audit_summary.md"),
        gold_audit_summary_md(&signoff),
    )?;
    std::fs::write(
        appendices_dir.join("benchmark_release_summary.md"),
        benchmark_release_summary_md(&release_summary),
    )?;

    copy_figures(args.figures_source.as_ref(), &figures_dir)?;
    let paper_summary = json!({
        "benchmark_version": args.benchmark_version.as_str(),
        "manifest_hash": release_summary.get("manifest_hash").and_then(|v| v.as_str()).unwrap_or(""),
        "eval_size": release_summary.get("eval_size").and_then(|v| v.as_u64()).unwrap_or(0),
        "systems": exp.systems,
        "providers": exp.providers,
        "seeds": exp.seeds,
        "required_annotation_pairs": coverage.get("required_pairs").and_then(|v| v.as_u64()).unwrap_or(0),
        "covered_annotation_pairs": coverage.get("covered_pairs").and_then(|v| v.as_u64()).unwrap_or(0),
        "gold_signoff_state": {
            "primary_reviewer": signoff.get("primary_reviewer").and_then(|v| v.as_str()).unwrap_or(""),
            "secondary_reviewer": signoff.get("secondary_reviewer").and_then(|v| v.as_str()).unwrap_or(""),
            "approved": signoff.get("approved").and_then(|v| v.as_bool()).unwrap_or(false),
        },
        "canonical_run_ids": args.canonical_run_ids,
        "paired_specs": pair_specs,
        "date": time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
    });
    std::fs::write(
        out_dir.join("paper_summary.json"),
        serde_json::to_vec_pretty(&paper_summary)?,
    )?;

    println!(
        "reports package: wrote paper artifact bundle at {}",
        out_dir.display()
    );
    Ok(())
}

fn discover_run_summaries(runs_root: &Path) -> Result<Vec<RunSummary>> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(runs_root) else {
        return Ok(out);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let bundle_path = path.join("results_bundle.json");
        let manifest_path = path.join("run_manifest.json");
        if !bundle_path.exists() {
            continue;
        }
        let bundle_raw = std::fs::read(&bundle_path)
            .with_context(|| format!("reading {}", bundle_path.display()))?;
        let bundle: ResultsBundle = match serde_json::from_slice::<ResultsBundle>(&bundle_raw) {
            Ok(b) => b,
            Err(e) => {
                eprintln!(
                    "reports aggregate: skipping {} ({}); run it again under the current \
                     metrics contract to include it in aggregates",
                    bundle_path.display(),
                    e
                );
                continue;
            }
        };
        if bundle.aggregate_metrics.metrics_version != cta_metrics::METRICS_VERSION {
            eprintln!(
                "reports aggregate: skipping {} (metrics_version={}; expected {})",
                bundle_path.display(),
                bundle.aggregate_metrics.metrics_version,
                cta_metrics::METRICS_VERSION,
            );
            continue;
        }

        let manifest: serde_json::Value = if manifest_path.exists() {
            let raw = std::fs::read(&manifest_path)
                .with_context(|| format!("reading {}", manifest_path.display()))?;
            serde_json::from_slice(&raw)
                .with_context(|| format!("parsing {}", manifest_path.display()))?
        } else {
            bundle.run_manifest.clone()
        };

        let run_id = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("run_unknown")
            .to_string();
        let system_id = RunSummary::system_from_manifest(&manifest)
            .unwrap_or_else(|| "system_unknown".to_string());
        let provider = RunSummary::provider_from_manifest(&manifest);
        let seed = RunSummary::seed_from_manifest(&manifest);
        let split = manifest
            .get("split")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| infer_split_from_run_id(&run_id))
            .to_string();

        out.push(RunSummary {
            run_id,
            system_id,
            provider,
            split,
            seed,
            bundle,
        });
    }
    out.sort_by(|a, b| a.run_id.cmp(&b.run_id));
    Ok(out)
}

fn infer_split_from_run_id(run_id: &str) -> &str {
    // run_<date>_<system>_<split>_<nnn>
    let parts: Vec<&str> = run_id.split('_').collect();
    if parts.len() >= 2 {
        parts[parts.len() - 2]
    } else {
        "unknown"
    }
}

fn parse_paired_spec(spec: &str) -> Option<(String, String)> {
    let mut a = None;
    let mut b = None;
    for part in spec.split(',') {
        let (k, v) = part.split_once('=')?;
        match k.trim() {
            "a" => a = Some(v.trim().to_string()),
            "b" => b = Some(v.trim().to_string()),
            _ => return None,
        }
    }
    match (a, b) {
        (Some(a), Some(b)) => Some((a, b)),
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
struct ExperimentConfigMeta {
    systems: Vec<String>,
    providers: Vec<String>,
    seeds: Vec<u64>,
}

fn load_experiment_config(path: &Path) -> Result<ExperimentConfigMeta> {
    let raw =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let exp = serde_json::from_str::<ExperimentConfigMeta>(&raw)
        .with_context(|| format!("parsing {}", path.display()))?;
    Ok(exp)
}

fn load_json(path: &Path) -> Result<serde_json::Value> {
    let raw =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?)
}

fn annotation_coverage_table(coverage: &serde_json::Value) -> String {
    format!(
        "\\begin{{tabular}}{{lrr}}\\n\\toprule\\nmetric & value \\\\n\\midrule\\nrequired\\_pairs & {} \\\\ncovered\\_pairs & {} \\\\nmissing\\_pairs & {} \\\\n\\bottomrule\\n\\end{{tabular}}\\n",
        coverage.get("required_pairs").and_then(|v| v.as_u64()).unwrap_or(0),
        coverage.get("covered_pairs").and_then(|v| v.as_u64()).unwrap_or(0),
        coverage.get("missing_pairs").and_then(|v| v.as_u64()).unwrap_or(0),
    )
}

fn adjudicated_examples_md(pack: &serde_json::Value) -> String {
    let mut out = String::from("# Adjudicated examples\n\n");
    if let Some(records) = pack.get("records").and_then(|v| v.as_array()) {
        for rec in records.iter().take(10) {
            let iid = rec
                .get("instance_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let sid = rec.get("system_id").and_then(|v| v.as_str()).unwrap_or("");
            out.push_str(&format!("- `{iid}` / `{sid}`\n"));
        }
    }
    out
}

fn gold_audit_summary_md(signoff: &serde_json::Value) -> String {
    format!(
        "# Gold audit summary\n\n- primary reviewer: `{}`\n- secondary reviewer: `{}`\n- approved: `{}`\n",
        signoff.get("primary_reviewer").and_then(|v| v.as_str()).unwrap_or(""),
        signoff.get("secondary_reviewer").and_then(|v| v.as_str()).unwrap_or(""),
        signoff.get("approved").and_then(|v| v.as_bool()).unwrap_or(false),
    )
}

fn benchmark_release_summary_md(release: &serde_json::Value) -> String {
    let status = release
        .get("release_validation")
        .and_then(|v| v.get("status"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    format!(
        "# Benchmark release summary\n\n- benchmark version: `{}`\n- manifest hash: `{}`\n- eval size: `{}`\n- dev size: `{}`\n- release validation: `{status}`\n",
        release.get("benchmark_version").and_then(|v| v.as_str()).unwrap_or(""),
        release.get("manifest_hash").and_then(|v| v.as_str()).unwrap_or(""),
        release.get("eval_size").and_then(|v| v.as_u64()).unwrap_or(0),
        release.get("dev_size").and_then(|v| v.as_u64()).unwrap_or(0),
    )
}

fn paired_tables(runs: &[RunSummary]) -> (Vec<String>, String) {
    let systems: BTreeSet<&str> = runs.iter().map(|r| r.system_id.as_str()).collect();
    let mut specs = Vec::new();
    let mut rows = String::from(
        "\\begin{tabular}{llrrr}\n\\toprule\nsystem a & system b & mean $\\Delta$ faith & mean $\\Delta$ cons & mean $\\Delta$ cov \\\\\n\\midrule\n",
    );
    let preferred = [
        ("text_only_v1", "full_method_v1"),
        ("code_only_v1", "full_method_v1"),
        ("naive_concat_v1", "full_method_v1"),
    ];
    for (a, b) in preferred {
        if systems.contains(a) && systems.contains(b) {
            let deltas = paired_deltas(runs, a, b);
            if deltas.is_empty() {
                continue;
            }
            let n = deltas.len() as f64;
            let (mut f, mut c, mut cov) = (0.0, 0.0, 0.0);
            for d in deltas {
                f += d.delta_faithfulness_fraction;
                c += d.delta_consistency_fraction;
                cov += d.delta_coverage_fraction;
            }
            rows.push_str(&format!(
                "{a} & {b} & {:.3} & {:.3} & {:.3} \\\\\n",
                f / n,
                c / n,
                cov / n
            ));
            specs.push(format!("a={a},b={b}"));
        }
    }
    rows.push_str("\\bottomrule\n\\end{tabular}\n");
    (specs, rows)
}

fn copy_figures(figures_source: Option<&PathBuf>, figures_dir: &Path) -> Result<()> {
    let expected = [
        "main_results.pdf",
        "domain_results.pdf",
        "provider_results.pdf",
        "failure_taxonomy.pdf",
        "annotation_progress.pdf",
    ];
    if let Some(source) = figures_source {
        for name in expected {
            let src = source.join(name);
            if src.is_file() {
                std::fs::copy(&src, figures_dir.join(name))
                    .with_context(|| format!("copying {}", src.display()))?;
            }
        }
    }
    let mut missing = Vec::new();
    for name in expected {
        if !figures_dir.join(name).is_file() {
            missing.push(name);
        }
    }
    if !missing.is_empty() {
        std::fs::write(
            figures_dir.join("README.md"),
            format!(
                "# Missing figure PDFs\n\nThe following files were not present at package time:\n{}\n",
                missing
                    .into_iter()
                    .map(|m| format!("- `{m}`"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
        )?;
    }
    Ok(())
}
