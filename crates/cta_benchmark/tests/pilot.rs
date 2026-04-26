//! Integration test: load the real pilot benchmark, validate, lint, and
//! build a manifest. This is the acceptance gate for Milestones 0 and 1.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use cta_benchmark::{build_manifest, lint_benchmark, load_benchmark};
use cta_core::{BenchmarkVersion, MetricsVersion, RubricVersion};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn pilot_v0_1_loads_and_lints_clean() {
    let bench_root = workspace_root().join("benchmark").join("v0.1");
    let version = BenchmarkVersion::new("v0.1").unwrap();
    let bench = load_benchmark(&bench_root, &version).expect("load pilot");
    assert!(
        bench.len() >= 12,
        "expected at least 12 pilot instances (6 domains x 2), got {}",
        bench.len()
    );

    let report = lint_benchmark(&bench);
    assert!(
        !report.has_errors(),
        "lint errors on pilot: {:#?}",
        report.issues
    );
}

#[test]
fn pilot_v0_1_manifest_hash_is_deterministic() {
    let bench_root = workspace_root().join("benchmark").join("v0.1");
    let version = BenchmarkVersion::new("v0.1").unwrap();
    let bench = load_benchmark(&bench_root, &version).unwrap();
    let rubric = RubricVersion::new("rubric_v1").unwrap();
    let metrics = MetricsVersion::new("metrics_v2").unwrap();

    let ts = "2026-04-21T00:00:00Z";
    let a = build_manifest(&bench, &rubric, &metrics, ts).unwrap();
    let b = build_manifest(&bench, &rubric, &metrics, ts).unwrap();
    assert_eq!(a.content_hash, b.content_hash);
    assert!(a.content_hash.starts_with("sha256:"));
}
