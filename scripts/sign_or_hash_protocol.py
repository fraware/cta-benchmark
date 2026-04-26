#!/usr/bin/env python3
"""Write benchmark/<v>/protocol_freeze.json with content hashes for paper preregistration."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    h.update(path.read_bytes())
    return f"sha256:{h.hexdigest()}"


def git_head() -> str:
    try:
        return (
            subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
        )
    except (subprocess.CalledProcessError, FileNotFoundError):
        return "unknown"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--benchmark-version", default="v0.3")
    ap.add_argument(
        "--out",
        type=Path,
        default=None,
        help="Output path (default benchmark/<v>/protocol_freeze.json)",
    )
    args = ap.parse_args()
    v = args.benchmark_version
    out = args.out or (ROOT / "benchmark" / v / "protocol_freeze.json")
    bench = ROOT / "benchmark" / v
    eval_path = bench / "splits" / "eval.json"
    exp_path = ROOT / "configs" / "experiments" / "benchmark_v03.json"
    summary = bench / "benchmark_paper_summary.json"
    repair_rule = ROOT / "repairs" / "hotspot_selection.csv"

    inputs: dict[str, str] = {}
    for label, p in (
        ("eval_split_json", eval_path),
        ("experiment_config_benchmark_v03", exp_path),
        ("benchmark_paper_summary", summary),
        ("repair_hotspot_selection", repair_rule),
    ):
        if p.is_file():
            inputs[label] = sha256_file(p)

    payload = {
        "schema_version": "protocol_freeze_v1",
        "benchmark_version": v,
        "annotation_wave_id": "wave_pipeline_v03_001",
        "stratification_seed": 42,
        "repair_selection_rule_id": "hotspot_low_faithfulness_top12",
        "metrics_version": "metrics_v2",
        "rubric_version": "rubric_v1",
        "repo_commit_at_freeze": git_head(),
        "generated_at_utc": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "input_hashes": inputs,
    }
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
