#!/usr/bin/env python3
"""
Publication-grade result tables: uncertainty summaries, family breakdowns,
failure-mode counts, instance-level export, composite sensitivity.

By default, if ``results/raw_metrics.json`` is absent, emits deterministic
demo-structured outputs from the manifest (CI convenience) and prints a
warning to stderr.

With ``--paper``, demo fallback is disabled: the script exits with an error
unless a valid raw metrics file is present.
"""

from __future__ import annotations

import argparse
import csv
import json
import math
import random
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

DEFAULT_SYSTEMS = ["text_only_v1", "code_only_v1", "naive_concat_v1", "full_method_v1"]


def load_manifest_rows(path: Path) -> list[dict]:
    rows: list[dict] = []
    with path.open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            rows.append(json.loads(line))
    return rows


def mean(xs: list[float]) -> float:
    return sum(xs) / len(xs) if xs else float("nan")


def std_sample(xs: list[float]) -> float:
    if len(xs) < 2:
        return float("nan")
    m = mean(xs)
    v = sum((x - m) ** 2 for x in xs) / (len(xs) - 1)
    return math.sqrt(v)


def median(xs: list[float]) -> float:
    ys = sorted(xs)
    n = len(ys)
    if n == 0:
        return float("nan")
    mid = n // 2
    if n % 2 == 1:
        return ys[mid]
    return 0.5 * (ys[mid - 1] + ys[mid])


def iqr(xs: list[float]) -> float:
    ys = sorted(xs)
    n = len(ys)
    if n < 2:
        return float("nan")

    def q(p: float) -> float:
        if n == 1:
            return ys[0]
        idx = p * (n - 1)
        lo = int(math.floor(idx))
        hi = int(math.ceil(idx))
        if lo == hi:
            return ys[lo]
        return ys[lo] + (ys[hi] - ys[lo]) * (idx - lo)

    return q(0.75) - q(0.25)


def bootstrap_ci95(xs: list[float], rng: random.Random, reps: int = 2000) -> tuple[float, float]:
    if not xs:
        return (float("nan"), float("nan"))
    n = len(xs)
    stats: list[float] = []
    for _ in range(reps):
        sample = [xs[rng.randrange(n)] for _ in range(n)]
        stats.append(mean(sample))
    stats.sort()
    lo = stats[int(0.025 * (reps - 1))]
    hi = stats[int(0.975 * (reps - 1))]
    return (lo, hi)


def domain_of_family(fam: str) -> str:
    if fam.startswith("arrays_"):
        return "arrays"
    if fam.startswith("sorting_"):
        return "sorting"
    if fam.startswith("graph_"):
        return "graph"
    if fam.startswith("greedy_"):
        return "greedy"
    if fam.startswith("dp_"):
        return "dp"
    if fam.startswith("trees_"):
        return "trees"
    return "unknown"


def load_raw_metrics(path: Path) -> tuple[list[dict], bool]:
    """Returns (rows, is_v2_instance_granular)."""
    payload = json.loads(path.read_text(encoding="utf-8"))
    rows = payload.get("rows") or []
    is_v2 = bool(rows) and "instance_id" in rows[0]
    return rows, is_v2


PRIMARY_METRICS = (
    "faithfulness_mean",
    "code_consistency_mean",
    "vacuity_rate",
    "proof_utility_mean",
)

SUMMARY_HEADER = [
    "system",
    "mean",
    "sd",
    "median",
    "iqr",
    "bootstrap_ci95_low",
    "bootstrap_ci95_high",
    "n",
]

# Default reliability weights (documented in system_reliability_summary.csv header comment via column names)
REL_WF, REL_WC, REL_WP = 0.34, 0.33, 0.33
REL_WV, REL_WX, REL_WM = 0.15, 0.10, 0.10


