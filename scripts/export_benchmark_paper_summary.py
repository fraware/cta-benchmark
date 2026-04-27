#!/usr/bin/env python3
"""Emit a stable JSON summary of benchmark v0.3 scale for manuscript tables."""

from __future__ import annotations

import csv
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
V3 = ROOT / "benchmark" / "v0.3"
OUT = V3 / "benchmark_paper_summary.json"
RAW = ROOT / "results" / "raw_metrics.json"

DEFAULT_FOUR = [
    "text_only_v1",
    "code_only_v1",
    "naive_concat_v1",
    "full_method_v1",
]


def systems_profiled_default() -> list[str]:
    if not RAW.is_file():
        return list(DEFAULT_FOUR)
    rows = json.loads(RAW.read_text(encoding="utf-8")).get("rows") or []
    if not rows or "system" not in rows[0]:
        return list(DEFAULT_FOUR)
    return sorted({str(r["system"]) for r in rows})


def main() -> int:
    rows: list[dict] = []
    with (ROOT / "benchmark" / "manifest.jsonl").open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            rows.append(json.loads(line))

    v3 = [r for r in rows if "v0.3" in (r.get("source_provenance") or "")]
    if not v3:
        v3 = rows

    by_split = Counter(r.get("split", "unknown") for r in v3)
    by_family = Counter(r.get("family", "unknown") for r in v3)
    by_diff = Counter(r.get("difficulty", "unknown") for r in v3)

    crit_by_family: dict[str, int] = {}
    for r in v3:
        fam = r.get("family", "unknown")
        n = int(r.get("critical_unit_count", 0) or 0)
        crit_by_family[fam] = crit_by_family.get(fam, 0) + n

    systems = systems_profiled_default()
    n_sys = len(systems)
    strict_n = 0
    strict_path = ROOT / "results" / "raw_metrics_strict.json"
    expanded_n = 0
    if RAW.is_file():
        raw_payload = json.loads(RAW.read_text(encoding="utf-8"))
        expanded_n = len(raw_payload.get("rows") or [])
    if strict_path.is_file():
        strict_payload = json.loads(strict_path.read_text(encoding="utf-8"))
        strict_n = len(strict_payload.get("rows") or [])
    elif RAW.is_file():
        raw_rows = json.loads(RAW.read_text(encoding="utf-8")).get("rows") or []
        strict_n = sum(
            1
            for r in raw_rows
            if str(r.get("annotation_origin", ""))
            in ("direct_human", "direct_adjudicated")
        )
    else:
        strict_n = 0
    headline_inst = strict_n
    inst_csv = ROOT / "results" / "instance_level.csv"
    if inst_csv.is_file():
        with inst_csv.open(encoding="utf-8", newline="") as f:
            headline_inst = max(0, sum(1 for _ in f) - 1)

    ag_audit = ROOT / "annotation" / "agreement_packet_ids.csv"
    ag_audit_rows = 0
    ag_strict_packets = 0
    ag_all_mapped = False
    if ag_audit.is_file():
        with ag_audit.open(encoding="utf-8", newline="") as f:
            rdr = csv.DictReader(f)
            ag_rows = list(rdr)
        ag_audit_rows = len(ag_rows)
        if ag_rows:
            origins = [str(x.get("annotation_origin", "")).strip() for x in ag_rows]
            ag_strict_packets = sum(
                1 for o in origins if o in ("direct_human", "direct_adjudicated")
            )
            ag_all_mapped = all(o == "mapped_from_canonical" for o in origins)

    payload = {
        "schema_version": "benchmark_paper_summary_v1",
        "benchmark_version": "v0.3",
        "paper_system_set": "four_baselines",
        "paper_headline_policy": "four_system_primary_study",
        "paper_alternate_scope_note": (
            "Optional three-headline-system scope: treat text_only_v1 as "
            "calibration-only and exclude it from primary tables while keeping "
            "it in appendix robustness exports."
        ),
        "paper_systems_ordered": list(DEFAULT_FOUR),
        "total_instances": len(v3),
        "systems_profiled": systems,
        "systems_profiled_count": n_sys,
        "expected_instance_level_rows_expanded_grid": len(v3) * n_sys,
        "expected_instance_level_rows_headline_strict_sparse": headline_inst,
        "expected_instance_level_rows": headline_inst,
        "expected_raw_metrics_rows": (
            expanded_n if expanded_n else len(v3) * n_sys
        ),
        "expected_raw_metrics_strict_rows": strict_n,
        "family_counts": dict(sorted(by_family.items())),
        "split_counts": dict(sorted(by_split.items())),
        "difficulty_counts": dict(sorted(by_diff.items())),
        "critical_units_sum_by_family": dict(sorted(crit_by_family.items())),
        "critical_units_total": sum(crit_by_family.values()),
        "eval_instance_count": len(
            json.loads(
                (V3 / "splits" / "eval.json").read_text(encoding="utf-8")
            )["instance_ids"]
        ),
        "source_manifest": "benchmark/manifest.jsonl",
    }
    if ag_audit_rows:
        payload["expected_agreement_packet_audit_rows"] = ag_audit_rows
        payload["agreement_audit_strict_independent_packet_count"] = ag_strict_packets
        payload["agreement_audit_all_packets_mapped_from_canonical"] = ag_all_mapped
        payload["agreement_audit_design_note"] = (
            "Audit rows are eval-split (instance, system) pairs; canonical "
            "template packets yield mapped_from_canonical unless instance_id "
            "equals the template stem."
        )
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {OUT}")
    write_paper_system_set_md(OUT, ROOT / "results" / "paper_system_set.md", payload)
    return 0


def write_paper_system_set_md(
    json_path: Path,
    md_path: Path,
    data: dict,
) -> None:
    """One-line-of-record for the manuscript: four vs three primary systems."""
    pss = str(data.get("paper_system_set", "four_baselines"))
    ph = str(data.get("paper_headline_policy", ""))
    systems = [str(s) for s in (data.get("paper_systems_ordered") or [])]
    note = str(data.get("paper_alternate_scope_note", ""))
    lines = [
        "# Paper system set (machine-generated)",
        "",
    ]
    if pss == "three_headline" or "three_system" in ph:
        lines.append(
            "Main paper evaluates **three** systems; `text_only_v1` is "
            "appendix-only calibration (not a primary headline comparator). "
            f"Headline systems: {', '.join(s for s in systems if s != 'text_only_v1') or 'code_only_v1, naive_concat_v1, full_method_v1'}."
        )
    else:
        pretty = ", ".join(systems) if systems else (
            "text_only_v1, code_only_v1, naive_concat_v1, full_method_v1"
        )
        lines.append(
            f"Main paper evaluates **four** systems: {pretty}."
        )
    if note:
        lines.extend(
            [
                "",
                f"Scope note (from `benchmark_paper_summary.json`): {note}",
            ]
        )
    try:
        rel = json_path.resolve().relative_to(ROOT.resolve())
    except ValueError:
        rel = json_path
    lines.extend(
        [
            "",
            f"Source: `{rel.as_posix()}` "
            f"(`paper_system_set={pss!r}`, `paper_headline_policy={ph!r}`).",
        ]
    )
    md_path.parent.mkdir(parents=True, exist_ok=True)
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {md_path}")


if __name__ == "__main__":
    raise SystemExit(main())
