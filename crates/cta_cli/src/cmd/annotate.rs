use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;
use cta_annotations::{adjudicate_set, load_dir, AdjudicationPolicy, AnnotationPack};
use cta_core::BenchmarkVersion;
use cta_schema::{SchemaName, SchemaRegistry};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

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
    /// `runs/annotation_packs/<version>-adjudicated.json`, or, when
    /// `--from-benchmark` is set, to the benchmark-local canonical path
    /// `benchmark/<version>/annotation/adjudicated_subset/pack.json`.
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Adjudication policy: `prefer-adjudicator` (default) or `majority`.
    #[arg(long, default_value = "prefer-adjudicator")]
    pub policy: String,
    /// Treat this invocation as producing the canonical, release-grade pack
    /// for the given benchmark version. Shifts the default output path from
    /// `runs/annotation_packs/` into the benchmark tree so that a fresh
    /// clone can compute paper-reportable metrics without prior runs/.
    /// Has no effect if `--out` is supplied explicitly.
    #[arg(long, default_value_t = false)]
    pub from_benchmark: bool,
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
    let (effective_input, cleanup_dir) = prepare_pack_input(workspace, &input)?;
    let set = load_dir(&effective_input, &registry)
        .with_context(|| format!("loading annotations from {}", effective_input.display()))?;
    let adjudicated = adjudicate_set(&set, policy).context("adjudicating annotation groups")?;
    let pack = AnnotationPack::from_adjudicated(&adjudicated)?;

    let out = args.out.unwrap_or_else(|| {
        if args.from_benchmark {
            workspace
                .join("benchmark")
                .join(args.version.as_str())
                .join("annotation")
                .join("adjudicated_subset")
                .join("pack.json")
        } else {
            workspace
                .join("runs")
                .join("annotation_packs")
                .join(format!("{}-adjudicated.json", args.version.as_str()))
        }
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
    if let Some(clean) = cleanup_dir {
        let _ = std::fs::remove_dir_all(clean);
    }
    Ok(())
}

fn prepare_pack_input(workspace: &Path, input: &Path) -> Result<(PathBuf, Option<PathBuf>)> {
    if !input.is_dir() {
        return Ok((input.to_path_buf(), None));
    }
    let has_root_json = std::fs::read_dir(input)?
        .filter_map(std::result::Result::ok)
        .any(|e| {
            e.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                && e.path().extension().and_then(|x| x.to_str()) == Some("json")
        });
    if !has_root_json {
        return Ok((input.to_path_buf(), None));
    }

    let ts = time::OffsetDateTime::now_utc().unix_timestamp_nanos();
    let staging = workspace
        .join("runs")
        .join("tmp")
        .join(format!("pack_input_{ts}"));
    std::fs::create_dir_all(&staging)?;
    for entry in walkdir::WalkDir::new(input)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let rel = match path.strip_prefix(input) {
            Ok(r) => r,
            Err(_) => continue,
        };
        // Only keep `<system>/<instance>.json` style files; skip metadata json
        // at the adjudicated_subset root.
        if rel.components().count() < 2 {
            continue;
        }
        let dest = staging.join(rel);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(path, &dest)?;
    }
    Ok((staging.clone(), Some(staging)))
}

