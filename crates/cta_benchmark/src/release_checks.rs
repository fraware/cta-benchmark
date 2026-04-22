//! Global release-coherence checks across benchmark contents, splits,
//! manifest, and experiment configs.
//!
//! These are the checks CI runs before a benchmark version is declared
//! paper-reportable. They complement the per-instance pass in [`crate::lint`]
//! by catching inconsistencies *between* artifacts:
//!
//! - split references an id that isn't in the instance set,
//! - split contains duplicate ids,
//! - eval split is empty,
//! - manifest's instance set disagrees with the loaded benchmark,
//! - manifest's per-domain counts disagree with loaded benchmark,
//! - manifest's content hash is stale,
//! - experiment config points at an empty split,
//! - experiment config references a system without a prompt config,
//! - experiment config references a provider config that is missing.
//!
//! Every check returns a stable machine-readable code; the CLI and CI
//! workflows can grep on codes without pinning to human-facing wording.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use cta_core::{BenchmarkVersion, MetricsVersion, RubricVersion};
use serde::{Deserialize, Serialize};

use crate::lint::{LintIssue, LintReport, LintSeverity};
use crate::loader::LoadedBenchmark;
use crate::manifest::{build_manifest, BenchmarkManifest};
use crate::splits::{Split, SplitName};

/// Minimal view of an experiment config used by the release-coherence pass.
///
/// Parsed directly from `configs/experiments/*.json` without going through
/// `cta_cli`, so this crate stays dependency-free at the workflow layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfigSummary {
    /// Canonical experiment id.
    pub experiment_id: String,
    /// Benchmark version this experiment targets.
    pub benchmark_version: BenchmarkVersion,
    /// Split the experiment runs on.
    pub split: String,
    /// System ids referenced by the experiment.
    pub systems: Vec<String>,
    /// Workspace-relative provider config paths.
    pub providers: Vec<String>,
    /// Source file path, for reporting.
    #[serde(skip, default)]
    pub source_path: PathBuf,
}

/// Inputs consumed by [`validate_release`].
///
/// The workspace root is required so we can resolve `providers` / `prompts`
/// paths without mirroring CLI-layer assumptions in this crate.
#[derive(Debug, Clone)]
pub struct ReleaseCheckContext<'a> {
    /// Workspace root (parent of `benchmark/`).
    pub workspace_root: &'a Path,
    /// Loaded benchmark contents.
    pub benchmark: &'a LoadedBenchmark,
    /// Splits keyed by canonical name. Missing optional splits are allowed.
    pub splits: &'a BTreeMap<SplitName, Split>,
    /// Persisted manifest (typically from `manifests/benchmark_manifest.json`).
    /// If `None`, only existence is reported.
    pub manifest: Option<&'a BenchmarkManifest>,
    /// Experiment configs to cross-check.
    pub experiments: &'a [ExperimentConfigSummary],
    /// Rubric version to rebuild the manifest under for hash comparison.
    pub rubric_version: &'a RubricVersion,
    /// Metrics version to rebuild the manifest under for hash comparison.
    pub metrics_version: &'a MetricsVersion,
}

/// Run every release-level coherence check. Findings are returned as a
/// [`LintReport`]; callers that want to gate CI should call
/// [`LintReport::has_errors`].
#[must_use]
pub fn validate_release(ctx: &ReleaseCheckContext<'_>) -> LintReport {
    let mut issues: Vec<LintIssue> = Vec::new();

    let instance_ids: BTreeSet<String> = ctx
        .benchmark
        .iter()
        .map(|(id, _)| id.as_str().to_string())
        .collect();

    check_splits(ctx.splits, &instance_ids, &mut issues);
    check_manifest(ctx, &instance_ids, &mut issues);
    check_experiments(ctx, &mut issues);

    LintReport { issues }
}