def write_system_metric_csv(
    path: Path,
    systems: list[str],
    metric_key: str,
    multi_by_system: dict[str, dict[str, list[float]]],
    rng: random.Random,
) -> None:
    with path.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(SUMMARY_HEADER)
        for sid in systems:
            xs = multi_by_system.get(sid, {}).get(metric_key, [])
            lo, hi = bootstrap_ci95(xs, rng) if xs else (float("nan"), float("nan"))
            w.writerow(
                [
                    sid,
                    f"{mean(xs):.4f}" if xs else "",
                    f"{std_sample(xs):.4f}" if xs else "",
                    f"{median(xs):.4f}" if xs else "",
                    f"{iqr(xs):.4f}" if xs else "",
                    f"{lo:.4f}" if xs and not math.isnan(lo) else "",
                    f"{hi:.4f}" if xs and not math.isnan(hi) else "",
                    len(xs),
                ]
            )


def instance_reliability(
    row: dict,
    wf: float,
    wc: float,
    wp: float,
    wv: float,
    wx: float,
    wm: float,
) -> float:
    """Weighted composite minus explicit penalties (all inputs in [0,1] scale except counts)."""
    f = float(row.get("faithfulness_mean", 0))
    c = float(row.get("code_consistency_mean", f))
    p = float(row.get("proof_utility_mean", f))
    v = float(row.get("vacuity_rate", 0))
    x = 1.0 if bool(row.get("contradiction_flag")) else 0.0
    miss = int(row.get("missing_critical_units", 0) or 0)
    mpen = min(1.0, miss / 6.0)
    return wf * f + wc * c + wp * p - wv * v - wx * x - wm * mpen


def write_family_metric_csv(
    path: Path,
    systems: list[str],
    fams: list[str],
    metric_key: str,
    multi_by_sys_fam: dict[tuple[str, str], dict[str, list[float]]],
    rng: random.Random,
) -> None:
    with path.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(
            [
                "family",
                "domain",
                "system",
                "mean",
                "sd",
                "median",
                "iqr",
                "bootstrap_ci95_low",
                "bootstrap_ci95_high",
                "n",
                "aggregate_metric",
            ]
        )
        for fam in fams:
            dom = domain_of_family(fam)
            for sid in systems:
                xs = multi_by_sys_fam.get((sid, fam), {}).get(metric_key, [])
                lo, hi = bootstrap_ci95(xs, rng, reps=2000)
                w.writerow(
                    [
                        fam,
                        dom,
                        sid,
                        f"{mean(xs):.4f}" if xs else "",
                        f"{std_sample(xs):.4f}" if xs else "",
                        f"{median(xs):.4f}" if xs else "",
                        f"{iqr(xs):.4f}" if xs else "",
                        f"{lo:.4f}" if xs else "",
                        f"{hi:.4f}" if xs else "",
                        len(xs),
                        metric_key,
                    ]
                )


def load_failure_slugs(ontology_path: Path) -> set[str]:
    data = json.loads(ontology_path.read_text(encoding="utf-8"))
    return {str(m.get("slug", "")) for m in data.get("modes", [])}


