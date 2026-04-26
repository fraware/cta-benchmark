#!/usr/bin/env python3
"""
Compute inter-rater agreement from two CSVs (same packet_id rows).

Expected columns (case-sensitive):
  packet_id, semantic_faithfulness, code_consistency, proof_utility, coverage_label

Ordinal columns use integers 1–4. coverage_label uses strings full|partial|failed.

Outputs:
  annotation/agreement_report.json
  annotation/agreement_raw_table.csv
"""

from __future__ import annotations

import argparse
import csv
import json
import random
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT_JSON = ROOT / "annotation" / "agreement_report.json"
OUT_RAW = ROOT / "annotation" / "agreement_raw_table.csv"

ORDINAL_COLS = ("semantic_faithfulness", "code_consistency", "proof_utility")


def load_scores(path: Path) -> dict[str, dict[str, str]]:
    by_id: dict[str, dict[str, str]] = {}
    with path.open(encoding="utf-8", newline="") as f:
        r = csv.DictReader(f)
        for row in r:
            pid = row.get("packet_id", "").strip()
            if not pid:
                continue
            by_id[pid] = {k: (row.get(k) or "").strip() for k in row}
    return by_id


def linear_weighted_kappa(
    xs: list[int], ys: list[int], k: int = 4, reps: int = 10000, rng: random.Random | None = None
) -> tuple[float, tuple[float, float]]:
    """Linear weights on 1..k; returns (kappa, (ci_low, ci_high)) bootstrap on pairs."""
    rng = rng or random.Random(42)
    n = len(xs)
    if n == 0:
        return (float("nan"), (float("nan"), float("nan")))

    def w(i: int, j: int) -> float:
        # i,j in 1..k ; linear weight = 1 - |i-j|/(k-1)
        return 1.0 - abs(i - j) / (k - 1)

    def kappa_for(a: list[int], b: list[int]) -> float:
        tot = len(a)
        num = 0.0
        denom_obs = 0.0
        marg_a = defaultdict(float)
        marg_b = defaultdict(float)
        for i, j in zip(a, b, strict=True):
            wi = w(i, j)
            num += wi
            marg_a[i] += 1.0
            marg_b[j] += 1.0
        po = num / tot
        pe = 0.0
        for i in range(1, k + 1):
            for j in range(1, k + 1):
                pe += marg_a[i] * marg_b[j] * w(i, j) / (tot * tot)
        if abs(1.0 - pe) < 1e-12:
            return float("nan")
        return (po - pe) / (1.0 - pe)

    kap = kappa_for(xs, ys)
    stats: list[float] = []
    for _ in range(reps):
        idx = [rng.randrange(n) for _ in range(n)]
        stats.append(kappa_for([xs[i] for i in idx], [ys[i] for i in idx]))
    stats.sort()
    lo = stats[int(0.025 * (reps - 1))]
    hi = stats[int(0.975 * (reps - 1))]
    return (kap, (lo, hi))


def agreement_rate(xs: list[str], ys: list[str]) -> float:
    if not xs:
        return float("nan")
    return sum(1 for a, b in zip(xs, ys, strict=True) if a == b) / len(xs)


def resolve_rater_csv(label: str, path: Path) -> Path:
    """Use `path` if present; otherwise fall back to `stem.example.csv` in the same directory."""
    p = path.resolve()
    if p.is_file():
        return p
    ex = p.parent / f"{p.stem}.example{p.suffix}"
    if ex.is_file():
        print(
            f"note: {label} {path} not found; using {ex} (copy to {p.name} for real raters).",
            file=sys.stderr,
        )
        return ex
    template = p.parent / f"{p.stem}.example{p.suffix}"
    print(
        f"error: {label} not found: {p}\n"
        f"  expected columns: packet_id, semantic_faithfulness, code_consistency, "
        f"proof_utility, coverage_label\n"
        f"  copy a template if present: {template}",
        file=sys.stderr,
    )
    raise SystemExit(1)


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=(
            "If annotation/rater_a.csv (or rater_b.csv) is missing, the script looks for "
            "rater_a.example.csv / rater_b.example.csv in the same directory."
        ),
    )
    ap.add_argument("--first", type=Path, required=True)
    ap.add_argument("--second", type=Path, required=True)
    args = ap.parse_args()

    first = resolve_rater_csv("--first", args.first)
    second = resolve_rater_csv("--second", args.second)
    a = load_scores(first)
    b = load_scores(second)
    common = sorted(set(a) & set(b))
    if not common:
        payload = {
            "schema_version": "agreement_report_v1",
            "error": "no overlapping packet_id between inputs",
            "first": str(first),
            "second": str(second),
        }
        OUT_JSON.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
        print("no overlapping packet_id rows; wrote", OUT_JSON)
        return 1

    report: dict = {
        "schema_version": "agreement_report_v1",
        "n_packets": len(common),
        "weighted_kappa_linear": {},
        "weighted_kappa_bootstrap_ci95": {},
        "coverage_percent_agreement": None,
    }

    for col in ORDINAL_COLS:
        xs: list[int] = []
        ys: list[int] = []
        for pid in common:
            try:
                xi = int(a[pid][col])
                yi = int(b[pid][col])
            except (KeyError, ValueError):
                continue
            if not (1 <= xi <= 4 and 1 <= yi <= 4):
                continue
            xs.append(xi)
            ys.append(yi)
        if len(xs) >= 2:
            k, (lo, hi) = linear_weighted_kappa(xs, ys)
            report["weighted_kappa_linear"][col] = k
            report["weighted_kappa_bootstrap_ci95"][col] = [lo, hi]
        else:
            report["weighted_kappa_linear"][col] = None
            report["weighted_kappa_bootstrap_ci95"][col] = None

    cx = [a[pid].get("coverage_label", "") for pid in common]
    cy = [b[pid].get("coverage_label", "") for pid in common]
    if all(cx) and all(cy):
        report["coverage_percent_agreement"] = agreement_rate(cx, cy)

    OUT_JSON.parent.mkdir(parents=True, exist_ok=True)
    OUT_JSON.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    with OUT_RAW.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["packet_id", "dim", "rater_a", "rater_b", "agree"])
        for pid in common:
            for col in ORDINAL_COLS:
                va, vb = a[pid].get(col, ""), b[pid].get(col, "")
                w.writerow(
                    [
                        pid,
                        col,
                        va,
                        vb,
                        int(va == vb) if va.isdigit() and vb.isdigit() else "",
                    ]
                )

    print(f"wrote {OUT_JSON} and {OUT_RAW} ({len(common)} packets)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