fn check_splits(
    splits: &BTreeMap<SplitName, Split>,
    instance_ids: &BTreeSet<String>,
    issues: &mut Vec<LintIssue>,
) {
    for name in SplitName::REQUIRED {
        if !splits.contains_key(name) {
            issues.push(LintIssue {
                instance_id: "<global>".to_string(),
                severity: LintSeverity::Error,
                code: "SPLIT_REQUIRED_MISSING",
                message: format!(
                    "required split '{}' is missing (expected file splits/{}.json)",
                    name.as_str(),
                    name.as_str()
                ),
                path: None,
            });
        }
    }

    for (name, split) in splits {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for iid in &split.instance_ids {
            if !seen.insert(iid.as_str()) {
                issues.push(LintIssue {
                    instance_id: iid.as_str().to_string(),
                    severity: LintSeverity::Error,
                    code: "SPLIT_DUPLICATE_INSTANCE",
                    message: format!(
                        "split '{split}' contains duplicate instance id '{iid}'",
                        split = name.as_str(),
                        iid = iid.as_str()
                    ),
                    path: Some(split.source_path.clone()),
                });
            }
            if !instance_ids.contains(iid.as_str()) {
                issues.push(LintIssue {
                    instance_id: iid.as_str().to_string(),
                    severity: LintSeverity::Error,
                    code: "SPLIT_UNKNOWN_INSTANCE",
                    message: format!(
                        "split '{split}' references instance '{iid}' which is not present in the benchmark",
                        split = name.as_str(),
                        iid = iid.as_str()
                    ),
                    path: Some(split.source_path.clone()),
                });
            }
        }
    }

    if let Some(eval) = splits.get(&SplitName::Eval) {
        if eval.is_empty() {
            issues.push(LintIssue {
                instance_id: "<global>".to_string(),
                severity: LintSeverity::Error,
                code: "SPLIT_EMPTY_EVAL",
                message: "eval split is empty; a released benchmark must have a non-empty eval"
                    .to_string(),
                path: Some(eval.source_path.clone()),
            });
        }
    }
}

fn check_manifest(
    ctx: &ReleaseCheckContext<'_>,
    instance_ids: &BTreeSet<String>,
    issues: &mut Vec<LintIssue>,
) {
    let Some(manifest) = ctx.manifest else {
        issues.push(LintIssue {
            instance_id: "<global>".to_string(),
            severity: LintSeverity::Error,
            code: "MANIFEST_MISSING",
            message: "benchmark manifest not found; regenerate with `cta benchmark manifest`"
                .to_string(),
            path: None,
        });
        return;
    };

    let manifest_ids: BTreeSet<String> = manifest.instance_hashes.keys().cloned().collect();
    if manifest_ids != *instance_ids {
        let only_in_manifest: Vec<_> = manifest_ids.difference(instance_ids).collect();
        let only_on_disk: Vec<_> = instance_ids.difference(&manifest_ids).collect();
        issues.push(LintIssue {
            instance_id: "<global>".to_string(),
            severity: LintSeverity::Error,
            code: "MANIFEST_INSTANCE_SET_MISMATCH",
            message: format!(
                "manifest instance set disagrees with loaded benchmark: only-in-manifest={only_in_manifest:?}, only-on-disk={only_on_disk:?}"
            ),
            path: None,
        });
    }

    let mut on_disk_counts: BTreeMap<String, u32> = BTreeMap::new();
    for (_, view) in ctx.benchmark.iter() {
        *on_disk_counts
            .entry(view.record.domain.as_str().to_string())
            .or_insert(0) += 1;
    }
    for d in cta_core::Domain::ALL {
        on_disk_counts.entry(d.as_str().to_string()).or_insert(0);
    }
    if on_disk_counts != manifest.instance_count_by_domain {
        issues.push(LintIssue {
            instance_id: "<global>".to_string(),
            severity: LintSeverity::Error,
            code: "MANIFEST_DOMAIN_COUNT_MISMATCH",
            message: format!(
                "manifest instance_count_by_domain={:?} disagrees with on-disk counts={:?}",
                manifest.instance_count_by_domain, on_disk_counts
            ),
            path: None,
        });
    }

    // Rebuild and compare content_hash. Timestamp does not enter the hash so
    // this is deterministic.
    match build_manifest(
        ctx.benchmark,
        ctx.rubric_version,
        ctx.metrics_version,
        manifest.created_at.as_str(),
    ) {
        Ok(fresh) => {
            if fresh.content_hash != manifest.content_hash {
                issues.push(LintIssue {
                    instance_id: "<global>".to_string(),
                    severity: LintSeverity::Error,
                    code: "MANIFEST_CONTENT_HASH_STALE",
                    message: format!(
                        "manifest content_hash={} does not match recomputed={}; regenerate with \
                         `cta benchmark manifest --version {}`",
                        manifest.content_hash,
                        fresh.content_hash,
                        manifest.benchmark_version,
                    ),
                    path: None,
                });
            }
        }
        Err(e) => {
            issues.push(LintIssue {
                instance_id: "<global>".to_string(),
                severity: LintSeverity::Error,
                code: "MANIFEST_RECOMPUTE_FAILED",
                message: format!("failed to recompute manifest for comparison: {e}"),
                path: None,
            });
        }
    }
}

