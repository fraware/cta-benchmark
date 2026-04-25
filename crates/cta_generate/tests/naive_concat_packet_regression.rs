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
        .join("naive_concat_v1")
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

fn assert_no_filler_theorems(instance_id: &str, packet: &Value) {
    for ob in benchmark_facing(packet) {
        let stmt = ob["lean_statement"]
            .as_str()
            .unwrap_or("")
            .to_ascii_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let gloss = ob["nl_gloss"].as_str().unwrap_or("").to_ascii_lowercase();
        assert!(
            stmt.trim() != "true"
                && !stmt.contains(": true")
                && !stmt.contains(": prop := by trivial")
                && !stmt.contains("-> true")
                && !stmt.contains("→ true")
                && !stmt.contains("∧ true")
                && !stmt.contains("placeholder")
                && !gloss.contains("placeholder")
                && !(gloss.contains("represents") && gloss.contains("need to")),
            "{instance_id}: benchmark-facing obligation is filler: {stmt}"
        );
    }
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

#[test]
fn regression_target_packets_are_benchmark_aligned() {
    let targets = [
        "arrays_binary_search_002",
        "graph_dijkstra_001",
        "graph_dijkstra_002",
        "graph_bfs_shortest_path_001",
        "greedy_interval_scheduling_001",
        "sorting_merge_sort_001",
        "trees_bst_insert_001",
        "dp_knapsack_01_001",
        "dp_knapsack_01_002",
    ];

    for instance_id in targets {
        let packet = load_packet(instance_id);
        assert_schema_consistency(instance_id, &packet);
        assert_no_filler_theorems(instance_id, &packet);
        assert_no_off_spec_theorems(instance_id, &packet);
        assert_direct_critical_coverage(instance_id, &packet);
        assert_benchmark_facing_cap(instance_id, &packet);
    }
}
