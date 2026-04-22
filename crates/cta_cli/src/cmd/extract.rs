use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;
use cta_benchmark::load_benchmark;
use cta_core::{BenchmarkVersion, InstanceId};

use super::benchmark_dir;

#[derive(Debug, Args)]
pub struct RustSummaryArgs {
    #[arg(long, default_value = "v0.1", value_parser = crate::parse_bench_version)]
    pub version: BenchmarkVersion,

    /// Instance id, e.g. `arrays_binary_search_001`.
    #[arg(long)]
    pub instance: String,

    /// Optional output path; default: stdout.
    #[arg(long)]
    pub out: Option<PathBuf>,
}

pub fn rust_summary(workspace: &Path, args: RustSummaryArgs) -> Result<()> {
    let root = benchmark_dir(workspace, args.version.as_str());
    let bench = load_benchmark(&root, &args.version)?;
    let id = InstanceId::new(args.instance.clone())
        .map_err(|e| anyhow::anyhow!("invalid instance id: {e}"))?;
    let view = bench
        .instances
        .get(&id)
        .with_context(|| format!("instance not found: {id}"))?;
    let summary = cta_rust_extract::extract_from_file(
        &view.reference_rs,
        &view.record.rust_reference.entry_fn,
    )?;
    let json = serde_json::to_string_pretty(&summary)?;
    if let Some(out) = args.out {
        if let Some(p) = out.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::write(&out, &json)?;
        println!("wrote {}", out.display());
    } else {
        println!("{json}");
    }
    Ok(())
}
