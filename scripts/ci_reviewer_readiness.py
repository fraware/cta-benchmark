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


def require_csv_columns(path: Path, required: list[str]) -> bool:
    if not path.is_file():
        rel = path.relative_to(ROOT).as_posix()
        print(f"error: missing required csv {rel}", file=sys.stderr)
        return False
    with path.open(encoding="utf-8", newline="") as f:
        cols = list(csv.DictReader(f).fieldnames or [])
    missing = [c for c in required if c not in cols]
    if missing:
        print(
            f"error: {path.relative_to(ROOT).as_posix()} missing required columns: {missing}",
            file=sys.stderr,
        )
        return False
    return True


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
        strict_unique = len(
            {
                str(r.get("instance_id", "")).strip()
                for r in strict_rows
                if str(r.get("instance_id", "")).strip()
            }
        )
        strict_mapped = sum(
            1
            for r in strict_rows
            if str(r.get("annotation_origin") or "").strip()
            == "mapped_from_canonical"
        )
        if strict_unique != 84:
            print(
                f"error: strict_unique_instances {strict_unique} != 84",
                file=sys.stderr,
            )
            return 1
        if strict_mapped != 0:
            print(
                "error: strict headline rows include mapped_from_canonical "
                f"({strict_mapped})",
                file=sys.stderr,
            )
            return 1
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

    hpv3_map = ROOT / "annotation" / "human_pass_v3" / "human_strict_packet_ids.csv"
    hpv3_rater = ROOT / "annotation" / "human_pass_v3" / "rater_b_human_strict_all.csv"
    hpv3_disagree = ROOT / "annotation" / "human_pass_v3" / "disagreement_log_strict_all.csv"
    if not hpv3_map.is_file() or not hpv3_rater.is_file() or not hpv3_disagree.is_file():
        print("error: missing required human_pass_v3 files", file=sys.stderr)
        return 1
    map_rows = load_csv_rows(hpv3_map)
    if len(map_rows) != 274:
        print(f"error: human_pass_v3.n_rows {len(map_rows)} != 274", file=sys.stderr)
        return 1
    map_instances = {(r.get("instance_id") or "").strip() for r in map_rows if (r.get("instance_id") or "").strip()}
    if len(map_instances) != 84:
        print(f"error: human_pass_v3.n_unique_instance_ids {len(map_instances)} != 84", file=sys.stderr)
        return 1
    if any((r.get("strict_row_id") or "").strip() == "" for r in map_rows):
        print("error: human_pass_v3 strict_row_id missing", file=sys.stderr)
        return 1
    if "mapped_from_canonical" in {c for row in map_rows for c in row.values()}:
        print("error: human_pass_v3 contains mapped_from_canonical rows", file=sys.stderr)
        return 1
    corr_rows = load_csv_rows(ROOT / "annotation" / "external_review" / "semantic_corrections_v3.csv")
    corr_keys = {(r.get("template_id") or "").strip() + "::" + (r.get("system_id") or "").strip() for r in corr_rows}
    map_keys = {(r.get("source_template_id") or "").strip() + "::" + (r.get("system_id") or "").strip() for r in map_rows}
    if not corr_keys.issubset(map_keys):
        print("error: semantic_corrections_v3 rows missing from human_pass_v3", file=sys.stderr)
        return 1
    strict_rows_ci = json.loads(strict_path.read_text(encoding="utf-8")).get("rows") or []
    high_risk = {
        (str(r.get("source_template_id") or r.get("instance_id")), str(r.get("system")))
        for r in strict_rows_ci
        if float(r.get("faithfulness_mean", 1.0)) < 0.5
        or int(r.get("missing_critical_units", 0) or 0) > 0
        or bool(r.get("contradiction_flag"))
        or float(r.get("vacuity_rate", 0.0)) > 0.0
    }
    hp_keys = {(r.get("source_template_id") or r.get("instance_id"), r.get("system_id")) for r in map_rows}
    if not high_risk.issubset(hp_keys):
        print("error: high-risk strict rows missing from human_pass_v3", file=sys.stderr)
        return 1
    for row in load_csv_rows(hpv3_disagree):
        txt = (row.get("resolution_reason") or "").strip().lower()
        if txt == "retain primary adjudicator label for ordinal metrics":
            print("error: generic disagreement resolution text present", file=sys.stderr)
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

    schema_checks = [
        (
            ROOT / "results" / "selection_robustness.csv",
            [
                "system_id",
                "instance_id",
                "selector",
                "semantic_faithfulness",
                "code_consistency",
                "vacuity_rate",
                "proof_utility",
                "missing_critical_units",
                "reliability",
            ],
        ),
        (
            ROOT / "results" / "prompt_token_accounting.csv",
            [
                "system_id",
                "mean_prompt_tokens",
                "median_prompt_tokens",
                "max_prompt_tokens",
                "mean_output_tokens",
                "median_output_tokens",
                "max_output_tokens",
                "truncated_rows",
                "mean_obligations_per_packet",
                "median_obligations_per_packet",
                "empty_packets",
            ],
        ),
        (
            ROOT / "results" / "cross_model_pilot_instance_level.csv",
            [
                "instance_id",
                "family",
                "regime",
                "model_tier",
                "system_id",
                "semantic_faithfulness",
                "code_consistency",
                "vacuity_rate",
                "proof_utility",
                "missing_critical_units",
            ],
        ),
        (
            ROOT / "repairs" / "repair_attempts.csv",
            [
                "candidate_failure_id",
                "selected_for_repair",
                "selection_reason",
                "repair_attempted",
                "repair_outcome",
                "human_minutes",
                "reference_obligations_used",
                "notes",
            ],
        ),
    ]
    for path, cols in schema_checks:
        if not require_csv_columns(path, cols):
            return 1

    valid_layers = {"human_gold", "synthetic_stress", "adjudicated"}
    prov = ROOT / "results" / "provenance_layer_registry.csv"
    if prov.is_file():
        rows = load_csv_rows(prov)
        for row in rows:
            layer = (row.get("layer") or "").strip()
            if layer not in valid_layers:
                print(
                    "error: invalid provenance layer "
                    f"{layer!r} in results/provenance_layer_registry.csv",
                    file=sys.stderr,
                )
                return 1

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
                f"non-empty lines {n_map_j} != expanded mapped "
                f"rows {mapped_exp}",
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

    art_manifest = ROOT / "artifacts" / "evidence_hardening_manifest.json"
    if art_manifest.is_file():
        art = json.loads(art_manifest.read_text(encoding="utf-8"))
        required_contents = set(str(x) for x in (art.get("required_contents") or []))
        must_have = {
            "annotation/human_pass_v3/human_strict_packet_ids.csv",
            "annotation/human_pass_v3/rater_b_human_strict_all.csv",
            "annotation/human_pass_v3/disagreement_log_strict_all.csv",
            "annotation/human_pass_v3/agreement_report_human_strict_all.json",
            "annotation/human_pass_v3/agreement_report_human_strict_all.md",
        }
        if not must_have.issubset(required_contents):
            print("error: artifact manifest missing human_pass_v3 required files", file=sys.stderr)
            return 1
        subprocess.check_call(
            [sys.executable, str(ROOT / "scripts" / "validate_release_artifact.py")],
            cwd=ROOT,
        )

    print("ci_reviewer_readiness: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
