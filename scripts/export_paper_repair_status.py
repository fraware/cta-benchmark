#!/usr/bin/env python3
"""Emit repair-status CSVs for manuscript repair-study transparency."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import subprocess
from datetime import UTC, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
M1_ALLOWLIST: set[tuple[str, str]] = {
    ("full_method_v1", "graph_dijkstra_001"),
    ("full_method_v1", "graph_dijkstra_002"),
    ("full_method_v1", "graph_bfs_shortest_path_002"),
    ("full_method_v1", "greedy_coin_change_canonical_002"),
    ("full_method_v1", "trees_lowest_common_ancestor_001"),
    ("full_method_v1", "trees_lowest_common_ancestor_002"),
    ("full_method_v1", "greedy_interval_scheduling_001"),
    ("full_method_v1", "greedy_interval_scheduling_002"),
    ("full_method_v1", "sorting_insertion_sort_001"),
    ("full_method_v1", "sorting_insertion_sort_002"),
    ("full_method_v1", "sorting_merge_sort_001"),
    ("full_method_v1", "sorting_merge_sort_002"),
    ("full_method_v1", "trees_bst_insert_001"),
    ("full_method_v1", "trees_bst_insert_002"),
    ("full_method_v1", "dp_knapsack_01_001"),
    ("full_method_v1", "dp_knapsack_01_002"),
    ("code_only_v1", "graph_dijkstra_001"),
    ("code_only_v1", "graph_dijkstra_002"),
    ("code_only_v1", "dp_knapsack_01_001"),
    ("code_only_v1", "dp_knapsack_01_002"),
    ("naive_concat_v1", "graph_dijkstra_001"),
    ("naive_concat_v1", "graph_dijkstra_002"),
    ("naive_concat_v1", "dp_knapsack_01_001"),
    ("naive_concat_v1", "dp_knapsack_01_002"),
    ("text_only_v1", "dp_knapsack_01_001"),
    ("text_only_v1", "dp_knapsack_01_002"),
    ("text_only_v1", "graph_dijkstra_001"),
    ("text_only_v1", "graph_dijkstra_002"),
}


def tool_versions() -> tuple[str, str]:
    lean_ver = ""
    lake_ver = ""
    lean_toolchain = ROOT / "lean" / "lean-toolchain"
    if lean_toolchain.is_file():
        lean_ver = lean_toolchain.read_text(encoding="utf-8").strip()
    try:
        out = subprocess.check_output(
            ["lake", "--version"],
            text=True,
            encoding="utf-8",
            cwd=ROOT / "lean",
            stderr=subprocess.STDOUT,
        )
        lake_ver = out.strip().splitlines()[0] if out.strip() else ""
    except (OSError, subprocess.CalledProcessError):
        lake_ver = ""
    return lean_ver, lake_ver


def sha256_file(path: Path) -> str:
    if not path.is_file():
        return ""
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def file_mtime_iso(path: Path) -> str:
    if not path.is_file():
        return ""
    return datetime.fromtimestamp(
        path.stat().st_mtime, tz=UTC
    ).isoformat()


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
    ap.add_argument(
        "--out-proof-subset",
        type=Path,
        default=ROOT / "repairs" / "paper_repair_proof_subset.csv",
    )
    ap.add_argument(
        "--out-proof-facing-subset",
        type=Path,
        default=ROOT / "repairs" / "paper_proof_facing_subset.csv",
    )
    args = ap.parse_args()
    lean_ver, lake_ver = tool_versions()

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

    hotspot_rows: list[dict[str, str]] = []
    with args.hotspot_selection.open(encoding="utf-8", newline="") as fin:
        reader = csv.DictReader(fin)
        rows_out: list[dict[str, str]] = []
        for row in reader:
            hotspot_rows.append({k: (v or "").strip() for k, v in row.items()})
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
    for row in hotspot_rows:
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

    # Proof-facing subset: only selected rows that elaborated successfully.
    proof_rows = [r for r in subset_rows if r.get("elaborated") == "true"]
    args.out_proof_subset.parent.mkdir(parents=True, exist_ok=True)
    with args.out_proof_subset.open("w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=succ_fields)
        w.writeheader()
        w.writerows(proof_rows)

    # Paper-facing subset across all hotspot rows with elaborated Lean proofs.
    proof_facing_fields = [
        "packet_id",
        "system_id",
        "instance_id",
        "elaborated",
        "admit_count",
        "axiom_count",
        "proof_mode",
        "imported_modules",
        "outcome_summary",
        "lean_version",
        "lake_version",
        "diagnostics_path",
        "diagnostics_sha256",
        "checked_command",
        "check_timestamp",
        "m1_allowlisted",
    ]
    imports_by_packet = {
        r["packet_id"]: r.get("imported_modules", "") for r in rows_out
    }
    proof_facing_rows: list[dict[str, str]] = []
    for row in hotspot_rows:
        pid = (row.get("packet_id") or "").strip()
        status_row = next((r for r in rows_out if r["packet_id"] == pid), {})
        if status_row.get("elaborated") != "true":
            continue
        diagnostics_rel = (row.get("lean_diagnostics_path") or "").strip()
        diagnostics_path = (
            (ROOT / diagnostics_rel) if diagnostics_rel else Path()
        )
        checked_command = ""
        if diagnostics_path.is_file():
            try:
                diag = json.loads(
                    diagnostics_path.read_text(encoding="utf-8")
                )
                cmd = diag.get("command")
                if isinstance(cmd, list):
                    checked_command = " ".join(str(x) for x in cmd)
            except (json.JSONDecodeError, OSError):
                checked_command = ""
        sid = row.get("system_id", "")
        iid = row.get("instance_id", "")
        template_id = (row.get("source_template_id") or iid).strip()
        allowlisted = (sid, template_id) in M1_ALLOWLIST
        proof_facing_rows.append(
            {
                "packet_id": pid,
                "system_id": sid,
                "instance_id": iid,
                "elaborated": status_row.get("elaborated", ""),
                "admit_count": status_row.get("admit_count", ""),
                "axiom_count": status_row.get("axiom_count", ""),
                "proof_mode": status_row.get("proof_mode", ""),
                "imported_modules": imports_by_packet.get(pid, ""),
                "outcome_summary": status_row.get("outcome_summary", ""),
                "lean_version": lean_ver,
                "lake_version": lake_ver,
                "diagnostics_path": diagnostics_rel,
                "diagnostics_sha256": sha256_file(diagnostics_path),
                "checked_command": checked_command,
                "check_timestamp": file_mtime_iso(diagnostics_path),
                "m1_allowlisted": "true" if allowlisted else "false",
            }
        )
    args.out_proof_facing_subset.parent.mkdir(parents=True, exist_ok=True)
    with args.out_proof_facing_subset.open(
        "w", newline="", encoding="utf-8"
    ) as f:
        w = csv.DictWriter(f, fieldnames=proof_facing_fields)
        w.writeheader()
        w.writerows(proof_facing_rows)

    print(f"wrote {args.out} ({len(rows_out)} rows)")
    print(f"wrote {args.out_success_subset} ({len(subset_rows)} rows)")
    print(f"wrote {args.out_proof_subset} ({len(proof_rows)} rows)")
    print(
        f"wrote {args.out_proof_facing_subset} ({len(proof_facing_rows)} rows)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