fn check_experiments(ctx: &ReleaseCheckContext<'_>, issues: &mut Vec<LintIssue>) {
    for exp in ctx.experiments {
        let Some(name) = SplitName::parse(&exp.split) else {
            issues.push(LintIssue {
                instance_id: "<global>".to_string(),
                severity: LintSeverity::Error,
                code: "EXPERIMENT_UNKNOWN_SPLIT",
                message: format!(
                    "experiment '{}' references unknown split '{}'",
                    exp.experiment_id, exp.split
                ),
                path: Some(exp.source_path.clone()),
            });
            continue;
        };
        match ctx.splits.get(&name) {
            Some(split) if !split.is_empty() => {}
            Some(split) => {
                issues.push(LintIssue {
                    instance_id: "<global>".to_string(),
                    severity: LintSeverity::Error,
                    code: "EXPERIMENT_REFERENCES_EMPTY_SPLIT",
                    message: format!(
                        "experiment '{exp}' references empty split '{split}' at {path}",
                        exp = exp.experiment_id,
                        split = name.as_str(),
                        path = split.source_path.display(),
                    ),
                    path: Some(exp.source_path.clone()),
                });
            }
            None => {
                issues.push(LintIssue {
                    instance_id: "<global>".to_string(),
                    severity: LintSeverity::Error,
                    code: "EXPERIMENT_REFERENCES_EMPTY_SPLIT",
                    message: format!(
                        "experiment '{exp}' references split '{split}' which is not shipped with this benchmark version",
                        exp = exp.experiment_id,
                        split = name.as_str(),
                    ),
                    path: Some(exp.source_path.clone()),
                });
            }
        }

        let prompts_root = ctx.workspace_root.join("configs").join("prompts");
        for sys in &exp.systems {
            let prompt_path = prompts_root.join(format!("{sys}.json"));
            if !prompt_path.is_file() {
                issues.push(LintIssue {
                    instance_id: "<global>".to_string(),
                    severity: LintSeverity::Error,
                    code: "EXPERIMENT_REFERENCES_UNKNOWN_SYSTEM",
                    message: format!(
                        "experiment '{exp}' references system '{sys}' but no prompt config exists at {path}",
                        exp = exp.experiment_id,
                        path = prompt_path.display(),
                    ),
                    path: Some(exp.source_path.clone()),
                });
            }
        }

        for prov in &exp.providers {
            let prov_path = ctx.workspace_root.join(prov);
            if !prov_path.is_file() {
                issues.push(LintIssue {
                    instance_id: "<global>".to_string(),
                    severity: LintSeverity::Error,
                    code: "EXPERIMENT_REFERENCES_MISSING_PROVIDER_CONFIG",
                    message: format!(
                        "experiment '{exp}' references provider config '{prov}' but {path} is missing",
                        exp = exp.experiment_id,
                        path = prov_path.display(),
                    ),
                    path: Some(exp.source_path.clone()),
                });
            }
        }
    }
}

/// Parse every `configs/experiments/*.json` under a workspace root into
/// [`ExperimentConfigSummary`] values. Files that fail to parse return a
/// synthetic issue in the second element of the tuple so the caller can
/// surface them without aborting the rest of the release pass.
///
/// # Errors
/// Returns IO errors reading the experiments directory. JSON parse errors
/// are reported as lint issues.
pub fn load_experiment_summaries(
    workspace_root: &Path,
) -> std::io::Result<(Vec<ExperimentConfigSummary>, Vec<LintIssue>)> {
    let dir = workspace_root.join("configs").join("experiments");
    let mut summaries: Vec<ExperimentConfigSummary> = Vec::new();
    let mut issues: Vec<LintIssue> = Vec::new();
    if !dir.is_dir() {
        return Ok((summaries, issues));
    }

    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    entries.sort();

    for path in entries {
        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                issues.push(LintIssue {
                    instance_id: "<global>".to_string(),
                    severity: LintSeverity::Error,
                    code: "EXPERIMENT_CONFIG_READ",
                    message: format!("failed to read {}: {e}", path.display()),
                    path: Some(path.clone()),
                });
                continue;
            }
        };
        match serde_json::from_str::<ExperimentConfigSummary>(&raw) {
            Ok(mut s) => {
                s.source_path = path.clone();
                summaries.push(s);
            }
            Err(e) => {
                issues.push(LintIssue {
                    instance_id: "<global>".to_string(),
                    severity: LintSeverity::Error,
                    code: "EXPERIMENT_CONFIG_PARSE",
                    message: format!("failed to parse {}: {e}", path.display()),
                    path: Some(path.clone()),
                });
            }
        }
    }
    Ok((summaries, issues))
}

