#!/usr/bin/env python3
"""
Emit publication-facing CSV tables under results/paper_table_*.csv from the
final metric summaries (four-system study; faithfulness is one column among
several explicit metrics).

When produced via ``compute_results.py --paper``, headline ``paper_table_*.csv``
files summarize **strict independent** rows (``raw_metrics_strict.json`` pipeline).
Canonical manuscript layer names::

  paper_strict_*        — strict independent evidence (copies / merged views).
  paper_expanded_*      — expanded propagated evidence (copied from
                          ``appendix_mapped_evidence/`` after that pass runs).

``paper_table_annotation_evidence.csv`` (written by ``compute_results.py``)
summarizes row-counts by ``annotation_origin`` for both views.
"""

from __future__ import annotations

import argparse
import csv
import json
import os
import shutil
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

METRIC_FILES = (
    ("faithfulness", "system_faithfulness_summary.csv"),
    ("consistency", "system_consistency_summary.csv"),
    ("vacuity", "system_vacuity_summary.csv"),
    ("proof_utility", "system_proof_utility_summary.csv"),
    ("reliability", "system_reliability_summary.csv"),
)


def read_system_metric_table(path: Path) -> dict[str, dict[str, str]]:
    by_sys: dict[str, dict[str, str]] = {}
    if not path.is_file():
        return by_sys
    with path.open(encoding="utf-8", newline="") as f:
        r = csv.DictReader(f)
        for row in r:
            sid = row.get("system", "").strip()
            if sid:
                by_sys[sid] = row
    return by_sys


def read_family_metric_table(path: Path) -> dict[tuple[str, str], dict[str, str]]:
    by_key: dict[tuple[str, str], dict[str, str]] = {}
    if not path.is_file():
        return by_key
    with path.open(encoding="utf-8", newline="") as f:
        r = csv.DictReader(f)
        for row in r:
            fam = row.get("family", "").strip()
            sid = row.get("system", "").strip()
            if fam and sid:
                by_key[(fam, sid)] = row
    return by_key


def write_merged_family_table(src_dir: Path, dest: Path) -> None:
    """Merge per-metric family summaries into one wide CSV (four primary metrics)."""
    ff = read_family_metric_table(src_dir / "family_faithfulness_summary.csv")
    fc = read_family_metric_table(src_dir / "family_consistency_summary.csv")
    fv = read_family_metric_table(src_dir / "family_vacuity_summary.csv")
    fp = read_family_metric_table(src_dir / "family_proof_utility_summary.csv")
    keys = sorted(ff.keys())
    dest.parent.mkdir(parents=True, exist_ok=True)
    with dest.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(
            [
                "family",
                "domain",
                "system",
                "faithfulness_mean",
                "faithfulness_ci95_low",
                "faithfulness_ci95_high",
                "code_consistency_mean",
                "code_consistency_ci95_low",
                "code_consistency_ci95_high",
                "vacuity_mean",
                "vacuity_ci95_low",
                "vacuity_ci95_high",
                "proof_utility_mean",
                "proof_utility_ci95_low",
                "proof_utility_ci95_high",
                "n_instances",
            ]
        )
        for key in keys:
            frow = ff.get(key, {})
            crow = fc.get(key, {})
            vrow = fv.get(key, {})
            prow = fp.get(key, {})
            w.writerow(
                [
                    frow.get("family", ""),
                    frow.get("domain", ""),
                    frow.get("system", ""),
                    frow.get("mean", ""),
                    frow.get("bootstrap_ci95_low", ""),
                    frow.get("bootstrap_ci95_high", ""),
                    crow.get("mean", ""),
                    crow.get("bootstrap_ci95_low", ""),
                    crow.get("bootstrap_ci95_high", ""),
                    vrow.get("mean", ""),
                    vrow.get("bootstrap_ci95_low", ""),
                    vrow.get("bootstrap_ci95_high", ""),
                    prow.get("mean", ""),
                    prow.get("bootstrap_ci95_low", ""),
                    prow.get("bootstrap_ci95_high", ""),
                    frow.get("n", ""),
                ]
            )


def write_paper_strict_metrics_long(results_dir: Path) -> None:
    """Stack system-level metric summaries for manuscript-friendly filtering."""
    out = results_dir / "paper_strict_system_metrics_long.csv"
    rows: list[list[str]] = []
    for metric_key, fname in METRIC_FILES:
        path = results_dir / fname
        if not path.is_file():
            continue
        with path.open(encoding="utf-8", newline="") as f:
            for row in csv.DictReader(f):
                sid = row.get("system", "").strip()
                if not sid:
                    continue
                rows.append(
                    [
                        metric_key,
                        sid,
                        row.get("mean", ""),
                        row.get("bootstrap_ci95_low", ""),
                        row.get("bootstrap_ci95_high", ""),
                        row.get("n", ""),
                    ]
                )
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(
            ["metric", "system", "mean", "bootstrap_ci95_low", "bootstrap_ci95_high", "n"]
        )
        for r in sorted(rows, key=lambda x: (x[0], x[1])):
            w.writerow(r)


