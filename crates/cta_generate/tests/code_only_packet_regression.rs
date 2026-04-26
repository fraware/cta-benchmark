#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn packet_path(instance_id: &str) -> PathBuf {
    workspace_root()
        .join("benchmark")
        .join("v0.2")
        .join("annotation")
        .join("review_packets")
        .join("code_only_v1")
        .join(instance_id)
        .join("packet.json")
}

fn load_packet(instance_id: &str) -> Value {
    let path = packet_path(instance_id);
    let raw = fs::read_to_string(&path).unwrap_or_else(|_| panic!("reading {}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|_| panic!("parsing {}", path.display()))
}

fn benchmark_facing(packet: &Value) -> Vec<Value> {
    packet["generated_obligations"]
        .as_array()
        .expect("generated_obligations array")
        .iter()
        .filter(|o| {
            o["layer"]
                .as_str()
                .map(|s| s == "benchmark_facing")
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}

fn critical_units(packet: &Value) -> HashSet<String> {
    packet["semantic_units"]
        .as_array()
        .expect("semantic_units array")
        .iter()
        .filter(|u| u["criticality"].as_str() == Some("critical"))
        .filter_map(|u| u["id"].as_str().map(str::to_string))
        .collect()
}

fn covered_direct_units(packet: &Value) -> HashSet<String> {
    packet["quality_summary"]["critical_units_covered_by_direct_theorems"]
        .as_array()
        .expect("direct coverage array")
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect()
}

fn assert_no_true_theorems(instance_id: &str, packet: &Value) {
    for ob in benchmark_facing(packet) {
        let stmt = ob["lean_statement"]
            .as_str()
            .unwrap_or("")
            .to_ascii_lowercase();
        assert!(
            stmt.trim() != "true"
                && !stmt.contains(": true := by trivial")
                && !stmt.contains("-> true")
                && !stmt.contains("→ true")
                && !stmt.contains("∧ true")
                && !stmt.contains("| none => true")
                && !stmt.contains("| some _ => true"),
            "{instance_id}: benchmark-facing obligation is vacuous: {stmt}"
        );
    }
}

fn assert_schema_consistency(instance_id: &str, packet: &Value) {
    let qs = packet
        .get("quality_summary")
        .and_then(|v| v.as_object())
        .expect("quality_summary object");
    for key in [
        "critical_units_covered_by_direct_theorems",
        "critical_units_only_indirectly_covered",
        "off_spec_theorems_present",
        "vacuous_theorems_present",
    ] {
        assert!(
            qs.contains_key(key),
            "{instance_id}: quality_summary missing `{key}`"
        );
    }
    for (idx, ob) in packet["generated_obligations"]
        .as_array()
        .expect("generated_obligations array")
        .iter()
        .enumerate()
    {
        for key in [
            "index",
            "kind",
            "layer",
            "lean_statement",
            "nl_gloss",
            "linked_semantic_units",
        ] {
            assert!(
                ob.get(key).is_some(),
                "{instance_id}: obligation {idx} missing `{key}`"
            );
        }
        let layer = ob["layer"].as_str().unwrap_or("");
        assert!(
            layer == "benchmark_facing" || layer == "auxiliary",
            "{instance_id}: obligation {idx} has invalid layer `{layer}`"
        );
    }
}

fn assert_quality_summary_matches_content(instance_id: &str, packet: &Value) {
    let bf = benchmark_facing(packet);
    let mut vacuous = false;
    let mut off_spec = false;
    for ob in &bf {
        let stmt = ob["lean_statement"]
            .as_str()
            .unwrap_or("")
            .to_ascii_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let gloss = ob["nl_gloss"].as_str().unwrap_or("").to_ascii_lowercase();
        if stmt == "true"
            || stmt.contains(": true := by trivial")
            || stmt.contains(": prop := by trivial")
            || stmt.contains("-> true")
            || stmt.contains("→ true")
            || stmt.contains("∧ true")
            || stmt.contains("| none => true")
            || stmt.contains("| some _ => true")
            || stmt.contains("placeholder")
            || gloss.contains("placeholder")
            || (gloss.contains("represents") && gloss.contains("need to"))
        {
            vacuous = true;
        }
        if stmt.contains("stable") || stmt.contains("stability") || gloss.contains("stability") {
            off_spec = true;
        }
    }
    assert_eq!(
        packet["quality_summary"]["vacuous_theorems_present"].as_bool(),
        Some(vacuous),
        "{instance_id}: vacuous flag mismatch"
    );
    assert_eq!(
        packet["quality_summary"]["off_spec_theorems_present"].as_bool(),
        Some(off_spec),
        "{instance_id}: off-spec flag mismatch"
    );
}

fn assert_no_off_spec_theorems(instance_id: &str, packet: &Value) {
    for ob in benchmark_facing(packet) {
        let stmt = ob["lean_statement"]
            .as_str()
            .unwrap_or("")
            .to_ascii_lowercase();
        let gloss = ob["nl_gloss"].as_str().unwrap_or("").to_ascii_lowercase();
        assert!(
            !stmt.contains("stable") && !stmt.contains("stability") && !gloss.contains("stability"),
            "{instance_id}: off-spec theorem in benchmark-facing layer"
        );
    }
}

fn assert_direct_critical_coverage(instance_id: &str, packet: &Value) {
    let critical = critical_units(packet);
    let direct = covered_direct_units(packet);
    let missing: Vec<String> = critical.difference(&direct).cloned().collect();
    assert!(
        missing.is_empty(),
        "{instance_id}: critical units missing direct theorem coverage: {missing:?}"
    );
}

fn assert_benchmark_facing_cap(instance_id: &str, packet: &Value) {
    let n = benchmark_facing(packet).len();
    assert!(
        n <= 6,
        "{instance_id}: benchmark-facing theorem count {n} exceeds cap 6"
    );
}

fn assert_no_tautological_or_universal_preconditions(instance_id: &str, packet: &Value) {
    for ob in benchmark_facing(packet) {
        let kind = ob["kind"].as_str().unwrap_or("");
        if kind != "precondition" {
            continue;
        }
        let stmt = ob["lean_statement"]
            .as_str()
            .unwrap_or("")
            .to_ascii_lowercase();
        assert!(
            !stmt.contains(":= h"),
            "{instance_id}: precondition is tautological by direct hypothesis return"
        );
        if instance_id == "arrays_binary_search_002" {
            assert!(
                !stmt.contains("theorem su1_sorted_nondec")
                    || stmt.contains("→")
                    || stmt.contains("->")
                    || stmt.contains("(h")
                    || stmt.contains("(hsorted"),
                "{instance_id}: sortedness precondition must be an assumption/contract, not a universal fact"
            );
        }
    }
}

fn assert_instance_specific_fixes(instance_id: &str, packet: &Value) {
    let bf = benchmark_facing(packet);
    let bf_text = bf
        .iter()
        .map(|ob| {
            ob["lean_statement"]
                .as_str()
                .unwrap_or("")
                .to_ascii_lowercase()
        })
        .collect::<Vec<_>>()
        .join("\n");
    match instance_id {
        "greedy_interval_scheduling_001" => {
            assert!(
                !bf_text.contains("∀ iv, iv ∈ s ↔ iv ∈ intervals"),
                "{instance_id}: witness theorem incorrectly equates selection with all intervals"
            );
            assert!(
                bf_text.contains("∀ iv, iv ∈ s → iv ∈ intervals")
                    || bf_text.contains("t ⊆ intervals")
                    || bf_text.contains("s ⊆ intervals"),
                "{instance_id}: witness theorem must encode subset selection from input"
            );
        }
        "graph_bfs_shortest_path_001" => {
            assert!(
                !bf_text.contains("p.get? i ∈ adj[p.get? i]"),
                "{instance_id}: malformed path-edge adjacency remains in witness/minimality theorem"
            );
            assert!(
                bf_text.contains("let u := p.get! i")
                    && bf_text.contains("let w := p.get! (i + 1)")
                    && (bf_text.contains("w ∈ adj[u]")
                        || bf_text.contains("list.mem w (adj[u].tolist)")
                        || bf_text.contains("(adj.get? u).getd []")),
                "{instance_id}: bfs edge clause must use consecutive vertices with adjacency membership"
            );
        }
        "graph_dijkstra_001" | "graph_dijkstra_002" => {
            assert!(
                !bf_text.contains("w ≥ 0") && !bf_text.contains("w >= 0"),
                "{instance_id}: dijkstra includes vacuous nonnegativity clause despite Nat weights"
            );
            let source = bf
                .iter()
                .find(|ob| {
                    ob["linked_semantic_units"]
                        .as_array()
                        .map(|arr| arr.len() == 1 && arr[0].as_str().unwrap_or("") == "SU3")
                        .unwrap_or(false)
                })
                .expect("dijkstra source theorem present");
            let links = source["linked_semantic_units"]
                .as_array()
                .expect("linked_semantic_units array")
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>();
            assert_eq!(
                links,
                vec!["SU3"],
                "{instance_id}: source theorem must link only directly covered source semantic unit"
            );
        }
        "trees_bst_insert_001" => {
            let has_bf_precondition = bf
                .iter()
                .any(|ob| ob["kind"].as_str().unwrap_or("") == "precondition");
            assert!(
                !has_bf_precondition,
                "{instance_id}: tautological BST precondition must not remain benchmark-facing"
            );
            assert!(
                !bf_text.contains("→ keys (bst_insert t key) = keys t ∪ {key} ∨"),
                "{instance_id}: malformed key-change disjunction theorem remains benchmark-facing"
            );
            let su3_bf = bf
                .iter()
                .filter(|ob| {
                    ob["linked_semantic_units"]
                        .as_array()
                        .map(|arr| arr.iter().any(|v| v.as_str() == Some("SU3")))
                        .unwrap_or(false)
                })
                .count();
            assert!(
                su3_bf >= 2,
                "{instance_id}: benchmark-facing SU3 should be split into absent-key and present-key multiset theorems"
            );
        }
        "trees_bst_insert_002" => {
            let has_bf_precondition = bf
                .iter()
                .any(|ob| ob["kind"].as_str().unwrap_or("") == "precondition");
            assert!(
                !has_bf_precondition,
                "{instance_id}: tautological BST precondition must not remain benchmark-facing"
            );
            assert!(
                bf_text.contains("isbst") && bf_text.contains("bstinsert"),
                "{instance_id}: benchmark-facing theorems must use IsBst / bstInsert scaffold names"
            );
            assert!(
                bf_text.contains("list.perm (keys (bstinsert")
                    || bf_text.contains("list.perm(keys(bstinsert"),
                "{instance_id}: multiset change must use List.Perm on keys (bstInsert …)"
            );
            let su3_bf = bf
                .iter()
                .filter(|ob| {
                    ob["linked_semantic_units"]
                        .as_array()
                        .map(|arr| arr.iter().any(|v| v.as_str() == Some("SU3")))
                        .unwrap_or(false)
                })
                .count();
            assert!(
                su3_bf >= 1,
                "{instance_id}: absent-key multiset theorem should link SU3"
            );
        }
        _ => {}
    }
}

fn assert_quality_summary_final_content(instance_id: &str, packet: &Value) {
    assert_eq!(
        packet["quality_summary"]["vacuous_theorems_present"].as_bool(),
        Some(false),
        "{instance_id}: vacuous_theorems_present must be false"
    );
    assert_eq!(
        packet["quality_summary"]["off_spec_theorems_present"].as_bool(),
        Some(false),
        "{instance_id}: off_spec_theorems_present must be false"
    );
    assert_eq!(
        packet["quality_summary"]["critical_units_only_indirectly_covered"]
            .as_array()
            .map(|v| v.is_empty()),
        Some(true),
        "{instance_id}: critical_units_only_indirectly_covered must be empty"
    );
}

#[test]
fn regression_target_packets_are_benchmark_aligned() {
    let targets = [
        "arrays_binary_search_001",
        "arrays_binary_search_002",
        "arrays_max_subarray_001",
        "arrays_max_subarray_002",
        "graph_bfs_shortest_path_001",
        "graph_bfs_shortest_path_002",
        "graph_dijkstra_001",
        "greedy_interval_scheduling_001",
        "sorting_insertion_sort_002",
        "sorting_merge_sort_001",
        "trees_bst_insert_001",
        "trees_lowest_common_ancestor_001",
        "trees_lowest_common_ancestor_002",
        "graph_dijkstra_002",
        "sorting_insertion_sort_001",
        "sorting_merge_sort_002",
        "dp_knapsack_01_001",
        "dp_knapsack_01_002",
        "dp_longest_common_subsequence_001",
        "trees_bst_insert_002",
    ];

    for instance_id in targets {
        let packet = load_packet(instance_id);
        assert_schema_consistency(instance_id, &packet);
        assert_quality_summary_matches_content(instance_id, &packet);
        assert_quality_summary_final_content(instance_id, &packet);
        assert_no_true_theorems(instance_id, &packet);
        assert_no_off_spec_theorems(instance_id, &packet);
        assert_no_tautological_or_universal_preconditions(instance_id, &packet);
        assert_instance_specific_fixes(instance_id, &packet);
        assert_direct_critical_coverage(instance_id, &packet);
        assert_benchmark_facing_cap(instance_id, &packet);
    }
}
