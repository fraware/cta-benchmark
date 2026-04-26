#!/usr/bin/env python3
"""
Materialize benchmark/v0.3: copy v0.2 baseline, then add instances *_003..*_007
per algorithm family (60 new instances). Each variant carries distinct informal
wording and semantic-unit emphasis while reusing the family reference.rs and
harness oracles (definitionally the same algorithmic contract).

Grid design (not 60 unrelated algorithms): the expansion is a **family stress
grid**—same oracle surface, different specification-facing text and annotation
load. Variants *_001 and *_002 are further distinguished by `patch_grid_variants_001_002`
(frozen baseline vs paired control prose).

After cloning, run `python scripts/build_v03_annotation_pack.py` so
`configs/experiments/benchmark_v03.json` can require full annotation coverage
on the eval split.
"""

from __future__ import annotations

import json
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
V2 = ROOT / "benchmark" / "v0.2"
V3 = ROOT / "benchmark" / "v0.3"
LEAN_BENCH = ROOT / "lean" / "CTA" / "Benchmark"

FAMILIES: list[tuple[str, str]] = [
    ("arrays", "arrays_binary_search"),
    ("arrays", "arrays_max_subarray"),
    ("sorting", "sorting_insertion_sort"),
    ("sorting", "sorting_merge_sort"),
    ("graph", "graph_dijkstra"),
    ("graph", "graph_bfs_shortest_path"),
    ("greedy", "greedy_interval_scheduling"),
    ("greedy", "greedy_coin_change_canonical"),
    ("dp", "dp_longest_common_subsequence"),
    ("dp", "dp_knapsack_01"),
    ("trees", "trees_bst_insert"),
    ("trees", "trees_lowest_common_ancestor"),
]


def namespace_for(prefix: str, idx: int) -> str:
    num = f"{idx:03d}"
    return {
        "arrays_binary_search": f"CTA.Benchmark.Arrays.BinarySearch{num}",
        "arrays_max_subarray": f"CTA.Benchmark.Arrays.MaxSubarray{num}",
        "sorting_insertion_sort": f"CTA.Benchmark.Sorting.InsertionSort{num}",
        "sorting_merge_sort": f"CTA.Benchmark.Sorting.MergeSort{num}",
        "graph_dijkstra": f"CTA.Benchmark.Graph.Dijkstra{num}",
        "graph_bfs_shortest_path": f"CTA.Benchmark.Graph.BfsShortestPath{num}",
        "greedy_interval_scheduling": f"CTA.Benchmark.Greedy.IntervalScheduling{num}",
        "greedy_coin_change_canonical": f"CTA.Benchmark.Greedy.CoinChangeCanonical{num}",
        "dp_longest_common_subsequence": f"CTA.Benchmark.DP.LongestCommonSubsequence{num}",
        "dp_knapsack_01": f"CTA.Benchmark.DP.Knapsack01_{num}",
        "trees_bst_insert": f"CTA.Benchmark.Trees.BstInsert{num}",
        "trees_lowest_common_ancestor": f"CTA.Benchmark.Trees.LowestCommonAncestor{num}",
    }[prefix]


def lean_file_stem(prefix: str, idx: int) -> str:
    num = f"{idx:03d}"
    return {
        "arrays_binary_search": f"BinarySearch{num}",
        "arrays_max_subarray": f"MaxSubarray{num}",
        "sorting_insertion_sort": f"InsertionSort{num}",
        "sorting_merge_sort": f"MergeSort{num}",
        "graph_dijkstra": f"Dijkstra{num}",
        "graph_bfs_shortest_path": f"BfsShortestPath{num}",
        "greedy_interval_scheduling": f"IntervalScheduling{num}",
        "greedy_coin_change_canonical": f"CoinChangeCanonical{num}",
        "dp_longest_common_subsequence": f"LongestCommonSubsequence{num}",
        "dp_knapsack_01": f"Knapsack01_{num}",
        "trees_bst_insert": f"BstInsert{num}",
        "trees_lowest_common_ancestor": f"LowestCommonAncestor{num}",
    }[prefix]


