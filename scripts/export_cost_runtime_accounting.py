#!/usr/bin/env python3
"""Export paper-facing cost/runtime accounting from available manifests."""

from __future__ import annotations

import argparse
import csv
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def scan_run_manifests() -> list[Path]:
    runs = ROOT / "runs"
    if not runs.is_dir():
        return []
    return sorted(p for p in runs.rglob("run_manifest.json") if p.is_file())


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--out",
        type=Path,
        default=ROOT / "results" / "paper_cost_runtime_accounting.csv",
    )
    args = ap.parse_args()

    manifests = scan_run_manifests()
    rows: list[dict[str, str]] = []
    for p in manifests:
        try:
            doc = json.loads(p.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            continue
        provider = doc.get("provider") or {}
        runner = doc.get("runner") or {}
        cost = doc.get("cost_reporting") or {}
        rows.append(
            {
                "run_id": str(doc.get("run_id", "")),
                "system_id": str(doc.get("system_id", "")),
                "provider_name": str(provider.get("name", "")),
                "provider_model": str(provider.get("model", "")),
                "total_input_tokens": str(doc.get("total_input_tokens", "")),
                "total_output_tokens": str(doc.get("total_output_tokens", "")),
                "wall_clock_seconds": str(doc.get("wall_clock_seconds", "")),
                "estimated_or_billed_cost_usd": str(cost.get("estimated_cost_usd", "")),
                "cost_status": str(cost.get("status", "")),
                "cost_reason_code": str(cost.get("reason_code", "")),
                "runner_hostname": str(runner.get("hostname", "")),
                "runner_os": str(runner.get("os", "")),
                "runner_arch": str(runner.get("arch", "")),
                "lean_check_time_seconds": str(doc.get("lean_check_time_seconds", "")),
                "manifest_path": str(p.relative_to(ROOT)).replace("\\", "/"),
            }
        )

    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("w", newline="", encoding="utf-8") as f:
        fields = [
            "run_id",
            "system_id",
            "provider_name",
            "provider_model",
            "total_input_tokens",
            "total_output_tokens",
            "wall_clock_seconds",
            "estimated_or_billed_cost_usd",
            "cost_status",
            "cost_reason_code",
            "runner_hostname",
            "runner_os",
            "runner_arch",
            "lean_check_time_seconds",
            "manifest_path",
        ]
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        w.writerows(rows)
    print(f"wrote {args.out} ({len(rows)} rows)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

