#!/usr/bin/env python3
"""
Reproduce agreement_report.json from committed auditable inputs:

  - annotation/agreement_packet_ids.csv (ordered population; n=192 for v0.3 eval×4 systems)
  - annotation/rater_a.csv, annotation/rater_b.csv (anonymized_packet_key join)

This is a thin wrapper around scripts/compute_agreement_stats.py so reviewers
can verify agreement numbers from the same CSVs tracked in git.
"""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    py = sys.executable
    second = ROOT / "annotation" / "rater_b.csv"
    human_second = ROOT / "annotation" / "rater_b_human.csv"
    if human_second.is_file():
        second = human_second
    argv = [
        py,
        str(ROOT / "scripts" / "compute_agreement_stats.py"),
        "--first",
        str(ROOT / "annotation" / "rater_a.csv"),
        "--second",
        str(second),
    ]
    print("reproduce_agreement_report:", " ".join(argv), file=sys.stderr)
    rc = subprocess.call(argv, cwd=ROOT)
    if rc != 0:
        return rc
    if second == human_second:
        # Keep explicit human-labeled artifacts for manuscript defaults.
        src_json = ROOT / "annotation" / "agreement_report.json"
        src_md = ROOT / "annotation" / "agreement_report.md"
        src_raw = ROOT / "annotation" / "agreement_raw_table.csv"
        dst_json = ROOT / "annotation" / "agreement_report_human.json"
        dst_md = ROOT / "annotation" / "agreement_report_human.md"
        dst_raw = ROOT / "annotation" / "agreement_raw_table_human.csv"
        if src_json.is_file():
            shutil.copyfile(src_json, dst_json)
        if src_md.is_file():
            shutil.copyfile(src_md, dst_md)
        if src_raw.is_file():
            shutil.copyfile(src_raw, dst_raw)
        evi = ROOT / "results" / "paper_table_agreement_evidence.csv"
        evi_h = ROOT / "results" / "paper_table_agreement_evidence_human.csv"
        if evi.is_file():
            shutil.copyfile(evi, evi_h)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
