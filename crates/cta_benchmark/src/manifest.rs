//! Deterministic benchmark manifest computation.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use cta_core::{BenchmarkVersion, Domain, MetricsVersion, RubricVersion};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::loader::LoadedBenchmark;

/// In-memory representation matching `benchmark_manifest.schema.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkManifest {
    /// Schema version constant.
    pub schema_version: String,
    /// Benchmark version.
    pub benchmark_version: BenchmarkVersion,
    /// Rubric version.
    pub rubric_version: RubricVersion,
    /// Metrics contract version.
    pub metrics_version: MetricsVersion,
    /// ISO-8601 UTC creation timestamp.
    pub created_at: String,
    /// Per-domain instance count.
    pub instance_count_by_domain: BTreeMap<String, u32>,
    /// Instance id -> canonical directory sha256.
    pub instance_hashes: BTreeMap<String, String>,
    /// Sha256 fingerprint of the full (sorted id, hash) list.
    pub content_hash: String,
}

/// Build a deterministic manifest from a loaded benchmark.
///
/// Timestamp is provided explicitly to keep this function pure and testable.
pub fn build_manifest(
    b: &LoadedBenchmark,
    rubric: &RubricVersion,
    metrics: &MetricsVersion,
    created_at: &str,
) -> std::io::Result<BenchmarkManifest> {
    let mut instance_hashes = BTreeMap::new();
    let mut instance_count_by_domain: BTreeMap<String, u32> = BTreeMap::new();

    for d in Domain::ALL {
        instance_count_by_domain.insert(d.as_str().to_string(), 0);
    }

    for (id, view) in b.iter() {
        let h = hash_instance_dir(&view.dir)?;
        instance_hashes.insert(id.to_string(), format!("sha256:{h}"));
        *instance_count_by_domain
            .entry(view.record.domain.as_str().to_string())
            .or_insert(0) += 1;
    }

    // Rollup fingerprint: sha256 over `id\0hash\n` lines in sorted order.
    let mut hasher = Sha256::new();
    for (id, h) in &instance_hashes {
        hasher.update(id.as_bytes());
        hasher.update([0u8]);
        hasher.update(h.as_bytes());
        hasher.update([b'\n']);
    }
    let content_hash = format!("sha256:{}", hex::encode(hasher.finalize()));

    Ok(BenchmarkManifest {
        schema_version: cta_core::SCHEMA_VERSION.to_string(),
        benchmark_version: b.version.clone(),
        rubric_version: rubric.clone(),
        metrics_version: metrics.clone(),
        created_at: created_at.to_string(),
        instance_count_by_domain,
        instance_hashes,
        content_hash,
    })
}

/// Canonical hash of an instance directory: hashes each file's relative path
/// and contents in sorted order. Directory metadata and ordering are ignored.
fn hash_instance_dir(dir: &Path) -> std::io::Result<String> {
    let mut files: Vec<_> = WalkDir::new(dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .collect();
    files.sort();

    let mut hasher = Sha256::new();
    for path in &files {
        let rel = path.strip_prefix(dir).unwrap_or(path);
        // Normalize path separators to forward slashes for cross-platform stability.
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        hasher.update(rel_str.as_bytes());
        hasher.update([0u8]);
        let contents = fs::read(path)?;
        hasher.update(&contents);
        hasher.update([b'\n']);
    }
    Ok(hex::encode(hasher.finalize()))
}
