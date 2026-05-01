#!/usr/bin/env python3
"""Enforce strict-vs-expanded evidence discipline for manuscript-facing numbers.

Validates:
  - results/raw_metrics_strict.json (row coverage, no mapped_from_canonical)
  - strict human-agreement artifact paths (packet map, raters, agreement outs)
  - results/paper_table_annotation_evidence.csv
  - results/paper_annotation_origin_counts.csv (if present)
  - results/appendix_mapped_evidence/ (directory + >= one CSV)
  - annotation/human_pass_v3/agreement_report_human_strict_all.json
  - results/table1_*.csv for instance and critical-unit totals
  - docs/paper/paper_claim_sources.yaml (author-maintained headline integers)

Optional: scan LaTeX sources for obvious expanded-only path references in
non-appendix files (heuristic; off by default).
"""
from __future__ import annotations

import argparse
import csv
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

STRICT_FORBIDDEN_ORIGIN = "mapped_from_canonical"

MANDATORY_HEADLINE_FILES = [
    "results/raw_metrics_strict.json",
    "results/paper_strict_system_summary.csv",
    "results/paper_strict_family_summary.csv",
    "results/paper_strict_failure_modes.csv",
    "results/paper_strict_instance_level.csv",
    "results/paper_strict_system_metrics_long.csv",
    "results/paper_table_annotation_evidence.csv",
    "results/paper_table_agreement_evidence.csv",
    "results/paper_annotation_origin_counts.csv",
]

STRICT_HUMAN_AGREEMENT_FILES = [
    "annotation/human_pass_v3/human_strict_packet_ids.csv",
    "annotation/rater_a_strict_all.csv",
    "annotation/human_pass_v3/rater_b_human_strict_all.csv",
    "annotation/human_pass_v3/agreement_report_human_strict_all.json",
    "annotation/human_pass_v3/agreement_report_human_strict_all.md",
    "annotation/human_pass_v3/disagreement_log_strict_all.csv",
]

APPENDIX_ONLY_FILES = [
    "results/raw_metrics_expanded.json",
    "results/paper_expanded_system_summary.csv",
    "results/paper_expanded_family_summary.csv",
    "results/paper_expanded_failure_modes.csv",
]

# Mandatory directory for appendix-only manuscript tables (robustness layer).
APPENDIX_MAPPED_DIR = ROOT / "results" / "appendix_mapped_evidence"


def load_flat_yaml_ints(path: Path) -> dict[str, int]:
    out: dict[str, int] = {}
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line or line.startswith("#") or ":" not in line:
            continue
        key, val = line.split(":", 1)
        key, val = key.strip(), val.strip()
        if not val:
            continue
        if val.startswith('"') and val.endswith('"'):
            val = val[1:-1]
        try:
            out[key] = int(val)
        except ValueError:
            continue
    return out