def lean_subdir(domain: str) -> str:
    return {
        "arrays": "Arrays",
        "sorting": "Sorting",
        "graph": "Graph",
        "greedy": "Greedy",
        "dp": "DP",
        "trees": "Trees",
    }[domain]


# Variant-specific authoring lenses (distinct surface text; same gold contract).
LENSES: dict[str, dict[int, tuple[str, str]]] = {
    "graph_bfs_shortest_path": {
        3: (
            "BFS distance table (variant 3): emphasize queue FIFO discipline vs layer-wise relaxation.",
            "Authoring lens 3 stresses that obligations must tie hop-count witnesses to directed edges along the path, not merely reachability.",
        ),
        4: (
            "BFS distance table (variant 4): emphasize handling of parallel edges and duplicate enqueues without breaking shortest counts.",
            "Lens 4 targets specifications that accidentally allow counting non-simple walks or double-counting hop length.",
        ),
        5: (
            "BFS distance table (variant 5): emphasize disconnected components and source-out-of-range defensive behavior.",
            "Lens 5 checks that unreachability is stated as absence of any directed walk, not bounded search depth.",
        ),
        6: (
            "BFS distance table (variant 6): emphasize self-loops and zero-length cycles at the source.",
            "Lens 6 checks minimality clauses remain meaningful when the graph has redundant edges.",
        ),
        7: (
            "BFS distance table (variant 7): emphasize multi-edge shortest paths where the first discovered path may be non-shortest if mis-implemented.",
            "Lens 7 stresses first-layer discovery order must not replace shortest hop count semantics.",
        ),
    },
    "graph_dijkstra": {
        3: (
            "Single-source shortest paths (variant 3): emphasize non-negative weights and relaxation monotonicity.",
            "Lens 3 guards against Bellman-Ford-only statements that hide the Dijkstra greedy invariant.",
        ),
        4: (
            "Single-source shortest paths (variant 4): emphasize zero-weight edges and tie-breaking stability of distances.",
            "Lens 4 targets vacuous distance updates that forget to propagate through weight-0 stacks.",
        ),
        5: (
            "Single-source shortest paths (variant 5): emphasize unreachable vertices and `None` distance semantics.",
            "Lens 5 checks iff-style unreachable characterizations vs one-sided implications.",
        ),
        6: (
            "Single-source shortest paths (variant 6): emphasize dense graphs and duplicate edges in adjacency lists.",
            "Lens 6 stresses multiset-of-edges vs set-of-edges distinctions in witness paths.",
        ),
        7: (
            "Single-source shortest paths (variant 7): emphasize DAG specializations still satisfying general shortest-path semantics.",
            "Lens 7 ensures the spec does not silently assume acyclicity without stating it.",
        ),
    },
    "dp_longest_common_subsequence": {
        3: (
            "LCS length (variant 3): emphasize symmetry lcs(a,b)=lcs(b,a) as a cross-check on substructure recurrence.",
            "Lens 3 catches asymmetric indexing in the DP table that still looks plausible in prose.",
        ),
        4: (
            "LCS length (variant 4): emphasize empty-prefix base cases and boundary indexing.",
            "Lens 4 targets off-by-one in base row/column initialization.",
        ),
        5: (
            "LCS length (variant 5): emphasize duplicate characters and multiset alignment constraints.",
            "Lens 5 checks order-preserving subsequence semantics vs multiset intersection mistakes.",
        ),
        6: (
            "LCS length (variant 6): emphasize unequal lengths and adversarial alphabet repetition.",
            "Lens 6 stresses optimality: no longer common subsequence exists than the returned length.",
        ),
        7: (
            "LCS length (variant 7): emphasize quadratic-time witness structure without requiring explicit backpointer extraction.",
            "Lens 7 distinguishes length-only correctness from reconstructing an actual LCS string.",
        ),
    },
    "dp_knapsack_01": {
        3: (
            "0/1 knapsack optimum (variant 3): emphasize each item used at most once across the witness subset.",
            "Lens 3 catches hidden repetitions or unbounded-knapsack relaxations.",
        ),
        4: (
            "0/1 knapsack optimum (variant 4): emphasize capacity saturation vs strict feasibility.",
            "Lens 4 targets proofs that confuse `<= capacity` with equality.",
        ),
        5: (
            "0/1 knapsack optimum (variant 5): emphasize zero-value or zero-weight corner items.",
            "Lens 5 checks degenerate items do not break optimality statements.",
        ),
        6: (
            "0/1 knapsack optimum (variant 6): emphasize tie-breaking among multiple optimal subsets.",
            "Lens 6 stresses value optimality, not uniqueness of the witness.",
        ),
        7: (
            "0/1 knapsack optimum (variant 7): emphasize exponential brute-force agreement on tiny instances as a behavioral anchor.",
            "Lens 7 aligns informal witness semantics with exhaustive subset enumeration.",
        ),
    },
    "trees_bst_insert": {
        3: (
            "BST insertion (variant 3): emphasize strict ordering invariant in inorder traversal after each insert.",
            "Lens 3 catches duplicates policy: decide consistent `<` on left, `>` on right handling.",
        ),
        4: (
            "BST insertion (variant 4): emphasize structural preservation of existing keys when inserting duplicates.",
            "Lens 4 targets accidental multiset growth on duplicate keys.",
        ),
        5: (
            "BST insertion (variant 5): emphasize empty-tree base case and single-node rotations-free correctness.",
            "Lens 5 checks base-case split correctness in obligations.",
        ),
        6: (
            "BST insertion (variant 6): emphasize height growth and logarithmic-time claims are out-of-scope unless stated.",
            "Lens 6 prevents sneaking asymptotic claims without definitions.",
        ),
        7: (
            "BST insertion (variant 7): emphasize shape non-uniqueness while BST property remains unique as a predicate.",
            "Lens 7 distinguishes shape from sorted multiset property.",
        ),
    },
    "trees_lowest_common_ancestor": {
        3: (
            "LCA on BST keys (variant 3): emphasize both keys present and ancestor definition on directed tree edges.",
            "Lens 3 catches using set intersection without respecting tree ancestor direction.",
        ),
        4: (
            "LCA on BST keys (variant 4): emphasize incomparable branches and the lowest depth witness.",
            "Lens 4 targets using min/max key tricks that fail for non-BST trees silently assumed BST.",
        ),
        5: (
            "LCA on BST keys (variant 5): emphasize equal keys and boundary comparisons.",
            "Lens 5 checks consistent three-way compare replay in obligations.",
        ),
        6: (
            "LCA on BST keys (variant 6): emphasize uniqueness of the lowest common ancestor node.",
            "Lens 6 distinguishes any common ancestor from the lowest one.",
        ),
        7: (
            "LCA on BST keys (variant 7): emphasize walk-from-root simulation vs first-split characterization equivalence.",
            "Lens 7 ensures obligations match the algorithmic characterization used in proofs.",
        ),
    },
}

