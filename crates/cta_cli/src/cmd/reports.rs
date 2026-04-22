use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;
use cta_metrics::ResultsBundle;
use cta_reports::render_all;

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
