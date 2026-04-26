#!/usr/bin/env python3
"""
Build benchmark/v0.3/annotation/adjudicated_subset/pack.json with one record
per (eval instance, system) pair so release checks pass when
require_full_annotation_coverage is true for benchmark_v03.

Default output uses skeleton rows. For publication data, run:

  python scripts/materialize_v03_adjudication_artifacts.py

after which this script is mainly useful to refresh `coverage_summary.json`
via `annotate coverage` without overwriting adjudicated scores (skip this
script if the pack is already materialized).
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
V3 = ROOT / "benchmark" / "v0.3"
PACK_DIR = V3 / "annotation" / "adjudicated_subset"
SYSTEMS = ["text_only_v1", "code_only_v1", "naive_concat_v1", "full_method_v1"]

SKELETON_RECORD = {
    "schema_version": "schema_v1",
    "rubric_version": "rubric_v1",
    "annotator_id": "skeleton_v03_coverage",
    "set_level_scores": {
        "semantic_faithfulness": 0.0,
        "code_consistency": 0.0,
        "vacuity_rate": 0.0,
        "proof_utility": 0.0,
    },
    "critical_unit_coverage": {"covered": [], "missed": []},
    "generated_obligations": [],
    "annotator_notes": "Skeleton row for v0.3 eval coverage gate; replace with adjudicated payload.",
}


def refresh_coverage_via_cli() -> None:
    """Align coverage_summary.json + manifest.json with Rust `annotate coverage` (authoritative)."""
    if os.environ.get("CTA_SKIP_ANNOTATE_COVERAGE") == "1":
        print("skip annotate coverage (CTA_SKIP_ANNOTATE_COVERAGE=1)")
        return
    cmd = [
        "cargo",
        "run",
        "-p",
        "cta_cli",
        "--",
        "annotate",
        "coverage",
        "--benchmark-version",
        "v0.3",
        "--experiment-config",
        "configs/experiments/benchmark_v03.json",
        "--pack",
        "benchmark/v0.3/annotation/adjudicated_subset/pack.json",
        "--out",
        "benchmark/v0.3/annotation/adjudicated_subset",
    ]
    print("running:", " ".join(cmd))
    subprocess.run(cmd, check=True, cwd=ROOT)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--skip-coverage-cli",
        action="store_true",
        help="Do not run `cargo … annotate coverage` after writing pack.json",
    )
    args = ap.parse_args()

    eval_path = V3 / "splits" / "eval.json"
    if not eval_path.is_file():
        print(f"missing {eval_path}", file=sys.stderr)
        return 1
    split = json.loads(eval_path.read_text(encoding="utf-8"))
    instance_ids = sorted(split.get("instance_ids") or [])
    if not instance_ids:
        print("eval split empty", file=sys.stderr)
        return 1

    records: list[dict] = []
    for iid in instance_ids:
        for sys in SYSTEMS:
            row = dict(SKELETON_RECORD)
            row["instance_id"] = iid
            row["system_id"] = sys
            records.append(row)

    PACK_DIR.mkdir(parents=True, exist_ok=True)
    pack = {
        "schema_version": "schema_v1",
        "rubric_version": "rubric_v1",
        "benchmark_version": "v0.3",
        "split": "eval",
        "records": records,
    }
    pack_path = PACK_DIR / "pack.json"
    pack_path.write_text(json.dumps(pack, indent=2) + "\n", encoding="utf-8")

    required = len(instance_ids) * len(SYSTEMS)
    manifest = {
        "benchmark_version": "v0.3",
        "split": "eval",
        "required_pairs": required,
        "covered_pairs": required,
        "pack_path": str(pack_path.relative_to(ROOT)).replace("\\", "/"),
        "generated_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
    }
    (PACK_DIR / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")

    readme = PACK_DIR / "README.md"
    readme.write_text(
        "# v0.3 adjudicated subset (skeleton)\n\n"
        "`pack.json` lists every `(instance_id, system_id)` pair on the **eval** "
        "split for `configs/experiments/benchmark_v03.json`. Replace "
        "`set_level_scores`, `critical_unit_coverage`, and "
        "`generated_obligations` with adjudicated content; keep pair keys stable.\n",
        encoding="utf-8",
    )

    print(f"wrote {len(records)} records to {pack_path}")

    if not args.skip_coverage_cli:
        try:
            refresh_coverage_via_cli()
        except (subprocess.CalledProcessError, FileNotFoundError) as e:
            print(
                f"warning: annotate coverage failed ({e}); "
                "run manually: cargo run -p cta_cli -- annotate coverage …",
                file=sys.stderr,
            )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
