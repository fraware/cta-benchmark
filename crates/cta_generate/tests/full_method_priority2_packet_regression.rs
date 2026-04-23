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
        .join("full_method_v1")
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
        .filter(|o| o["layer"].as_str() == Some("benchmark_facing"))
        .cloned()
        .collect()
}

fn critical_unit_ids(packet: &Value) -> HashSet<String> {
    packet["semantic_units"]
        .as_array()
        .expect("semantic_units array")
        .iter()
        .filter(|u| u["criticality"].as_str() == Some("critical"))
        .filter_map(|u| u["id"].as_str().map(str::to_string))
        .collect()
}

fn assert_quality_summary(instance_id: &str, packet: &Value) {
    let qs = packet
        .get("quality_summary")
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| panic!("{instance_id}: missing quality_summary"));
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
    let covered: HashSet<String> = qs["critical_units_covered_by_direct_theorems"]
        .as_array()
        .expect("covered array")
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    let critical = critical_unit_ids(packet);
    for id in &critical {
        assert!(
            covered.contains(id),
            "{instance_id}: critical unit {id} not listed as directly covered in quality_summary"
        );
    }
}

fn assert_no_vacuous_true_shells(instance_id: &str, packet: &Value) {
    for ob in benchmark_facing(packet) {
        let stmt = ob["lean_statement"]
            .as_str()
            .unwrap_or("")
            .to_ascii_lowercase();
        let stmt_norm = stmt.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            !stmt.contains("-> true") && !stmt.contains("→ true"),
            "{instance_id}: benchmark-facing theorem uses implication-to-True shell"
        );
        assert!(
            !stmt.contains("∧ true"),
            "{instance_id}: benchmark-facing theorem uses conjunction-with-True filler"
        );
        assert!(
            !stmt_norm.contains(": true := by trivial")
                && !stmt_norm.contains(": true := by simp")
                && stmt_norm.trim() != "true",
            "{instance_id}: benchmark-facing theorem body collapses to True"
        );
        assert!(
            !stmt.contains("| none => true") && !stmt.contains("| some _ => true"),
            "{instance_id}: benchmark-facing match arm collapses to True"
        );
        assert!(
            !stmt.contains("∃ n : nat, true") && !stmt.contains("exists n : nat, true"),
            "{instance_id}: existential-True termination placeholder"
        );
    }
}

fn assert_family_shapes(instance_id: &str, packet: &Value) {
    let joined = benchmark_facing(packet)
        .iter()
        .map(|o| {
            o["lean_statement"]
                .as_str()
                .unwrap_or("")
                .to_ascii_lowercase()
        })
        .collect::<Vec<_>>()
        .join("\n");

    match instance_id {
        "dp_knapsack_01_001" | "dp_knapsack_01_002" => {
            assert!(
                joined.contains("∃ sel") || joined.contains("exists sel"),
                "{instance_id}: knapsack witness must existentially quantify a selection"
            );
            assert!(
                joined.contains("knapsack01") && joined.contains("totalvalue"),
                "{instance_id}: knapsack witness must relate totalValue to knapsack01"
            );
            assert!(
                joined.contains("totalvalue values sel ≤ knapsack01"),
                "{instance_id}: knapsack optimality must upper-bound competitor values by knapsack01"
            );
        }
        "trees_lowest_common_ancestor_001" | "trees_lowest_common_ancestor_002" => {
            assert!(
                joined.contains("issubtree") && joined.contains("haskey"),
                "{instance_id}: LCA witness must mention subtree containment and key presence"
            );
            assert!(
                joined.contains("ispropersubtree") && joined.contains("false"),
                "{instance_id}: LCA lowestness must quantify proper subtrees and conclude False for impossible cases"
            );
        }
        "arrays_binary_search_001" | "arrays_binary_search_002" => {
            assert!(
                !joined.contains(": sorted a := h"),
                "{instance_id}: must not encode sortedness as a tautological `Sorted a := h` shell"
            );
            assert!(
                joined.contains("binary_search_absent_when_sorted")
                    && joined.contains("sorted a")
                    && joined.contains("binarysearch")
                    && joined.contains("none"),
                "{instance_id}: absence theorem must assume Sorted and rule out hits on None"
            );
            assert!(
                joined.contains("binary_search_success")
                    && joined.contains("some i")
                    && joined.contains("get?"),
                "{instance_id}: success theorem must tie Some(i) to arr.get? / bounds"
            );
        }
        _ => {}
    }
}

#[test]
fn full_method_v1_priority2_packets_reject_vacuity_and_tautologies() {
    let ids = [
        "dp_knapsack_01_001",
        "dp_knapsack_01_002",
        "trees_lowest_common_ancestor_001",
        "trees_lowest_common_ancestor_002",
        "arrays_binary_search_001",
        "arrays_binary_search_002",
    ];
    for id in ids {
        let packet = load_packet(id);
        for (idx, ob) in packet["generated_obligations"]
            .as_array()
            .expect("generated_obligations")
            .iter()
            .enumerate()
        {
            assert!(
                ob.get("layer").and_then(|v| v.as_str()).is_some(),
                "{id}: obligation {idx} missing layer"
            );
        }
        assert_quality_summary(id, &packet);
        assert_no_vacuous_true_shells(id, &packet);
        assert_family_shapes(id, &packet);
    }
}
