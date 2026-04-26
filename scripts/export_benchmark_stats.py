#!/usr/bin/env python3
"""Emit CSV summaries for Table 1 (benchmark inventory) from manifest.jsonl."""

from __future__ import annotations

import argparse
import csv
import json
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--manifest", type=Path, default=ROOT / "benchmark" / "manifest.jsonl")
    ap.add_argument(
        "--out-dir",
        type=Path,
        default=ROOT / "results",
    )
    args = ap.parse_args()

    rows: list[dict] = []
    with args.manifest.open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            rows.append(json.loads(line))

    args.out_dir.mkdir(parents=True, exist_ok=True)
    t1 = args.out_dir / "table1_benchmark_overview.csv"
    with t1.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["metric", "value"])
        w.writerow(["total_instances", len(rows)])
        by_split = Counter(r["split"] for r in rows)
        for k in sorted(by_split):
            w.writerow([f"split_{k}", by_split[k]])
        by_fam = Counter(r["family"] for r in rows)
        for fam in sorted(by_fam):
            w.writerow([f"family_count:{fam}", by_fam[fam]])
        diff = Counter(r["difficulty"] for r in rows)
        for d in sorted(diff):
            w.writerow([f"difficulty_{d}", diff[d]])

    fam_units = defaultdict(int)
    fam_crit = defaultdict(int)
    for r in rows:
        fam_units[r["family"]] += len(r.get("semantic_units") or [])
        fam_crit[r["family"]] += int(r.get("critical_unit_count") or 0)

    t1b = args.out_dir / "table1_family_semantic_load.csv"
    with t1b.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["family", "instances", "semantic_units_total", "critical_units_sum"])
        fams = sorted(set(r["family"] for r in rows))
        for fam in fams:
            inst_c = sum(1 for r in rows if r["family"] == fam)
            w.writerow([fam, inst_c, fam_units[fam], fam_crit[fam]])

    print(f"wrote {t1} and {t1b}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