def jload(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def read_annotation_evidence(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as f:
        return [dict(r) for r in csv.DictReader(f)]


def sum_critical_units(path: Path) -> int:
    with path.open(encoding="utf-8", newline="") as f:
        rows = list(csv.DictReader(f))
    return sum(int(r["critical_units_sum"]) for r in rows)


def table1_total_instances(path: Path) -> int:
    with path.open(encoding="utf-8", newline="") as f:
        for row in csv.DictReader(f):
            if row.get("metric") == "total_instances":
                return int(row["value"])
    raise RuntimeError(f"total_instances not found in {path}")


def family_count(path: Path) -> int:
    with path.open(encoding="utf-8", newline="") as f:
        return sum(1 for row in csv.DictReader(f) if row.get("metric", "").startswith("family_count:"))


def err(msg: str) -> None:
    print(f"check_paper_claim_sources: ERROR: {msg}", file=sys.stderr)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--claims-yaml",
        type=Path,
        default=ROOT / "docs" / "paper" / "paper_claim_sources.yaml",
    )
    ap.add_argument(
        "--scan-tex",
        action="store_true",
        help="Warn if .tex files reference raw_metrics_expanded (heuristic).",
    )
    args = ap.parse_args()

    problems = 0

    for rel in MANDATORY_HEADLINE_FILES:
        p = ROOT / rel
        if not p.is_file():
            err(f"missing mandatory headline file: {rel}")
            problems += 1
    if problems:
        return 1

    for rel in STRICT_HUMAN_AGREEMENT_FILES:
        p = ROOT / rel
        if not p.is_file():
            err(f"missing strict human-agreement artifact: {rel}")
            problems += 1
    if problems:
        return 1

    if not APPENDIX_MAPPED_DIR.is_dir():
        err(
            "missing mandatory appendix directory: "
            f"{APPENDIX_MAPPED_DIR.relative_to(ROOT)}"
        )
        problems += 1
    elif not any(APPENDIX_MAPPED_DIR.glob("*.csv")):
        err(
            "results/appendix_mapped_evidence must contain at least one .csv "
            "(appendix manuscript exports)"
        )
        problems += 1
    if problems:
        return 1

    for rel in APPENDIX_ONLY_FILES:
        p = ROOT / rel
        if not p.is_file():
            err(f"missing appendix evidence file: {rel}")
            problems += 1
    if problems:
        return 1

    raw_strict = jload(ROOT / "results" / "raw_metrics_strict.json")
    strict_rows: list[dict] = raw_strict.get("rows") or []
    n_strict = len(strict_rows)
    inst_u = {str(r.get("instance_id", "")) for r in strict_rows}
    inst_u.discard("")
    bad_origin = [r for r in strict_rows if (r.get("annotation_origin") or "") == STRICT_FORBIDDEN_ORIGIN]
    if bad_origin:
        err(
            f"raw_metrics_strict.json must not contain annotation_origin={STRICT_FORBIDDEN_ORIGIN!r} "
            f"({len(bad_origin)} rows)"
        )
        problems += 1

    ev_rows = read_annotation_evidence(ROOT / "results" / "paper_table_annotation_evidence.csv")
    by_view = {r["metrics_view"]: r for r in ev_rows}
    st = by_view.get("strict_independent")
    ex = by_view.get("expanded_mapped")
    if not st or not ex:
        err("paper_table_annotation_evidence.csv must have strict_independent and expanded_mapped rows")
        return 1

    def intcell(r: dict[str, str], k: str) -> int:
        return int((r.get(k) or "0").strip())

    if intcell(st, "n_eval_rows") != n_strict:
        err(
            f"strict row count mismatch: raw_metrics_strict has {n_strict}, "
            f"paper_table_annotation_evidence strict_independent n_eval_rows={intcell(st, 'n_eval_rows')}"
        )
        problems += 1
    if intcell(st, "n_unique_instance_ids") != len(inst_u):
        err(
            f"strict unique instance mismatch: json unique={len(inst_u)}, "
            f"evidence csv n_unique_instance_ids={intcell(st, 'n_unique_instance_ids')}"
        )
        problems += 1
    if intcell(st, "n_mapped_from_canonical") != 0:
        err("strict_independent n_mapped_from_canonical must be 0 in paper_table_annotation_evidence.csv")
        problems += 1
    if intcell(ex, "n_mapped_from_canonical") < 1:
        err("expanded_mapped n_mapped_from_canonical must be >0 for appendix discipline")
        problems += 1

    origin_path = ROOT / "results" / "paper_annotation_origin_counts.csv"
    o_rows = {r["category"]: int(r["count"]) for r in read_annotation_evidence(origin_path)}
    if o_rows.get("strict_independent_mapped_from_canonical", -1) != 0:
        err("paper_annotation_origin_counts: strict_independent_mapped_from_canonical must be 0")
        problems += 1
    if o_rows.get("strict_independent_n_eval_rows", 0) != n_strict:
        err("paper_annotation_origin_counts strict_independent_n_eval_rows must match raw_metrics_strict row count")
        problems += 1

    agree_path = ROOT / "annotation" / "human_pass_v3" / "agreement_report_human_strict_all.json"
    agree = jload(agree_path)
    if agree.get("n_rows") != n_strict:
        err(
            f"agreement_report n_rows {agree.get('n_rows')} != raw_metrics_strict rows {n_strict}"
        )
        problems += 1
    if agree.get("n_mapped_from_canonical") != 0:
        err("agreement strict layer n_mapped_from_canonical must be 0")
        problems += 1
    if agree.get("n_unique_instance_ids") != len(inst_u):
        err("agreement n_unique_instance_ids must match strict unique instances")
        problems += 1

    t1_inst = table1_total_instances(ROOT / "results" / "table1_benchmark_overview.csv")
    t1_fam = family_count(ROOT / "results" / "table1_benchmark_overview.csv")
    crit_sum = sum_critical_units(ROOT / "results" / "table1_family_semantic_load.csv")
    if t1_inst != len(inst_u):
        err(f"table1 total_instances {t1_inst} != unique strict instances {len(inst_u)}")
        problems += 1

    yaml_claims = load_flat_yaml_ints(args.claims_yaml)
    expected_checks = {
        "strict_eval_rows": n_strict,
        "strict_unique_instances": len(inst_u),
        "strict_mapped_from_canonical": 0,
        "strict_direct_first_pass": intcell(st, "n_direct_first_pass"),
        "expanded_eval_rows": intcell(ex, "n_eval_rows"),
        "expanded_unique_instances": intcell(ex, "n_unique_instance_ids"),
        "expanded_direct_first_pass": intcell(ex, "n_direct_first_pass"),
        "expanded_mapped_from_canonical": intcell(ex, "n_mapped_from_canonical"),
        "benchmark_instances": t1_inst,
        "algorithm_families": t1_fam,
        "critical_semantic_units_total": crit_sum,
        "human_agreement_strict_rows": agree["n_rows"],
        "human_agreement_unique_instances": agree["n_unique_instance_ids"],
        "human_agreement_mapped_from_canonical": agree["n_mapped_from_canonical"],
    }
    for key, actual in expected_checks.items():
        if key not in yaml_claims:
            err(f"paper_claim_sources.yaml missing key: {key}")
            problems += 1
            continue
        if yaml_claims[key] != actual:
            err(
                f"paper_claim_sources.yaml {key}: yaml says {yaml_claims[key]}, "
                f"artifacts compute {actual}"
            )
            problems += 1

    if args.scan_tex:
        tex_files = list(ROOT.glob("**/*.tex"))
        # Skip common vendor paths if ever added
        tex_files = [p for p in tex_files if "build" not in p.parts]
        for tf in tex_files:
            txt = tf.read_text(encoding="utf-8", errors="replace")
            if "raw_metrics_expanded" in txt or "paper_expanded_" in txt:
                print(
                    f"check_paper_claim_sources: WARN: {tf.relative_to(ROOT)} references expanded metrics; "
                    "ensure only appendix.",
                    file=sys.stderr,
                )

    if problems:
        err(f"{problems} issue(s); fix artifacts or docs/paper/paper_claim_sources.yaml")
        return 1

    print(
        "check_paper_claim_sources: OK - strict headline discipline, yaml manifest, "
        "and mandatory paths verified."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