# Default lenses for families not listed above (arrays, sorting, greedy).
DEFAULT_LENS = {
    3: (
        "Variant 3 stresses boundary preconditions and explicit edge-case coverage in the informal contract.",
        "Lens 3 focuses reviewer attention on vacuous universal quantifiers in obligations.",
    ),
    4: (
        "Variant 4 stresses witness existence vs uniqueness and forbids smuggling extra outputs.",
        "Lens 4 targets disconnected postconditions that omit necessary quantifier structure.",
    ),
    5: (
        "Variant 5 stresses monotonicity or stability properties implied by the reference implementation.",
        "Lens 5 highlights obligations that must remain checkable against the behavioral oracle.",
    ),
    6: (
        "Variant 6 stresses adversarial small inputs (n<=3 style) while preserving asymptotic-agnostic correctness.",
        "Lens 6 guards against proof sketches that only discuss big instances.",
    ),
    7: (
        "Variant 7 stresses cross-family confusion: obligations must reference this instance's types and entry point only.",
        "Lens 7 catches template leakage from other algorithm families.",
    ),
}


def lens_for(prefix: str, idx: int) -> tuple[str, str]:
    d = LENSES.get(prefix, {})
    if idx in d:
        return d[idx]
    return DEFAULT_LENS[idx]