/// Read a `benchmark_manifest.json` from the conventional location
/// `<version_root>/manifests/benchmark_manifest.json`.
///
/// Returns `Ok(None)` if the file does not exist so callers can decide
/// whether a missing manifest is an error (it is, in release mode).
///
/// # Errors
/// Returns an error if the file exists but fails to parse.
pub fn load_manifest(version_root: &Path) -> anyhow::Result<Option<BenchmarkManifest>> {
    let path = version_root
        .join("manifests")
        .join("benchmark_manifest.json");
    if !path.is_file() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)?;
    let m: BenchmarkManifest = serde_json::from_str(&raw)?;
    Ok(Some(m))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::LoadedBenchmark;
    use crate::splits::{Split, SplitName};
    use cta_core::{BenchmarkVersion, InstanceId, MetricsVersion, RubricVersion};
    use std::collections::BTreeMap;

    fn empty_benchmark() -> LoadedBenchmark {
        LoadedBenchmark {
            version: BenchmarkVersion::new("v0.1").unwrap(),
            root: PathBuf::new(),
            instances: BTreeMap::new(),
        }
    }

    fn split(name: SplitName, ids: &[&str]) -> Split {
        Split {
            schema_version: "schema_v1".to_string(),
            benchmark_version: BenchmarkVersion::new("v0.1").unwrap(),
            split: name,
            instance_ids: ids
                .iter()
                .map(|s| InstanceId::new(*s).unwrap())
                .collect(),
            source_path: PathBuf::from(format!("splits/{}.json", name.as_str())),
        }
    }

    #[test]
    fn empty_eval_is_error() {
        let bench = empty_benchmark();
        let mut splits = BTreeMap::new();
        splits.insert(
            SplitName::Dev,
            split(SplitName::Dev, &["arrays_binary_search_001"]),
        );
        splits.insert(SplitName::Eval, split(SplitName::Eval, &[]));
        let rubric = RubricVersion::new("rubric_v1").unwrap();
        let metrics = MetricsVersion::new("metrics_v2").unwrap();
        let ctx = ReleaseCheckContext {
            workspace_root: Path::new("/tmp"),
            benchmark: &bench,
            splits: &splits,
            manifest: None,
            experiments: &[],
            rubric_version: &rubric,
            metrics_version: &metrics,
        };
        let report = validate_release(&ctx);
        assert!(report
            .issues
            .iter()
            .any(|i| i.code == "SPLIT_EMPTY_EVAL"));
    }

    #[test]
    fn unknown_instance_in_split_is_error() {
        let bench = empty_benchmark();
        let mut splits = BTreeMap::new();
        splits.insert(
            SplitName::Dev,
            split(SplitName::Dev, &["arrays_binary_search_001"]),
        );
        splits.insert(SplitName::Eval, split(SplitName::Eval, &["sorting_merge_sort_001"]));
        let rubric = RubricVersion::new("rubric_v1").unwrap();
        let metrics = MetricsVersion::new("metrics_v2").unwrap();
        let ctx = ReleaseCheckContext {
            workspace_root: Path::new("/tmp"),
            benchmark: &bench,
            splits: &splits,
            manifest: None,
            experiments: &[],
            rubric_version: &rubric,
            metrics_version: &metrics,
        };
        let report = validate_release(&ctx);
        assert!(report
            .issues
            .iter()
            .any(|i| i.code == "SPLIT_UNKNOWN_INSTANCE"));
    }

    #[test]
    fn missing_manifest_is_error() {
        let bench = empty_benchmark();
        let mut splits = BTreeMap::new();
        splits.insert(
            SplitName::Dev,
            split(SplitName::Dev, &[]),
        );
        splits.insert(SplitName::Eval, split(SplitName::Eval, &[]));
        let rubric = RubricVersion::new("rubric_v1").unwrap();
        let metrics = MetricsVersion::new("metrics_v2").unwrap();
        let ctx = ReleaseCheckContext {
            workspace_root: Path::new("/tmp"),
            benchmark: &bench,
            splits: &splits,
            manifest: None,
            experiments: &[],
            rubric_version: &rubric,
            metrics_version: &metrics,
        };
        let report = validate_release(&ctx);
        assert!(report.issues.iter().any(|i| i.code == "MANIFEST_MISSING"));
    }
}
