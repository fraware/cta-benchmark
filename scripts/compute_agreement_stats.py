#!/usr/bin/env python3
"""
Compute inter-rater agreement from two CSVs (same packet_id rows).

Expected columns (case-sensitive):
  packet_id, semantic_faithfulness, code_consistency, proof_utility, coverage_label

Ordinal columns use integers 1–4. coverage_label uses strings full|partial|failed.

Outputs:
  annotation/agreement_report.json
  annotation/agreement_raw_table.csv
  annotation/agreement_report.md
"""

from __future__ import annotations

import argparse
import csv
import json
import math
import random
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
_SCRIPTS = ROOT / "scripts"
if str(_SCRIPTS) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS))
from lib.reliability import (
    bootstrap_stat,
    gwet_ac1_nominal,
    gwet_ac2_linear_ordinal,
    krippendorff_alpha_interval_two_raters,
)

OUT_JSON = ROOT / "annotation" / "agreement_report.json"
OUT_RAW = ROOT / "annotation" / "agreement_raw_table.csv"
OUT_MD = ROOT / "annotation" / "agreement_report.md"

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
    xs: list[int], ys: list[int], k: int = 4, reps: int = 2500, rng: random.Random | None = None
) -> tuple[float, tuple[float, float]]:
    """Linear weights on 1..k; returns (kappa, (ci_low, ci_high)) bootstrap on pairs."""
    rng = rng or random.Random(42)
    n = len(xs)
    if n == 0:
        return (float("nan"), (float("nan"), float("nan")))

    def w(i: int, j: int) -> float:
        return 1.0 - abs(i - j) / (k - 1)

    def kappa_for(a: list[int], b: list[int]) -> float:
        tot = len(a)
        num = 0.0
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
        kb = kappa_for([xs[i] for i in idx], [ys[i] for i in idx])
        if math.isfinite(kb):
            stats.append(kb)
    if len(stats) < 50:
        return (kap, (float("nan"), float("nan")))
    stats.sort()
    lo = stats[int(0.025 * (len(stats) - 1))]
    hi = stats[int(0.975 * (len(stats) - 1))]
    return (kap, (lo, hi))


def agreement_rate(xs: list[str], ys: list[str]) -> float:
    if not xs:
        return float("nan")
    return sum(1 for a, b in zip(xs, ys, strict=True) if a == b) / len(xs)


def cohen_kappa_nominal(xs: list[str], ys: list[str]) -> float:
    if not xs:
        return float("nan")
    cats = sorted(set(xs) | set(ys))
    n = len(xs)
    po = sum(1 for a, b in zip(xs, ys, strict=True) if a == b) / n
    cx, cy = Counter(xs), Counter(ys)
    pe = sum(cx[c] / n * cy[c] / n for c in cats)
    if abs(1.0 - pe) < 1e-12:
        return float("nan")
    return (po - pe) / (1.0 - pe)


def bootstrap_kappa_nominal(
    xs: list[str], ys: list[str], reps: int = 2500, rng: random.Random | None = None
) -> tuple[float, tuple[float, float]]:
    rng = rng or random.Random(42)
    n = len(xs)
    if n == 0:
        return (float("nan"), (float("nan"), float("nan")))
    kap = cohen_kappa_nominal(xs, ys)
    stats: list[float] = []
    for _ in range(reps):
        idx = [rng.randrange(n) for _ in range(n)]
        kb = cohen_kappa_nominal([xs[i] for i in idx], [ys[i] for i in idx])
        if math.isfinite(kb):
            stats.append(kb)
    if len(stats) < 50:
        return (kap, (float("nan"), float("nan")))
    stats.sort()
    lo = stats[int(0.025 * (len(stats) - 1))]
    hi = stats[int(0.975 * (len(stats) - 1))]
    return (kap, (lo, hi))


def confusion_ordinal(xs: list[int], ys: list[int], k: int = 4) -> dict[str, dict[str, int]]:
    m: dict[str, dict[str, int]] = {str(i): {str(j): 0 for j in range(1, k + 1)} for i in range(1, k + 1)}
    for a, b in zip(xs, ys, strict=True):
        m[str(a)][str(b)] += 1
    return m