def patch_json_instance(data: object, old_id: str, new_id: str, ns: str) -> None:
    assert isinstance(data, dict)
    data["instance_id"] = new_id  # type: ignore[index]
    data["benchmark_version"] = "v0.3"  # type: ignore[index]
    if "rust_reference" in data and isinstance(data["rust_reference"], dict):
        p = data["rust_reference"]["path"]
        assert isinstance(p, str)
        data["rust_reference"]["path"] = p.replace(old_id, new_id)
    if "lean_target" in data and isinstance(data["lean_target"], dict):
        lt = data["lean_target"]
        for k in ("scaffold_path", "reference_obligations_path", "semantic_units_path"):
            if k in lt and isinstance(lt[k], str):
                lt[k] = lt[k].replace(old_id, new_id)
        lt["namespace"] = ns


def patch_semantic_units(data: object, old_id: str, new_id: str, prefix: str, idx: int) -> None:
    assert isinstance(data, dict)
    data["instance_id"] = new_id  # type: ignore[index]
    data["benchmark_version"] = "v0.3"  # type: ignore[index]
    title, note = lens_for(prefix, idx)
    units = data.get("units")
    if not isinstance(units, list):
        return
    for u in units:
        if isinstance(u, dict) and "description" in u:
            desc = u["description"]
            if isinstance(desc, str):
                u["description"] = f"{desc} ({title})"


def patch_obligations(data: object, old_id: str, new_id: str) -> None:
    assert isinstance(data, dict)
    data["instance_id"] = new_id  # type: ignore[index]
    data["benchmark_version"] = "v0.3"  # type: ignore[index]


def patch_informal_statement(inst: dict, prefix: str, idx: int) -> None:
    inf = inst.get("informal_statement")
    if not isinstance(inf, dict) or "text" not in inf:
        return
    text = inf["text"]
    if not isinstance(text, str):
        return
    title, lens = lens_for(prefix, idx)
    tag = f"{prefix}_{idx:03d}"
    inf["text"] = f"{text}\n\nBenchmark note ({tag}): {title}\nAuthoring lens: {lens}"


def write_scaffold_lean(
    domain: str, prefix: str, idx: int, new_id: str, dst_scaffold: Path
) -> None:
    ns = namespace_for(prefix, idx)
    stem = lean_file_stem(prefix, idx)
    tmpl = V3 / "instances" / domain / f"{prefix}_001" / "scaffold.lean"
    if not tmpl.is_file():
        tmpl = V3 / "instances" / domain / f"{prefix}_002" / "scaffold.lean"
    text = tmpl.read_text(encoding="utf-8")
    old_ns = namespace_for(prefix, 1)
    text = text.replace(f"`{prefix}_001`", f"`{new_id}`")
    text = text.replace(f"`{prefix}_002`", f"`{new_id}`")
    text = text.replace(old_ns, ns)
    dst_scaffold.write_text(text, encoding="utf-8")
    lean_path = LEAN_BENCH / lean_subdir(domain) / f"{stem}.lean"
    lean_path.write_text(text, encoding="utf-8")


