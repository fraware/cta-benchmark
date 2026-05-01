#!/usr/bin/env python3
"""Run final CI parity commands and write a markdown evidence log."""

from __future__ import annotations

import argparse
import datetime as dt
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

COMMANDS = [
    ["cargo", "fmt", "--all", "--", "--check"],
    ["cargo", "clippy", "--workspace", "--all-targets", "--no-deps"],
    ["cargo", "test", "--workspace", "--all-targets"],
    ["cargo", "test", "--workspace", "--doc"],
    ["cargo", "run", "-p", "cta_cli", "--", "validate", "schemas"],
    [
        "cargo",
        "run",
        "-p",
        "cta_cli",
        "--",
        "validate",
        "benchmark",
        "--version",
        "v0.3",
        "--release",
    ],
    [
        "cargo",
        "run",
        "-p",
        "cta_cli",
        "--",
        "benchmark",
        "lint",
        "--version",
        "v0.3",
        "--release",
    ],
    [sys.executable, "scripts/materialize_v03_adjudication_artifacts.py"],
    [sys.executable, "scripts/reproduce_agreement_report.py"],
    [sys.executable, "scripts/compute_results.py", "--paper"],
    [sys.executable, "scripts/implement_evidence_hardening.py"],
    [sys.executable, "scripts/export_external_annotation_review_bundle.py"],
    [
        sys.executable,
        "scripts/compute_human_strict_agreement.py",
        "--packet-map",
        "annotation/human_pass_v3/human_strict_packet_ids.csv",
        "--rater-a",
        "annotation/rater_a_strict_all.csv",
        "--rater-b",
        "annotation/human_pass_v3/rater_b_human_strict_all.csv",
        "--out-json",
        "annotation/human_pass_v3/agreement_report_human_strict_all.json",
        "--out-md",
        "annotation/human_pass_v3/agreement_report_human_strict_all.md",
        "--out-disagreements",
        (
            "annotation/human_pass_v3/"
            "disagreement_log_strict_all.csv"
        ),
    ],
    [sys.executable, "scripts/export_benchmark_paper_summary.py"],
    [sys.executable, "scripts/compute_results.py", "--paper"],
    [sys.executable, "scripts/export_external_annotation_review_bundle.py"],
    [sys.executable, "scripts/export_benchmark_paper_summary.py"],
    [sys.executable, "scripts/implement_evidence_hardening.py"],
    [sys.executable, "scripts/validate_release_artifact.py"],
    [sys.executable, "scripts/ci_reviewer_readiness.py"],
    [sys.executable, "scripts/check_paper_claim_sources.py"],
]


def run_cmd(argv: list[str], cwd: Path) -> tuple[int, str]:
    p = subprocess.run(
        argv,
        cwd=cwd,
        text=True,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    return p.returncode, p.stdout


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--out",
        type=Path,
        default=ROOT / "artifacts" / f"final_ci_run_{dt.datetime.now():%Y%m%d}.md",
    )
    args = ap.parse_args()
    args.out.parent.mkdir(parents=True, exist_ok=True)

    logs: list[dict[str, object]] = []
    failed = False
    for cmd in COMMANDS:
        rc, out = run_cmd(cmd, ROOT)
        logs.append({"cmd": cmd, "returncode": rc, "output": out})
        if rc != 0:
            failed = True
            break
    rc_lean, out_lean = run_cmd(["lake", "build"], ROOT / "lean")
    logs.append(
        {
            "cmd": ["cd", "lean", "&&", "lake", "build"],
            "returncode": rc_lean,
            "output": out_lean,
        }
    )
    if rc_lean != 0:
        failed = True

    lines = [
        "# Final CI Parity Evidence",
        "",
        f"- Timestamp (UTC): `{dt.datetime.now(dt.timezone.utc).isoformat()}`",
        f"- Repo root: `{ROOT.as_posix()}`",
        "",
    ]
    for item in logs:
        cmd = " ".join(str(x) for x in item["cmd"])
        status = "PASS" if item["returncode"] == 0 else "FAIL"
        lines += [
            f"## `{cmd}`",
            "",
            f"- Status: **{status}**",
            "",
            "```text",
            str(item["output"]).rstrip(),
            "```",
            "",
        ]
    args.out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {args.out}")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
