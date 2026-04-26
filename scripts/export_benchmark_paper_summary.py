#!/usr/bin/env python3
"""Emit a stable JSON summary of benchmark v0.3 scale for manuscript tables."""

from __future__ import annotations

import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
V3 = ROOT / "benchmark" / "v0.3"
OUT = V3 / "benchmark_paper_summary.json"
RAW = ROOT / "results" / "raw_metrics.json"


def systems_profiled_default() -> list[str]:
    if not RAW.is_file():
        return ["text_only_v1", "code_only_v1", "naive_concat_v1", "full_method_v1"]
    rows = json.loads(RAW.read_text(encoding="utf-8")).get("rows") or []
    if not rows or "system" not in rows[0]:
        return ["text_only_v1", "code_only_v1", "naive_concat_v1", "full_method_v1"]
    return sorted({str(r["system"]) for r in rows})


def main() -> int:
    rows: list[dict] = []
    with (ROOT / "benchmark" / "manifest.jsonl").open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            rows.append(json.loads(line))

    v3 = [r for r in rows if "v0.3" in (r.get("source_provenance") or "")]
    if not v3:
        v3 = rows

    by_split = Counter(r.get("split", "unknown") for r in v3)
    by_family = Counter(r.get("family", "unknown") for r in v3)
    by_diff = Counter(r.get("difficulty", "unknown") for r in v3)

    crit_by_family: dict[str, int] = {}
    for r in v3:
        fam = r.get("family", "unknown")
        n = int(r.get("critical_unit_count", 0) or 0)
        crit_by_family[fam] = crit_by_family.get(fam, 0) + n

    systems = systems_profiled_default()
    n_sys = len(systems)
    payload = {
        "schema_version": "benchmark_paper_summary_v1",
        "benchmark_version": "v0.3",
        "total_instances": len(v3),
        "systems_profiled": systems,
        "systems_profiled_count": n_sys,
        "expected_instance_level_rows": len(v3) * n_sys,
        "expected_raw_metrics_rows": len(v3) * n_sys,
        "family_counts": dict(sorted(by_family.items())),
        "split_counts": dict(sorted(by_split.items())),
        "difficulty_counts": dict(sorted(by_diff.items())),
        "critical_units_sum_by_family": dict(sorted(crit_by_family.items())),
        "critical_units_total": sum(crit_by_family.values()),
        "eval_instance_count": len(json.loads((V3 / "splits" / "eval.json").read_text(encoding="utf-8"))["instance_ids"]),
        "source_manifest": "benchmark/manifest.jsonl",
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
