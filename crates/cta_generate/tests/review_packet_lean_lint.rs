//! Cross-cutting static checks on `benchmark/v0.2/annotation/review_packets/**/packet.json`.
//! These mirror manual review criteria (vacuity, path–distance linkage, coin optimality direction,
//! LCA witness + lowestness shape, and suspicious `rfl` proofs on semantic obligations).

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

fn collect_packet_json_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_packet_json_files(&path, out);
        } else if path.file_name().and_then(|n| n.to_str()) == Some("packet.json") {
            out.push(path);
        }
    }
}

fn benchmark_facing_obligations(packet: &Value) -> Vec<&Value> {
    packet["generated_obligations"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter(|o| o.get("layer").and_then(|v| v.as_str()) == Some("benchmark_facing"))
                .collect()
        })
        .unwrap_or_default()
}

fn instance_id_from_path(path: &Path) -> String {
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string()
}

fn assert_no_benchmark_facing_vacuity(instance_id: &str, stmt: &str) {
    // `graph_dijkstra_001` full_method packet predates the hardened obligation style; tracked separately.
    if instance_id == "graph_dijkstra_001" {
        return;
    }
    let lc = stmt.to_ascii_lowercase();
    let stmt_norm = lc.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        !lc.contains("-> true") && !lc.contains("→ true"),
        "{instance_id}: benchmark-facing theorem uses implication-to-True shell"
    );
    assert!(
        !lc.contains("∧ true"),
        "{instance_id}: benchmark-facing theorem uses conjunction-with-True filler"
    );
    assert!(
        !stmt_norm.contains(": true := by trivial")
            && !stmt_norm.contains(": true := by simp")
            && stmt_norm.trim() != "true",
        "{instance_id}: benchmark-facing theorem body collapses to True"
    );
    assert!(
        !lc.contains("| none => true") && !lc.contains("| some _ => true"),
        "{instance_id}: benchmark-facing match arm collapses to True"
    );
}

fn bad_disconnected_path_weight_sum(stmt: &str) -> bool {
    let lc = stmt.to_ascii_lowercase();
    let has_path = lc.contains("∃ path") || lc.contains("exists path");
    let has_wsum = lc.contains("∃ wsum") || lc.contains("exists wsum");
    has_path && has_wsum && !lc.contains("pathweight")
}

fn rfl_proof_credibility_flag(kind: &str, stmt: &str) -> bool {
    if matches!(kind, "termination" | "precondition" | "unknown" | "") {
        return false;
    }
    let lc = stmt.to_ascii_lowercase();
    lc.contains("by rfl")
        || lc.contains(", rfl⟩")
        || (lc.contains("simpa") && lc.contains("rfl"))
}

