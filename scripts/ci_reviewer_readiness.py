#!/usr/bin/env python3
"""CI checks: paper summary vs CSV rows, JSON validation (cargo), denylist."""

from __future__ import annotations

import csv
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def cargo_validate(schema: str, path: Path) -> None:
    subprocess.check_call(
        [
            "cargo",
            "run",
            "-p",
            "cta_cli",
            "--quiet",
            "--",
            "validate",
            "file",
            "--schema",
            schema,
            "--path",
            str(path),
        ],
        cwd=ROOT,
    )


def count_csv_rows(path: Path) -> int:
    with path.open(encoding="utf-8", newline="") as f:
        return max(0, sum(1 for _ in f) - 1)


def load_csv_rows(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as f:
        return list(csv.DictReader(f))


def count_nonempty_jsonl_lines(path: Path) -> int:
    with path.open(encoding="utf-8") as f:
        return sum(1 for line in f if line.strip())


def main() -> int:
    summary_path = ROOT / "benchmark" / "v0.3" / "benchmark_paper_summary.json"
    summary = json.loads(summary_path.read_text(encoding="utf-8"))
    expected = int(summary["expected_instance_level_rows"])
    inst = ROOT / "results" / "instance_level.csv"
    n = count_csv_rows(inst)
    if n != expected:
        print(
            f"error: instance_level.csv rows {n} != "
            f"benchmark_paper_summary expected {expected}",
            file=sys.stderr,
        )
        return 1

    raw_path = ROOT / "results" / "raw_metrics.json"
    if raw_path.is_file():
        raw = json.loads(raw_path.read_text(encoding="utf-8"))
        rcount = len(raw.get("rows") or [])
        exp_raw = int(summary.get("expected_raw_metrics_rows", expected))
        if rcount != exp_raw:
            print(
                f"error: raw_metrics rows {rcount} != "
                f"expected_raw_metrics_rows {exp_raw}",
                file=sys.stderr,
            )
            return 1

    strict_path = ROOT / "results" / "raw_metrics_strict.json"
    if strict_path.is_file():
        strict_body = json.loads(strict_path.read_text(encoding="utf-8"))
        strict_rows = strict_body.get("rows") or []
        strict_cnt = len(strict_rows)
        exp_strict = summary.get("expected_raw_metrics_strict_rows")
        if exp_strict is not None and int(exp_strict) != strict_cnt:
            print(
                f"error: raw_metrics_strict rows {strict_cnt} != "
                f"expected_raw_metrics_strict_rows {exp_strict}",
                file=sys.stderr,
            )
            return 1
        ext_strict = (
            ROOT / "annotation" / "external_review" / "strict_review_queue.jsonl"
        )
        if ext_strict.is_file():
            n_ext = count_nonempty_jsonl_lines(ext_strict)
            if n_ext != strict_cnt:
                print(
                    "error: annotation/external_review/strict_review_queue.jsonl "
                    f"non-empty lines {n_ext} != raw_metrics_strict rows {strict_cnt}",
                    file=sys.stderr,
                )
                return 1

    ag_path = ROOT / "annotation" / "agreement_packet_ids.csv"
    exp_ag = summary.get("expected_agreement_packet_audit_rows")
    if exp_ag is not None and ag_path.is_file():
        n_ag = count_csv_rows(ag_path)
        if n_ag != int(exp_ag):
            print(
                f"error: agreement_packet_ids.csv rows {n_ag} != "
                f"expected_agreement_packet_audit_rows {exp_ag}",
                file=sys.stderr,
            )
            return 1

    rep_path = ROOT / "annotation" / "agreement_report.json"
    if rep_path.is_file() and exp_ag is not None:
        rep = json.loads(rep_path.read_text(encoding="utf-8"))
        n_rep = rep.get("n_packets")
        if n_rep is not None and int(n_rep) != int(exp_ag):
            print(
                f"error: agreement_report.json n_packets {n_rep} != "
                f"expected_agreement_packet_audit_rows {exp_ag}",
                file=sys.stderr,
            )
            return 1

    ev_path = ROOT / "results" / "paper_table_annotation_evidence.csv"
    exp_strict = summary.get("expected_raw_metrics_strict_rows")
    if ev_path.is_file() and exp_strict is not None:
        found_strict_row = False
        with ev_path.open(encoding="utf-8", newline="") as f:
            for row in csv.DictReader(f):
                mview = (row.get("metrics_view") or "").strip()
                if mview != "strict_independent":
                    continue
                found_strict_row = True
                try:
                    n_ev = int((row.get("n_eval_rows") or "").strip())
                except ValueError:
                    print(
                        "error: paper_table_annotation_evidence.csv "
                        "strict row has non-integer n_eval_rows",
                        file=sys.stderr,
                    )
                    return 1
                if n_ev != int(exp_strict):
                    print(
                        f"error: paper_table_annotation_evidence strict "
                        f"n_eval_rows {n_ev} != "
                        f"expected_raw_metrics_strict_rows {exp_strict}",
                        file=sys.stderr,
                    )
                    return 1
                break
        if not found_strict_row:
            print(
                "error: paper_table_annotation_evidence.csv missing "
                "strict_independent row",
                file=sys.stderr,
            )
            return 1

    exp_ag_strict = summary.get(
        "agreement_audit_strict_independent_packet_count"
    )
    ag_evi_path = ROOT / "results" / "paper_table_agreement_evidence.csv"
    if exp_ag_strict is not None:
        if not ag_evi_path.is_file():
            rel_ag = ag_evi_path.relative_to(ROOT)
            print(
                "error: benchmark_paper_summary.json declares "
                f"agreement_audit_strict_independent_packet_count="
                f"{exp_ag_strict} but {rel_ag} is missing",
                file=sys.stderr,
            )
            return 1
        found_ag_strict_row = False
        with ag_evi_path.open(encoding="utf-8", newline="") as f:
            for row in csv.DictReader(f):
                ag_sub = (row.get("agreement_subset") or "").strip()
                if ag_sub != "strict_independent_only":
                    continue
                found_ag_strict_row = True
                try:
                    n_ag_pk = int((row.get("n_packets") or "").strip())
                except ValueError:
                    print(
                        "error: paper_table_agreement_evidence.csv "
                        "strict_independent_only row has non-integer "
                        "n_packets",
                        file=sys.stderr,
                    )
                    return 1
                if n_ag_pk != int(exp_ag_strict):
                    print(
                        "error: paper_table_agreement_evidence "
                        "strict_independent_only n_packets "
                        f"{n_ag_pk} != "
                        "agreement_audit_strict_independent_packet_count "
                        f"{exp_ag_strict}",
                        file=sys.stderr,
                    )
                    return 1
                break
        if not found_ag_strict_row:
            print(
                "error: paper_table_agreement_evidence.csv missing "
                "strict_independent_only row",
                file=sys.stderr,
            )
            return 1

    primary_registry = ROOT / "results" / "paper_primary_model_registry.csv"
    if not primary_registry.is_file():
        print(
            "error: missing results/paper_primary_model_registry.csv",
            file=sys.stderr,
        )
        return 1
    primary_rows = load_csv_rows(primary_registry)
    if len(primary_rows) != 4:
        print(
            "error: paper_primary_model_registry.csv must contain exactly 4 rows "
            f"(found {len(primary_rows)})",
            file=sys.stderr,
        )
        return 1
    headline_set = {
        str(x)
        for x in (summary.get("paper_systems_ordered") or [])
        if str(x).strip()
    }
    allowed_status = {"matched", "historical_manifest_mismatch_explained"}
    for row in primary_rows:
        sid = (row.get("system_id") or "").strip()
        if sid not in headline_set:
            print(
                "error: paper_primary_model_registry system_id "
                f"{sid!r} not in paper_systems_ordered",
                file=sys.stderr,
            )
            return 1
        status = (row.get("model_metadata_status") or "").strip()
        if status not in allowed_status:
            print(
                "error: invalid model_metadata_status in paper_primary_model_registry: "
                f"{status!r}",
                file=sys.stderr,
            )
            return 1

    man = (
        ROOT
        / "benchmark"
        / "v0.3"
        / "annotation"
        / "adjudicated_subset"
        / "manifest.json"
    )
    if man.is_file():
        cargo_validate("annotation_pack_manifest", man)

    pf = ROOT / "benchmark" / "v0.3" / "protocol_freeze.json"
    if pf.is_file():
        cargo_validate("protocol_freeze", pf)

    ont = ROOT / "schemas" / "failure_mode_v1.json"
    if ont.is_file():
        cargo_validate("failure_mode_ontology", ont)

    deny = re.compile(
        r"\b(TODO|placeholder|skeleton|fill\s+after)\b",
        re.IGNORECASE,
    )
    scan_roots = [
        ROOT / "annotation",
        ROOT / "results",
    ]
    for base in scan_roots:
        if not base.is_dir():
            continue
        for p in base.rglob("*"):
            if not p.is_file():
                continue
            rel = p.relative_to(ROOT).as_posix()
            rel_slash = rel.replace("\\", "/")
            if "/external_review/" in rel_slash:
                continue
            skip_wave = (
                ".example." in p.name or "/human_wave_v03/" in rel_slash
            )
            if skip_wave:
                continue
            text_suffixes = {".csv", ".json", ".jsonl", ".md", ".txt"}
            if p.suffix.lower() not in text_suffixes:
                continue
            try:
                txt = p.read_text(encoding="utf-8", errors="ignore")
            except OSError:
                continue
            if deny.search(txt):
                msg = f"error: placeholder denylist matched {rel}"
                print(msg, file=sys.stderr)
                return 1

    v3_ann = ROOT / "benchmark" / "v0.3" / "annotation"
    if v3_ann.is_dir():
        for p in v3_ann.rglob("*"):
            if not p.is_file():
                continue
            try:
                rel_ann = p.relative_to(v3_ann)
            except ValueError:
                continue
            if rel_ann.parts[:1] == ("review_packets",):
                continue
            rel = p.relative_to(ROOT).as_posix()
            rel_slash_ann = rel.replace("\\", "/")
            skip_ann = (
                ".example." in p.name or "/human_wave_v03/" in rel_slash_ann
            )
            if skip_ann:
                continue
            suf = p.suffix.lower()
            if suf == ".csv":
                pass
            elif p.name == "manifest.json" and suf == ".json":
                pass
            elif p.name.lower() == "readme.md" and suf == ".md":
                pass
            else:
                continue
            try:
                txt = p.read_text(encoding="utf-8", errors="ignore")
            except OSError:
                continue
            if deny.search(txt):
                msg_ann = f"error: placeholder denylist matched {rel}"
                print(msg_ann, file=sys.stderr)
                return 1

    ext_mapped = (
        ROOT / "annotation" / "external_review" / "mapped_review_queue.jsonl"
    )
    if raw_path.is_file() and ext_mapped.is_file():
        raw_mapped = json.loads(raw_path.read_text(encoding="utf-8"))
        rows_m = raw_mapped.get("rows") or []
        mapped_exp = sum(
            1
            for r in rows_m
            if str(r.get("annotation_origin") or "") == "mapped_from_canonical"
        )
        n_map_j = count_nonempty_jsonl_lines(ext_mapped)
        if n_map_j != mapped_exp:
            print(
                "error: annotation/external_review/mapped_review_queue.jsonl "
                f"non-empty lines {n_map_j} != expanded mapped rows {mapped_exp}",
                file=sys.stderr,
            )
            return 1

    onto = json.loads(ont.read_text(encoding="utf-8"))
    allowed = {str(m.get("slug", "")) for m in onto.get("modes", [])}
    raw_doc = json.loads(raw_path.read_text(encoding="utf-8"))
    raw_rows_list = raw_doc.get("rows") or []
    for row in raw_rows_list:
        lab = str(row.get("failure_mode_label") or "").strip()
        if lab and lab not in allowed:
            print(
                f"error: failure_mode_label {lab!r} not in ontology",
                file=sys.stderr,
            )
            return 1

    print("ci_reviewer_readiness: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
