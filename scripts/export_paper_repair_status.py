#!/usr/bin/env python3
"""Emit repair-status CSVs for manuscript repair-study transparency."""

from __future__ import annotations

import argparse
import csv
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def load_repair_log_index(path: Path) -> dict[str, dict]:
    by_id: dict[str, dict] = {}
    if not path.is_file():
        return by_id
    with path.open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            rec = json.loads(line)
            pid = str(rec.get("packet_id") or "").strip()
            if pid:
                by_id[pid] = rec
    return by_id


def scaffold_imports(scaffold_path: Path) -> str:
    if not scaffold_path.is_file():
        return ""
    try:
        text = scaffold_path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""
    imports: list[str] = []
    for ln in text.splitlines()[:60]:
        s = ln.strip()
        if s.startswith("import "):
            imports.append(s)
    return "; ".join(imports)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--hotspot-selection",
        type=Path,
        default=ROOT / "repairs" / "hotspot_selection.csv",
    )
    ap.add_argument(
        "--repair-log",
        type=Path,
        default=ROOT / "repairs" / "repair_log.jsonl",
    )
    ap.add_argument(
        "--out",
        type=Path,
        default=ROOT / "repairs" / "paper_repair_status.csv",
    )
    ap.add_argument(
        "--out-success-subset",
        type=Path,
        default=ROOT / "repairs" / "paper_repair_success_subset.csv",
    )
    args = ap.parse_args()

    log_idx = load_repair_log_index(args.repair_log)
    args.out.parent.mkdir(parents=True, exist_ok=True)

    fieldnames = [
        "packet_id",
        "selected_for_repair_budget",
        "annotation_origin",
        "pre_repair_failure_type",
        "elaborated",
        "admit_count",
        "axiom_count",
        "proof_mode",
        "imported_modules",
        "outcome_summary",
    ]

    if not args.hotspot_selection.is_file():
        with args.out.open("w", newline="", encoding="utf-8") as f:
            w = csv.DictWriter(f, fieldnames=fieldnames)
            w.writeheader()
        print(f"wrote empty {args.out} (no {args.hotspot_selection})")
        return 0

    with args.hotspot_selection.open(encoding="utf-8", newline="") as fin:
        reader = csv.DictReader(fin)
        rows_out: list[dict[str, str]] = []
        for row in reader:
            pid = (row.get("packet_id") or "").strip()
            pkt_rel = (row.get("packet_json_path") or "").strip()
            pkt_path = (ROOT / pkt_rel) if pkt_rel else Path()
            lc: dict = {}
            ctx: dict = {}
            if pkt_path.is_file():
                try:
                    doc = json.loads(pkt_path.read_text(encoding="utf-8"))
                    raw_lc = doc.get("lean_check")
                    lc = raw_lc if isinstance(raw_lc, dict) else {}
                    raw_ctx = doc.get("context")
                    ctx = raw_ctx if isinstance(raw_ctx, dict) else {}
                except (json.JSONDecodeError, OSError):
                    lc, ctx = {}, {}
            sp = (ctx.get("scaffold_path") or "").strip()
            scaffold = (ROOT / sp) if sp else Path()

            sel = (row.get("selected") or "").strip().lower() == "true"
            log_rec = log_idx.get(pid, {})
            outcome = (
                (row.get("outcome") or "").strip()
                or str(log_rec.get("outcome_summary") or "").strip()
                or str(log_rec.get("attempt_status") or "").strip()
            )

            axioms = lc.get("axiom_dependencies")
            axiom_n = len(axioms) if isinstance(axioms, list) else 0

            rows_out.append(
                {
                    "packet_id": pid,
                    "selected_for_repair_budget": "true" if sel else "false",
                    "annotation_origin": (
                        row.get("annotation_origin") or ""
                    ).strip(),
                    "pre_repair_failure_type": (
                        row.get("candidate_reason") or ""
                    ).strip(),
                    "elaborated": str(lc.get("elaborated", "")).lower()
                    if "elaborated" in lc
                    else "",
                    "admit_count": str(lc.get("admit_count", "")),
                    "axiom_count": str(axiom_n) if pkt_path.is_file() else "",
                    "proof_mode": str(lc.get("proof_mode", "")),
                    "imported_modules": scaffold_imports(scaffold),
                    "outcome_summary": outcome,
                }
            )

    with args.out.open("w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        w.writerows(rows_out)

    succ_fields = [
        "packet_id",
        "system_id",
        "instance_id",
        "selected_for_repair_budget",
        "repair_success",
        "elaborated",
        "admit_count",
        "axiom_count",
        "proof_mode",
        "pre_repair_failure_type",
        "outcome_summary",
    ]
    subset_rows: list[dict[str, str]] = []
    with args.hotspot_selection.open(encoding="utf-8", newline="") as fin:
        for row in csv.DictReader(fin):
            if (row.get("selected") or "").strip().lower() != "true":
                continue
            pid = (row.get("packet_id") or "").strip()
            status_row = next(
                (r for r in rows_out if r["packet_id"] == pid),
                {},
            )
            out = status_row.get("outcome_summary", "").strip().lower()
            repair_success = (
                out.startswith("repaired_") or out == "repair_success"
            )
            subset_rows.append(
                {
                    "packet_id": pid,
                    "system_id": (row.get("system_id") or "").strip(),
                    "instance_id": (row.get("instance_id") or "").strip(),
                    "selected_for_repair_budget": "true",
                    "repair_success": "true" if repair_success else "false",
                    "elaborated": status_row.get("elaborated", ""),
                    "admit_count": status_row.get("admit_count", ""),
                    "axiom_count": status_row.get("axiom_count", ""),
                    "proof_mode": status_row.get("proof_mode", ""),
                    "pre_repair_failure_type": status_row.get(
                        "pre_repair_failure_type", ""
                    ),
                    "outcome_summary": status_row.get("outcome_summary", ""),
                }
            )
    args.out_success_subset.parent.mkdir(parents=True, exist_ok=True)
    with args.out_success_subset.open("w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=succ_fields)
        w.writeheader()
        w.writerows(subset_rows)

    print(f"wrote {args.out} ({len(rows_out)} rows)")
    print(f"wrote {args.out_success_subset} ({len(subset_rows)} rows)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
