use std::path::Path;

use anyhow::{anyhow, Context, Result};
use clap::Args;
use cta_benchmark::loader::load_benchmark;
use cta_core::{BenchmarkVersion, InstanceId, RunId, SystemId};
use cta_generate::{
    build_context, build_from_config, generate, GenerateParams, PromptTemplate, Provider,
    ProviderConfig, StubProvider,
};
use serde_json::json;

#[derive(Debug, Args)]
pub struct GenerateArgs {
    /// System id (e.g. `full_method_v1`).
    #[arg(long)]
    pub system: String,

    /// Split name (dev | eval | challenge).
    #[arg(long)]
    pub split: String,

    /// Benchmark version.
    #[arg(long, default_value = "v0.1")]
    pub version: String,

    /// Provider name. `stub` uses the offline provider. Otherwise the CLI
    /// loads `configs/providers/<provider>.json` and routes through the
    /// wire-level provider.
    #[arg(long, default_value = "stub")]
    pub provider: String,

    /// Seed forwarded to the provider.
    #[arg(long, default_value_t = 0)]
    pub seed: u64,

    /// Max tokens per request.
    #[arg(long, default_value_t = 2048)]
    pub max_tokens: u32,

    /// Temperature.
    #[arg(long, default_value_t = 0.0)]
    pub temperature: f64,

    /// Override the run id. If omitted, a deterministic one of the form
    /// `run_YYYY_MM_DD_<system>_<split>_001` is synthesized.
    #[arg(long)]
    pub run_id: Option<String>,
}

pub fn run(workspace: &Path, args: GenerateArgs) -> Result<()> {
    let version = BenchmarkVersion::new(&args.version)
        .map_err(|e| anyhow!("invalid benchmark version: {e}"))?;
    let benchmark = load_benchmark(workspace.join("benchmark").join(&args.version), &version)
        .context("failed to load benchmark")?;

    let split_path = workspace
        .join("benchmark")
        .join(&args.version)
        .join("splits")
        .join(format!("{}.json", args.split));
    let split_raw = std::fs::read_to_string(&split_path)
        .with_context(|| format!("reading split: {}", split_path.display()))?;
    let split_json: serde_json::Value = serde_json::from_str(&split_raw)?;
    let instance_ids: Vec<String> = split_json
        .get("instance_ids")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("split {} missing instance_ids", args.split))?
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();

    let system_id = SystemId::new(&args.system)
        .map_err(|e| anyhow!("invalid system id '{}': {e}", args.system))?;
    let prompt_path = workspace
        .join("configs")
        .join("prompts")
        .join(format!("{}.json", args.system));
    let template = PromptTemplate::load(&prompt_path)
        .with_context(|| format!("loading prompt template: {}", prompt_path.display()))?;

    let provider: Box<dyn Provider> = if args.provider == "stub" {
        Box::new(StubProvider::default())
    } else {
        let cfg_path = workspace
            .join("configs")
            .join("providers")
            .join(format!("{}.json", args.provider));
        let cfg_raw = std::fs::read_to_string(&cfg_path)
            .with_context(|| format!("reading provider config: {}", cfg_path.display()))?;
        let cfg: ProviderConfig = serde_json::from_str(&cfg_raw)
            .with_context(|| format!("parsing provider config: {}", cfg_path.display()))?;
        build_from_config(cfg)
    };

    let run_id_str = args.run_id.clone().unwrap_or_else(|| {
        let date = current_date_ymd();
        format!(
            "run_{date}_{system}_{split}_001",
            system = args.system,
            split = args.split,
        )
    });
    let run_id =
        RunId::new(&run_id_str).map_err(|e| anyhow!("invalid run id '{run_id_str}': {e}"))?;

    let run_dir = workspace.join("runs").join(run_id.as_str());
    let generated_dir = run_dir.join("generated").join(args.system.as_str());
    let raw_dir = generated_dir.join("raw");
    std::fs::create_dir_all(&raw_dir)?;

    let mut generated = 0usize;
    for iid in &instance_ids {
        let instance_id =
            InstanceId::new(iid).map_err(|e| anyhow!("invalid instance id '{iid}': {e}"))?;
        let view = benchmark
            .instances
            .get(&instance_id)
            .ok_or_else(|| anyhow!("split references unknown instance: {iid}"))?;
        let informal_statement = view.record.informal_statement.text.clone();
        let ctx = build_context(
            template.kind,
            &view.dir,
            &informal_statement,
            &view.scaffold_lean,
            &view.semantic_units,
        )?;
        let raw_rel = format!("generated/{}/raw/{iid}.txt", args.system);
        let params = GenerateParams {
            run_id: run_id.clone(),
            system_id: system_id.clone(),
            instance_id: instance_id.clone(),
            seed: args.seed,
            max_tokens: args.max_tokens,
            temperature: args.temperature,
            raw_output_path: raw_rel.clone(),
        };
        let outcome = generate(provider.as_ref(), &template, &ctx, &params)
            .with_context(|| format!("generating obligations for {iid}"))?;

        let raw_path = run_dir.join(&raw_rel);
        if let Some(parent) = raw_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&raw_path, &outcome.raw)?;

        let bundle_path = generated_dir.join(format!("{iid}.json"));
        let json = serde_json::to_string_pretty(&outcome.bundle)?;
        std::fs::write(&bundle_path, json)?;
        generated += 1;
    }

    // run_manifest.json
    let manifest = build_run_manifest(
        &run_id,
        &system_id,
        &template,
        provider.as_ref(),
        args.seed,
        args.max_tokens,
        args.temperature,
        &args.version,
    )?;
    let manifest_path = run_dir.join("run_manifest.json");
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

    println!(
        "generated {} bundle(s) for system '{}' split '{}' under {}",
        generated,
        args.system,
        args.split,
        run_dir.display()
    );
    Ok(())
}