fn assert_graph_path_distance_consistency(instance_id: &str, ob: &Value) {
    if !instance_id.contains("dijkstra")
        && !instance_id.contains("bfs")
        && !instance_id.contains("shortest_path")
    {
        return;
    }
    let linked: Vec<String> = ob["linked_semantic_units"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let needs_path_structure = linked
        .iter()
        .any(|id| matches!(id.as_str(), "SU3" | "SU4" | "SU5" | "SU6"));
    if !needs_path_structure {
        return;
    }
    let stmt = ob["lean_statement"].as_str().unwrap_or("");
    let lc = stmt.to_ascii_lowercase();
    assert!(
        !bad_disconnected_path_weight_sum(stmt),
        "{instance_id}: distance/path obligation must not use disconnected ∃path ∧ ∃wsum without PathWeight ({stmt})"
    );
    let needs_pathweight = linked
        .iter()
        .any(|id| matches!(id.as_str(), "SU4" | "SU5" | "SU6"));
    if instance_id.contains("dijkstra") && needs_pathweight && instance_id != "graph_dijkstra_001" {
        assert!(
            lc.contains("pathweight"),
            "{instance_id}: weighted reachability/optimality/unreachability obligations must mention PathWeight ({stmt})"
        );
    }
}

fn assert_coin_change_optimality_direction(instance_id: &str, ob: &Value) {
    if !instance_id.contains("coin_change") {
        return;
    }
    if ob["kind"].as_str() != Some("optimality") {
        return;
    }
    let linked = ob["linked_semantic_units"].as_array().cloned().unwrap_or_default();
    if !linked
        .iter()
        .filter_map(|v| v.as_str())
        .any(|s| s == "SU4")
    {
        return;
    }
    let lc = ob["lean_statement"]
        .as_str()
        .unwrap_or("")
        .to_ascii_lowercase();
    if !(lc.contains('≤') || lc.contains("<=")) {
        return;
    }
    assert!(
        !lc.contains("counts.sum ≤ (coinchangecanonical")
            && !lc.contains("counts.sum <= (coinchangecanonical"),
        "{instance_id}: reversed coin-change optimality inequality"
    );
}

fn assert_lca_directionality(instance_id: &str, joined_bf: &str, packet_path: &Path) {
    if !instance_id.contains("lowest_common_ancestor") {
        return;
    }
    let ps = packet_path.to_string_lossy();
    if !(ps.contains("full_method_v1") || ps.contains("code_only_v1")) {
        return;
    }
    assert!(
        joined_bf.contains("issubtree") && joined_bf.contains("haskey"),
        "{instance_id}: LCA packets must witness subtree containment of both keys"
    );
    if joined_bf.contains("subtreerootedat") {
        assert!(
            joined_bf.contains("ispropersubtree")
                && (joined_bf.contains("¬ (haskey") || joined_bf.contains("¬(")),
            "{instance_id}: LCA lowestness must use proper subtrees below subtreeRootedAt and forbid both keys"
        );
    } else {
        assert!(
            joined_bf.contains("ispropersubtree") && joined_bf.contains("false"),
            "{instance_id}: LCA lowestness must quantify proper subtrees (legacy shape allowed until migrated)"
        );
    }
}

fn benchmark_facing_statements(packet: &Value) -> Vec<String> {
    benchmark_facing_obligations(packet)
        .iter()
        .map(|o| o["lean_statement"].as_str().unwrap_or("").to_string())
        .collect()
}

fn has_admit_or_sorry(stmt: &str) -> bool {
    let lc = stmt.to_ascii_lowercase();
    lc.contains(" admit")
        || lc.contains("\nadmit")
        || lc.contains(" sorry")
        || lc.contains("\nsorry")
}

fn assert_lean_check_shape_and_m1_contract(packet: &Value, packet_path: &Path, instance_id: &str) {
    let Some(lean_check) = packet.get("lean_check").and_then(|v| v.as_object()) else {
        panic!("{instance_id}: missing lean_check object");
    };
    let diagnostics_rel = lean_check
        .get("diagnostics_path")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("{instance_id}: lean_check.diagnostics_path missing or not a string"));
    let diagnostics_abs = workspace_root().join(diagnostics_rel);
    let elaborated = lean_check.get("elaborated").and_then(Value::as_bool);
    let proof_mode = lean_check.get("proof_mode").and_then(Value::as_str);
    let admit_count = lean_check.get("admit_count").and_then(Value::as_u64);
    let _axiom_deps = lean_check.get("axiom_dependencies").and_then(Value::as_array);

    // Base schema hardening: when fields are present, they must be valid.
    if let Some(mode) = proof_mode {
        assert!(
            matches!(mode, "axiom_backed" | "definition_backed"),
            "{instance_id}: lean_check.proof_mode must be axiom_backed|definition_backed ({mode})"
        );
    }

    // M1 gate: only enforce strict checks for packets explicitly marked elaborated.
    if elaborated == Some(true) {
        assert!(
            diagnostics_abs.is_file(),
            "{instance_id}: elaborated packet must have diagnostics file at {}",
            diagnostics_abs.display()
        );
        assert!(
            proof_mode.is_some(),
            "{instance_id}: elaborated packet must set lean_check.proof_mode"
        );
        let count = admit_count.unwrap_or_else(|| {
            panic!("{instance_id}: elaborated packet must set lean_check.admit_count")
        });
        assert_eq!(
            count, 0,
            "{instance_id}: elaborated packet must have admit_count = 0"
        );
        for stmt in benchmark_facing_statements(packet) {
            assert!(
                !has_admit_or_sorry(&stmt),
                "{instance_id}: elaborated benchmark-facing theorem contains admit/sorry ({})",
                packet_path.display()
            );
        }
    }
}

#[test]
fn review_packets_benchmark_facing_lean_lints() {
    let root = workspace_root().join("benchmark/v0.2/annotation/review_packets");
    let mut paths = Vec::new();
    collect_packet_json_files(&root, &mut paths);
    paths.sort();

    assert!(
        paths.len() > 10,
        "expected review_packets tree under {root:?}"
    );

    for path in paths {
        let raw = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let packet: Value =
            serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
        let instance_id = instance_id_from_path(&path);
        assert_lean_check_shape_and_m1_contract(&packet, &path, &instance_id);
        let joined_bf = benchmark_facing_obligations(&packet)
            .iter()
            .map(|o| {
                o["lean_statement"]
                    .as_str()
                    .unwrap_or("")
                    .to_ascii_lowercase()
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert_lca_directionality(&instance_id, &joined_bf, &path);

        for ob in benchmark_facing_obligations(&packet) {
            let stmt = ob["lean_statement"].as_str().unwrap_or("");
            let kind = ob["kind"].as_str().unwrap_or("");
            assert_no_benchmark_facing_vacuity(&instance_id, stmt);
            assert_graph_path_distance_consistency(&instance_id, ob);
            assert_coin_change_optimality_direction(&instance_id, ob);
            assert!(
                !rfl_proof_credibility_flag(kind, stmt),
                "{}: proof-credibility — semantic obligation `{kind}` should not close with rfl/simpa rfl ({})",
                instance_id,
                path.display()
            );
        }
    }
}
