use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;
use cta_metrics::ResultsBundle;
use cta_reports::{
    aggregate_by_system, domain_breakdown, domain_breakdown_latex, paired_deltas,
    paired_deltas_csv, provider_breakdown, provider_breakdown_latex, render_all,
    summary_primary_latex, BootstrapConfig, RunSummary,
};

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

pub fn aggregate(workspace: &Path, args: AggregateArgs) -> Result<()> {
    let runs_root = args
        .runs_root
        .unwrap_or_else(|| workspace.join("runs"));
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
