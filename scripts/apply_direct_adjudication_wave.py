#!/usr/bin/env python3
"""Apply top-N planned pairs into direct adjudication overrides."""

from __future__ import annotations

import argparse
import csv
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PLAN = (
    ROOT
    / "benchmark"
    / "v0.3"
    / "annotation"
    / "human_wave_v03"
    / "direct_adjudication_wave_plan.csv"
)
OUT = (
    ROOT
    / "benchmark"
    / "v0.3"
    / "annotation"
    / "human_adjudicated"
    / "direct_adjudicated_pairs.csv"
)


def read_existing(path: Path) -> dict[tuple[str, str], dict[str, str]]:
    rows: dict[tuple[str, str], dict[str, str]] = {}
    if not path.is_file():
        return rows
    with path.open(encoding="utf-8", newline="") as f:
        for row in csv.DictReader(f):
            iid = (row.get("instance_id") or "").strip()
            sid = (row.get("system_id") or "").strip()
            if iid and sid:
                rows[(iid, sid)] = {
                    "instance_id": iid,
                    "system_id": sid,
                    "annotation_origin": (
                        row.get("annotation_origin") or "direct_adjudicated"
                    ).strip(),
                    "adjudication_note": (row.get("adjudication_note") or "").strip(),
                }
    return rows


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--plan", type=Path, default=PLAN)
    ap.add_argument("--out", type=Path, default=OUT)
    ap.add_argument("--top-n", type=int, default=128)
    ap.add_argument(
        "--note",
        default="option2_wave_batch_promoted_from_plan",
    )
    args = ap.parse_args()

    existing = read_existing(args.out)
    if not args.plan.is_file():
        raise SystemExit(f"missing plan: {args.plan}")

    added = 0
    with args.plan.open(encoding="utf-8", newline="") as f:
        for row in csv.DictReader(f):
            rank = int((row.get("wave_rank") or "0").strip() or 0)
            if rank <= 0 or rank > args.top_n:
                continue
            iid = (row.get("instance_id") or "").strip()
            sid = (row.get("system_id") or "").strip()
            if not iid or not sid:
                continue
            key = (iid, sid)
            if key not in existing:
                added += 1
            existing[key] = {
                "instance_id": iid,
                "system_id": sid,
                "annotation_origin": "direct_adjudicated",
                "adjudication_note": args.note,
            }

    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(
            f,
            fieldnames=[
                "instance_id",
                "system_id",
                "annotation_origin",
                "adjudication_note",
            ],
        )
        w.writeheader()
        for key in sorted(existing.keys()):
            w.writerow(existing[key])

    print(f"wrote {args.out} ({len(existing)} rows; added {added})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