#[derive(Debug, Deserialize)]
struct ExperimentConfig {
    split: String,
    systems: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SplitFile {
    instance_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PairRecord {
    instance_id: String,
    system_id: String,
}

#[derive(Debug, Deserialize)]
struct InstanceRecord {
    domain: String,
    difficulty: String,
    informal_statement: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct SemanticUnitsFile {
    #[serde(alias = "semantic_units")]
    units: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ReferenceObligationsFile {
    obligations: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct GeneratedOutput {
    normalized_obligations: Vec<serde_json::Value>,
    raw_output_path: Option<String>,
}

#[derive(Debug, Args)]
pub struct PlanArgs {
    /// Benchmark version (e.g. `v0.2`).
    #[arg(long, value_parser = parse_bench_version)]
    pub benchmark_version: BenchmarkVersion,
    /// Path to experiment config JSON.
    #[arg(long)]
    pub experiment_config: PathBuf,
    /// Output directory for task-board artifacts.
    #[arg(long)]
    pub out: PathBuf,
    /// Optional path to adjudicated pack; defaults to benchmark-local pack.
    #[arg(long)]
    pub pack: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct CoverageArgs {
    /// Benchmark version (e.g. `v0.2`).
    #[arg(long, value_parser = parse_bench_version)]
    pub benchmark_version: BenchmarkVersion,
    /// Path to experiment config JSON.
    #[arg(long)]
    pub experiment_config: PathBuf,
    /// Path to adjudicated pack JSON.
    #[arg(long)]
    pub pack: PathBuf,
    /// Output directory for adjudicated-subset artifacts.
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Debug, Args)]
pub struct BatchesArgs {
    /// Benchmark version (e.g. `v0.2`).
    #[arg(long, value_parser = parse_bench_version)]
    pub benchmark_version: BenchmarkVersion,
    /// Path to task-board missing pairs JSON.
    #[arg(long)]
    pub missing_pairs: PathBuf,
    /// Output directory for strict batch queue files.
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Debug, Args)]
pub struct SyncReviewPacketsArgs {
    /// Benchmark version (e.g. `v0.2`).
    #[arg(long, value_parser = parse_bench_version)]
    pub benchmark_version: BenchmarkVersion,
    /// Source root containing `batch_*/<system>/*__adjudicator.json`.
    #[arg(long)]
    pub from: PathBuf,
    /// Destination root for adjudicated files.
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Debug, Args)]
pub struct BuildReviewPacketsArgs {
    /// Benchmark version (e.g. `v0.2`).
    #[arg(long, value_parser = parse_bench_version)]
    pub benchmark_version: BenchmarkVersion,
    /// Path to experiment config JSON.
    #[arg(long)]
    pub experiment_config: PathBuf,
    /// Path to required pair list (typically missing_pairs.json).
    #[arg(long)]
    pub pairs: PathBuf,
    /// Output root for self-contained review packets.
    #[arg(long)]
    pub out: PathBuf,
    /// Optional limit for packet creation (for calibration batches).
    #[arg(long)]
    pub limit: Option<usize>,
    /// Continue when generated outputs are missing; emit unresolved report.
    #[arg(long, default_value_t = false)]
    pub allow_missing_output: bool,
}

#[derive(Debug, Args)]
pub struct IngestDraftArgs {
    /// Benchmark version (e.g. `v0.2`).
    #[arg(long, value_parser = parse_bench_version)]
    pub benchmark_version: BenchmarkVersion,
    /// Source directory of assistant draft annotation JSON files.
    #[arg(long)]
    pub from: PathBuf,
    /// Destination raw annotation root (benchmark/<v>/annotation/raw).
    #[arg(long)]
    pub into: PathBuf,
}

#[derive(Debug, Args)]
pub struct VerifyReviewPacketsArgs {
    /// Benchmark version (e.g. `v0.2`).
    #[arg(long, value_parser = parse_bench_version)]
    pub benchmark_version: BenchmarkVersion,
    /// Review packet root (e.g. benchmark/v0.2/annotation/review_packets).
    #[arg(long)]
    pub packets_root: PathBuf,
    /// Formal JSON schema path for packet.json validation.
    #[arg(long, default_value = "schemas/review_packet.schema.json")]
    pub schema: PathBuf,
    /// Output path for signed verification summary.
    #[arg(
        long,
        default_value = "benchmark/v0.2/annotation/review_packets/verification_summary.signed.json"
    )]
    pub out: PathBuf,
}

#[derive(Debug, Deserialize)]
struct DraftAnnotation {
    benchmark_version: String,
    instance_id: String,
    system_id: String,
    annotator_id: String,
    source_packet: String,
    set_level_scores: serde_json::Value,
    critical_unit_coverage: serde_json::Value,
    generated_obligations: Vec<serde_json::Value>,
    summary_rationale: String,
    recommended_disposition: String,
}

pub fn plan(workspace: &Path, args: PlanArgs) -> Result<()> {
    let (exp, required_pairs) =
        load_required_pairs(workspace, &args.benchmark_version, &args.experiment_config)?;
    let pack_path = args.pack.unwrap_or_else(|| {
        workspace
            .join("benchmark")
            .join(args.benchmark_version.as_str())
            .join("annotation")
            .join("adjudicated_subset")
            .join("pack.json")
    });
    let present_pairs = load_pack_pairs(&pack_path).with_context(|| {
        format!(
            "loading adjudicated pairs from {}",
            pack_path.as_path().display()
        )
    })?;
    let missing_pairs: Vec<(String, String)> = required_pairs
        .iter()
        .filter(|(iid, sid)| !present_pairs.contains(&(iid.clone(), sid.clone())))
        .cloned()
        .collect();

    std::fs::create_dir_all(&args.out)?;
    let required_json: Vec<_> = required_pairs
        .iter()
        .map(|(iid, sid)| json!({ "instance_id": iid, "system_id": sid }))
        .collect();
    let missing_json: Vec<_> = missing_pairs
        .iter()
        .map(|(iid, sid)| json!({ "instance_id": iid, "system_id": sid }))
        .collect();
    std::fs::write(
        args.out.join("required_pairs.json"),
        serde_json::to_vec_pretty(&required_json)?,
    )?;
    std::fs::write(
        args.out.join("missing_pairs.json"),
        serde_json::to_vec_pretty(&missing_json)?,
    )?;
    write_assignment_matrix(
        &args.out.join("assignment_matrix.csv"),
        args.benchmark_version.as_str(),
        &required_pairs,
        &present_pairs,
    )?;
    write_completion_log(&args.out.join("completion_log.csv"), &missing_pairs)?;

    println!(
        "annotate plan: wrote task board to {} (required={}, missing={})",
        args.out.display(),
        required_pairs.len(),
        missing_pairs.len()
    );
    println!(
        "annotate plan: split={} systems={}",
        exp.split,
        exp.systems.join(",")
    );
    Ok(())
}

pub fn coverage(workspace: &Path, args: CoverageArgs) -> Result<()> {
    let (exp, required_pairs) =
        load_required_pairs(workspace, &args.benchmark_version, &args.experiment_config)?;
    let present_pairs = load_pack_pairs(&args.pack).with_context(|| {
        format!(
            "loading adjudicated pairs from {}",
            args.pack.as_path().display()
        )
    })?;
    let missing_pairs: Vec<(String, String)> = required_pairs
        .iter()
        .filter(|(iid, sid)| !present_pairs.contains(&(iid.clone(), sid.clone())))
        .cloned()
        .collect();
    let covered_pairs = required_pairs.len().saturating_sub(missing_pairs.len());

    std::fs::create_dir_all(&args.out)?;
    let coverage_summary = json!({
        "benchmark_version": args.benchmark_version.as_str(),
        "split": exp.split,
        "systems": exp.systems,
        "required_pairs": required_pairs.len(),
        "covered_pairs": covered_pairs,
        "missing_pairs": missing_pairs.len(),
        "missing_examples": missing_pairs
            .iter()
            .take(20)
            .map(|(iid, sid)| json!({"instance_id": iid, "system_id": sid}))
            .collect::<Vec<_>>(),
    });
    let manifest = json!({
        "benchmark_version": args.benchmark_version.as_str(),
        "split": coverage_summary["split"],
        "required_pairs": required_pairs.len(),
        "covered_pairs": covered_pairs,
        "pack_path": normalize_workspace_path(workspace, &args.pack),
        "generated_at": time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
    });
    std::fs::write(
        args.out.join("coverage_summary.json"),
        serde_json::to_vec_pretty(&coverage_summary)?,
    )?;
    std::fs::write(
        args.out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;

    println!(
        "annotate coverage: wrote {} and {} (covered={}/{})",
        args.out.join("coverage_summary.json").display(),
        args.out.join("manifest.json").display(),
        covered_pairs,
        required_pairs.len()
    );
    Ok(())
}

pub fn batches(_workspace: &Path, args: BatchesArgs) -> Result<()> {
    let raw = std::fs::read_to_string(&args.missing_pairs)
        .with_context(|| format!("reading {}", args.missing_pairs.display()))?;
    let pairs: Vec<PairRecord> = serde_json::from_str(&raw)
        .with_context(|| format!("parsing {}", args.missing_pairs.display()))?;
    let systems = [
        "text_only_v1",
        "code_only_v1",
        "naive_concat_v1",
        "full_method_v1",
    ];
    std::fs::create_dir_all(&args.out)?;

    let mut manifest_rows = Vec::new();
    for (idx, system_id) in systems.iter().enumerate() {
        let mut system_pairs: Vec<&PairRecord> =
            pairs.iter().filter(|p| p.system_id == *system_id).collect();
        system_pairs.sort_by(|a, b| a.instance_id.cmp(&b.instance_id));
        let batch_id = format!("batch_{:02}", idx + 1);
        let queue_path = args.out.join(format!("{batch_id}_{system_id}.csv"));
        let mut lines = vec!["instance_id,system_id,status,notes".to_string()];
        for p in &system_pairs {
            lines.push(format!("{},{},queued,", p.instance_id, p.system_id));
        }
        std::fs::write(&queue_path, lines.join("\n"))?;
        manifest_rows.push(json!({
            "batch_id": batch_id,
            "system_id": system_id,
            "pair_count": system_pairs.len(),
            "queue_path": path_to_slash_string(&queue_path)
        }));
    }
    let manifest = json!({
        "benchmark_version": args.benchmark_version.as_str(),
        "source_missing_pairs": path_to_slash_string(&args.missing_pairs),
        "policy": "strict_per_system_batches",
        "batches": manifest_rows,
        "generated_at": time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
    });
    std::fs::write(
        args.out.join("batch_manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!(
        "annotate batches: wrote strict batch queues to {}",
        args.out.display()
    );
    Ok(())
}

pub fn sync_review_packets(_workspace: &Path, args: SyncReviewPacketsArgs) -> Result<()> {
    let mut copied = 0usize;
    for entry in walkdir::WalkDir::new(&args.from)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let name = entry.file_name().to_string_lossy();
        if !name.ends_with("__adjudicator.json") {
            continue;
        }
        let parts: Vec<&str> = name
            .trim_end_matches("__adjudicator.json")
            .split("__")
            .collect();
        if parts.len() != 2 {
            continue;
        }
        let instance_id = parts[0];
        let system_id = parts[1];
        let dest = args.out.join(system_id).join(format!("{instance_id}.json"));
        let src_bytes = std::fs::read(entry.path())
            .with_context(|| format!("reading {}", entry.path().display()))?;
        if dest.is_file() {
            let dst_bytes =
                std::fs::read(&dest).with_context(|| format!("reading {}", dest.display()))?;
            if src_bytes != dst_bytes {
                anyhow::bail!(
                    "refusing to overwrite differing adjudicated file at {}",
                    dest.display()
                );
            }
            continue;
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&dest, src_bytes).with_context(|| format!("writing {}", dest.display()))?;
        copied += 1;
    }
    println!(
        "annotate sync-review-packets: copied {copied} adjudicated record(s) into {}",
        args.out.display()
    );
    Ok(())
}

pub fn build_review_packets(workspace: &Path, args: BuildReviewPacketsArgs) -> Result<()> {
    let raw = std::fs::read_to_string(&args.pairs)
        .with_context(|| format!("reading {}", args.pairs.display()))?;
    let mut pairs: Vec<PairRecord> =
        serde_json::from_str(&raw).with_context(|| format!("parsing {}", args.pairs.display()))?;
    if let Some(limit) = args.limit {
        pairs.truncate(limit);
    }
    let run_index = build_run_index(workspace)?;
    let mut built = 0usize;
    let mut unresolved: Vec<serde_json::Value> = Vec::new();
    for pair in &pairs {
        let instance_root = find_instance_root(
            workspace,
            args.benchmark_version.as_str(),
            &pair.instance_id,
        )?;
        let generated_path = match resolve_generated_output_path(
            workspace,
            &run_index,
            &pair.system_id,
            &pair.instance_id,
        ) {
            Ok(p) => p,
            Err(e) if args.allow_missing_output => {
                unresolved.push(json!({
                    "instance_id": pair.instance_id,
                    "system_id": pair.system_id,
                    "reason": format!("{e}")
                }));
                continue;
            }
            Err(e) => return Err(e),
        };
        let generated_raw = std::fs::read_to_string(&generated_path)
            .with_context(|| format!("reading {}", generated_path.display()))?;
        let generated: GeneratedOutput = serde_json::from_str(&generated_raw)
            .with_context(|| format!("parsing {}", generated_path.display()))?;
        if generated.normalized_obligations.is_empty() {
            anyhow::bail!(
                "generated obligations empty for pair ({}, {})",
                pair.instance_id,
                pair.system_id
            );
        }

        let packet_dir = args.out.join(&pair.system_id).join(&pair.instance_id);
        std::fs::create_dir_all(&packet_dir)?;
        let instance_json_path = instance_root.join("instance.json");
        let instance_record: InstanceRecord = serde_json::from_str(
            &std::fs::read_to_string(&instance_json_path)
                .with_context(|| format!("reading {}", instance_json_path.display()))?,
        )
        .with_context(|| format!("parsing {}", instance_json_path.display()))?;
        let sem_path = instance_root.join("semantic_units.json");
        let ref_obl_path = instance_root.join("reference_obligations.json");
        let sem: SemanticUnitsFile = serde_json::from_str(
            &std::fs::read_to_string(&sem_path)
                .with_context(|| format!("reading {}", sem_path.display()))?,
        )
        .with_context(|| format!("parsing {}", sem_path.display()))?;
        let refs: ReferenceObligationsFile = serde_json::from_str(
            &std::fs::read_to_string(&ref_obl_path)
                .with_context(|| format!("reading {}", ref_obl_path.display()))?,
        )
        .with_context(|| format!("parsing {}", ref_obl_path.display()))?;

        std::fs::copy(&sem_path, packet_dir.join("semantic_units.json"))?;
        std::fs::copy(&ref_obl_path, packet_dir.join("reference_obligations.json"))?;
        std::fs::copy(&generated_path, packet_dir.join("generated_output.json"))?;
        std::fs::copy(
            instance_root.join("scaffold.lean"),
            packet_dir.join("scaffold.lean"),
        )?;
        std::fs::copy(
            instance_root.join("reference.rs"),
            packet_dir.join("reference.rs"),
        )?;

        let run_root = generated_path
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| workspace.join("runs"));
        let raw_output_src = generated
            .raw_output_path
            .as_deref()
            .map(|p| run_root.join(p))
            .filter(|p| p.is_file())
            .unwrap_or_else(|| {
                generated_path
                    .parent()
                    .unwrap_or(workspace)
                    .join("raw")
                    .join(format!("{}.txt", pair.instance_id))
            });
        if raw_output_src.is_file() {
            std::fs::copy(&raw_output_src, packet_dir.join("raw_output.txt"))?;
        } else {
            std::fs::write(packet_dir.join("raw_output.txt"), "")?;
        }

        // Keep packet self-contained even when optional artifacts are unavailable.
        std::fs::write(
            packet_dir.join("rust_summary.json"),
            "{\"available\":false}",
        )?;
        std::fs::write(
            packet_dir.join("lean_diagnostics.json"),
            "{\"available\":false}",
        )?;
        std::fs::write(
            packet_dir.join("behavior_report.json"),
            "{\"available\":false}",
        )?;

        let critical_units: HashSet<String> = sem
            .units
            .iter()
            .filter(|u| {
                u.get("criticality")
                    .and_then(|c| c.as_str())
                    .map(|c| c == "critical")
                    .unwrap_or(false)
            })
            .filter_map(|u| u.get("id").and_then(|id| id.as_str()).map(str::to_string))
            .collect();
        let generated_obligations = generated
            .normalized_obligations
            .iter()
            .enumerate()
            .map(|(idx, g)| {
                let kind = g
                    .get("kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let linked_semantic_units =
                    g.get("linked_semantic_units").cloned().unwrap_or(json!([]));
                let stmt = g
                    .get("lean_statement")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                let gloss = g
                    .get("nl_gloss")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                let inferred = infer_critical_units_for_obligation(&sem.units, &stmt, &gloss);
                let linked_semantic_units =
                    merge_semantic_units(&linked_semantic_units, inferred.as_slice());
                let layer = obligation_layer(&kind, &linked_semantic_units, &critical_units);
                json!({
                    "index": idx,
                    "kind": kind,
                    "layer": layer,
                    "lean_statement": g.get("lean_statement").cloned().unwrap_or(json!("")),
                    "nl_gloss": g.get("nl_gloss").cloned().unwrap_or(json!("")),
                    "linked_semantic_units": linked_semantic_units,
                    "raw_source": "model"
                })
            })
            .collect::<Vec<_>>();
        let quality_summary = build_quality_summary(&sem.units, &generated_obligations);

        let packet = json!({
            "benchmark_version": args.benchmark_version.as_str(),
            "instance_id": pair.instance_id,
            "system_id": pair.system_id,
            "domain": instance_record.domain,
            "difficulty": instance_record.difficulty,
            "instance_path": normalize_workspace_path(workspace, &instance_root),
            "generated_output_path": normalize_workspace_path(workspace, &generated_path),
            "informal_statement": instance_record.informal_statement,
            "semantic_units": sem.units,
            "reference_obligations": refs.obligations,
            "generated_obligations": generated_obligations,
            "quality_summary": quality_summary,
            "lean_check": {
                "elaborated": serde_json::Value::Null,
                "diagnostics_path": normalize_workspace_path(workspace, &packet_dir.join("lean_diagnostics.json"))
            },
            "behavior_check": {
                "report_path": normalize_workspace_path(workspace, &packet_dir.join("behavior_report.json")),
                "summary": {
                    "has_counterexample": false,
                    "counterexample_count": 0
                }
            },
            "context": {
                "scaffold_path": normalize_workspace_path(workspace, &packet_dir.join("scaffold.lean")),
                "reference_rs_path": normalize_workspace_path(workspace, &packet_dir.join("reference.rs")),
                "rust_summary_path": normalize_workspace_path(workspace, &packet_dir.join("rust_summary.json"))
            }
        });
        std::fs::write(
            packet_dir.join("packet.json"),
            serde_json::to_vec_pretty(&packet)?,
        )?;
        built += 1;
    }
    if args.allow_missing_output {
        let report = json!({
            "benchmark_version": args.benchmark_version.as_str(),
            "requested_pairs": pairs.len(),
            "built_packets": built,
            "unresolved_pairs": unresolved.len(),
            "unresolved": unresolved
        });
        std::fs::write(
            args.out.join("_build_report.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
    }
    println!(
        "annotate build-review-packets: wrote {built} packet(s) under {}{}",
        args.out.display(),
        if args.allow_missing_output {
            format!(
                " (report: {})",
                args.out.join("_build_report.json").display()
            )
        } else {
            String::new()
        }
    );
    Ok(())
}

pub fn ingest_draft(_workspace: &Path, args: IngestDraftArgs) -> Result<()> {
    let mut imported = 0usize;
    let draft_root = args.into.join("assistant_draft");
    std::fs::create_dir_all(&draft_root)?;
    for entry in walkdir::WalkDir::new(&args.from)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let raw =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let draft: DraftAnnotation =
            serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?;
        if draft.benchmark_version != args.benchmark_version.as_str() {
            anyhow::bail!(
                "draft {} benchmark_version={} does not match {}",
                path.display(),
                draft.benchmark_version,
                args.benchmark_version.as_str()
            );
        }
        if draft.annotator_id != "assistant_draft" {
            anyhow::bail!(
                "draft {} must use annotator_id=assistant_draft",
                path.display()
            );
        }
        if draft.generated_obligations.is_empty() {
            anyhow::bail!("draft {} has no generated_obligations", path.display());
        }
        if draft.summary_rationale.trim().is_empty() {
            anyhow::bail!("draft {} has empty summary_rationale", path.display());
        }
        let _ = (
            &draft.source_packet,
            &draft.set_level_scores,
            &draft.critical_unit_coverage,
            &draft.recommended_disposition,
        );
        let dest = draft_root
            .join(&draft.system_id)
            .join(format!("{}__assistant_draft.json", draft.instance_id));
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Preserve verbatim rationale/body by writing the original raw bytes.
        std::fs::write(&dest, raw)?;
        imported += 1;
    }
    println!(
        "annotate ingest-draft: imported {imported} draft annotation(s) into {}",
        draft_root.display()
    );
    Ok(())
}

pub fn verify_review_packets(workspace: &Path, args: VerifyReviewPacketsArgs) -> Result<()> {
    let schema_path = if args.schema.is_absolute() {
        args.schema.clone()
    } else {
        workspace.join(&args.schema)
    };
    let packets_root = if args.packets_root.is_absolute() {
        args.packets_root.clone()
    } else {
        workspace.join(&args.packets_root)
    };
    let out_path = if args.out.is_absolute() {
        args.out.clone()
    } else {
        workspace.join(&args.out)
    };
    let _schema_value: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&schema_path)
            .with_context(|| format!("reading {}", schema_path.display()))?,
    )
    .with_context(|| format!("parsing {}", schema_path.display()))?;
    let registry = SchemaRegistry::load(workspace.join("schemas"))
        .with_context(|| "loading schema registry from workspace")?;

    let mut packet_paths: Vec<PathBuf> = walkdir::WalkDir::new(&packets_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file() && e.file_name().to_string_lossy() == "packet.json")
        .map(|e| e.into_path())
        .collect();
    packet_paths.sort();

    let required_files = [
        "packet.json",
        "semantic_units.json",
        "reference_obligations.json",
        "generated_output.json",
        "raw_output.txt",
        "scaffold.lean",
        "reference.rs",
        "rust_summary.json",
        "lean_diagnostics.json",
        "behavior_report.json",
    ];

    let mut failures = Vec::new();
    let mut packet_hashes = Vec::new();
    let mut passed = 0usize;

    for packet_path in &packet_paths {
        let packet_dir = packet_path.parent().unwrap_or(&packets_root);
        let packet_bytes = std::fs::read(packet_path)
            .with_context(|| format!("reading {}", packet_path.display()))?;
        let packet_value: serde_json::Value = serde_json::from_slice(&packet_bytes)
            .with_context(|| format!("parsing {}", packet_path.display()))?;
        let mut packet_issues: Vec<String> = Vec::new();

        if let Err(e) = registry.validate(SchemaName::ReviewPacket, &packet_value) {
            packet_issues.push(e.to_string());
        }

        let generated_obligations_non_empty = packet_value
            .get("generated_obligations")
            .and_then(|v| v.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);
        if !generated_obligations_non_empty {
            packet_issues.push("generated_obligations must be non-empty".to_string());
        }

        for req in required_files {
            let p = packet_dir.join(req);
            if !p.is_file() {
                packet_issues.push(format!("missing required file: {}", p.display()));
            }
        }

        let raw_output_path = packet_dir.join("raw_output.txt");
        if raw_output_path.is_file() {
            let raw_output = std::fs::read_to_string(&raw_output_path)
                .with_context(|| format!("reading {}", raw_output_path.display()))?;
            if raw_output.trim().is_empty() {
                packet_issues.push(format!(
                    "empty raw_output.txt: {}",
                    raw_output_path.display()
                ));
            }
        }

        let instance_id = packet_value
            .get("instance_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown_instance")
            .to_string();
        let system_id = packet_value
            .get("system_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown_system")
            .to_string();
        let packet_hash = sha256_hex(&packet_bytes);
        packet_hashes.push(json!({
            "instance_id": instance_id,
            "system_id": system_id,
            "packet_path": normalize_workspace_path(workspace, packet_path),
            "sha256": packet_hash
        }));

        if packet_issues.is_empty() {
            passed += 1;
        } else {
            failures.push(json!({
                "packet_path": normalize_workspace_path(workspace, packet_path),
                "issues": packet_issues
            }));
        }
    }

    let generated_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());
    let unsigned_summary = json!({
        "benchmark_version": args.benchmark_version.as_str(),
        "packets_root": normalize_workspace_path(workspace, &packets_root),
        "schema_path": normalize_workspace_path(workspace, &schema_path),
        "generated_at": generated_at,
        "total_packets": packet_paths.len(),
        "passed_packets": passed,
        "failed_packets": failures.len(),
        "packet_hashes": packet_hashes,
        "failures": failures
    });
    let signature = sha256_hex(&serde_json::to_vec(&unsigned_summary)?);
    let signed_summary = json!({
        "signature": {
            "algorithm": "sha256",
            "value": signature
        },
        "verification": unsigned_summary
    });

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out_path, serde_json::to_vec_pretty(&signed_summary)?)
        .with_context(|| format!("writing {}", out_path.display()))?;
    println!(
        "annotate verify-review-packets: wrote signed verification summary to {} (passed={}, failed={})",
        out_path.display(),
        passed,
        failures.len()
    );

    if !signed_summary["verification"]["failed_packets"]
        .as_u64()
        .map(|n| n == 0)
        .unwrap_or(false)
    {
        anyhow::bail!(
            "packet verification failed: {} packet(s) have issues; see {}",
            signed_summary["verification"]["failed_packets"]
                .as_u64()
                .unwrap_or(0),
            out_path.display()
        );
    }

    Ok(())
}

