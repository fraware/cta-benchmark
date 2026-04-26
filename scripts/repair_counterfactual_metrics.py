#!/usr/bin/env python3
"""
Compare mean faithfulness on the repair cohort vs a simple counterfactual.

Counterfactual (documented, conservative): for each (instance_id, system) in
the repair subset, substitute faithfulness with that system's mean
faithfulness over eval-split instances that are *not* in the repair subset.
This does not recover pre-repair per-instance scores; it bounds a population
shift under a transparent proxy.
"""

from __future__ import annotations

import argparse
import csv
import json
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def mean(xs: list[float]) -> float:
    return sum(xs) / len(xs) if xs else float("nan")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--instance-level",
        type=Path,
        default=ROOT / "results" / "instance_level.csv",
    )
    ap.add_argument(
        "--eval-split",
        type=Path,
        default=ROOT / "benchmark" / "v0.3" / "splits" / "eval.json",
    )
    ap.add_argument(
        "--out",
        type=Path,
        default=ROOT / "results" / "repair_impact_summary.json",
    )
    args = ap.parse_args()

    if not args.instance_level.is_file():
        print(f"missing {args.instance_level}", file=sys.stderr)
        return 1
    eval_ids: set[str] = set()
    if args.eval_split.is_file():
        eval_ids = set(json.loads(args.eval_split.read_text(encoding="utf-8")).get("instance_ids") or [])

    rows: list[dict[str, str]] = []
    with args.instance_level.open(encoding="utf-8", newline="") as f:
        for row in csv.DictReader(f):
            rows.append(row)

    by_system_non_repair: dict[str, list[float]] = defaultdict(list)
    by_system_repair: dict[str, list[float]] = defaultdict(list)
    for r in rows:
        split_ok = (not eval_ids) or ((r.get("split") or "").strip() == "eval")
        if not split_ok:
            continue
        try:
            fth = float(r.get("faithfulness_mean") or "")
        except ValueError:
            continue
        sid = (r.get("system") or "").strip()
        if not sid:
            continue
        if (r.get("repair_subset") or "").strip().lower() == "yes":
            by_system_repair[sid].append(fth)
        else:
            by_system_non_repair[sid].append(fth)

    per_system: dict[str, dict[str, float | int]] = {}
    for sid in sorted(set(by_system_repair) | set(by_system_non_repair)):
        obs = by_system_repair.get(sid, [])
        base = by_system_non_repair.get(sid, [])
        if not obs:
            continue
        base_mean = mean(base) if base else None
        per_system[sid] = {
            "repair_n": len(obs),
            "observed_mean_faithfulness": mean(obs),
            "counterfactual_mean_faithfulness": base_mean,
            "delta_obs_minus_counterfactual": (
                (mean(obs) - base_mean) if base_mean is not None else None
            ),
            "non_repair_eval_n": len(base),
        }

    payload = {
        "schema_version": "repair_impact_summary_v1",
        "counterfactual_definition": (
            "For each system with repair_subset=yes rows on the eval split, counterfactual_mean_faithfulness "
            "is the mean faithfulness over eval-split rows for that system with repair_subset!=yes. "
            "Each repaired instance is compared against that system-level mean (not a reconstructed pre-repair trace)."
        ),
        "per_system": per_system,
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(payload, indent=2, allow_nan=False) + "\n", encoding="utf-8")
    print(f"wrote {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