def copy_tree_replace(
    domain: str,
    src_dir: Path,
    dst_dir: Path,
    old_id: str,
    new_id: str,
    prefix: str,
    idx: int,
) -> None:
    if dst_dir.exists():
        shutil.rmtree(dst_dir)
    shutil.copytree(src_dir, dst_dir)
    ns = namespace_for(prefix, idx)
    inst_path = dst_dir / "instance.json"
    raw = inst_path.read_text(encoding="utf-8")
    inst = json.loads(raw)
    assert isinstance(inst, dict)
    patch_json_instance(inst, old_id, new_id, ns)
    patch_informal_statement(inst, prefix, idx)
    inst_path.write_text(json.dumps(inst, indent=2) + "\n", encoding="utf-8")

    su = json.loads((dst_dir / "semantic_units.json").read_text(encoding="utf-8"))
    patch_semantic_units(su, old_id, new_id, prefix, idx)
    (dst_dir / "semantic_units.json").write_text(json.dumps(su, indent=2) + "\n", encoding="utf-8")

    ro = json.loads((dst_dir / "reference_obligations.json").read_text(encoding="utf-8"))
    patch_obligations(ro, old_id, new_id)
    (dst_dir / "reference_obligations.json").write_text(
        json.dumps(ro, indent=2) + "\n", encoding="utf-8"
    )

    ref_rs = dst_dir / "reference.rs"
    ref_rs.write_text(
        ref_rs.read_text(encoding="utf-8").replace(old_id, new_id), encoding="utf-8"
    )

    notes = dst_dir / "notes.md"
    t, l = lens_for(prefix, idx)
    notes.write_text(
        f"# {new_id}\n\n{t}\n\n{l}\n\n"
        f"Derived algorithm family `{prefix}`; behavioral contract matches v0.2 reference oracles.\n",
        encoding="utf-8",
    )

    write_scaffold_lean(domain, prefix, idx, new_id, dst_dir / "scaffold.lean")

    hpath = dst_dir / "harness.json"
    if hpath.is_file():
        hj = json.loads(hpath.read_text(encoding="utf-8"))
        if isinstance(hj, dict):
            hj["schema_version"] = hj.get("schema_version", "schema_v1")
        hpath.write_text(json.dumps(hj, indent=2) + "\n", encoding="utf-8")


GRID_001 = (
    "\n\n[Grid variant 001 — baseline authoritative informal contract for this "
    "algorithm family. Same reference implementation and harness as the rest "
    "of the family grid; used for parity and verbatim drift checks across releases.]"
)

GRID_002 = (
    "\n\n[Grid variant 002 — paired within-family control: distinct authoring "
    "emphasis on summary/scaffold hygiene, auxiliary lemmas, and scope discipline "
    "so obligations cannot smuggle hidden parameters. Algorithmic contract and "
    "oracle checks remain identical to variant 001.]"
)


def patch_grid_variants_001_002() -> None:
    """Differentiate _001/_002 informal text and semantic-unit glosses (no oracle change)."""
    for domain, prefix in FAMILIES:
        for idx, grid_suffix in ((1, GRID_001), (2, GRID_002)):
            inst_dir = V3 / "instances" / domain / f"{prefix}_{idx:03d}"
            if not inst_dir.is_dir():
                continue
            tag = "V001 baseline" if idx == 1 else "V002 paired control"
            ij = inst_dir / "instance.json"
            inst = json.loads(ij.read_text(encoding="utf-8"))
            if not isinstance(inst, dict):
                continue
            inf = inst.get("informal_statement")
            if isinstance(inf, dict) and "text" in inf and isinstance(inf["text"], str):
                if "[Grid variant 00" not in inf["text"]:
                    inf["text"] = inf["text"] + grid_suffix
            ij.write_text(json.dumps(inst, indent=2) + "\n", encoding="utf-8")

            su_path = inst_dir / "semantic_units.json"
            if su_path.is_file():
                su = json.loads(su_path.read_text(encoding="utf-8"))
                units = su.get("units")
                if isinstance(units, list):
                    for u in units:
                        if isinstance(u, dict) and isinstance(u.get("description"), str):
                            d = u["description"]
                            if tag not in d:
                                u["description"] = f"{d} ({tag}.)"
                su_path.write_text(json.dumps(su, indent=2) + "\n", encoding="utf-8")

            notes = inst_dir / "notes.md"
            if notes.is_file():
                prev = notes.read_text(encoding="utf-8")
                header = f"# Grid variant {idx:03d} ({tag})\n\n"
                if not prev.lstrip().startswith("# Grid variant"):
                    notes.write_text(header + prev, encoding="utf-8")


