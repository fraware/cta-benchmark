use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;
use cta_annotations::{load_dir, AnnotationPack};
use cta_metrics::{
    agreement, compute_results_bundle_with_agreement, InstanceInputs, InstanceSignal,
};
use cta_schema::{SchemaName, SchemaRegistry};

#[derive(Debug, Args)]
pub struct ComputeArgs {
    /// Run id to compute metrics over (directory `runs/<run>` must exist).
    #[arg(long)]
    pub run: String,
    /// Path to the adjudicated annotation pack.
    #[arg(long)]
    pub annotations: PathBuf,
    /// Optional path for the results bundle output. Defaults to
    /// `runs/<run>/results_bundle.json`.
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Optional path to a raw annotator directory (pre-adjudication). When
    /// supplied, the runner computes inter-annotator agreement metrics.
    #[arg(long)]
    pub raw_annotations: Option<PathBuf>,
}

pub fn compute(workspace: &Path, args: ComputeArgs) -> Result<()> {
    let run_dir = workspace.join("runs").join(&args.run);
    if !run_dir.is_dir() {
        anyhow::bail!("run directory not found: {}", run_dir.display());
    }
    let manifest_path = run_dir.join("run_manifest.json");
    let run_manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&manifest_path)
            .with_context(|| format!("reading {}", manifest_path.display()))?,
    )
    .with_context(|| format!("parsing {}", manifest_path.display()))?;
    let system_id = run_manifest
        .get("system_id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);

    let pack_raw = std::fs::read(&args.annotations)
        .with_context(|| format!("reading {}", args.annotations.display()))?;
    let pack: AnnotationPack = serde_json::from_slice(&pack_raw)
        .with_context(|| format!("parsing {}", args.annotations.display()))?;

    let inputs = collect_instance_inputs(&run_dir, &pack, system_id.as_deref())?;
    let registry = SchemaRegistry::load(workspace.join("schemas"))
        .context("loading schemas for results_bundle validation")?;

    let agreement = if let Some(dir) = args.raw_annotations.as_ref() {
        let set = load_dir(dir, &registry)
            .with_context(|| format!("loading raw annotations from {}", dir.display()))?;
        agreement::from_annotation_set(&set)
    } else {
        None
    };

    let bundle = compute_results_bundle_with_agreement(run_manifest, &pack, &inputs, agreement);
    let bundle_value = serde_json::to_value(&bundle)?;
    registry
        .validate(SchemaName::ResultsBundle, &bundle_value)
        .context("results_bundle failed schema validation")?;

    let out = args
        .out
        .unwrap_or_else(|| run_dir.join("results_bundle.json"));
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out, serde_json::to_string_pretty(&bundle_value)?)?;

    let p = &bundle.aggregate_metrics.primary;
    println!(
        "metrics compute: wrote {path}\n\
         elaboration={elab:.3} faith={faith:.3} coverage={cov:.3} consistency={cons:.3} vacuity={vac:.3} proof={proof:.3}",
        path = out.display(),
        elab = p.elaboration_rate,
        faith = p.semantic_faithfulness_mean,
        cov = p.critical_unit_coverage,
        cons = p.rust_consistency_rate,
        vac = p.vacuity_rate,
        proof = p.proof_utility,
    );
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
        let generated = resolve_generated_path(&generated_root, system_id, &inst_id);

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

        let lean_rel = format!("lean/{inst_id}.diagnostics.json");
        let lean_diagnostics_path = run_dir.join(&lean_rel).exists().then(|| lean_rel.clone());
        let behavior_rel = format!("behavior/{inst_id}.report.json");
        let behavior_report_path = run_dir
            .join(&behavior_rel)
            .exists()
            .then(|| behavior_rel.clone());

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
                lean_diagnostics_path,
                behavior_report_path,
            },
        );
    }

    Ok(out)
}

fn resolve_generated_path(
    generated_root: &Path,
    system_id: Option<&str>,
    inst_id: &str,
) -> PathBuf {
    let file = format!("{inst_id}.json");
    if let Some(sid) = system_id {
        let p = generated_root.join(sid).join(&file);
        if p.exists() {
            return p;
        }
    }
    let flat = generated_root.join(&file);
    if flat.exists() {
        return flat;
    }
    if let Ok(rd) = std::fs::read_dir(generated_root) {
        for e in rd.flatten() {
            if e.path().is_dir() {
                let p = e.path().join(&file);
                if p.exists() {
                    return p;
                }
            }
        }
    }
    generated_root.join(&file)
}
