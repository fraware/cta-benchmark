//! `cta experiment run` — config-driven, end-to-end orchestration of the CTA
//! pipeline.
//!
//! The experiment config enumerates the cartesian product of
//! `systems x providers x seeds`. For every combination, the runner:
//!
//! 1. Loads the declared split and prompt template.
//! 2. Builds a canonical, deterministic `run_id` of the form
//!    `run_YYYY_MM_DD_<system>_<split>_<NNN>`.
//! 3. Generates normalised obligation bundles via [`cta_generate`].
//! 4. Writes a schema-valid `run_manifest.json` alongside the bundles.
//! 5. If the config supplies an `annotation_pack`, computes a results bundle
//!    and renders CSV/Markdown/LaTeX reports.
//!
//! The orchestrator never swallows errors silently — the first failed run
//! terminates the experiment with a non-zero exit.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::Args;
use cta_annotations::AnnotationPack;
use cta_benchmark::loader::load_benchmark;
use cta_core::{BenchmarkVersion, InstanceId, RunId, SystemId};
use cta_generate::{
    build_context, build_from_config, generate, GenerateParams, PromptTemplate, Provider,
    ProviderConfig, StubProvider,
};
use cta_metrics::{compute_results_bundle, InstanceInputs, InstanceSignal, ResultsBundle};
use cta_reports::render_all;
use cta_schema::{SchemaName, SchemaRegistry};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, info_span, instrument};

#[derive(Debug, Args)]
pub struct ExperimentArgs {
    /// Path to an experiment config JSON (validated against `experiment.schema.json`).
    #[arg(long)]
    pub config: PathBuf,

    /// If set, print the materialised run plan but don't execute any runs.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExperimentConfig {
    schema_version: String,
    experiment_id: String,
    benchmark_version: String,
    split: String,
    systems: Vec<String>,
    providers: Vec<String>,
    seeds: Vec<u64>,
    #[serde(default)]
    generation_parameters: GenerationParameters,
    #[serde(default)]
    annotation_pack: Option<String>,
    #[serde(default)]
    reports: Option<ReportsConfig>,
    #[serde(default)]
    notes: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct GenerationParameters {
    #[serde(default)]
    temperature: Option<f64>,
    #[serde(default)]
    max_tokens: Option<u32>,
    #[serde(default)]
    top_p: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ReportsConfig {
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default = "default_reports_subdir")]
    output_subdir: String,
}

fn default_true() -> bool {
    true
}

fn default_reports_subdir() -> String {
    "reports".to_string()
}

/// Single materialised entry in the run plan.
#[derive(Debug, Clone, Serialize)]
struct PlannedRun {
    run_id: String,
    system_id: String,
    provider_config: String,
    seed: u64,
    temperature: f64,
    max_tokens: u32,
}

/// Aggregated per-experiment summary written to `runs/experiments/<id>/summary.json`.
#[derive(Debug, Clone, Serialize)]
struct ExperimentSummary {
    experiment_id: String,
    benchmark_version: String,
    split: String,
    started_at: String,
    finished_at: String,
    total_runs: usize,
    runs: Vec<RunSummary>,
    metrics_computed: bool,
}

#[derive(Debug, Clone, Serialize)]
struct RunSummary {
    run_id: String,
    system_id: String,
    provider: String,
    seed: u64,
    instances_generated: usize,
    run_dir: PathBuf,
    results_bundle: Option<PathBuf>,
    reports_dir: Option<PathBuf>,
}

