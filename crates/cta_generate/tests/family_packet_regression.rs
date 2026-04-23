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

fn load_packet(instance_id: &str) -> Value {
    let path = workspace_root()
        .join("benchmark")
        .join("v0.2")
        .join("annotation")
        .join("review_packets")
        .join("code_only_v1")
        .join(instance_id)
        .join("packet.json");
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

fn bf_text(packet: &Value) -> String {
    benchmark_facing(packet)
        .iter()
        .map(|o| {
            o["lean_statement"]
                .as_str()
                .unwrap_or("")
                .to_ascii_lowercase()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn family_specific_shape_guards_hold() {
    let ids = [
        "dp_longest_common_subsequence_002",
        "greedy_interval_scheduling_002",
        "graph_bfs_shortest_path_001",
        "graph_bfs_shortest_path_002",
        "trees_lowest_common_ancestor_001",
        "trees_lowest_common_ancestor_002",
        "arrays_binary_search_002",
        "greedy_coin_change_canonical_001",
        "greedy_coin_change_canonical_002",
    ];
    for id in ids {
        let packet = load_packet(id);
        let txt = bf_text(&packet);
        assert!(
            !txt.contains("{{"),
            "{id}: unresolved placeholder in benchmark-facing theorem"
        );
        assert!(
            !txt.contains("-> true")
                && !txt.contains("→ true")
                && !txt.contains("∨ true")
                && !txt.contains("| none => true")
                && !txt.contains("| some _ => true"),
            "{id}: benchmark-facing theorem has vacuous true-shell form"
        );
    }

    let lcs = bf_text(&load_packet("dp_longest_common_subsequence_002"));
    assert!(
        lcs.contains("strictlyincreasing") || lcs.contains("strictly increasing"),
        "dp_longest_common_subsequence_002: subsequence witness must use increasing indices"
    );
    assert!(
        !lcs.contains("substring"),
        "dp_longest_common_subsequence_002: contiguous-substring semantics leaked into subsequence definition"
    );

    let interval = bf_text(&load_packet("greedy_interval_scheduling_002"));
    assert!(
        interval.contains("iv ∈ s → iv ∈ intervals")
            || interval.contains("list.mem iv s -> list.mem iv intervals"),
        "greedy_interval_scheduling_002: feasible witness must use subset implication"
    );

    for bfs_id in ["graph_bfs_shortest_path_001", "graph_bfs_shortest_path_002"] {
        let bfs = bf_text(&load_packet(bfs_id));
        assert!(
            bfs.contains("let u := p.get! i")
                && bfs.contains("let w := p.get! (i + 1)")
                && (bfs.contains("w ∈ adj[u]")
                    || bfs.contains("list.mem w (adj[u].tolist)")
                    || bfs.contains("(adj.get? u).getd []")),
            "{bfs_id}: bfs witness/minimality edge clause must use consecutive adjacency"
        );
        assert!(
            !bfs.contains("p.get? i ∈ adj[p.get? i]"),
            "{bfs_id}: malformed bfs edge shape persists"
        );
    }

    for lca_id in [
        "trees_lowest_common_ancestor_001",
        "trees_lowest_common_ancestor_002",
    ] {
        let lca = bf_text(&load_packet(lca_id));
        assert!(
            lca.contains("proper_descendants_at")
                || lca.contains("proper descendant")
                || lca.contains("∀ d"),
            "{lca_id}: benchmark-facing theorem must directly express descendant exclusion"
        );
    }

    let bs = bf_text(&load_packet("arrays_binary_search_002"));
    assert!(
        !bs.contains("i < arr.size →"),
        "arrays_binary_search_002: success theorem must conclude bounds, not assume them"
    );
    assert!(
        bs.contains("= some i ? i < arr.size")
            || bs.contains("= some i -> i < arr.size")
            || bs.contains("= some i → i < arr.size")
            || bs.contains("= some i → i < arr.length")
            || bs.contains("= some i) → i < arr.length")
            || bs.contains("= some i) -> i < arr.length"),
        "arrays_binary_search_002: success theorem must derive in-bounds from return value"
    );

    for coin_id in [
        "greedy_coin_change_canonical_001",
        "greedy_coin_change_canonical_002",
    ] {
        let coin = bf_text(&load_packet(coin_id));
        assert!(
            coin.contains("canonicaldenoms") && coin.contains("optimality"),
            "{coin_id}: canonicality must appear explicitly in optimality theorem shape"
        );
    }
}
