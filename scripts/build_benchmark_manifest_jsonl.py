#!/usr/bin/env python3
"""
Emit benchmark/manifest.jsonl from a versioned benchmark tree (default v0.3).
Each line is one JSON object with reviewer-facing audit fields.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def family_from_id(instance_id: str) -> str:
    m = re.match(r"^(.*)_(\d{3})$", instance_id)
    if not m:
        return instance_id
    return m.group(1)


def load_split_ids(path: Path) -> set[str]:
    if not path.is_file():
        return set()
    data = json.loads(path.read_text(encoding="utf-8"))
    return set(data.get("instance_ids") or [])


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--benchmark-version",
        default="v0.3",
        help="Directory under benchmark/ (default: v0.3)",
    )
    ap.add_argument(
        "--out",
        type=Path,
        default=ROOT / "benchmark" / "manifest.jsonl",
        help="Output JSONL path",
    )
    args = ap.parse_args()

    bench = ROOT / "benchmark" / args.benchmark_version
    inst_root = bench / "instances"
    if not inst_root.is_dir():
        print(f"missing {inst_root}", file=sys.stderr)
        return 1

    dev_ids = load_split_ids(bench / "splits" / "dev.json")
    eval_ids = load_split_ids(bench / "splits" / "eval.json")

    rows: list[dict] = []
    for domain_dir in sorted(inst_root.iterdir()):
        if not domain_dir.is_dir():
            continue
        for inst_dir in sorted(domain_dir.iterdir()):
            if not inst_dir.is_dir():
                continue
            ij = inst_dir / "instance.json"
            su_path = inst_dir / "semantic_units.json"
            ro_path = inst_dir / "reference_obligations.json"
            if not ij.is_file():
                continue
            inst = json.loads(ij.read_text(encoding="utf-8"))
            su = json.loads(su_path.read_text(encoding="utf-8")) if su_path.is_file() else {}
            ro = json.loads(ro_path.read_text(encoding="utf-8")) if ro_path.is_file() else {}
            iid = inst["instance_id"]
            units = su.get("units") or []
            crit = sum(1 for u in units if u.get("criticality") == "critical")
            obls = ro.get("obligations") or []
            informal = inst.get("informal_statement") or {}
            stmt_text = informal.get("text") or ""

            if iid in dev_ids and iid in eval_ids:
                split = "dev_and_eval"
            elif iid in dev_ids:
                split = "dev"
            elif iid in eval_ids:
                split = "eval"
            else:
                split = "unassigned"

            rel = inst_dir.relative_to(bench)
            code_paths = [
                str(rel / "instance.json"),
                str(rel / "reference.rs"),
                str(rel / "scaffold.lean"),
                str(rel / "reference_obligations.json"),
                str(rel / "semantic_units.json"),
                str(rel / "harness.json"),
                str(rel / "notes.md"),
            ]

            rows.append(
                {
                    "instance_id": iid,
                    "family": family_from_id(iid),
                    "difficulty": inst.get("difficulty", ""),
                    "source_provenance": (
                        f"CTA benchmark {args.benchmark_version}; internally authored gold; "
                        f"algorithm family `{family_from_id(iid)}`; behavioral oracle in harness.json."
                    ),
                    "informal_statement": stmt_text,
                    "semantic_units": units,
                    "critical_unit_count": crit,
                    "reference_obligations": obls,
                    "code_context_paths": code_paths,
                    "split": split,
                    "license_status": "MIT (repository root LICENSE applies to benchmark artifacts)",
                }
            )

    rows.sort(key=lambda r: r["instance_id"])
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("w", encoding="utf-8") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    print(f"wrote {len(rows)} rows to {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
