//! Benchmark scanner and typed loader.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use cta_core::{BenchmarkVersion, InstanceId};
use thiserror::Error;
use walkdir::WalkDir;

use crate::model::{InstanceRecord, InstanceView};

/// Errors produced while loading benchmark contents.
#[derive(Debug, Error)]
pub enum LoadError {
    /// The benchmark version directory does not exist.
    #[error("benchmark version directory not found: {0}")]
    VersionRootMissing(PathBuf),

    /// IO failure.
    #[error("io error at {path}: {source}")]
    Io {
        /// Path being accessed.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// JSON parse failure.
    #[error("failed to parse {path}: {source}")]
    Json {
        /// Path being parsed.
        path: PathBuf,
        /// Underlying serde_json error.
        #[source]
        source: serde_json::Error,
    },

    /// Instance's declared id did not match its directory layout.
    #[error("instance id mismatch at {dir}: instance_id='{declared}', directory='{dir_name}'")]
    InstanceIdMismatch {
        /// Instance directory.
        dir: PathBuf,
        /// Id declared in instance.json.
        declared: String,
        /// Instance directory basename.
        dir_name: String,
    },

    /// Instance declared the wrong benchmark version.
    #[error("benchmark version mismatch at {dir}: declared='{declared}', expected='{expected}'")]
    VersionMismatch {
        /// Instance directory.
        dir: PathBuf,
        /// Declared version.
        declared: String,
        /// Loader-expected version.
        expected: String,
    },
}

/// Result alias.
pub type Result<T> = std::result::Result<T, LoadError>;

/// Fully loaded benchmark, indexed by [`InstanceId`].
#[derive(Debug)]
pub struct LoadedBenchmark {
    /// Version (e.g. `v0.1`).
    pub version: BenchmarkVersion,
    /// Root path (e.g. `benchmark/v0.1`).
    pub root: PathBuf,
    /// Map of instance id to its loaded view.
    pub instances: BTreeMap<InstanceId, InstanceView>,
}

impl LoadedBenchmark {
    /// Iterate instances in id order.
    pub fn iter(&self) -> impl Iterator<Item = (&InstanceId, &InstanceView)> {
        self.instances.iter()
    }

    /// Number of loaded instances.
    #[must_use]
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Whether the benchmark is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
}

/// Load a versioned benchmark directory, e.g. `benchmark/v0.1`.
pub fn load_benchmark(
    version_root: impl AsRef<Path>,
    version: &BenchmarkVersion,
) -> Result<LoadedBenchmark> {
    let root = version_root.as_ref().to_path_buf();
    if !root.is_dir() {
        return Err(LoadError::VersionRootMissing(root));
    }

    let instances_root = root.join("instances");
    if !instances_root.is_dir() {
        return Err(LoadError::VersionRootMissing(instances_root));
    }

    let mut instances = BTreeMap::new();

    // Instance directories are exactly two levels deep: instances/<domain>/<id>/
    for domain_entry in read_dir(&instances_root)? {
        if !domain_entry.is_dir() {
            continue;
        }
        for inst_entry in read_dir(&domain_entry)? {
            if !inst_entry.is_dir() {
                continue;
            }
            let instance_json = inst_entry.join("instance.json");
            if !instance_json.is_file() {
                continue;
            }
            let view = load_instance(&inst_entry, &instance_json, version, &root)?;
            instances.insert(view.record.instance_id.clone(), view);
        }
    }

    Ok(LoadedBenchmark {
        version: version.clone(),
        root,
        instances,
    })
}

fn read_dir(path: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in WalkDir::new(path).min_depth(1).max_depth(1) {
        let entry = entry.map_err(|e| LoadError::Io {
            path: path.to_path_buf(),
            source: std::io::Error::other(e.to_string()),
        })?;
        out.push(entry.into_path());
    }
    out.sort();
    Ok(out)
}

fn load_instance(
    dir: &Path,
    instance_json: &Path,
    expected_version: &BenchmarkVersion,
    bench_root: &Path,
) -> Result<InstanceView> {
    let raw = fs::read_to_string(instance_json).map_err(|source| LoadError::Io {
        path: instance_json.to_path_buf(),
        source,
    })?;
    let record: InstanceRecord = serde_json::from_str(&raw).map_err(|source| LoadError::Json {
        path: instance_json.to_path_buf(),
        source,
    })?;

    let dir_name = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    if dir_name != record.instance_id.as_str() {
        return Err(LoadError::InstanceIdMismatch {
            dir: dir.to_path_buf(),
            declared: record.instance_id.to_string(),
            dir_name,
        });
    }

    if record.benchmark_version.as_str() != expected_version.as_str() {
        return Err(LoadError::VersionMismatch {
            dir: dir.to_path_buf(),
            declared: record.benchmark_version.to_string(),
            expected: expected_version.to_string(),
        });
    }

    let reference_rs = bench_root.join(&record.rust_reference.path);
    let scaffold_lean = bench_root.join(&record.lean_target.scaffold_path);
    let reference_obligations = bench_root.join(&record.lean_target.reference_obligations_path);
    let semantic_units = bench_root.join(&record.lean_target.semantic_units_path);
    let harness = bench_root.join(&record.behavioral_oracle.harness_path);

    Ok(InstanceView {
        record,
        dir: dir.to_path_buf(),
        instance_json: instance_json.to_path_buf(),
        reference_rs,
        scaffold_lean,
        reference_obligations,
        semantic_units,
        harness,
    })
}