fn load_required_pairs(
    workspace: &Path,
    benchmark_version: &BenchmarkVersion,
    experiment_config_path: &Path,
) -> Result<(ExperimentConfig, Vec<(String, String)>)> {
    let exp_raw = std::fs::read_to_string(experiment_config_path)
        .with_context(|| format!("reading {}", experiment_config_path.display()))?;
    let exp: ExperimentConfig = serde_json::from_str(&exp_raw)
        .with_context(|| format!("parsing {}", experiment_config_path.display()))?;

    let split_path = workspace
        .join("benchmark")
        .join(benchmark_version.as_str())
        .join("splits")
        .join(format!("{}.json", exp.split));
    let split_raw = std::fs::read_to_string(&split_path)
        .with_context(|| format!("reading {}", split_path.display()))?;
    let split: SplitFile = serde_json::from_str(&split_raw)
        .with_context(|| format!("parsing {}", split_path.display()))?;

    let mut required = Vec::new();
    for iid in &split.instance_ids {
        for sid in &exp.systems {
            required.push((iid.clone(), sid.clone()));
        }
    }
    Ok((exp, required))
}

fn load_pack_pairs(path: &Path) -> Result<HashSet<(String, String)>> {
    if !path.is_file() {
        return Ok(HashSet::new());
    }
    let raw = std::fs::read_to_string(path)?;
    let pack: AnnotationPack = serde_json::from_str(&raw)?;
    Ok(pack
        .records
        .into_iter()
        .map(|r| (r.instance_id.to_string(), r.system_id.to_string()))
        .collect())
}