def load_hotspot_repair_keys(path: Path) -> set[tuple[str, str]]:
    if not path.is_file():
        return set()
    keys: set[tuple[str, str]] = set()
    with path.open(encoding="utf-8", newline="") as f:
        r = csv.DictReader(f)
        for row in r:
            if (row.get("selected") or "").strip().lower() != "true":
                continue
            iid = (row.get("instance_id") or "").strip()
            sid = (row.get("system_id") or "").strip()
            if iid and sid:
                keys.add((iid, sid))
    return keys


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--manifest", type=Path, default=ROOT / "benchmark" / "manifest.jsonl")
    ap.add_argument("--raw-metrics", type=Path, default=ROOT / "results" / "raw_metrics.json")
    ap.add_argument("--out-dir", type=Path, default=ROOT / "results")
    ap.add_argument("--seed", type=int, default=42)
    ap.add_argument(
        "--failure-ontology",
        type=Path,
        default=ROOT / "schemas" / "failure_mode_v1.json",
        help="JSON list of allowed failure_mode_label slugs (see failure_mode_ontology_v1).",
    )
    ap.add_argument(
        "--hotspot-selection",
        type=Path,
        default=ROOT / "repairs" / "hotspot_selection.csv",
        help="When present, rows with selected=true set repair_subset=yes in instance_level.csv.",
    )
    ap.add_argument(
        "--paper",
        action="store_true",
        help="Require results/raw_metrics.json; never emit demo fallback.",
    )
    args = ap.parse_args()

    rng = random.Random(args.seed)
    mrows = load_manifest_rows(args.manifest)
    fams = sorted({r["family"] for r in mrows})

    metrics_by_system: dict[str, list[float]] = defaultdict(list)
    by_sys_fam: dict[tuple[str, str], list[float]] = defaultdict(list)
    by_sys_inst: dict[tuple[str, str], dict] = {}
    failure_by: list[dict] = []
    systems: list[str] = list(DEFAULT_SYSTEMS)
    multi_by_system: dict[str, dict[str, list[float]]] = defaultdict(
        lambda: {m: [] for m in PRIMARY_METRICS}
    )
    multi_by_sys_fam: dict[tuple[str, str], dict[str, list[float]]] = defaultdict(
        lambda: {m: [] for m in PRIMARY_METRICS}
    )

    use_demo = False
    if args.raw_metrics.is_file():
        raw_rows, is_v2 = load_raw_metrics(args.raw_metrics)
        if not raw_rows:
            if args.paper:
                print("error: raw_metrics.json has no rows (--paper)", file=sys.stderr)
                return 1
            use_demo = True
        elif is_v2:
            allowed_failures = (
                load_failure_slugs(args.failure_ontology) if args.failure_ontology.is_file() else set()
            )
            seen_sys: set[str] = set()
            for row in raw_rows:
                sid = str(row["system"])
                fam = str(row["family"])
                iid = str(row["instance_id"])
                seen_sys.add(sid)
                score = float(row["faithfulness_mean"])
                metrics_by_system[sid].append(score)
                by_sys_fam[(sid, fam)].append(score)
                by_sys_inst[(sid, iid)] = row
                for m in PRIMARY_METRICS:
                    if m in row:
                        multi_by_system[sid][m].append(float(row[m]))
                        multi_by_sys_fam[(sid, fam)][m].append(float(row[m]))
                fml = (row.get("failure_mode_label") or "").strip()
                if args.paper and allowed_failures and fml not in allowed_failures:
                    print(
                        f"error: failure_mode_label {fml!r} not in ontology "
                        f"({args.failure_ontology}) (--paper)",
                        file=sys.stderr,
                    )
                    return 1
                if fml:
                    failure_by.append({"system": sid, "family": fam, "failure_mode": fml, "count": 1})
                elif float(row["faithfulness_mean"]) < 0.45:
                    failure_by.append(
                        {"system": sid, "family": fam, "failure_mode": "low_faithfulness", "count": 1}
                    )
            systems = sorted(seen_sys) if seen_sys else list(DEFAULT_SYSTEMS)
        else:
            # legacy aggregate rows (family × system)
            for row in raw_rows:
                sid = str(row["system"])
                fam = str(row["family"])
                score = float(row["faithfulness_mean"])
                metrics_by_system[sid].append(score)
                by_sys_fam[(sid, fam)].append(score)
            systems = sorted(metrics_by_system.keys()) if metrics_by_system else list(DEFAULT_SYSTEMS)
    else:
        if args.paper:
            print(
                f"error: raw metrics missing at {args.raw_metrics} (--paper forbids demo fallback)",
                file=sys.stderr,
            )
            return 1
        use_demo = True
        print(
            f"warning: raw_metrics.json not found at {args.raw_metrics}; demo fallback active.",
            file=sys.stderr,
        )

    if use_demo:
        systems = ["code_only_v1", "naive_concat_v1", "full_method_v1"]
        for sid in systems:
            base = {"code_only_v1": 2.2, "naive_concat_v1": 2.6, "full_method_v1": 3.1}[sid]
            for fam in fams:
                for _i in range(7):
                    noise = (hash((sid, fam, _i)) % 1000) / 1000.0 - 0.5
                    s = max(1.0, min(4.0, base + 0.35 * noise))
                    metrics_by_system[sid].append(s)
                    by_sys_fam[(sid, fam)].append(s)
                    if s < 2.5:
                        failure_by.append(
                            {
                                "system": sid,
                                "family": fam,
                                "failure_mode": "low_faithfulness",
                                "count": 1,
                            }
                        )

    args.out_dir.mkdir(parents=True, exist_ok=True)
    repair_keys = load_hotspot_repair_keys(args.hotspot_selection)

    # Per-metric system summaries (no ambiguous "system_summary" compression).
    sys_faith = args.out_dir / "system_faithfulness_summary.csv"
    sys_cons = args.out_dir / "system_consistency_summary.csv"
    sys_vac = args.out_dir / "system_vacuity_summary.csv"
    sys_pu = args.out_dir / "system_proof_utility_summary.csv"
    if not use_demo and by_sys_inst:
        write_system_metric_csv(sys_faith, systems, "faithfulness_mean", multi_by_system, rng)
        write_system_metric_csv(sys_cons, systems, "code_consistency_mean", multi_by_system, rng)
        write_system_metric_csv(sys_vac, systems, "vacuity_rate", multi_by_system, rng)
        write_system_metric_csv(sys_pu, systems, "proof_utility_mean", multi_by_system, rng)
    else:
        for p in (sys_faith, sys_cons, sys_vac, sys_pu):
            with p.open("w", newline="", encoding="utf-8") as f:
                w = csv.writer(f)
                w.writerow(SUMMARY_HEADER)
                for sid in systems:
                    xs = metrics_by_system.get(sid, [])
                    lo, hi = bootstrap_ci95(xs, rng)
                    w.writerow(
                        [
                            sid,
                            f"{mean(xs):.4f}",
                            f"{std_sample(xs):.4f}",
                            f"{median(xs):.4f}",
                            f"{iqr(xs):.4f}",
                            f"{lo:.4f}",
                            f"{hi:.4f}",
                            len(xs),
                        ]
                    )

    # Deprecated alias: faithfulness-only (same as system_faithfulness_summary.csv).
    sys_path = args.out_dir / "system_summary.csv"
    if sys_faith.is_file():
        sys_path.write_text(sys_faith.read_text(encoding="utf-8"), encoding="utf-8")
    else:
        with sys_path.open("w", newline="", encoding="utf-8") as f:
            w = csv.writer(f)
            w.writerow(SUMMARY_HEADER)
            for sid in systems:
                xs = metrics_by_system.get(sid, [])
                lo, hi = bootstrap_ci95(xs, rng)
                w.writerow(
                    [
                        sid,
                        f"{mean(xs):.4f}",
                        f"{std_sample(xs):.4f}",
                        f"{median(xs):.4f}",
                        f"{iqr(xs):.4f}",
                        f"{lo:.4f}",
                        f"{hi:.4f}",
                        len(xs),
                    ]
                )

    fam_faith = args.out_dir / "family_faithfulness_summary.csv"
    fam_cons = args.out_dir / "family_consistency_summary.csv"
    fam_vac = args.out_dir / "family_vacuity_summary.csv"
    fam_pu = args.out_dir / "family_proof_utility_summary.csv"
    if not use_demo and by_sys_inst:
        write_family_metric_csv(
            fam_faith, systems, fams, "faithfulness_mean", multi_by_sys_fam, rng
        )
        write_family_metric_csv(
            fam_cons, systems, fams, "code_consistency_mean", multi_by_sys_fam, rng
        )
        write_family_metric_csv(fam_vac, systems, fams, "vacuity_rate", multi_by_sys_fam, rng)
        write_family_metric_csv(
            fam_pu, systems, fams, "proof_utility_mean", multi_by_sys_fam, rng
        )
    fam_path = args.out_dir / "family_summary.csv"
    if fam_faith.is_file():
        fam_path.write_text(fam_faith.read_text(encoding="utf-8"), encoding="utf-8")
    else:
        with fam_path.open("w", newline="", encoding="utf-8") as f:
            w = csv.writer(f)
            w.writerow(
                [
                    "family",
                    "domain",
                    "system",
                    "mean",
                    "sd",
                    "median",
                    "iqr",
                    "bootstrap_ci95_low",
                    "bootstrap_ci95_high",
                    "n",
                    "aggregate_metric",
                ]
            )
            for fam in fams:
                dom = domain_of_family(fam)
                for sid in systems:
                    xs = by_sys_fam.get((sid, fam), [])
                    lo, hi = bootstrap_ci95(xs, rng, reps=2000)
                    w.writerow(
                        [
                            fam,
                            dom,
                            sid,
                            f"{mean(xs):.4f}",
                            f"{std_sample(xs):.4f}",
                            f"{median(xs):.4f}",
                            f"{iqr(xs):.4f}",
                            f"{lo:.4f}",
                            f"{hi:.4f}",
                            len(xs),
                            "faithfulness_mean",
                        ]
                    )

    # Explicit reliability composite (documented weights; not a hidden "mean").
    rel_path = args.out_dir / "system_reliability_summary.csv"
    rel_sens_path = args.out_dir / "system_reliability_sensitivity.csv"
    if not use_demo and by_sys_inst:
        rel_by_sys: dict[str, list[float]] = defaultdict(list)
        for sid in systems:
            for mr in mrows:
                row = by_sys_inst.get((sid, mr["instance_id"]))
                if not row:
                    continue
                rel_by_sys[sid].append(
                    instance_reliability(
                        row, REL_WF, REL_WC, REL_WP, REL_WV, REL_WX, REL_WM
                    )
                )
        with rel_path.open("w", newline="", encoding="utf-8") as f:
            w = csv.writer(f)
            w.writerow(
                SUMMARY_HEADER
                + [
                    "w_faithfulness",
                    "w_code_consistency",
                    "w_proof_utility",
                    "w_vacuity",
                    "w_contradiction",
                    "w_missing_critical",
                    "reliability_definition",
                ]
            )
            definition = (
                "wf*faithfulness + wc*code_consistency + wp*proof_utility "
                "- wv*vacuity_rate - wx*I(contradiction) - wm*min(1,missing/6)"
            )
            for sid in systems:
                xs = rel_by_sys.get(sid, [])
                lo, hi = bootstrap_ci95(xs, rng)
                w.writerow(
                    [
                        sid,
                        f"{mean(xs):.4f}" if xs else "",
                        f"{std_sample(xs):.4f}" if xs else "",
                        f"{median(xs):.4f}" if xs else "",
                        f"{iqr(xs):.4f}" if xs else "",
                        f"{lo:.4f}" if xs else "",
                        f"{hi:.4f}" if xs else "",
                        len(xs),
                        REL_WF,
                        REL_WC,
                        REL_WP,
                        REL_WV,
                        REL_WX,
                        REL_WM,
                        definition,
                    ]
                )
        triples = [
            (REL_WF, REL_WC, REL_WP),
            (0.5, 0.25, 0.25),
            (0.4, 0.3, 0.3),
            (0.6, 0.2, 0.2),
            (0.45, 0.275, 0.275),
        ]
        with rel_sens_path.open("w", newline="", encoding="utf-8") as f:
            w = csv.writer(f)
            w.writerow(
                [
                    "w_faithfulness",
                    "w_code_consistency",
                    "w_proof_utility",
                    "w_vacuity",
                    "w_contradiction",
                    "w_missing_critical",
                    "system",
                    "reliability_mean",
                ]
            )
            for wf, wc, wp in triples:
                if abs(wf + wc + wp - 1.0) > 1e-6:
                    continue
                for sid in systems:
                    xs = []
                    for mr in mrows:
                        row = by_sys_inst.get((sid, mr["instance_id"]))
                        if row:
                            xs.append(
                                instance_reliability(
                                    row, wf, wc, wp, REL_WV, REL_WX, REL_WM
                                )
                            )
                    if xs:
                        w.writerow(
                            [
                                wf,
                                wc,
                                wp,
                                REL_WV,
                                REL_WX,
                                REL_WM,
                                sid,
                                f"{mean(xs):.4f}",
                            ]
                        )

    # failure_mode_counts.csv
    fail_path = args.out_dir / "failure_mode_counts.csv"
    agg: dict[tuple[str, str, str], int] = defaultdict(int)
    for row in failure_by:
        key = (row["system"], row["family"], row["failure_mode"])
        agg[key] += row["count"]
    with fail_path.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(["system", "family", "failure_mode", "count"])
        for (sid, fam, mode), c in sorted(agg.items()):
            w.writerow([sid, fam, mode, c])
        if not agg:
            w.writerow(["code_only_v1", "global", "no_failures_recorded", 0])

    # instance_level.csv
    inst_path = args.out_dir / "instance_level.csv"
    with inst_path.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(
            [
                "instance_id",
                "family",
                "domain",
                "split",
                "system",
                "faithfulness_mean",
                "code_consistency_mean",
                "vacuity_rate",
                "proof_utility_mean",
                "contradiction_flag",
                "missing_critical_units",
                "failure_mode_label",
                "annotation_origin",
                "source_template_id",
                "repair_subset",
            ]
        )
        for r in mrows:
            dom = domain_of_family(r["family"])
            iid = r["instance_id"]
            for sid in systems:
                row = by_sys_inst.get((sid, iid))
                if row:
                    w.writerow(
                        [
                            iid,
                            r["family"],
                            dom,
                            r["split"],
                            sid,
                            f"{float(row['faithfulness_mean']):.4f}",
                            f"{float(row.get('code_consistency_mean', 0)):.4f}",
                            f"{float(row.get('vacuity_rate', 0)):.4f}",
                            f"{float(row.get('proof_utility_mean', 0)):.4f}",
                            str(bool(row.get("contradiction_flag", False))).lower(),
                            int(row.get("missing_critical_units", 0)),
                            row.get("failure_mode_label") or "",
                            str(row.get("annotation_origin", "")),
                            str(row.get("source_template_id", "")),
                            "yes" if (iid, sid) in repair_keys else "no",
                        ]
                    )
                else:
                    xs = by_sys_fam.get((sid, r["family"]), [])
                    demo = mean(xs) if xs else float("nan")
                    w.writerow(
                        [
                            iid,
                            r["family"],
                            dom,
                            r["split"],
                            sid,
                            f"{demo:.4f}",
                            "",
                            "",
                            "",
                            "",
                            "",
                            "",
                            "",
                            "",
                            "yes" if (iid, sid) in repair_keys else "no",
                        ]
                    )

    # composite sensitivity
    comp_path = args.out_dir / "composite_sensitivity.csv"
    weights_grid = [
        (0.5, 0.25, 0.25),
        (0.4, 0.3, 0.3),
        (0.34, 0.33, 0.33),
        (0.6, 0.2, 0.2),
    ]
    with comp_path.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(
            ["w_faithfulness", "w_code", "w_proof", "system", "composite_mean", "source_mode"]
        )
        mode = "demo_synthetic" if use_demo else "raw_metrics"
        for wf, wc, wp in weights_grid:
            if abs(wf + wc + wp - 1.0) > 1e-6:
                continue
            for sid in systems:
                if use_demo:
                    xs = metrics_by_system.get(sid, [])
                    comp = [wf * x + wc * x + wp * x for x in xs]
                    w.writerow([wf, wc, wp, sid, f"{mean(comp):.4f}", mode])
                else:
                    comps: list[float] = []
                    for r in mrows:
                        row = by_sys_inst.get((sid, r["instance_id"]))
                        if not row:
                            continue
                        fsc = float(row["faithfulness_mean"])
                        csc = float(row.get("code_consistency_mean", fsc))
                        psc = float(row.get("proof_utility_mean", fsc))
                        comps.append(wf * fsc + wc * csc + wp * psc)
                    if comps:
                        w.writerow([wf, wc, wp, sid, f"{mean(comps):.4f}", mode])
                    else:
                        xs = metrics_by_system.get(sid, [])
                        comp = [wf * x + wc * x + wp * x for x in xs]
                        w.writerow([wf, wc, wp, sid, f"{mean(comp):.4f}", mode])

    ci_path = args.out_dir / "system_summary_with_ci.json"
    if by_sys_inst and not use_demo:
        per_system: dict[str, dict[str, dict[str, object]]] = {}
        for sid in systems:
            per_system[sid] = {}
            for m in PRIMARY_METRICS:
                xs = multi_by_system.get(sid, {}).get(m, [])
                if not xs:
                    per_system[sid][m] = {"mean": None, "ci95": [None, None], "n": 0}
                    continue
                lo, hi = bootstrap_ci95(xs, rng, reps=2000)
                per_system[sid][m] = {"mean": mean(xs), "ci95": [lo, hi], "n": len(xs)}
        per_system_family: dict[str, dict[str, dict[str, dict[str, object]]]] = {}
        for sid in systems:
            per_system_family[sid] = {}
            for fam in fams:
                per_system_family[sid][fam] = {}
                for m in PRIMARY_METRICS:
                    xs = multi_by_sys_fam.get((sid, fam), {}).get(m, [])
                    if not xs:
                        per_system_family[sid][fam][m] = {
                            "mean": None,
                            "ci95": [None, None],
                            "n": 0,
                        }
                        continue
                    lo, hi = bootstrap_ci95(xs, rng, reps=2000)
                    per_system_family[sid][fam][m] = {
                        "mean": mean(xs),
                        "ci95": [lo, hi],
                        "n": len(xs),
                    }
        ci_payload = {
            "schema_version": "system_summary_with_ci_v1",
            "seed": args.seed,
            "bootstrap_reps": 2000,
            "aggregate_scope": "all_rows_joining_manifest_instances_to_raw_metrics_v2",
            "note_instance_vs_aggregate": (
                "Per-instance values live in results/raw_metrics.json; this file summarizes "
                "bootstrap uncertainty for means pooled over instances (not per-instance CIs)."
            ),
            "primary_metrics": list(PRIMARY_METRICS),
            "per_system": per_system,
            "per_system_family": per_system_family,
        }
        ci_path.write_text(
            json.dumps(ci_payload, indent=2, allow_nan=False) + "\n",
            encoding="utf-8",
        )

    if not use_demo and by_sys_inst:
        subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "export_paper_tables.py")],
            cwd=str(ROOT),
            check=True,
        )

    extra_paths = [
        p
        for p in (sys_faith, sys_cons, sys_vac, sys_pu, rel_path, rel_sens_path)
        if p.is_file()
    ]
    extra = (", " + ", ".join(str(p) for p in extra_paths)) if extra_paths else ""
    print(
        f"wrote {sys_path}, {fam_path}, {fail_path}, {inst_path}, {comp_path}"
        + (f", {ci_path}" if ci_path.is_file() else "")
        + extra
        + (
            ", paper_table_*.csv"
            if (args.out_dir / "paper_table_systems.csv").is_file()
            else ""
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
