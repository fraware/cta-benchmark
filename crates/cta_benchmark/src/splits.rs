//! Typed loader for benchmark split files (`splits/<name>.json`).
//!
//! The loader is deliberately strict: each split file must deserialise into a
//! [`Split`] with its declared `benchmark_version` matching the version root
//! it was loaded from, and its `split` discriminant matching the file stem.
//!
//! Splits are the handshake between the benchmark release and every
//! experiment config. They are consumed by:
//!
//! - the experiment runner (selecting which instances to generate against),
//! - the release-time coherence layer (see `release_checks`),
//! - the reports layer (slicing per-split metrics).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use cta_core::{BenchmarkVersion, InstanceId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Canonical split names recognised by the release pipeline.
///
/// `Challenge` is optional: releases may ship without a `challenge.json` file
/// and the release lint will not error in that case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SplitName {
    /// Diagnostic split used while designing prompts and tuning systems.
    Dev,
    /// Frozen evaluation split. Paper tables are computed on this split.
    Eval,
    /// Optional stress split (hard, adversarial). Empty/missing by default.
    Challenge,
}

impl SplitName {
    /// Canonical string form (matches the file stem and the experiment
    /// config `split` enum).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            SplitName::Dev => "dev",
            SplitName::Eval => "eval",
            SplitName::Challenge => "challenge",
        }
    }

    /// Parse from canonical string form.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(SplitName::Dev),
            "eval" => Some(SplitName::Eval),
            "challenge" => Some(SplitName::Challenge),
            _ => None,
        }
    }

    /// Splits that must exist and be non-empty in a released benchmark.
    ///
    /// `Dev` may be empty in theory, but convention (and every lint code we
    /// emit) assumes it is populated. `Eval` is the paper-facing split and
    /// is the only one enforced as mandatory non-empty.
    pub const REQUIRED: &'static [SplitName] = &[SplitName::Dev, SplitName::Eval];
}

/// Errors produced while loading or validating split files.
#[derive(Debug, Error)]
pub enum SplitError {
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
    #[error("failed to parse split file {path}: {source}")]
    Json {
        /// Path being parsed.
        path: PathBuf,
        /// Underlying serde_json error.
        #[source]
        source: serde_json::Error,
    },

    /// Split file's `benchmark_version` does not match its directory.
    #[error("benchmark version mismatch at {path}: declared='{declared}', expected='{expected}'")]
    VersionMismatch {
        /// Path being parsed.
        path: PathBuf,
        /// Declared version.
        declared: String,
        /// Expected version.
        expected: String,
    },

    /// Split file's `split` field does not match its file stem.
    #[error("split name mismatch at {path}: declared='{declared}', file stem='{stem}'")]
    NameMismatch {
        /// Path being parsed.
        path: PathBuf,
        /// Declared split name.
        declared: String,
        /// File stem.
        stem: String,
    },
}

/// Result alias.
pub type Result<T> = std::result::Result<T, SplitError>;

/// In-memory representation of a single split file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Split {
    /// Schema version constant (`schema_v1`).
    pub schema_version: String,
    /// Declared benchmark version (e.g. `v0.1`).
    pub benchmark_version: BenchmarkVersion,
    /// Split discriminant (matches the file stem).
    pub split: SplitName,
    /// Instance ids selected for this split. Order matters: we preserve the
    /// author's ordering but release lints enforce uniqueness.
    pub instance_ids: Vec<InstanceId>,
    /// Absolute path the split was loaded from (set by [`load_splits`], not
    /// serialised). Keeps lint messages informative.
    #[serde(skip, default)]
    pub source_path: PathBuf,
}

impl Split {
    /// True if this split has no instances.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.instance_ids.is_empty()
    }

    /// Number of instance ids in this split.
    #[must_use]
    pub fn len(&self) -> usize {
        self.instance_ids.len()
    }
}

