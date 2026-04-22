use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Args;
use cta_benchmark::load_benchmark;
use cta_core::BenchmarkVersion;
use cta_schema::{SchemaName, SchemaRegistry};
use walkdir::WalkDir;

use super::{benchmark_dir, schemas_dir};

/// Validate a single JSON artifact against a named schema.
#[derive(Debug, Args)]
pub struct FileArgs {
    /// Schema name (e.g. `instance`, `run_manifest`, `results_bundle`, `experiment`).
    #[arg(long)]
    pub schema: String,
    /// Path to the JSON artifact.
    #[arg(long)]
    pub path: PathBuf,
    /// Optional schemas root override.
    #[arg(long)]
    pub schemas: Option<PathBuf>,
}

pub fn file(workspace: &Path, args: FileArgs) -> Result<()> {
    let schemas_root = args.schemas.unwrap_or_else(|| schemas_dir(workspace));
    let reg = SchemaRegistry::load(&schemas_root)
        .with_context(|| format!("loading schemas from {}", schemas_root.display()))?;
    let schema_name = SchemaName::parse(&args.schema)
        .ok_or_else(|| anyhow::anyhow!("unknown schema name: {}", args.schema))?;
    reg.validate_file(schema_name, &args.path)
        .with_context(|| format!("validating {}", args.path.display()))?;
    println!(
        "ok: {} validates against {:?}",
        args.path.display(),
        schema_name
    );
    Ok(())
}

#[derive(Debug, Args)]
pub struct SchemasArgs {
    /// Path to schemas directory (defaults to `<workspace>/schemas`).
    #[arg(long)]
    pub schemas: Option<PathBuf>,
}

pub fn schemas(workspace: &Path, args: SchemasArgs) -> Result<()> {
    let dir = args.schemas.unwrap_or_else(|| schemas_dir(workspace));
    let reg = SchemaRegistry::load(&dir)
        .with_context(|| format!("failed to load schemas from {}", dir.display()))?;
    println!(
        "loaded {} canonical schemas from {}",
        SchemaName::ALL.len(),
        reg.root().display()
    );
    for name in SchemaName::ALL {
        println!("  - {:?} ({})", name, name.file_name());
    }
    Ok(())
}

#[derive(Debug, Args)]
pub struct BenchmarkArgs {
    /// Benchmark version, e.g. `v0.1`.
    #[arg(long, default_value = "v0.1", value_parser = crate::parse_bench_version)]
    pub version: BenchmarkVersion,
}

pub fn benchmark(workspace: &Path, args: BenchmarkArgs) -> Result<()> {
    let schemas_root = schemas_dir(workspace);
    let reg = SchemaRegistry::load(&schemas_root)
        .with_context(|| format!("loading schemas from {}", schemas_root.display()))?;
    let bench_root = benchmark_dir(workspace, args.version.as_str());
    let bench = load_benchmark(&bench_root, &args.version)?;

    let mut failures = 0usize;
    for (id, view) in bench.iter() {
        if let Err(e) = reg.validate_file(SchemaName::Instance, &view.instance_json) {
            failures += 1;
            eprintln!("[fail] {id}: {e}");
            continue;
        }
        if view.reference_obligations.is_file() {
            if let Err(e) = reg.validate_file(SchemaName::Obligation, &view.reference_obligations) {
                failures += 1;
                eprintln!("[fail] {id} obligations: {e}");
            }
        }
        if view.semantic_units.is_file() {
            if let Err(e) = reg.validate_file(SchemaName::SemanticUnits, &view.semantic_units) {
                failures += 1;
                eprintln!("[fail] {id} semantic_units: {e}");
            }
        }
        if view.harness.is_file() {
            if let Err(e) = reg.validate_file(SchemaName::Harness, &view.harness) {
                failures += 1;
                eprintln!("[fail] {id} harness: {e}");
            }
        }
    }

    // Additionally, if a benchmark_manifest.json exists under manifests/, validate it.
    let manifests = bench_root.join("manifests");
    if manifests.is_dir() {
        for entry in WalkDir::new(&manifests).min_depth(1).max_depth(1) {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Err(e) = reg.validate_file(SchemaName::BenchmarkManifest, path) {
                    failures += 1;
                    eprintln!("[fail] manifest {}: {e}", path.display());
                }
            }
        }
    }

    if failures > 0 {
        anyhow::bail!("{failures} validation failure(s)");
    }
    println!(
        "ok: validated {} instance(s) under {}",
        bench.len(),
        bench_root.display()
    );
    Ok(())
}
