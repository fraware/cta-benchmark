#!/usr/bin/env python3
"""Generate and validate the fixed 13x4 strict-gap adjudication worklist."""

from __future__ import annotations

import argparse
import csv
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SYSTEMS = [
    "text_only_v1",
    "code_only_v1",
    "naive_concat_v1",
    "full_method_v1",
]
STRICT_GAP_INSTANCES = [
    "arrays_binary_search_003",
    "arrays_max_subarray_003",
    "dp_knapsack_01_003",
    "dp_knapsack_01_007",
    "dp_longest_common_subsequence_003",
    "graph_bfs_shortest_path_003",
    "graph_dijkstra_003",
    "greedy_coin_change_canonical_003",
    "greedy_interval_scheduling_003",
    "sorting_insertion_sort_003",
    "sorting_merge_sort_003",
    "trees_bst_insert_003",
    "trees_lowest_common_ancestor_003",
]


def load_overrides(path: Path) -> set[tuple[str, str]]:
    done: set[tuple[str, str]] = set()
    if not path.is_file():
        return done
    with path.open(encoding="utf-8", newline="") as f:
        for row in csv.DictReader(f):
            iid = (row.get("instance_id") or "").strip()
            sid = (row.get("system_id") or "").strip()
            origin = (row.get("annotation_origin") or "").strip()
            if not iid or not sid:
                continue
            if origin in {"direct_adjudicated", "direct_human"}:
                done.add((iid, sid))
    return done


def family_of(instance_id: str) -> str:
    return "_".join(instance_id.split("_")[:-1])


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--direct-pairs",
        type=Path,
        default=(
            ROOT
            / "benchmark"
            / "v0.3"
            / "annotation"
            / "human_adjudicated"
            / "direct_adjudicated_pairs.csv"
        ),
    )
    ap.add_argument(
        "--out-worklist",
        type=Path,
        default=(
            ROOT
            / "benchmark"
            / "v0.3"
            / "annotation"
            / "human_wave_v03"
            / "strict_gap_13x4_worklist.csv"
        ),
    )
    ap.add_argument(
        "--out-completion",
        type=Path,
        default=(
            ROOT
            / "benchmark"
            / "v0.3"
            / "annotation"
            / "human_wave_v03"
            / "strict_gap_13x4_completion.csv"
        ),
    )
    ap.add_argument(
        "--check-complete",
        action="store_true",
        help=(
            "Exit non-zero unless all 52 target pairs are completed "
            "in direct-pairs."
        ),
    )
    args = ap.parse_args()

    done = load_overrides(args.direct_pairs)
    rows = []
    for iid in STRICT_GAP_INSTANCES:
        for sid in SYSTEMS:
            complete = (iid, sid) in done
            rows.append(
                {
                    "instance_id": iid,
                    "family": family_of(iid),
                    "system_id": sid,
                    "target_annotation_origin": "direct_adjudicated",
                    "completed": "true" if complete else "false",
                }
            )

    args.out_worklist.parent.mkdir(parents=True, exist_ok=True)
    with args.out_worklist.open("w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(
            f,
            fieldnames=[
                "instance_id",
                "family",
                "system_id",
                "target_annotation_origin",
                "completed",
            ],
        )
        w.writeheader()
        w.writerows(rows)

    completed_n = sum(1 for r in rows if r["completed"] == "true")
    total_n = len(rows)
    with args.out_completion.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["metric", "value"])
        w.writerow(["target_pairs", total_n])
        w.writerow(["completed_pairs", completed_n])
        w.writerow(["missing_pairs", total_n - completed_n])

    print(f"wrote {args.out_worklist} ({total_n} rows)")
    print(f"wrote {args.out_completion}")
    if args.check_complete and completed_n != total_n:
        print(
            "strict_gap_13x4 incomplete: "
            f"{completed_n}/{total_n} pairs completed",
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