#[instrument(skip_all, fields(config = %args.config.display(), dry_run = args.dry_run))]
pub fn run(workspace: &Path, args: ExperimentArgs) -> Result<()> {
    let registry = SchemaRegistry::load(workspace.join("schemas"))
        .context("loading schemas for experiment validation")?;

    let config_raw = std::fs::read(&args.config)
        .with_context(|| format!("reading experiment config: {}", args.config.display()))?;
    let config_value: serde_json::Value = serde_json::from_slice(&config_raw)
        .with_context(|| format!("parsing experiment config: {}", args.config.display()))?;
    registry
        .validate(SchemaName::Experiment, &config_value)
        .with_context(|| format!("{} failed schema validation", args.config.display()))?;
    let config: ExperimentConfig = serde_json::from_value(config_value)
        .with_context(|| format!("deserialising {}", args.config.display()))?;

    let started_at = current_rfc3339()?;
    let plan = materialise_plan(&config)?;
    info!(
        experiment_id = %config.experiment_id,
        benchmark_version = %config.benchmark_version,
        split = %config.split,
        runs = plan.len(),
        "experiment plan materialised"
    );

    if args.dry_run {
        println!(
            "experiment {id}: dry-run ({total} planned run(s))",
            id = config.experiment_id,
            total = plan.len()
        );
        for r in &plan {
            println!(
                "  {run_id}  system={system} provider={prov} seed={seed} temp={temp:.3} max_tokens={max}",
                run_id = r.run_id,
                system = r.system_id,
                prov = r.provider_config,
                seed = r.seed,
                temp = r.temperature,
                max = r.max_tokens,
            );
        }
        return Ok(());
    }

    let version = BenchmarkVersion::new(&config.benchmark_version)
        .map_err(|e| anyhow!("invalid benchmark version: {e}"))?;
    let benchmark = load_benchmark(
        workspace.join("benchmark").join(&config.benchmark_version),
        &version,
    )
    .context("loading benchmark")?;

    let split_path = workspace
        .join("benchmark")
        .join(&config.benchmark_version)
        .join("splits")
        .join(format!("{}.json", config.split));
    let split_raw = std::fs::read_to_string(&split_path)
        .with_context(|| format!("reading split: {}", split_path.display()))?;
    let split_value: serde_json::Value = serde_json::from_str(&split_raw)?;
    let instance_ids: Vec<String> = split_value
        .get("instance_ids")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("split {} missing instance_ids", config.split))?
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();

    let annotation_pack: Option<AnnotationPack> = match config.annotation_pack.as_deref() {
        Some(rel) => {
            let path = workspace.join(rel);
            let raw = std::fs::read(&path)
                .with_context(|| format!("reading annotation pack: {}", path.display()))?;
            Some(serde_json::from_slice(&raw)?)
        }
        None => None,
    };

    let mut run_summaries = Vec::with_capacity(plan.len());

    for entry in &plan {
        let span = info_span!(
            "run",
            run_id = %entry.run_id,
            system = %entry.system_id,
            seed = entry.seed,
        );
        let summary = span.in_scope(|| {
            execute_run(
                workspace,
                &registry,
                &benchmark,
                &config,
                &instance_ids,
                entry,
                annotation_pack.as_ref(),
            )
        })?;
        info!(
            run_id = %summary.run_id,
            instances = summary.instances_generated,
            metrics = summary.results_bundle.is_some(),
            "run completed"
        );
        run_summaries.push(summary);
    }

    let finished_at = current_rfc3339()?;
    let summary = ExperimentSummary {
        experiment_id: config.experiment_id.clone(),
        benchmark_version: config.benchmark_version.clone(),
        split: config.split.clone(),
        started_at,
        finished_at,
        total_runs: run_summaries.len(),
        metrics_computed: annotation_pack.is_some(),
        runs: run_summaries,
    };
    let summary_dir = workspace
        .join("runs")
        .join("experiments")
        .join(&config.experiment_id);
    std::fs::create_dir_all(&summary_dir)?;
    let summary_path = summary_dir.join("summary.json");
    std::fs::write(&summary_path, serde_json::to_string_pretty(&summary)?)?;

    println!(
        "experiment {id}: executed {n} run(s); summary={path}",
        id = config.experiment_id,
        n = summary.total_runs,
        path = summary_path.display()
    );
    Ok(())
}