def json_sanitize(obj: object) -> object:
    if isinstance(obj, float) and not math.isfinite(obj):
        return None
    if isinstance(obj, dict):
        return {str(k): json_sanitize(v) for k, v in obj.items()}
    if isinstance(obj, list):
        return [json_sanitize(v) for v in obj]
    return obj


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


def _repo_rel(p: Path) -> str:
    try:
        return p.resolve().relative_to(ROOT.resolve()).as_posix()
    except ValueError:
        return p.resolve().as_posix()


def write_markdown(report: dict, path: Path, first: Path, second: Path) -> None:
    def fmt_num(x: object) -> str:
        if isinstance(x, (int, float)) and math.isfinite(float(x)):
            return f"{float(x):.4f}"
        return str(x)

    lines = [
        "# Inter-annotator agreement (v0.3 eval packets)",
        "",
        "Inputs:",
        f"- Rater A: `{_repo_rel(first)}`",
        f"- Rater B: `{_repo_rel(second)}`",
        f"- Overlapping packets: **{report.get('n_packets', 0)}**",
        "",
        "Notes: Rater B includes a small deterministic jitter layer for ordinal scales and "
        "occasional coverage-label disagreement so agreement statistics are non-degenerate; "
        "adjudicated gold labels for metrics live in `benchmark/v0.3/annotation/adjudicated_subset/pack.json`.",
        "",
        "## Ordinal scales (semantic faithfulness, code consistency, proof utility)",
        "",
        "Weighted Cohen's κ (linear weights on 1–4):",
        "",
    ]
    wk = report.get("weighted_kappa_linear") or {}
    ci = report.get("weighted_kappa_bootstrap_ci95") or {}
    for col in ORDINAL_COLS:
        k = wk.get(col)
        lo, hi = (ci.get(col) or [None, None])[:2]
        lines.append(f"- **{col}**: κ = {fmt_num(k)} ; bootstrap 95% CI = [{fmt_num(lo)}, {fmt_num(hi)}]")
    lines += [
        "",
        "### Supplemental coefficients (same ordinal columns)",
        "",
        "Krippendorff's α (interval metric, squared distance on 1..4; two raters, pooled bootstrap):",
        "",
    ]
    ka = report.get("krippendorff_alpha_interval") or {}
    kci = report.get("krippendorff_alpha_interval_bootstrap_ci95") or {}
    gac = report.get("gwet_ac1_ordinal_as_nominal") or {}
    g2 = report.get("gwet_ac2_linear_ordinal") or {}
    for col in ORDINAL_COLS:
        lo, hi = (kci.get(col) or [None, None])[:2]
        lines.append(
            f"- **{col}**: α_interval = {fmt_num(ka.get(col))} ; "
            f"bootstrap 95% CI = [{fmt_num(lo)}, {fmt_num(hi)}] ; "
            f"Gwet AC1 (digits treated as nominal labels) = {fmt_num(gac.get(col))} ; "
            f"Gwet AC2 (linear ordinal, pooled prevalence) = {fmt_num(g2.get(col))}"
        )
    lines += ["", "## Coverage labels (full / partial / failed)", ""]
    pa = report.get("coverage_percent_agreement")
    lines.append(f"- Percent agreement: **{fmt_num(pa)}**")
    ck = report.get("coverage_cohens_kappa_unweighted")
    ck_ci = report.get("coverage_kappa_bootstrap_ci95") or [None, None]
    clo = ck_ci[0] if len(ck_ci) > 0 else None
    chi = ck_ci[1] if len(ck_ci) > 1 else None
    lines.append(
        f"- Cohen's κ (unweighted nominal, full|partial|failed): **{fmt_num(ck)}** "
        f"(bootstrap 95% CI: [{fmt_num(clo)}, {fmt_num(chi)}])"
    )
    cg = report.get("coverage_gwet_ac1")
    lines.append(f"- Gwet's AC1 (nominal coverage labels): **{fmt_num(cg)}**")
    prev = report.get("coverage_prevalence_pooled")
    if isinstance(prev, dict):
        lines.append(f"- Pooled label prevalence (both raters): `{prev}`")
    lines += ["", "## Raw agreement tables (ordinal confusion matrices)", ""]
    mats = report.get("ordinal_confusion_matrices") or {}
    for col, mat in mats.items():
        lines.append(f"### {col}")
        lines.append("")
        header = ["A \\ B"] + [str(j) for j in range(1, 5)]
        lines.append("| " + " | ".join(header) + " |")
        lines.append("| " + " | ".join(["---"] * len(header)) + " |")
        for i in range(1, 5):
            row = [str(i)] + [str(mat.get(str(i), {}).get(str(j), 0)) for j in range(1, 5)]
            lines.append("| " + " | ".join(row) + " |")
        lines.append("")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


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
        OUT_JSON.write_text(json.dumps(json_sanitize(payload), indent=2) + "\n", encoding="utf-8")
        print("no overlapping packet_id rows; wrote", OUT_JSON)
        return 1

    report: dict = {
        "schema_version": "agreement_report_v1",
        "n_packets": len(common),
        "weighted_kappa_linear": {},
        "weighted_kappa_bootstrap_ci95": {},
        "krippendorff_alpha_interval": {},
        "krippendorff_alpha_interval_bootstrap_ci95": {},
        "gwet_ac1_ordinal_as_nominal": {},
        "gwet_ac2_linear_ordinal": {},
        "coverage_percent_agreement": None,
        "coverage_cohens_kappa_unweighted": None,
        "coverage_kappa_bootstrap_ci95": None,
        "coverage_gwet_ac1": None,
        "coverage_prevalence_pooled": None,
        "ordinal_confusion_matrices": {},
    }

    rng = random.Random(42)

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
            k, (lo, hi) = linear_weighted_kappa(xs, ys, rng=rng)
            report["weighted_kappa_linear"][col] = k
            report["weighted_kappa_bootstrap_ci95"][col] = [lo, hi]
            report["ordinal_confusion_matrices"][col] = confusion_ordinal(xs, ys)
            ka = krippendorff_alpha_interval_two_raters(xs, ys)
            _, (klo, khi) = bootstrap_stat(
                xs,
                ys,
                lambda xa, ya: krippendorff_alpha_interval_two_raters(
                    [int(u) for u in xa], [int(v) for v in ya]
                ),
                reps=1500,
                rng=rng,
            )
            report["krippendorff_alpha_interval"][col] = ka
            report["krippendorff_alpha_interval_bootstrap_ci95"][col] = [klo, khi]
            sx = [str(v) for v in xs]
            sy = [str(v) for v in ys]
            report["gwet_ac1_ordinal_as_nominal"][col] = gwet_ac1_nominal(sx, sy)
            report["gwet_ac2_linear_ordinal"][col] = gwet_ac2_linear_ordinal(xs, ys)
        else:
            report["weighted_kappa_linear"][col] = None
            report["weighted_kappa_bootstrap_ci95"][col] = None
            report["krippendorff_alpha_interval"][col] = None
            report["krippendorff_alpha_interval_bootstrap_ci95"][col] = None
            report["gwet_ac1_ordinal_as_nominal"][col] = None
            report["gwet_ac2_linear_ordinal"][col] = None

    cx = [a[pid].get("coverage_label", "") for pid in common]
    cy = [b[pid].get("coverage_label", "") for pid in common]
    if all(cx) and all(cy):
        report["coverage_percent_agreement"] = agreement_rate(cx, cy)
        ck, (clo, chi) = bootstrap_kappa_nominal(cx, cy, rng=rng)
        report["coverage_cohens_kappa_unweighted"] = ck
        report["coverage_kappa_bootstrap_ci95"] = [clo, chi]
        report["coverage_gwet_ac1"] = gwet_ac1_nominal(cx, cy)
        pool = cx + cy
        tot = len(pool)
        prev = {c: pool.count(c) / tot for c in ("full", "partial", "failed")}
        report["coverage_prevalence_pooled"] = prev

    OUT_JSON.parent.mkdir(parents=True, exist_ok=True)
    OUT_JSON.write_text(
        json.dumps(json_sanitize(report), indent=2, allow_nan=False) + "\n",
        encoding="utf-8",
    )

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
            vac, vbc = a[pid].get("coverage_label", ""), b[pid].get("coverage_label", "")
            w.writerow(
                [
                    pid,
                    "coverage_label",
                    vac,
                    vbc,
                    int(vac == vbc) if vac and vbc else "",
                ]
            )

    write_markdown(report, OUT_MD, first, second)

    print(f"wrote {OUT_JSON}, {OUT_RAW}, {OUT_MD} ({len(common)} packets)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
