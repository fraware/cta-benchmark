#!/usr/bin/env python3
"""
Emit publication-facing CSV tables under results/paper_table_*.csv from the
final metric summaries (four-system study; faithfulness is one column among
several explicit metrics).

When produced via ``compute_results.py --paper``, headline ``paper_table_*.csv``
files summarize **strict independent** rows (``raw_metrics_strict.json`` pipeline).
Expanded mapped tables live under ``results/appendix_mapped_evidence/``.
``paper_table_annotation_evidence.csv`` (written by ``compute_results.py``) is the
manuscript-ready row-count table by ``annotation_origin`` for both views.
"""

from __future__ import annotations

import argparse
import csv
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


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

    print(f"wrote {out_sys}, {out_fam}, {out_fail}, {out_rep}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
