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
                && !stmt.contains("∧ true"),
            "{instance_id}: benchmark-facing obligation is vacuous: {stmt}"
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
        "graph_bfs_shortest_path_001",
        "greedy_interval_scheduling_001",
        "sorting_merge_sort_001",
        "trees_bst_insert_001",
    ];

    for instance_id in targets {
        let packet = load_packet(instance_id);
        assert_no_true_theorems(instance_id, &packet);
        assert_no_off_spec_theorems(instance_id, &packet);
        assert_direct_critical_coverage(instance_id, &packet);
        assert_benchmark_facing_cap(instance_id, &packet);
    }
}
