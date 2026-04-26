//! Golden-shape tests across the 12 pilot instances.
//!
//! These tests guarantee that `cta_rust_extract` produces structurally
//! stable summaries for every pilot reference implementation: every
//! summary parses, every summary has a non-empty parameter list, the
//! return-kind classifier produces a value from the stable enum, and the
//! set of semantic tags is within the published closed set.
//!
//! The intent is to catch extractor regressions that silently strip
//! information on a specific instance family without requiring a full
//! JSON-level golden file per instance (those are deferred to M2's
//! golden-fixtures sprint).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use cta_rust_extract::extract_from_file;

/// One row per pilot instance: (repo-relative path to reference.rs, entry fn).
const PILOTS: &[(&str, &str)] = &[
    (
        "benchmark/v0.1/instances/arrays/arrays_binary_search_001/reference.rs",
        "binary_search",
    ),
    (
        "benchmark/v0.1/instances/arrays/arrays_max_subarray_001/reference.rs",
        "max_subarray",
    ),
    (
        "benchmark/v0.1/instances/sorting/sorting_insertion_sort_001/reference.rs",
        "insertion_sort",
    ),
    (
        "benchmark/v0.1/instances/sorting/sorting_merge_sort_001/reference.rs",
        "merge_sort",
    ),
    (
        "benchmark/v0.1/instances/graph/graph_dijkstra_001/reference.rs",
        "dijkstra",
    ),
    (
        "benchmark/v0.1/instances/graph/graph_bfs_shortest_path_001/reference.rs",
        "bfs_shortest_path",
    ),
    (
        "benchmark/v0.1/instances/greedy/greedy_interval_scheduling_001/reference.rs",
        "interval_scheduling",
    ),
    (
        "benchmark/v0.1/instances/greedy/greedy_coin_change_canonical_001/reference.rs",
        "coin_change_canonical",
    ),
    (
        "benchmark/v0.1/instances/dp/dp_longest_common_subsequence_001/reference.rs",
        "lcs_length",
    ),
    (
        "benchmark/v0.1/instances/dp/dp_knapsack_01_001/reference.rs",
        "knapsack_01",
    ),
    (
        "benchmark/v0.1/instances/trees/trees_bst_insert_001/reference.rs",
        "bst_insert",
    ),
    (
        "benchmark/v0.1/instances/trees/trees_lowest_common_ancestor_001/reference.rs",
        "lca_bst",
    ),
];

/// Stable closed set of semantic tags the extractor is allowed to emit.
fn allowed_semantic_tags() -> BTreeSet<&'static str> {
    [
        "iterative",
        "mixed_control",
        "recursive",
        "straight_line",
        "uses_early_return",
        "uses_mutable_state",
        "uses_option_return",
        "uses_result_return",
    ]
    .into_iter()
    .collect()
}

/// Stable closed set of return-kind classifier outputs.
fn allowed_return_kinds() -> BTreeSet<&'static str> {
    [
        "bool", "numeric", "option", "other", "result", "unit", "vec",
    ]
    .into_iter()
    .collect()
}

fn workspace_root() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .ancestors()
        .nth(2)
        .expect("workspace root is two levels above crate dir")
        .to_path_buf()
}

#[test]
fn all_pilots_extract_cleanly() {
    let root = workspace_root();
    let tags = allowed_semantic_tags();
    let kinds = allowed_return_kinds();

    for (rel, entry) in PILOTS {
        let path = root.join(rel);
        let summary = extract_from_file(&path, entry).unwrap_or_else(|e| {
            panic!("{}: {e}", rel);
        });
        assert_eq!(&summary.fn_name, entry, "{rel}: fn_name mismatch");
        assert!(!summary.params.is_empty(), "{rel}: extracted no parameters");
        assert!(
            !summary.return_type.is_empty(),
            "{rel}: return_type is empty"
        );
        assert!(
            kinds.contains(summary.return_kind.as_str()),
            "{rel}: return_kind `{}` outside the allowed closed set",
            summary.return_kind
        );
        for t in &summary.semantic_tags {
            assert!(
                tags.contains(t.as_str()),
                "{rel}: unexpected semantic tag `{t}`"
            );
        }
        // Sanity: the four control-flow control tags are mutually exclusive.
        let cf_tags: Vec<&str> = summary
            .semantic_tags
            .iter()
            .map(String::as_str)
            .filter(|t| {
                matches!(
                    *t,
                    "iterative" | "recursive" | "mixed_control" | "straight_line"
                )
            })
            .collect();
        assert_eq!(
            cf_tags.len(),
            1,
            "{rel}: expected exactly one control-flow tag, got {cf_tags:?}"
        );
    }
}

#[test]
fn summary_serialization_roundtrips() {
    let root = workspace_root();
    for (rel, entry) in PILOTS {
        let path = root.join(rel);
        let summary = extract_from_file(&path, entry).expect("extraction ok");
        let json = serde_json::to_string(&summary).expect("serializes");
        let decoded: cta_rust_extract::RustSummary =
            serde_json::from_str(&json).expect("deserializes");
        assert_eq!(decoded.fn_name, summary.fn_name);
        assert_eq!(decoded.return_kind, summary.return_kind);
        assert_eq!(decoded.mutable_locals, summary.mutable_locals);
    }
}