fn write_assignment_matrix(
    path: &Path,
    benchmark_version: &str,
    required_pairs: &[(String, String)],
    present_pairs: &HashSet<(String, String)>,
) -> Result<()> {
    let mut lines = vec![
        "instance_id,system_id,annotator_1,annotator_2,adjudicator,status,raw_ann_01_path,raw_ann_02_path,adjudicated_path,notes".to_string(),
    ];
    for (iid, sid) in required_pairs {
        let status = if present_pairs.contains(&(iid.clone(), sid.clone())) {
            "packed"
        } else {
            "unassigned"
        };
        lines.push(format!(
            "{iid},{sid},,,,{status},benchmark/{benchmark_version}/annotation/raw/ann_01/{iid}__{sid}.json,benchmark/{benchmark_version}/annotation/raw/ann_02/{iid}__{sid}.json,benchmark/{benchmark_version}/annotation/raw/adjudicator/{iid}__{sid}.json,"
        ));
    }
    std::fs::write(path, lines.join("\n"))?;
    Ok(())
}

fn write_completion_log(path: &Path, missing_pairs: &[(String, String)]) -> Result<()> {
    let mut lines = vec!["timestamp,instance_id,system_id,status,notes".to_string()];
    if missing_pairs.is_empty() {
        let ts = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());
        lines.push(format!(
            "{ts},,,packed,all required pairs currently covered"
        ));
    }
    std::fs::write(path, lines.join("\n"))?;
    Ok(())
}

