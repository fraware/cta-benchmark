use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::Args;
use cta_behavior::{AdapterRegistry, HarnessConfig};
use cta_benchmark::loader::load_benchmark;
use cta_core::{BenchmarkVersion, InstanceId};

#[derive(Debug, Args)]
pub struct CheckArgs {
    /// Instance id to run the harness against.
    #[arg(long)]
    pub instance: String,

    /// Benchmark version (e.g. `v0.1`).
    #[arg(long, default_value = "v0.1")]
    pub version: String,

    /// Override the harness config file. If omitted, the adapter uses the
    /// instance's `harness.json`.
    #[arg(long, value_name = "FILE")]
    pub harness: Option<PathBuf>,

    /// Override the number of trials from the harness config.
    #[arg(long)]
    pub trials: Option<u32>,

    /// Override the seed from the harness config.
    #[arg(long)]
    pub seed: Option<u64>,

    /// Optional output path for the JSON report.
    #[arg(long, value_name = "FILE")]
    pub out: Option<PathBuf>,
}

pub fn check(workspace: &Path, args: CheckArgs) -> Result<()> {
    let version = BenchmarkVersion::new(&args.version)
        .map_err(|e| anyhow!("invalid benchmark version: {e}"))?;
    let benchmark = load_benchmark(workspace.join("benchmark").join(&args.version), &version)
        .context("failed to load benchmark")?;
    let instance_id =
        InstanceId::new(&args.instance).map_err(|e| anyhow!("invalid instance id: {e}"))?;
    let view = benchmark
        .instances
        .get(&instance_id)
        .ok_or_else(|| anyhow!("instance not found: {}", args.instance))?;

    let harness_path = args.harness.clone().unwrap_or_else(|| view.harness.clone());
    let harness_raw = std::fs::read_to_string(&harness_path)
        .with_context(|| format!("reading harness config: {}", harness_path.display()))?;
    let mut config: HarnessConfig = serde_json::from_str(&harness_raw)
        .with_context(|| format!("parsing harness config: {}", harness_path.display()))?;
    if let Some(t) = args.trials {
        config.num_trials = t;
    }
    if let Some(s) = args.seed {
        config.seed = s;
    }

    let registry = AdapterRegistry::with_pilot();
    let report =
        cta_behavior::run(&registry, &instance_id, &config).context("harness execution failed")?;

    let json = serde_json::to_string_pretty(&report)?;
    if let Some(out) = &args.out {
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(out, &json)?;
    } else {
        println!("{json}");
    }

    if report.any_falsified() {
        Err(anyhow!(
            "{} falsification(s) across {} oracle(s) over {} trials",
            report.falsifications.len(),
            report.oracle_stats.len(),
            report.trials_run
        ))
    } else {
        Ok(())
    }
}
