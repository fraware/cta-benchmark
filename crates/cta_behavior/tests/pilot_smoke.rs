//! Smoke test: run every pilot adapter against its committed `harness.json`
//! with a small trial count, asserting no falsifications are observed.
//!
//! Clippy: this file deliberately uses `expect`/`panic` for fixture traversal;
//! suppress noisy lints that do not apply to integration smoke tests.
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use std::path::PathBuf;

use cta_behavior::{AdapterRegistry, HarnessConfig};

fn workspace_root() -> PathBuf {
    // From `target/debug/deps` or from the crate dir, climb until we see
    // `benchmark/`.
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !p.join("benchmark").is_dir() {
        if !p.pop() {
            panic!(
                "could not locate workspace root from {:?}",
                env!("CARGO_MANIFEST_DIR")
            );
        }
    }
    p
}

#[test]
fn all_pilot_adapters_are_clean_on_small_trials() {
    let root = workspace_root();
    let instances_root = root.join("benchmark").join("v0.1").join("instances");
    let registry = AdapterRegistry::with_pilot();

    let mut checked = 0usize;
    for domain_dir in std::fs::read_dir(&instances_root).expect("read instances") {
        let domain_dir = domain_dir.expect("dir entry").path();
        if !domain_dir.is_dir() {
            continue;
        }
        for inst_dir in std::fs::read_dir(&domain_dir).expect("read domain") {
            let inst_dir = inst_dir.expect("dir entry").path();
            if !inst_dir.is_dir() {
                continue;
            }
            let instance_id = inst_dir
                .file_name()
                .expect("file name")
                .to_string_lossy()
                .to_string();
            let harness_path = inst_dir.join("harness.json");
            if !harness_path.is_file() {
                continue;
            }
            let raw = std::fs::read_to_string(&harness_path).expect("read harness");
            let mut config: HarnessConfig = serde_json::from_str(&raw).expect("parse harness");
            // Keep the test fast: cap trials.
            config.num_trials = config.num_trials.min(20);

            let adapter = registry
                .get(&instance_id)
                .unwrap_or_else(|| panic!("no adapter for {instance_id}"));
            let report = adapter.run(&config).expect("run harness");
            assert!(
                !report.any_falsified(),
                "falsifications in {instance_id}: {:?}",
                report.falsifications
            );
            checked += 1;
        }
    }
    assert_eq!(
        checked, 12,
        "expected 12 pilot instances, checked {checked}"
    );
}