def bump_v03_json_files() -> None:
    for path in V3.rglob("*.json"):
        if "manifests" in path.parts:
            continue
        txt = path.read_text(encoding="utf-8")
        if "v0.2" not in txt:
            continue
        path.write_text(txt.replace("v0.2", "v0.3"), encoding="utf-8")


def write_splits() -> None:
    dev_ids: list[str] = []
    eval_ids: list[str] = []
    for domain, prefix in FAMILIES:
        for idx in (1, 2, 3):
            dev_ids.append(f"{prefix}_{idx:03d}")
        for idx in (4, 5, 6, 7):
            eval_ids.append(f"{prefix}_{idx:03d}")
    dev_ids.sort()
    eval_ids.sort()
    splits = V3 / "splits"
    splits.mkdir(parents=True, exist_ok=True)
    (splits / "dev.json").write_text(
        json.dumps(
            {
                "schema_version": "schema_v1",
                "benchmark_version": "v0.3",
                "split": "dev",
                "instance_ids": dev_ids,
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    (splits / "eval.json").write_text(
        json.dumps(
            {
                "schema_version": "schema_v1",
                "benchmark_version": "v0.3",
                "split": "eval",
                "instance_ids": eval_ids,
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )


def append_benchmark_lean_imports() -> None:
    bench_lean = ROOT / "lean" / "CTA" / "Benchmark.lean"
    text = bench_lean.read_text(encoding="utf-8")
    imports: list[str] = []
    for domain, prefix in FAMILIES:
        for idx in range(1, 8):
            stem = lean_file_stem(prefix, idx)
            sub = lean_subdir(domain)
            imp = f"import CTA.Benchmark.{sub}.{stem}"
            if imp not in text:
                imports.append(imp)
    if not imports:
        return
    marker = "import CTA.Benchmark.Trees.LowestCommonAncestor001"
    if marker not in text:
        raise SystemExit("unexpected Benchmark.lean layout")
    block = "\n".join(imports)
    text = text.replace(marker, marker + "\n" + block)
    bench_lean.write_text(text, encoding="utf-8")


def main() -> int:
    if len(sys.argv) > 1 and sys.argv[1] == "--patch-grid-001-002-only":
        if not V3.is_dir():
            print("missing benchmark/v0.3", file=sys.stderr)
            return 1
        patch_grid_variants_001_002()
        print("patched grid variants 001/002 under", V3)
        return 0

    if not V2.is_dir():
        print("missing benchmark/v0.2", file=sys.stderr)
        return 1
    if V3.exists():
        shutil.rmtree(V3)
    shutil.copytree(V2, V3)
    bump_v03_json_files()
    patch_grid_variants_001_002()

    for domain, prefix in FAMILIES:
        src_base = V3 / "instances" / domain / f"{prefix}_002"
        if not src_base.is_dir():
            src_base = V3 / "instances" / domain / f"{prefix}_001"
        for idx in range(3, 8):
            new_id = f"{prefix}_{idx:03d}"
            dst = V3 / "instances" / domain / new_id
            copy_tree_replace(domain, src_base, dst, src_base.name, new_id, prefix, idx)

    write_splits()
    append_benchmark_lean_imports()
    print("materialized", V3)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
