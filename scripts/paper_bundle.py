#!/usr/bin/env python3
"""
One-command rebuild for v0.3 paper tables and audit metadata.

Steps: hash key inputs, run agreement + results (--paper) + benchmark summary
export + repair materializer + repair counterfactual summary + protocol freeze.

When PAPER_STRICT=1, fail if committed results still contain demo_synthetic markers
intended only for non-paper demo fallback.
"""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
BUILD = ROOT / "build"
OUT_BUILD = BUILD / "paper_build.json"


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    h.update(path.read_bytes())
    return f"sha256:{h.hexdigest()}"


def git_head() -> str:
    try:
        return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return "unknown"


def run_step(label: str, argv: list[str]) -> None:
    print(f"paper_bundle: {label}: {' '.join(argv)}", file=sys.stderr)
    subprocess.check_call(argv, cwd=ROOT)


def scan_paper_strict_denylist() -> None:
    needles = ("demo_synthetic",)
    roots = [
        ROOT / "results",
        ROOT / "annotation",
        ROOT / "reports",
    ]
    hits: list[str] = []
    for base in roots:
        if not base.is_dir():
            continue
        for p in base.rglob("*"):
            if not p.is_file():
                continue
            if ".example." in p.name:
                continue
            try:
                text = p.read_text(encoding="utf-8", errors="ignore")
            except OSError:
                continue
            for n in needles:
                if n in text:
                    hits.append(f"{p.relative_to(ROOT).as_posix()}: contains {n!r}")
    if hits:
        print("PAPER_STRICT=1: denylist hits:\n" + "\n".join(hits), file=sys.stderr)
        raise SystemExit(1)


def main() -> int:
    strict = os.environ.get("PAPER_STRICT") == "1"
    BUILD.mkdir(parents=True, exist_ok=True)

    input_paths = [
        ROOT / "benchmark" / "manifest.jsonl",
        ROOT / "benchmark" / "v0.3" / "splits" / "eval.json",
        ROOT / "results" / "raw_metrics.json",
        ROOT / "results" / "raw_metrics_strict.json",
        ROOT / "results" / "raw_metrics_expanded.json",
        ROOT / "annotation" / "agreement_packet_ids.csv",
        ROOT / "configs" / "experiments" / "benchmark_v03.json",
        ROOT / "benchmark" / "v0.3" / "benchmark_paper_summary.json",
        ROOT / "benchmark" / "v0.3" / "protocol_freeze.json",
        ROOT / "annotation" / "rater_a.csv",
        ROOT / "annotation" / "rater_b.csv",
    ]
    input_hashes: dict[str, str | None] = {}
    for p in input_paths:
        key = p.relative_to(ROOT).as_posix()
        input_hashes[key] = sha256_file(p) if p.is_file() else None

    steps: list[dict[str, Any]] = []
    py = sys.executable

    def record(label: str, argv: list[str]) -> None:
        run_step(label, argv)
        steps.append({"step": label, "argv": argv, "ok": True})

    record(
        "compute_agreement_stats",
        [
            py,
            str(ROOT / "scripts" / "compute_agreement_stats.py"),
            "--first",
            str(ROOT / "annotation" / "rater_a.csv"),
            "--second",
            str(ROOT / "annotation" / "rater_b.csv"),
        ],
    )
    record(
        "materialize_repair_hotspot_artifacts",
        [py, str(ROOT / "scripts" / "materialize_repair_hotspot_artifacts.py")],
    )
    record(
        "compute_results_paper",
        [py, str(ROOT / "scripts" / "compute_results.py"), "--paper"],
    )
    record(
        "export_benchmark_paper_summary",
        [py, str(ROOT / "scripts" / "export_benchmark_paper_summary.py")],
    )
    record(
        "repair_counterfactual_metrics",
        [py, str(ROOT / "scripts" / "repair_counterfactual_metrics.py")],
    )
    record(
        "sign_or_hash_protocol",
        [py, str(ROOT / "scripts" / "sign_or_hash_protocol.py"), "--benchmark-version", "v0.3"],
    )

    if strict:
        scan_paper_strict_denylist()

    output_paths = [
        ROOT / "annotation" / "agreement_report.json",
        ROOT / "results" / "system_summary.csv",
        ROOT / "results" / "system_summary_with_ci.json",
        ROOT / "results" / "instance_level.csv",
        ROOT / "results" / "paper_table_annotation_evidence.csv",
        ROOT / "results" / "paper_table_agreement_evidence.csv",
        ROOT / "results" / "paper_table_systems.csv",
        ROOT / "results" / "appendix_mapped_evidence" / "paper_table_systems.csv",
        ROOT / "results" / "repair_impact_summary.json",
        ROOT / "benchmark" / "v0.3" / "benchmark_paper_summary.json",
        ROOT / "benchmark" / "v0.3" / "protocol_freeze.json",
        ROOT / "repairs" / "hotspot_selection.csv",
    ]
    output_hashes: dict[str, str | None] = {}
    for p in output_paths:
        key = p.relative_to(ROOT).as_posix()
        output_hashes[key] = sha256_file(p) if p.is_file() else None

    try:
        from importlib import metadata as importlib_metadata

        rust_v = subprocess.check_output(["rustc", "-V"], cwd=ROOT, text=True).strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        rust_v = "unknown"

    payload = {
        "schema_version": "paper_build_v1",
        "git_commit": git_head(),
        "python_version": sys.version.split()[0],
        "rustc_version": rust_v,
        "paper_strict": strict,
        "command_argv": [str(x) for x in sys.argv],
        "input_hashes": input_hashes,
        "output_hashes": output_hashes,
        "steps": steps,
    }
    OUT_BUILD.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {OUT_BUILD}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
