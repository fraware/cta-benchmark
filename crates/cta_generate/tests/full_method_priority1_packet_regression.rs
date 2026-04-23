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

fn packet_path(system_id: &str, instance_id: &str) -> PathBuf {
    workspace_root()
        .join("benchmark")
        .join("v0.2")
        .join("annotation")
        .join("review_packets")
        .join(system_id)
        .join(instance_id)
        .join("packet.json")
}

fn load_packet(system_id: &str, instance_id: &str) -> Value {
    let path = packet_path(system_id, instance_id);
    let raw = fs::read_to_string(&path).unwrap_or_else(|_| panic!("reading {}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|_| panic!("parsing {}", path.display()))
}

fn benchmark_facing(packet: &Value) -> Vec<Value> {
    packet["generated_obligations"]
        .as_array()
        .expect("generated_obligations array")
        .iter()
        .filter(|o| o["layer"].as_str() == Some("benchmark_facing"))
        .cloned()
        .collect()
}

fn assert_no_known_bad_graph_patterns(instance_id: &str, packet: &Value) {
    for ob in benchmark_facing(packet) {
        let stmt = ob["lean_statement"]
            .as_str()
            .unwrap_or("")
            .to_ascii_lowercase();
        assert!(
            !stmt.contains("d' < d"),
            "{instance_id}: forbidden optimality shell pattern `d' < d`"
        );
        assert!(
            !stmt.contains("d' < d → false") && !stmt.contains("d' < d -> false"),
            "{instance_id}: forbidden optimality contradiction encoding"
        );

        let linked = ob["linked_semantic_units"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let is_unreachability = linked
            .iter()
            .filter_map(|v| v.as_str())
            .any(|s| s == "SU5" || s == "SU6");
        if is_unreachability {
            assert!(
                !(stmt.contains("get? v = some none ↔")
                    && stmt.contains("get? v = some (some k)")),
                "{instance_id}: forbidden unreachability encoding tying None to absence of Some(k) in the output table"
            );
        }
    }
}

fn assert_path_linked_obligations_tie_distance_to_paths(instance_id: &str, packet: &Value) {
    for ob in benchmark_facing(packet) {
        let linked = ob["linked_semantic_units"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let linked_ids: Vec<String> = linked
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();

        let needs_path_weight_linkage = linked_ids
            .iter()
            .any(|id| matches!(id.as_str(), "SU4" | "SU5" | "SU6"));
        if !needs_path_weight_linkage {
            continue;
        }

        let stmt = ob["lean_statement"]
            .as_str()
            .unwrap_or("")
            .to_ascii_lowercase();
        assert!(
            stmt.contains("pathweight"),
            "{instance_id}: obligation linked to {:?} must use `PathWeight` to tie distances to path weights ({stmt})",
            linked_ids
        );
        if linked_ids.iter().any(|id| id == "SU5") {
            assert!(
                stmt.contains("≤") || stmt.contains("<="),
                "{instance_id}: SU5 optimality must relate returned distance to path weights via `≤` ({stmt})"
            );
        }
    }
}

fn assert_bfs_path_obligations(instance_id: &str, packet: &Value) {
    assert_no_known_bad_graph_patterns(instance_id, packet);

    for su in ["SU3", "SU4", "SU5"] {
        let mut found = false;
        for ob in benchmark_facing(packet) {
            let linked = ob["linked_semantic_units"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            if !linked.iter().filter_map(|v| v.as_str()).any(|s| s == su) {
                continue;
            }
            found = true;
            let stmt = ob["lean_statement"]
                .as_str()
                .unwrap_or("")
                .to_ascii_lowercase();
            assert!(
                stmt.contains("path") && stmt.contains("path.length"),
                "{instance_id}: {su} theorem must mention a path witness and relate it to hop-count via `path.length`"
            );
            assert!(
                stmt.contains("∈ (adj.get?") || stmt.contains("in (adj.get?"),
                "{instance_id}: {su} theorem must relate consecutive vertices using adjacency membership"
            );
        }
        assert!(
            found,
            "{instance_id}: missing benchmark-facing theorem for {su}"
        );
    }
}

fn assert_coin_change_optimality_direction(instance_id: &str, packet: &Value) {
    let mut opt_stmt: Option<String> = None;
    for ob in benchmark_facing(packet) {
        if ob["kind"].as_str() != Some("optimality") {
            continue;
        }
        let linked = ob["linked_semantic_units"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        if !linked.iter().filter_map(|v| v.as_str()).any(|s| s == "SU4") {
            continue;
        }
        opt_stmt = ob["lean_statement"].as_str().map(str::to_string);
        break;
    }
    let stmt = opt_stmt
        .unwrap_or_else(|| panic!("{instance_id}: missing optimality theorem linked to SU4"));
    let lc = stmt.to_ascii_lowercase();
    assert!(
        lc.contains("canonical"),
        "{instance_id}: optimality must mention canonicality explicitly"
    );
    assert!(
        lc.contains("coinchangecanonical") && lc.contains(".sum") && lc.contains("≤"),
        "{instance_id}: optimality must compare greedy sum against alternatives using `≤`"
    );
    assert!(
        !lc.contains("counts.sum ≤ (coinchangecanonical")
            && !lc.contains("counts.sum <= (coinchangecanonical"),
        "{instance_id}: reversed coin-change optimality inequality regressed"
    );
}

#[test]
fn full_method_v1_priority1_semantic_hardening_packets() {
    let dijkstra = load_packet("full_method_v1", "graph_dijkstra_002");
    assert_no_known_bad_graph_patterns("graph_dijkstra_002", &dijkstra);
    assert_path_linked_obligations_tie_distance_to_paths("graph_dijkstra_002", &dijkstra);

    let bfs = load_packet("full_method_v1", "graph_bfs_shortest_path_002");
    assert_bfs_path_obligations("graph_bfs_shortest_path_002", &bfs);

    let coin = load_packet("full_method_v1", "greedy_coin_change_canonical_002");
    assert_coin_change_optimality_direction("greedy_coin_change_canonical_002", &coin);
}
