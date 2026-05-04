#!/usr/bin/env python3
"""
Emit common-cell artifacts for strict direct headline rows (NeurIPS E&D).

Writes:
  - data/common_cell_instances.csv — instance_id rows in the 4-system common cell (48 rows).
  - data/common_cell_system_summary.csv — per-system aggregates over those instances only.
"""

from __future__ import annotations

import argparse
import csv
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PRIMARY_SYSTEMS = frozenset(
    {"text_only_v1", "code_only_v1", "naive_concat_v1", "full_method_v1"}
)


def compute_common_instance_ids(strict_csv: Path) -> list[str]:
    by_instance: dict[str, set[str]] = defaultdict(set)
    with strict_csv.open(encoding="utf-8", newline="") as f:
        reader = csv.DictReader(f)
        for row in reader:
            if (row.get("annotation_origin") or "").strip() != "direct_adjudicated":
                continue
            sid = (row.get("system") or "").strip()
            if sid not in PRIMARY_SYSTEMS:
                continue
            iid = (row.get("instance_id") or "").strip()
            if iid:
                by_instance[iid].add(sid)

    return sorted(
        iid
        for iid, systems in by_instance.items()
        if systems == PRIMARY_SYSTEMS
    )


def write_instances_csv(path: Path, instance_ids: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=["instance_id"])
        w.writeheader()
        for iid in instance_ids:
            w.writerow({"instance_id": iid})


def write_system_summary_csv(path: Path, strict_csv: Path, instance_ids: list[str]) -> None:
    import pandas as pd

    cell = set(instance_ids)
    df = pd.read_csv(strict_csv)
    df = df.loc[
        (df["annotation_origin"] == "direct_adjudicated")
        & (df["system"].isin(PRIMARY_SYSTEMS))
        & (df["instance_id"].isin(cell))
    ].copy()
    df["missing_critical_units"] = pd.to_numeric(
        df["missing_critical_units"], errors="coerce"
    ).fillna(0)

    g = df.groupby("system", sort=True).agg(
        rows=("instance_id", "count"),
        instances=("instance_id", "nunique"),
        faithfulness_mean=("faithfulness_mean", "mean"),
        code_consistency_mean=("code_consistency_mean", "mean"),
        vacuity_rate=("vacuity_rate", "mean"),
        proof_utility_mean=("proof_utility_mean", "mean"),
        missing_critical_units=("missing_critical_units", "sum"),
    )
    g = g.reset_index()
    for col in (
        "faithfulness_mean",
        "code_consistency_mean",
        "vacuity_rate",
        "proof_utility_mean",
    ):
        g[col] = g[col].round(6)

    cols = [
        "system",
        "rows",
        "instances",
        "faithfulness_mean",
        "code_consistency_mean",
        "vacuity_rate",
        "proof_utility_mean",
        "missing_critical_units",
    ]
    path.parent.mkdir(parents=True, exist_ok=True)
    g[cols].to_csv(path, index=False)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--strict-csv",
        type=Path,
        default=ROOT / "results" / "paper_strict_instance_level.csv",
    )
    ap.add_argument(
        "--out-instances",
        type=Path,
        default=ROOT / "hf_release" / "data" / "common_cell_instances.csv",
    )
    ap.add_argument(
        "--out-system-summary",
        type=Path,
        default=ROOT / "hf_release" / "data" / "common_cell_system_summary.csv",
    )
    args = ap.parse_args()

    if not args.strict_csv.is_file():
        raise SystemExit(f"missing strict CSV: {args.strict_csv}")

    ids = compute_common_instance_ids(args.strict_csv)
    write_instances_csv(args.out_instances, ids)
    write_system_summary_csv(args.out_system_summary, args.strict_csv, ids)

    print(f"wrote {args.out_instances} ({len(ids)} instance_id rows)")
    print(f"wrote {args.out_system_summary} ({len(PRIMARY_SYSTEMS)} systems)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