def copy_paper_strict_metric_aliases(results_dir: Path) -> None:
    alias_map = (
        ("paper_strict_system_faithfulness_summary.csv", "system_faithfulness_summary.csv"),
        ("paper_strict_system_consistency_summary.csv", "system_consistency_summary.csv"),
        ("paper_strict_system_vacuity_summary.csv", "system_vacuity_summary.csv"),
        ("paper_strict_system_proof_utility_summary.csv", "system_proof_utility_summary.csv"),
        ("paper_strict_system_reliability_summary.csv", "system_reliability_summary.csv"),
    )
    for dst_name, src_name in alias_map:
        src = results_dir / src_name
        dst = results_dir / dst_name
        if src.is_file():
            shutil.copyfile(src, dst)


def write_failure_mode_export(
    src_counts_csv: Path,
    out_csv: Path,
    evidence_view: str,
) -> None:
    """Write manuscript-ready failure table with share columns and view tag."""
    out_csv.parent.mkdir(parents=True, exist_ok=True)
    systems = sorted(
        read_system_metric_table(
            ROOT / "results" / "system_faithfulness_summary.csv"
        ).keys()
    )

    # Count family-level failures from the pipeline count table first.
    counts_fam: dict[tuple[str, str, str], int] = defaultdict(int)
    if src_counts_csv.is_file():
        with src_counts_csv.open(encoding="utf-8", newline="") as f:
            for row in csv.DictReader(f):
                sid = (row.get("system") or "").strip()
                fam = (row.get("family") or "").strip() or "global"
                mode = (row.get("failure_mode") or "").strip()
                c = int((row.get("count") or "0").strip() or 0)
                if sid and mode:
                    counts_fam[(sid, fam, mode)] += c

    # Add operational signals from hotspot candidate_reason so strict view does
    # not collapse to a placeholder when ontology-tagged failures are sparse.
    hs = ROOT / "repairs" / "hotspot_selection.csv"
    if hs.is_file():
        with hs.open(encoding="utf-8", newline="") as f:
            for row in csv.DictReader(f):
                origin = (row.get("annotation_origin") or "").strip()
                if evidence_view == "strict_independent":
                    if origin not in {"direct_human", "direct_adjudicated"}:
                        continue
                reason = (row.get("candidate_reason") or "").strip()
                if not reason or reason == "routine_eval_obligation_hygiene":
                    continue
                iid = (row.get("instance_id") or "").strip()
                fam = "_".join(iid.split("_")[:-1]) if "_" in iid else "global"
                sid = (row.get("system_id") or "").strip()
                if not sid:
                    continue
                for tok in reason.split(";"):
                    mode = tok.strip()
                    if (
                        evidence_view == "strict_independent"
                        and mode == "missing_critical_semantic_unit"
                    ):
                        continue
                    if mode:
                        counts_fam[(sid, fam, mode)] += 1

    # Strict independent: missing-critical row counts come from ``raw_metrics_strict``
    # (pack-derived ``missing_critical_units``), not hotspot proxy tokens.
    if evidence_view == "strict_independent":
        strip = [k for k in counts_fam if k[2] == "missing_critical_semantic_unit"]
        for k in strip:
            del counts_fam[k]
        rm_path = ROOT / "results" / "raw_metrics_strict.json"
        if rm_path.is_file():
            payload = json.loads(rm_path.read_text(encoding="utf-8"))
            for row in payload.get("rows") or []:
                origin = (row.get("annotation_origin") or "").strip()
                if origin not in {"direct_human", "direct_adjudicated"}:
                    continue
                if int(row.get("missing_critical_units", 0) or 0) <= 0:
                    continue
                sid = (row.get("system") or "").strip()
                iid = (row.get("instance_id") or "").strip()
                if not sid or not iid:
                    continue
                fam = "_".join(iid.split("_")[:-1]) if "_" in iid else "global"
                counts_fam[(sid, fam, "missing_critical_semantic_unit")] += 1

    # Build denominator stats from raw-metrics rows in the matching evidence view.
    raw_name = (
        "raw_metrics_strict.json"
        if evidence_view == "strict_independent"
        else "raw_metrics.json"
    )
    raw_path = ROOT / "results" / raw_name
    rows_for_system: dict[str, int] = defaultdict(int)
    miss_crit_for_system: dict[str, int] = defaultdict(int)
    crit_by_instance: dict[str, int] = {}
    manifest = ROOT / "benchmark" / "manifest.jsonl"
    if manifest.is_file():
        with manifest.open(encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                row = json.loads(line)
                iid = str(row.get("instance_id", "")).strip()
                if not iid:
                    continue
                crit_by_instance[iid] = int(row.get("critical_unit_count", 0) or 0)
    crit_opps_for_system: dict[str, int] = defaultdict(int)
    if raw_path.is_file():
        payload = json.loads(raw_path.read_text(encoding="utf-8"))
        for row in payload.get("rows") or []:
            sid = str(row.get("system", "")).strip()
            iid = str(row.get("instance_id", "")).strip()
            if not sid or not iid:
                continue
            rows_for_system[sid] += 1
            miss_crit_for_system[sid] += int(row.get("missing_critical_units", 0) or 0)
            crit_opps_for_system[sid] += crit_by_instance.get(iid, 0)

    # Build complete system x mode global table with explicit zeros.
    mode_set = sorted({k[2] for k in counts_fam} or {"no_failures_recorded"})
    if not systems:
        systems = sorted({k[0] for k in counts_fam} or ["global"])
    counts_global: dict[tuple[str, str], int] = defaultdict(int)
    for (sid, _fam, mode), c in counts_fam.items():
        counts_global[(sid, mode)] += c
    rows: list[dict[str, str]] = []
    for sid in systems:
        for mode in mode_set:
            rows.append(
                {
                    "system": sid,
                    "family": "global",
                    "failure_mode": mode,
                    "count": str(counts_global.get((sid, mode), 0)),
                }
            )

    total = sum(int(r.get("count") or "0") for r in rows)
    by_system: dict[str, int] = defaultdict(int)
    for r in rows:
        by_system[r["system"]] += int(r["count"])

    with out_csv.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(
            [
                "evidence_view",
                "system",
                "family",
                "failure_mode",
                "count",
                "share_within_system",
                "share_global",
                "n_rows_for_system",
                "critical_unit_opportunities",
                "failure_events_per_row",
                "missing_critical_units_per_critical_unit_opportunity",
            ]
        )
        for r in sorted(
            rows,
            key=lambda x: (
                x.get("system", ""),
                x.get("family", ""),
                x.get("failure_mode", ""),
            ),
        ):
            c = int(r.get("count") or "0")
            sid = r.get("system", "")
            sys_total = by_system.get(sid, 0)
            share_sys = (c / sys_total) if sys_total else 0.0
            share_global = (c / total) if total else 0.0
            n_rows = rows_for_system.get(sid, 0)
            crit_opps = crit_opps_for_system.get(sid, 0)
            events_per_row = (c / n_rows) if n_rows else 0.0
            miss_rate = (c / crit_opps) if crit_opps else 0.0
            w.writerow(
                [
                    evidence_view,
                    sid,
                    r.get("family", "") or "global",
                    r.get("failure_mode", ""),
                    c,
                    f"{share_sys:.6f}",
                    f"{share_global:.6f}",
                    n_rows,
                    crit_opps,
                    f"{events_per_row:.6f}",
                    f"{miss_rate:.6f}",
                ]
            )


def finalize_strict_paper_layer(results_dir: Path) -> None:
    """Canonical strict-independent filenames under ``results/`` (not for appendix tmp)."""
    if os.environ.get("CTA_COMPUTE_APPENDIX") == "1":
        return
    pt_sys = results_dir / "paper_table_systems.csv"
    if not pt_sys.is_file():
        return
    shutil.copyfile(pt_sys, results_dir / "paper_strict_system_summary.csv")

    write_merged_family_table(results_dir, results_dir / "paper_strict_family_summary.csv")
    fam_rel = results_dir / "family_reliability_summary.csv"
    if fam_rel.is_file():
        shutil.copyfile(
            fam_rel,
            results_dir / "paper_strict_family_reliability_summary.csv",
        )

    write_failure_mode_export(
        results_dir / "failure_mode_counts.csv",
        results_dir / "paper_strict_failure_modes.csv",
        "strict_independent",
    )

    inst = results_dir / "instance_level.csv"
    if inst.is_file():
        shutil.copyfile(inst, results_dir / "paper_strict_instance_level.csv")

    copy_paper_strict_metric_aliases(results_dir)
    write_paper_strict_metrics_long(results_dir)


def finalize_expanded_paper_layer(results_root: Path) -> None:
    """Promote appendix expanded mapped CSVs to explicit ``paper_expanded_*`` names."""
    apx = results_root / "appendix_mapped_evidence"
    apx_sys = apx / "paper_table_systems.csv"
    if not apx_sys.is_file():
        return
    shutil.copyfile(apx_sys, results_root / "paper_expanded_system_summary.csv")

    write_merged_family_table(apx, results_root / "paper_expanded_family_summary.csv")
    apx_fam_rel = apx / "family_reliability_summary.csv"
    if apx_fam_rel.is_file():
        shutil.copyfile(
            apx_fam_rel,
            results_root / "paper_expanded_family_reliability_summary.csv",
        )

    write_failure_mode_export(
        apx / "failure_mode_counts.csv",
        results_root / "paper_expanded_failure_modes.csv",
        "expanded_propagated",
    )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--results-dir", type=Path, default=ROOT / "results")
    args = ap.parse_args()
    d = args.results_dir

    faith = read_system_metric_table(d / "system_faithfulness_summary.csv")
    cons = read_system_metric_table(d / "system_consistency_summary.csv")
    vac = read_system_metric_table(d / "system_vacuity_summary.csv")
    pu = read_system_metric_table(d / "system_proof_utility_summary.csv")
    rel = read_system_metric_table(d / "system_reliability_summary.csv")

    systems = sorted(faith.keys()) if faith else []
    out_sys = d / "paper_table_systems.csv"
    with out_sys.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(
            [
                "system",
                "faithfulness_mean",
                "faithfulness_ci95_low",
                "faithfulness_ci95_high",
                "code_consistency_mean",
                "code_consistency_ci95_low",
                "code_consistency_ci95_high",
                "vacuity_mean",
                "vacuity_ci95_low",
                "vacuity_ci95_high",
                "proof_utility_mean",
                "proof_utility_ci95_low",
                "proof_utility_ci95_high",
                "reliability_mean",
                "reliability_ci95_low",
                "reliability_ci95_high",
            ]
        )
        for sid in systems:
            frow = faith.get(sid, {})
            crow = cons.get(sid, {})
            vrow = vac.get(sid, {})
            prow = pu.get(sid, {})
            rrow = rel.get(sid, {})
            w.writerow(
                [
                    sid,
                    frow.get("mean", ""),
                    frow.get("bootstrap_ci95_low", ""),
                    frow.get("bootstrap_ci95_high", ""),
                    crow.get("mean", ""),
                    crow.get("bootstrap_ci95_low", ""),
                    crow.get("bootstrap_ci95_high", ""),
                    vrow.get("mean", ""),
                    vrow.get("bootstrap_ci95_low", ""),
                    vrow.get("bootstrap_ci95_high", ""),
                    prow.get("mean", ""),
                    prow.get("bootstrap_ci95_low", ""),
                    prow.get("bootstrap_ci95_high", ""),
                    rrow.get("mean", ""),
                    rrow.get("bootstrap_ci95_low", ""),
                    rrow.get("bootstrap_ci95_high", ""),
                ]
            )

    fam_faith = d / "family_faithfulness_summary.csv"
    out_fam = d / "paper_table_families.csv"
    with out_fam.open("w", newline="", encoding="utf-8") as fout:
        w = csv.writer(fout)
        w.writerow(
            [
                "family",
                "domain",
                "system",
                "faithfulness_mean",
                "faithfulness_ci95_low",
                "faithfulness_ci95_high",
                "n_instances",
            ]
        )
        if fam_faith.is_file():
            with fam_faith.open(encoding="utf-8", newline="") as fin:
                r = csv.DictReader(fin)
                for row in r:
                    w.writerow(
                        [
                            row.get("family", ""),
                            row.get("domain", ""),
                            row.get("system", ""),
                            row.get("mean", ""),
                            row.get("bootstrap_ci95_low", ""),
                            row.get("bootstrap_ci95_high", ""),
                            row.get("n", ""),
                        ]
                    )

    fail_src = d / "failure_mode_counts.csv"
    out_fail = d / "paper_table_failure_modes.csv"
    if fail_src.is_file():
        out_fail.write_text(fail_src.read_text(encoding="utf-8"), encoding="utf-8")

    rep_src = d / "repair_impact_summary.json"
    out_rep = d / "paper_table_repairs.csv"
    if rep_src.is_file():
        payload = json.loads(rep_src.read_text(encoding="utf-8"))
        rows = payload.get("per_system") or {}
        with out_rep.open("w", newline="", encoding="utf-8") as f:
            w = csv.writer(f)
            w.writerow(
                [
                    "system",
                    "repair_n",
                    "observed_mean_faithfulness",
                    "counterfactual_mean_faithfulness",
                    "delta_obs_minus_counterfactual",
                    "non_repair_eval_n",
                ]
            )
            for sid, block in sorted(rows.items()):
                if not isinstance(block, dict):
                    continue
                w.writerow(
                    [
                        sid,
                        block.get("repair_n", ""),
                        block.get("observed_mean_faithfulness", ""),
                        block.get("counterfactual_mean_faithfulness", ""),
                        block.get("delta_obs_minus_counterfactual", ""),
                        block.get("non_repair_eval_n", ""),
                    ]
                )

    finalize_strict_paper_layer(d)

    print(f"wrote {out_sys}, {out_fam}, {out_fail}, {out_rep}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
