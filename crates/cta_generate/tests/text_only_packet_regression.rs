//! Curated `text_only_v1` review packets that already use the same
//! `benchmark_facing` / `auxiliary` layering and `quality_summary` shape as
//! `code_only_v1` / `naive_concat_v1` gold rosters.
//!
//! Most legacy `text_only_v1` packets still omit `quality_summary` and use
//! `kind: "unknown"` without `layer`; extend the `targets` list only when a
//! packet is migrated to that schema. In addition to both knapsack pilots,
//! `graph_dijkstra_{001,002}` are migrated to the same `layer` +
//! `quality_summary` + `DijkstraTheory`-backed obligations as other Dijkstra
//! review packets.

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
        .join("text_only_v1")
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
        "dp_knapsack_01_001",
        "dp_knapsack_01_002",
        "graph_dijkstra_001",
        "graph_dijkstra_002",
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

/// `text_only_v1/graph_dijkstra_001` reuses the same archived model JSON as
/// `code_only_v1/graph_dijkstra_001` (`raw_output.txt` is byte-identical). The
/// normalized bundle must stay aligned with that body (not Fin/`dijkstraLike`
/// text copied from `graph_dijkstra_002`).
#[test]
fn text_only_graph_dijkstra_001_generated_output_tracks_code_only_lineage() {
    fn review_packet_dir(system_id: &str, instance_id: &str) -> PathBuf {
        workspace_root()
            .join("benchmark")
            .join("v0.2")
            .join("annotation")
            .join("review_packets")
            .join(system_id)
            .join(instance_id)
    }

    let text_dir = review_packet_dir("text_only_v1", "graph_dijkstra_001");
    let code_dir = review_packet_dir("code_only_v1", "graph_dijkstra_001");
    let text_raw = fs::read_to_string(text_dir.join("raw_output.txt"))
        .unwrap_or_else(|e| panic!("read text_only raw: {e}"));
    let code_raw = fs::read_to_string(code_dir.join("raw_output.txt"))
        .unwrap_or_else(|e| panic!("read code_only raw: {e}"));
    assert_eq!(
        text_raw, code_raw,
        "graph_dijkstra_001: text_only and code_only raw_output.txt must match"
    );

    let text_gen: Value = serde_json::from_str(
        &fs::read_to_string(text_dir.join("generated_output.json"))
            .expect("read text_only generated_output.json"),
    )
    .expect("parse text_only generated_output.json");
    let code_gen: Value = serde_json::from_str(
        &fs::read_to_string(code_dir.join("generated_output.json"))
            .expect("read code_only generated_output.json"),
    )
    .expect("parse code_only generated_output.json");

    assert_eq!(
        text_gen["normalized_obligations"], code_gen["normalized_obligations"],
        "graph_dijkstra_001: normalized_obligations must match code_only (shared raw)"
    );
    assert_eq!(
        text_gen["prompt_hash"], code_gen["prompt_hash"],
        "graph_dijkstra_001: prompt_hash must match code_only normalization lineage"
    );
    assert_eq!(
        text_gen["seed"], code_gen["seed"],
        "graph_dijkstra_001: seed must match code_only bundle"
    );

    let joined = text_gen["normalized_obligations"]
        .as_array()
        .expect("normalized_obligations array")
        .iter()
        .filter_map(|o| o["lean_statement"].as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !joined.contains("dijkstraLike"),
        "graph_dijkstra_001: text_only bundle must not regress to Fin/dijkstraLike obligations"
    );
}