fn materialise_plan(config: &ExperimentConfig) -> Result<Vec<PlannedRun>> {
    let date = current_date_ymd()?;
    let temperature = config.generation_parameters.temperature.unwrap_or(0.0);
    let max_tokens = config.generation_parameters.max_tokens.unwrap_or(2048);

    let mut plan =
        Vec::with_capacity(config.systems.len() * config.providers.len() * config.seeds.len());
    let mut counter: BTreeMap<(String, String), u32> = BTreeMap::new();
    for system in &config.systems {
        for provider in &config.providers {
            for seed in &config.seeds {
                let key = (system.clone(), config.split.clone());
                let n = counter.entry(key).or_insert(0);
                *n += 1;
                let run_id = format!(
                    "run_{date}_{sys}_{split}_{nnn:03}",
                    sys = system,
                    split = config.split,
                    nnn = *n,
                );
                plan.push(PlannedRun {
                    run_id,
                    system_id: system.clone(),
                    provider_config: provider.clone(),
                    seed: *seed,
                    temperature,
                    max_tokens,
                });
            }
        }
    }
    Ok(plan)
}

#[allow(clippy::too_many_lines)]
fn execute_run(
    workspace: &Path,
    registry: &SchemaRegistry,
    benchmark: &cta_benchmark::LoadedBenchmark,
    config: &ExperimentConfig,
    instance_ids: &[String],
    entry: &PlannedRun,
    annotation_pack: Option<&AnnotationPack>,
) -> Result<RunSummary> {
    let system_id = SystemId::new(&entry.system_id)
        .map_err(|e| anyhow!("invalid system id '{}': {e}", entry.system_id))?;
    let run_id =
        RunId::new(&entry.run_id).map_err(|e| anyhow!("invalid run id '{}': {e}", entry.run_id))?;

    let prompt_path = workspace
        .join("configs")
        .join("prompts")
        .join(format!("{}.json", entry.system_id));
    let template = PromptTemplate::load(&prompt_path)
        .with_context(|| format!("loading prompt template: {}", prompt_path.display()))?;

    let provider: Box<dyn Provider> =
        if entry.provider_config.ends_with("local_stub.json") || entry.provider_config == "stub" {
            Box::<StubProvider>::default()
        } else {
            let cfg_path = workspace.join(&entry.provider_config);
            let cfg_raw = std::fs::read_to_string(&cfg_path)
                .with_context(|| format!("reading provider config: {}", cfg_path.display()))?;
            let cfg: ProviderConfig = serde_json::from_str(&cfg_raw)
                .with_context(|| format!("parsing provider config: {}", cfg_path.display()))?;
            build_from_config(cfg)
        };

    let run_dir = workspace.join("runs").join(run_id.as_str());
    let generated_dir = run_dir.join("generated").join(entry.system_id.as_str());
    let raw_dir = generated_dir.join("raw");
    std::fs::create_dir_all(&raw_dir)?;

    let mut instances_generated = 0usize;
    for iid in instance_ids {
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
        let raw_rel = format!("generated/{}/raw/{iid}.txt", entry.system_id);
        let params = GenerateParams {
            run_id: run_id.clone(),
            system_id: system_id.clone(),
            instance_id: instance_id.clone(),
            seed: entry.seed,
            max_tokens: entry.max_tokens,
            temperature: entry.temperature,
            raw_output_path: raw_rel.clone(),
        };
        let outcome = generate(provider.as_ref(), &template, &ctx, &params)
            .with_context(|| format!("generating obligations for {iid}"))?;
        debug!(
            instance = %iid,
            parse_ok = outcome.bundle.parse_status.ok,
            obligations = outcome.bundle.normalized_obligations.len(),
            "instance generated"
        );

        let raw_path = run_dir.join(&raw_rel);
        if let Some(parent) = raw_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&raw_path, &outcome.raw)?;

        let bundle_path = generated_dir.join(format!("{iid}.json"));
        std::fs::write(&bundle_path, serde_json::to_string_pretty(&outcome.bundle)?)?;
        instances_generated += 1;
    }

    let manifest = super::generate::build_run_manifest_public(
        &run_id,
        &system_id,
        &template,
        provider.as_ref(),
        entry.seed,
        entry.max_tokens,
        entry.temperature,
        &config.benchmark_version,
    )?;
    registry
        .validate(SchemaName::RunManifest, &manifest)
        .context("run_manifest failed schema validation")?;
    let manifest_path = run_dir.join("run_manifest.json");
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

    let (results_bundle, reports_dir) = if let Some(pack) = annotation_pack {
        let inputs = collect_instance_inputs(&run_dir, pack, Some(entry.system_id.as_str()))?;
        let bundle = compute_results_bundle(manifest, pack, &inputs);
        let bundle_value = serde_json::to_value(&bundle)?;
        registry
            .validate(SchemaName::ResultsBundle, &bundle_value)
            .context("results_bundle failed schema validation")?;
        let bundle_path = run_dir.join("results_bundle.json");
        std::fs::write(&bundle_path, serde_json::to_string_pretty(&bundle_value)?)?;

        let reports_dir = match config.reports.as_ref() {
            Some(r) if r.enabled => Some(run_dir.join(&r.output_subdir)),
            None => Some(run_dir.join("reports")),
            Some(_) => None,
        };
        if let Some(dir) = reports_dir.as_ref() {
            render_reports(dir, entry.system_id.as_str(), &bundle)?;
        }
        (Some(bundle_path), reports_dir)
    } else {
        (None, None)
    };

    Ok(RunSummary {
        run_id: run_id.as_str().to_string(),
        system_id: entry.system_id.clone(),
        provider: entry.provider_config.clone(),
        seed: entry.seed,
        instances_generated,
        run_dir,
        results_bundle,
        reports_dir,
    })
}

