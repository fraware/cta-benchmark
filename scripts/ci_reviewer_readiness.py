#!/usr/bin/env python3
"""CI checks: paper summary row expectations, JSON validation (cargo), placeholder denylist."""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def cargo_validate(schema: str, path: Path) -> None:
    subprocess.check_call(
        [
            "cargo",
            "run",
            "-p",
            "cta_cli",
            "--quiet",
            "--",
            "validate",
            "file",
            "--schema",
            schema,
            "--path",
            str(path),
        ],
        cwd=ROOT,
    )


def count_csv_rows(path: Path) -> int:
    with path.open(encoding="utf-8", newline="") as f:
        return max(0, sum(1 for _ in f) - 1)


def main() -> int:
    summary_path = ROOT / "benchmark" / "v0.3" / "benchmark_paper_summary.json"
    summary = json.loads(summary_path.read_text(encoding="utf-8"))
    expected = int(summary["expected_instance_level_rows"])
    inst = ROOT / "results" / "instance_level.csv"
    n = count_csv_rows(inst)
    if n != expected:
        print(
            f"error: instance_level.csv rows {n} != benchmark_paper_summary expected {expected}",
            file=sys.stderr,
        )
        return 1

    raw_path = ROOT / "results" / "raw_metrics.json"
    if raw_path.is_file():
        raw = json.loads(raw_path.read_text(encoding="utf-8"))
        rcount = len(raw.get("rows") or [])
        exp_raw = int(summary.get("expected_raw_metrics_rows", expected))
        if rcount != exp_raw:
            print(
                f"error: raw_metrics rows {rcount} != expected_raw_metrics_rows {exp_raw}",
                file=sys.stderr,
            )
            return 1

    strict_path = ROOT / "results" / "raw_metrics_strict.json"
    if strict_path.is_file():
        strict_rows = json.loads(strict_path.read_text(encoding="utf-8")).get("rows") or []
        strict_cnt = len(strict_rows)
        exp_strict = summary.get("expected_raw_metrics_strict_rows")
        if exp_strict is not None and int(exp_strict) != strict_cnt:
            print(
                f"error: raw_metrics_strict rows {strict_cnt} != "
                f"expected_raw_metrics_strict_rows {exp_strict}",
                file=sys.stderr,
            )
            return 1

    man = ROOT / "benchmark" / "v0.3" / "annotation" / "adjudicated_subset" / "manifest.json"
    if man.is_file():
        cargo_validate("annotation_pack_manifest", man)

    pf = ROOT / "benchmark" / "v0.3" / "protocol_freeze.json"
    if pf.is_file():
        cargo_validate("protocol_freeze", pf)

    ont = ROOT / "schemas" / "failure_mode_v1.json"
    if ont.is_file():
        cargo_validate("failure_mode_ontology", ont)

    deny = re.compile(
        r"\b(TODO|placeholder|skeleton|fill\s+after)\b",
        re.IGNORECASE,
    )
    scan_roots = [
        ROOT / "annotation",
        ROOT / "results",
    ]
    for base in scan_roots:
        if not base.is_dir():
            continue
        for p in base.rglob("*"):
            if not p.is_file():
                continue
            rel = p.relative_to(ROOT).as_posix()
            if ".example." in p.name or "/human_wave_v03/" in rel.replace("\\", "/"):
                continue
            if p.suffix.lower() not in {".csv", ".json", ".jsonl", ".md", ".txt"}:
                continue
            try:
                txt = p.read_text(encoding="utf-8", errors="ignore")
            except OSError:
                continue
            if deny.search(txt):
                print(f"error: placeholder denylist matched {rel}", file=sys.stderr)
                return 1

    v3_ann = ROOT / "benchmark" / "v0.3" / "annotation"
    if v3_ann.is_dir():
        for p in v3_ann.rglob("*"):
            if not p.is_file():
                continue
            try:
                rel_ann = p.relative_to(v3_ann)
            except ValueError:
                continue
            if rel_ann.parts[:1] == ("review_packets",):
                continue
            rel = p.relative_to(ROOT).as_posix()
            if ".example." in p.name or "/human_wave_v03/" in rel.replace("\\", "/"):
                continue
            suf = p.suffix.lower()
            if suf == ".csv":
                pass
            elif p.name == "manifest.json" and suf == ".json":
                pass
            elif p.name.lower() == "readme.md" and suf == ".md":
                pass
            else:
                continue
            try:
                txt = p.read_text(encoding="utf-8", errors="ignore")
            except OSError:
                continue
            if deny.search(txt):
                print(f"error: placeholder denylist matched {rel}", file=sys.stderr)
                return 1

    onto = json.loads(ont.read_text(encoding="utf-8"))
    allowed = {str(m.get("slug", "")) for m in onto.get("modes", [])}
    raw_rows_list = json.loads(raw_path.read_text(encoding="utf-8")).get("rows") or []
    for row in raw_rows_list:
        lab = str(row.get("failure_mode_label") or "").strip()
        if lab and lab not in allowed:
            print(f"error: failure_mode_label {lab!r} not in ontology", file=sys.stderr)
            return 1

    print("ci_reviewer_readiness: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
