#!/usr/bin/env python3
"""
Reproduce annotation/agreement_report.json from the committed auditable inputs:

  - annotation/agreement_packet_ids.csv (ordered population; n=192 for v0.3 eval×4 systems)
  - annotation/rater_a.csv, annotation/rater_b.csv (anonymized_packet_key join)

This is a thin wrapper around scripts/compute_agreement_stats.py so reviewers
can verify agreement numbers from the same CSVs tracked in git.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    py = sys.executable
    argv = [
        py,
        str(ROOT / "scripts" / "compute_agreement_stats.py"),
        "--first",
        str(ROOT / "annotation" / "rater_a.csv"),
        "--second",
        str(ROOT / "annotation" / "rater_b.csv"),
    ]
    print("reproduce_agreement_report:", " ".join(argv), file=sys.stderr)
    return subprocess.call(argv, cwd=ROOT)


if __name__ == "__main__":
    raise SystemExit(main())