/// Discover and load every split under `<version_root>/splits/*.json`.
///
/// Returns a map keyed by [`SplitName`]. Only canonical split names
/// (`dev`, `eval`, `challenge`) are recognised; other JSON files in the
/// `splits/` directory are ignored so future tooling can drop supplemental
/// metadata (e.g. sampling seeds) alongside the splits without this loader
/// erroring.
///
/// # Errors
/// Returns the first parse/consistency error encountered.
pub fn load_splits(
    version_root: impl AsRef<Path>,
    expected_version: &BenchmarkVersion,
) -> Result<BTreeMap<SplitName, Split>> {
    let splits_dir = version_root.as_ref().join("splits");
    let mut out = BTreeMap::new();
    if !splits_dir.is_dir() {
        return Ok(out);
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(&splits_dir)
        .map_err(|source| SplitError::Io {
            path: splits_dir.clone(),
            source,
        })?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    entries.sort();

    for path in entries {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let Some(name) = SplitName::parse(&stem) else {
            continue;
        };

        let raw = fs::read_to_string(&path).map_err(|source| SplitError::Io {
            path: path.clone(),
            source,
        })?;
        let mut split: Split = serde_json::from_str(&raw).map_err(|source| SplitError::Json {
            path: path.clone(),
            source,
        })?;
        if split.benchmark_version.as_str() != expected_version.as_str() {
            return Err(SplitError::VersionMismatch {
                path,
                declared: split.benchmark_version.as_str().to_string(),
                expected: expected_version.as_str().to_string(),
            });
        }
        if split.split != name {
            return Err(SplitError::NameMismatch {
                path,
                declared: split.split.as_str().to_string(),
                stem,
            });
        }
        split.source_path = path.clone();
        out.insert(name, split);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write(path: &Path, json: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = fs::File::create(path).unwrap();
        f.write_all(json.as_bytes()).unwrap();
    }

    #[test]
    fn split_name_round_trip() {
        for name in [SplitName::Dev, SplitName::Eval, SplitName::Challenge] {
            assert_eq!(SplitName::parse(name.as_str()), Some(name));
        }
        assert!(SplitName::parse("nope").is_none());
    }

    #[test]
    fn load_splits_reads_only_canonical_stems() {
        let tmp = tempfile::tempdir().unwrap();
        let splits_dir = tmp.path().join("splits");
        write(
            &splits_dir.join("dev.json"),
            r#"{"schema_version":"schema_v1","benchmark_version":"v0.1","split":"dev","instance_ids":["arrays_binary_search_001"]}"#,
        );
        write(
            &splits_dir.join("eval.json"),
            r#"{"schema_version":"schema_v1","benchmark_version":"v0.1","split":"eval","instance_ids":[]}"#,
        );
        write(&splits_dir.join("notes.json"), r#"{"unrelated":true}"#);
        let v = BenchmarkVersion::new("v0.1").unwrap();
        let got = load_splits(tmp.path(), &v).unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[&SplitName::Dev].len(), 1);
        assert!(got[&SplitName::Eval].is_empty());
    }

    #[test]
    fn load_splits_rejects_version_mismatch() {
        let tmp = tempfile::tempdir().unwrap();
        let splits_dir = tmp.path().join("splits");
        write(
            &splits_dir.join("dev.json"),
            r#"{"schema_version":"schema_v1","benchmark_version":"v0.2","split":"dev","instance_ids":[]}"#,
        );
        let v = BenchmarkVersion::new("v0.1").unwrap();
        let err = load_splits(tmp.path(), &v).unwrap_err();
        assert!(matches!(err, SplitError::VersionMismatch { .. }));
    }

    #[test]
    fn load_splits_rejects_name_mismatch() {
        let tmp = tempfile::tempdir().unwrap();
        let splits_dir = tmp.path().join("splits");
        write(
            &splits_dir.join("dev.json"),
            r#"{"schema_version":"schema_v1","benchmark_version":"v0.1","split":"eval","instance_ids":[]}"#,
        );
        let v = BenchmarkVersion::new("v0.1").unwrap();
        let err = load_splits(tmp.path(), &v).unwrap_err();
        assert!(matches!(err, SplitError::NameMismatch { .. }));
    }

    #[test]
    fn missing_splits_dir_is_not_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        let v = BenchmarkVersion::new("v0.1").unwrap();
        let got = load_splits(tmp.path(), &v).unwrap();
        assert!(got.is_empty());
    }
}