fn normalize_workspace_path(workspace: &Path, p: &Path) -> String {
    p.strip_prefix(workspace)
        .map(|r| r.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| p.to_string_lossy().replace('\\', "/"))
}

fn obligation_layer(
    kind: &str,
    linked_semantic_units: &serde_json::Value,
    critical_units: &HashSet<String>,
) -> &'static str {
    let has_critical_links = linked_semantic_units
        .as_array()
        .map(|a| {
            a.iter().any(|v| {
                v.as_str()
                    .map(|id| critical_units.contains(id))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);
    if has_critical_links
        && matches!(
            kind,
            "precondition" | "postcondition" | "optimality" | "termination"
        )
    {
        "benchmark_facing"
    } else {
        "auxiliary"
    }
}

fn build_quality_summary(
    semantic_units: &[serde_json::Value],
    generated_obligations: &[serde_json::Value],
) -> serde_json::Value {
    let critical_units: Vec<String> = semantic_units
        .iter()
        .filter(|u| {
            u.get("criticality")
                .and_then(|c| c.as_str())
                .map(|c| c == "critical")
                .unwrap_or(false)
        })
        .filter_map(|u| u.get("id").and_then(|id| id.as_str()).map(str::to_string))
        .collect();
    let optional_units: HashSet<String> = semantic_units
        .iter()
        .filter(|u| {
            u.get("criticality")
                .and_then(|c| c.as_str())
                .map(|c| c == "optional")
                .unwrap_or(false)
        })
        .filter_map(|u| u.get("id").and_then(|id| id.as_str()).map(str::to_string))
        .collect();

    let mut covered_direct = HashSet::new();
    let mut covered_indirect = HashSet::new();
    let mut off_spec_theorems_present = false;
    let mut vacuous_theorems_present = false;
    for ob in generated_obligations {
        let layer = ob
            .get("layer")
            .and_then(|v| v.as_str())
            .unwrap_or("auxiliary");
        let stmt = ob
            .get("lean_statement")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let stmt_norm = normalize_text(&stmt);
        let gloss = ob
            .get("nl_gloss")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let linked = ob
            .get("linked_semantic_units")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let linked_ids: Vec<String> = linked
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        if layer == "benchmark_facing" {
            if is_vacuous_or_filler(&stmt_norm, &gloss) {
                vacuous_theorems_present = true;
            }
            let linked_only_optional =
                !linked_ids.is_empty() && linked_ids.iter().all(|id| optional_units.contains(id));
            if stmt_norm.contains("stable")
                || stmt_norm.contains("stability")
                || gloss.contains("stability")
                || linked_only_optional
            {
                off_spec_theorems_present = true;
            }
        }
        for su in linked {
            if let Some(id) = su.as_str() {
                if layer == "benchmark_facing" {
                    covered_direct.insert(id.to_string());
                } else {
                    covered_indirect.insert(id.to_string());
                }
            }
        }
        let inferred = infer_critical_units_for_obligation(semantic_units, &stmt, &gloss);
        for id in inferred {
            if layer == "benchmark_facing" {
                covered_direct.insert(id);
            } else {
                covered_indirect.insert(id);
            }
        }
    }
    let critical_units_covered_by_direct_theorems: Vec<String> = critical_units
        .iter()
        .filter(|id| covered_direct.contains(*id))
        .cloned()
        .collect();
    let critical_units_only_indirectly_covered: Vec<String> = critical_units
        .iter()
        .filter(|id| !covered_direct.contains(*id) && covered_indirect.contains(*id))
        .cloned()
        .collect();

    json!({
        "critical_units_covered_by_direct_theorems": critical_units_covered_by_direct_theorems,
        "critical_units_only_indirectly_covered": critical_units_only_indirectly_covered,
        "off_spec_theorems_present": off_spec_theorems_present,
        "vacuous_theorems_present": vacuous_theorems_present
    })
}

fn normalize_text(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_vacuous_or_filler(stmt_norm: &str, gloss_lc: &str) -> bool {
    stmt_norm == "true"
        || stmt_norm.contains(": true := by trivial")
        || stmt_norm.contains(": true := by simp")
        || stmt_norm.contains(": prop := by trivial")
        || stmt_norm.contains(": prop := by simp")
        || stmt_norm.contains("-> true")
        || stmt_norm.contains("→ true")
        || stmt_norm.contains("∧ true")
        || stmt_norm.contains("placeholder")
        || gloss_lc.contains("placeholder")
        || (gloss_lc.contains("represents") && gloss_lc.contains("need to"))
}

fn infer_critical_units_for_obligation(
    semantic_units: &[serde_json::Value],
    statement_lc: &str,
    gloss_lc: &str,
) -> Vec<String> {
    let text = format!("{statement_lc} {gloss_lc}");
    let mut out = Vec::new();
    for su in semantic_units {
        let id = match su.get("id").and_then(|v| v.as_str()) {
            Some(v) => v,
            None => continue,
        };
        let critical = su
            .get("criticality")
            .and_then(|v| v.as_str())
            .map(|v| v == "critical")
            .unwrap_or(false);
        if !critical {
            continue;
        }
        let desc = su
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let matches = (desc.contains("sorted") && text.contains("sorted"))
            || (desc.contains("length") && text.contains("length"))
            || (desc.contains("source") && text.contains("source") && text.contains("some 0"))
            || (desc.contains("non-negative")
                && (text.contains("non-negative")
                    || text.contains(">= 0")
                    || text.contains("≥ 0")))
            || (desc.contains("start < stop")
                && ((text.contains(".1 <") && text.contains(".2"))
                    || text.contains("start < stop")))
            || (desc.contains("pairwise non-overlapping")
                && (text.contains("pairwise") || text.contains("non-overlapping")))
            || (desc.contains("optimality")
                && (text.contains("optimality")
                    || text.contains("no path")
                    || text.contains("strictly larger")))
            || (desc.contains("unreachability")
                && (text.contains("none")
                    && (text.contains("no path") || text.contains("unreachable"))));
        if matches {
            out.push(id.to_string());
        }
    }
    out
}

fn merge_semantic_units(existing: &serde_json::Value, inferred: &[String]) -> serde_json::Value {
    let mut seen = HashSet::new();
    let mut merged = Vec::new();
    if let Some(arr) = existing.as_array() {
        for v in arr {
            if let Some(id) = v.as_str() {
                if id.starts_with("SU") && seen.insert(id.to_string()) {
                    merged.push(serde_json::Value::String(id.to_string()));
                }
            }
        }
    }
    for id in inferred {
        if seen.insert(id.clone()) {
            merged.push(serde_json::Value::String(id.clone()));
        }
    }
    serde_json::Value::Array(merged)
}

fn path_to_slash_string(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

fn find_instance_root(
    workspace: &Path,
    benchmark_version: &str,
    instance_id: &str,
) -> Result<PathBuf> {
    let instances_root = workspace
        .join("benchmark")
        .join(benchmark_version)
        .join("instances");
    for entry in walkdir::WalkDir::new(&instances_root)
        .min_depth(2)
        .max_depth(2)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_dir())
    {
        if entry.file_name().to_string_lossy() == instance_id {
            return Ok(entry.into_path());
        }
    }
    anyhow::bail!(
        "instance {} not found under {}",
        instance_id,
        instances_root.display()
    )
}

fn build_run_index(workspace: &Path) -> Result<BTreeMap<String, Vec<PathBuf>>> {
    let runs_root = workspace.join("runs");
    let mut index: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    if !runs_root.is_dir() {
        return Ok(index);
    }
    for entry in std::fs::read_dir(&runs_root)? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.path().is_dir() {
            continue;
        }
        let run_id = entry.file_name().to_string_lossy().to_string();
        if !run_id.contains("_eval_") {
            continue;
        }
        index.entry(run_id).or_default().push(entry.path());
    }
    Ok(index)
}

fn resolve_generated_output_path(
    workspace: &Path,
    run_index: &BTreeMap<String, Vec<PathBuf>>,
    system_id: &str,
    instance_id: &str,
) -> Result<PathBuf> {
    let mut candidate_runs: Vec<&String> = run_index
        .keys()
        .filter(|run_id| run_id.contains(system_id) && run_id.contains("_eval_"))
        .collect();
    candidate_runs.sort();
    candidate_runs.reverse();
    for run_id in candidate_runs {
        let p = workspace
            .join("runs")
            .join(run_id)
            .join("generated")
            .join(system_id)
            .join(format!("{instance_id}.json"));
        if p.is_file() {
            return Ok(p);
        }
    }
    anyhow::bail!("no generated output found for ({instance_id}, {system_id}) under runs/*_eval_*")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