fn render_reports(out_dir: &Path, system_id: &str, bundle: &ResultsBundle) -> Result<()> {
    std::fs::create_dir_all(out_dir)?;
    let rendered = render_all(system_id, bundle);
    std::fs::write(out_dir.join("primary_metrics.csv"), &rendered.primary_csv)?;
    std::fs::write(out_dir.join("instance_results.csv"), &rendered.instance_csv)?;
    std::fs::write(out_dir.join("results.md"), &rendered.markdown)?;
    std::fs::write(out_dir.join("results.tex"), &rendered.latex)?;
    Ok(())
}

fn collect_instance_inputs(
    run_dir: &Path,
    pack: &AnnotationPack,
    system_id: Option<&str>,
) -> Result<BTreeMap<String, InstanceInputs>> {
    let mut out: BTreeMap<String, InstanceInputs> = BTreeMap::new();
    let generated_root = run_dir.join("generated");
    for ann in &pack.records {
        let inst_id = ann.instance_id.as_str().to_string();
        let file = format!("{inst_id}.json");
        let generated = system_id
            .map(|s| generated_root.join(s).join(&file))
            .filter(|p| p.exists())
            .unwrap_or_else(|| generated_root.join(&file));

        let elaborated = if generated.exists() {
            serde_json::from_slice::<serde_json::Value>(&std::fs::read(&generated)?)
                .ok()
                .and_then(|v| v.get("parse_status").cloned())
                .and_then(|ps| ps.get("ok").cloned())
                .and_then(|ok| ok.as_bool())
                .unwrap_or(false)
        } else {
            false
        };

        let critical_units_total = u32::try_from(
            ann.critical_unit_coverage.covered.len() + ann.critical_unit_coverage.missed.len(),
        )
        .unwrap_or(u32::MAX);
        out.insert(
            inst_id,
            InstanceInputs {
                signal: InstanceSignal {
                    elaborated,
                    proof_used: false,
                    critical_units_total,
                },
                lean_diagnostics_path: None,
                behavior_report_path: None,
            },
        );
    }
    Ok(out)
}

fn current_date_ymd() -> Result<String> {
    let now = time::OffsetDateTime::now_utc();
    Ok(format!(
        "{:04}_{:02}_{:02}",
        now.year(),
        u8::from(now.month()),
        now.day()
    ))
}

fn current_rfc3339() -> Result<String> {
    Ok(time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339)?)
}
