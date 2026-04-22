use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;
use cta_annotations::{adjudicate_set, load_dir, AdjudicationPolicy, AnnotationPack};
use cta_core::BenchmarkVersion;
use cta_schema::SchemaRegistry;

use crate::parse_bench_version;

#[derive(Debug, Args)]
pub struct PackArgs {
    /// Benchmark version (e.g. `v0.1`).
    #[arg(long, value_parser = parse_bench_version)]
    pub version: BenchmarkVersion,
    /// Root directory of annotation files (recursive).
    /// Defaults to `benchmark/<version>/annotation/adjudicated_subset`.
    #[arg(long)]
    pub input: Option<PathBuf>,
    /// Output path for the adjudicated pack JSON. Defaults to
    /// `runs/annotation_packs/<version>-adjudicated.json`.
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Adjudication policy: `prefer-adjudicator` (default) or `majority`.
    #[arg(long, default_value = "prefer-adjudicator")]
    pub policy: String,
}

pub fn pack(workspace: &Path, args: PackArgs) -> Result<()> {
    let policy = match args.policy.as_str() {
        "prefer-adjudicator" => AdjudicationPolicy::PreferAdjudicator,
        "majority" => AdjudicationPolicy::AlwaysMajority,
        other => {
            anyhow::bail!("unknown --policy {other}; expected prefer-adjudicator or majority")
        }
    };

    let input = args.input.unwrap_or_else(|| {
        workspace
            .join("benchmark")
            .join(args.version.as_str())
            .join("annotation")
            .join("adjudicated_subset")
    });

    let schemas = workspace.join("schemas");
    let registry = SchemaRegistry::load(&schemas)
        .with_context(|| format!("loading schemas from {}", schemas.display()))?;
    let set = load_dir(&input, &registry)
        .with_context(|| format!("loading annotations from {}", input.display()))?;
    let adjudicated = adjudicate_set(&set, policy).context("adjudicating annotation groups")?;
    let pack = AnnotationPack::from_adjudicated(&adjudicated)?;

    let out = args.out.unwrap_or_else(|| {
        workspace
            .join("runs")
            .join("annotation_packs")
            .join(format!("{}-adjudicated.json", args.version.as_str()))
    });
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&pack)?;
    std::fs::write(&out, &text)?;

    println!(
        "annotate pack: wrote {records} adjudicated records ({total} source files) to {path}",
        records = pack.records.len(),
        total = set.total_records(),
        path = out.display()
    );
    Ok(())
}