fn current_date_ymd() -> String {
    use time::OffsetDateTime;
    let now = OffsetDateTime::now_utc();
    format!(
        "{:04}_{:02}_{:02}",
        now.year(),
        u8::from(now.month()),
        now.day()
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_run_manifest_public(
    run_id: &RunId,
    system_id: &SystemId,
    template: &PromptTemplate,
    provider: &dyn Provider,
    seed: u64,
    max_tokens: u32,
    temperature: f64,
    benchmark_version: &str,
) -> Result<serde_json::Value> {
    build_run_manifest(
        run_id,
        system_id,
        template,
        provider,
        seed,
        max_tokens,
        temperature,
        benchmark_version,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_run_manifest(
    run_id: &RunId,
    system_id: &SystemId,
    template: &PromptTemplate,
    provider: &dyn Provider,
    seed: u64,
    max_tokens: u32,
    temperature: f64,
    benchmark_version: &str,
) -> Result<serde_json::Value> {
    let prompt_hash = cta_generate::hash_prompt(&template.body);
    let repo_commit = detect_repo_commit().unwrap_or_else(|| "0000000".to_string());
    let created_at =
        time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339)?;
    let rust_version = env!("CARGO_PKG_RUST_VERSION").to_string();
    let hostname = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".into());
    let provider_block = json!({
        "name": provider.name(),
        "model": provider.model(),
        "model_version": "unknown"
    });
    Ok(json!({
        "schema_version": "schema_v1",
        "run_id": run_id.as_str(),
        "repo_commit": repo_commit,
        "benchmark_version": benchmark_version,
        "schema_versions": {
            "instance": "schema_v1",
            "obligation": "schema_v1",
            "annotation": "schema_v1",
            "generated_output": "schema_v1",
            "results_bundle": "schema_v1",
            "metrics": cta_metrics::METRICS_VERSION,
            "rubric": "rubric_v1"
        },
        "system_id": system_id.as_str(),
        "provider": provider_block,
        "prompt_template_hash": prompt_hash,
        "seed": seed,
        "generation_parameters": {
            "temperature": temperature,
            "max_tokens": max_tokens
        },
        "toolchains": {
            "rust": rust_version,
            "lean": "leanprover/lean4:v4.12.0"
        },
        "created_at": created_at,
        "runner": {
            "hostname": hostname,
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH
        }
    }))
}

fn detect_repo_commit() -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--short=10", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    if s.chars().all(|c| c.is_ascii_hexdigit()) && s.len() >= 7 {
        Some(s)
    } else {
        None
    }
}
