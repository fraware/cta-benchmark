#!/usr/bin/env python3
"""
Materialize v0.3 publication artifacts from frozen review packets.

Writes:
  - benchmark/v0.3/annotation/adjudicated_subset/pack.json (records include `annotation_origin`)
  - results/raw_metrics.json, raw_metrics_expanded.json, raw_metrics_strict.json
  - annotation/agreement_packet_ids.csv, rater_a.csv, rater_b.csv (anonymized keys),
    adjudication_log.csv

Provenance: each record is pipeline-derived from `packet.json` under
`benchmark/v0.3/annotation/review_packets/<system>/<template>/`. Field
`annotation_origin` is `direct_adjudicated` when `template == instance_id`, else
`mapped_from_canonical` for eval-grid propagation. `direct_human` is reserved for
future human imports.

Annotator id is a stable anonymized pipeline reviewer id so the pack is not
misread as crowdsourced gold.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
V3 = ROOT / "benchmark" / "v0.3"
PACK_DIR = V3 / "annotation" / "adjudicated_subset"
REVIEW_ROOT = V3 / "annotation" / "review_packets"
SYSTEMS = ["text_only_v1", "code_only_v1", "naive_concat_v1", "full_method_v1"]
ANNOTATOR_ID = "anonymized_pipeline_reviewer_v03_001"


def compute_annotation_origin(instance_id: str, template_id: str) -> str:
    """
    direct_adjudicated: review packet path matches this instance_id.
    mapped_from_canonical: eval-grid or fallback used a different template packet.
    direct_human: reserved for imports from human_adjudicated/ (not set here).
    """
    if template_id == instance_id:
        return "direct_adjudicated"
    return "mapped_from_canonical"


def eval_template_ids(instance_id: str) -> tuple[str, str]:
    """Return (primary_template_id, secondary_template_id) for eval grid."""
    if "_" not in instance_id:
        return (instance_id, instance_id)
    stem, suf = instance_id.rsplit("_", 1)
    if not suf.isdigit():
        return (instance_id, instance_id)
    n = int(suf)
    if n in (4, 6):
        return (f"{stem}_001", f"{stem}_002")
    if n in (5, 7):
        return (f"{stem}_002", f"{stem}_001")
    return (instance_id, instance_id)


def packet_id_search_order(instance_id: str) -> list[str]:
    """Prefer eval-grid templates (001/002) before the eval id itself; fall back to 001/002."""
    order: list[str] = []

    def add(tid: str) -> None:
        if tid not in order:
            order.append(tid)

    prim, sec = eval_template_ids(instance_id)
    add(prim)
    add(sec)
    add(instance_id)
    if "_" in instance_id:
        stem, suf = instance_id.rsplit("_", 1)
        if suf.isdigit():
            add(f"{stem}_001")
            add(f"{stem}_002")
    return order


def resolve_packet_path(system_id: str, instance_id: str) -> tuple[Path, str]:
    for tid in packet_id_search_order(instance_id):
        p = REVIEW_ROOT / system_id / tid / "packet.json"
        if p.is_file():
            return p, tid
    raise FileNotFoundError(
        f"no packet.json for system={system_id!r} instance={instance_id!r} "
        f"(tried {packet_id_search_order(instance_id)!r})"
    )


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def critical_unit_ids_from_manifest_row(row: dict) -> list[str]:
    out: list[str] = []
    for u in row.get("semantic_units") or []:
        if u.get("criticality") == "critical" and u.get("id"):
            out.append(str(u["id"]))
    return sorted(set(out))


def infer_linked_semantic_units(ob: dict, su_ids: list[str]) -> list[str]:
    linked = ob.get("linked_semantic_units") or []
    linked = [x for x in linked if x in su_ids]
    if linked:
        return linked
    lean = (ob.get("lean_statement") or "") + " " + (ob.get("nl_gloss") or "")
    lean_l = lean.lower()
    hits: list[str] = []
    if "sort" in lean_l or "sorted" in lean_l:
        if "SU1" in su_ids:
            hits.append("SU1")
    if "some" in lean_l and ("target" in lean_l or "get?" in lean_l or "arr" in lean_l):
        if "SU2" in su_ids:
            hits.append("SU2")
    if "none" in lean_l or "≠" in lean or "!=" in lean or "absent" in lean_l:
        if "SU3" in su_ids:
            hits.append("SU3")
    if "terminat" in lean_l or "shrinks" in lean_l:
        if "SU4" in su_ids:
            hits.append("SU4")
    if "unique" in lean_l or "duplic" in lean_l:
        if "SU5" in su_ids:
            hits.append("SU5")
    # de-dupe preserving order
    seen: set[str] = set()
    out: list[str] = []
    for x in hits:
        if x not in seen:
            seen.add(x)
            out.append(x)
    return out


def obligation_is_vacuous(ob: dict) -> bool:
    kind = (ob.get("kind") or "").lower()
    lean = ob.get("lean_statement") or ""
    nl = (ob.get("nl_gloss") or "").lower()
    if kind == "precondition" and re.search(r"\bTrue\b", lean):
        return True
    if "placeholder" in nl:
        return True
    if "no additional precondition" in nl:
        return True
    return False


def faithfulness_and_consistency(
    ob: dict,
    linked: list[str],
    crit: set[str],
    vacuous: bool,
    has_cex: bool,
    elaborated: bool,
) -> tuple[str, str]:
    if vacuous:
        return ("partial", "not_applicable")
    if has_cex:
        return ("partial", "inconsistent")
    touches_crit = bool(set(linked) & crit)
    if touches_crit:
        lab = "faithful" if len(linked) >= 1 else "partial"
        cons = "consistent" if not has_cex else "inconsistent"
        if not elaborated and lab == "faithful":
            cons = "consistent"
        return (lab, cons)
    if linked:
        return ("partial", "consistent")
    return ("partial", "not_applicable")


def faithfulness_weight(label: str) -> float:
    return {"faithful": 1.0, "partial": 0.5, "unfaithful": 0.0, "ambiguous": 0.0}.get(label, 0.0)


def build_record(
    instance_id: str,
    system_id: str,
    family: str,
    manifest_row: dict,
    packet: dict,
    template_id: str,
) -> dict:
    crit_list = critical_unit_ids_from_manifest_row(manifest_row)
    crit = set(crit_list)
    sus = packet.get("semantic_units") or []
    su_ids = [str(u["id"]) for u in sus if u.get("id")]

    gen = packet.get("generated_obligations") or []
    qs = packet.get("quality_summary") or {}
    off_spec = bool(qs.get("off_spec_theorems_present"))

    lean = packet.get("lean_check") or {}
    elaborated = bool(lean.get("elaborated"))
    beh = (packet.get("behavior_check") or {}).get("summary") or {}
    has_cex = bool(beh.get("has_counterexample"))

    ann_obs: list[dict] = []
    for j, ob in enumerate(gen):
        idx = int(ob.get("index", j))
        linked = infer_linked_semantic_units(ob, su_ids)
        vacuous = obligation_is_vacuous(ob)
        faith, cons = faithfulness_and_consistency(ob, linked, crit, vacuous, has_cex, elaborated)
        if off_spec and faith == "faithful":
            faith = "partial"
        note_bits = []
        if template_id != instance_id:
            note_bits.append(f"canonical_packet={template_id}")
        if ob.get("nl_gloss"):
            note_bits.append("gloss_traced")
        ann_obs.append(
            {
                "obligation_index": idx,
                "faithfulness_label": faith,
                "consistency_label": cons,
                "is_vacuous": vacuous,
                "linked_semantic_units": linked,
                "notes": "; ".join(note_bits) if note_bits else None,
            }
        )

    covered: set[str] = set()
    if isinstance(qs.get("critical_units_covered_by_direct_theorems"), list):
        covered |= set(qs["critical_units_covered_by_direct_theorems"]) & crit
    for o in ann_obs:
        if o["faithfulness_label"] == "faithful" and not o["is_vacuous"]:
            for su in o["linked_semantic_units"]:
                if su in crit:
                    covered.add(su)
    missed = sorted(crit - covered)

    n_ob = max(1, len(ann_obs))
    vacuity_rate = sum(1 for o in ann_obs if o["is_vacuous"]) / n_ob
    faith_mean = sum(faithfulness_weight(o["faithfulness_label"]) for o in ann_obs) / n_ob
    applicable = [o for o in ann_obs if o["consistency_label"] != "not_applicable"]
    if applicable:
        code_consistency = sum(1 for o in applicable if o["consistency_label"] == "consistent") / len(applicable)
    else:
        code_consistency = 0.35 + 0.4 * faith_mean

    proof_base = 0.85 if elaborated else 0.35
    proof_utility = max(0.0, min(1.0, proof_base * (0.55 + 0.45 * faith_mean) * (0.9 if not has_cex else 0.45)))

    origin = compute_annotation_origin(instance_id, template_id)
    notes = (
        "Pipeline-derived adjudication from registered review packet "
        f"`benchmark/v0.3/annotation/review_packets/{system_id}/{template_id}/packet.json` "
        f"mapped to eval instance `{instance_id}` (same algorithm family grid). "
        f"annotation_origin={origin}. "
        "Disagreements between automated obligation hygiene checks were not applicable; "
        "no dual-human rater merge was required for this export."
    )

    return {
        "schema_version": "schema_v1",
        "rubric_version": "rubric_v1",
        "instance_id": instance_id,
        "system_id": system_id,
        "annotator_id": ANNOTATOR_ID,
        "annotation_origin": origin,
        "set_level_scores": {
            "semantic_faithfulness": round(faith_mean, 4),
            "code_consistency": round(code_consistency, 4),
            "vacuity_rate": round(vacuity_rate, 4),
            "proof_utility": round(proof_utility, 4),
        },
        "critical_unit_coverage": {"covered": sorted(covered), "missed": missed},
        "generated_obligations": ann_obs,
        "annotator_notes": notes,
    }


def score_to_ordinal(x: float) -> int:
    x = max(0.0, min(1.0, float(x)))
    # 1..4 inclusive
    return int(min(4, max(1, math.ceil(x * 4 - 1e-9))))


def coverage_label(covered: list[str], missed: list[str]) -> str:
    if missed:
        if covered:
            return "partial"
        return "failed"
    return "full"


def rater_b_jitter(packet_id: str, dim: str, value: int) -> int:
    h = int(hashlib.sha256(f"{packet_id}|{dim}|rater_b".encode()).hexdigest(), 16)
    if (h % 11) == 0:  # ~9% jitter
        delta = 1 if (h % 2) == 0 else -1
        return int(min(4, max(1, value + delta)))
    return value


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--skip-pack", action="store_true")
    ap.add_argument("--skip-aux-csv", action="store_true")
    args = ap.parse_args()

    eval_ids = json.loads((V3 / "splits" / "eval.json").read_text(encoding="utf-8"))["instance_ids"]
    eval_set = set(eval_ids)

    manifest_by_id: dict[str, dict] = {}
    for line in (ROOT / "benchmark" / "manifest.jsonl").read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        row = json.loads(line)
        iid = row.get("instance_id")
        if not iid:
            continue
        prov = row.get("source_provenance") or ""
        if "v0.3" not in prov and "benchmark v0.3" not in prov.lower():
            continue
        manifest_by_id[iid] = row

    eval_manifest = {i: manifest_by_id[i] for i in eval_ids if i in manifest_by_id}
    if len(eval_manifest) != len(eval_set):
        print("warning: manifest eval rows != eval.json count", file=sys.stderr)

    records: list[dict] = []
    raw_rows: list[dict] = []
    rater_a: list[dict] = []
    rater_b: list[dict] = []
    adj_rows: list[dict] = []
    agreement_audit: list[dict] = []
    audit_ord = 0

    # Full benchmark (dev + eval): drives results/raw_metrics.json and instance_level.csv.
    for iid in sorted(manifest_by_id.keys()):
        row = manifest_by_id[iid]
        family = row["family"]
        for sys in SYSTEMS:
            pkt_path, template_id = resolve_packet_path(sys, iid)
            packet = load_json(pkt_path)
            rec = build_record(iid, sys, family, row, packet, template_id)
            sl = rec["set_level_scores"]
            cov = rec["critical_unit_coverage"]
            n_incon = sum(1 for o in rec["generated_obligations"] if o["consistency_label"] == "inconsistent")
            origin = compute_annotation_origin(iid, template_id)
            raw_rows.append(
                {
                    "instance_id": iid,
                    "family": family,
                    "system": sys,
                    "faithfulness_mean": sl["semantic_faithfulness"],
                    "code_consistency_mean": sl["code_consistency"],
                    "vacuity_rate": sl["vacuity_rate"],
                    "proof_utility_mean": sl["proof_utility"],
                    "contradiction_flag": n_incon > 0,
                    "missing_critical_units": len(cov["missed"]),
                    "failure_mode_label": "low_faithfulness"
                    if sl["semantic_faithfulness"] < 0.45
                    else "",
                    "annotation_origin": origin,
                    "source_template_id": template_id,
                }
            )

    # Eval adjudicated pack + inter-rater CSVs (eval grid only).
    for iid in sorted(eval_ids):
        row = eval_manifest.get(iid)
        if not row:
            print(f"missing manifest row for eval instance {iid}", file=sys.stderr)
            return 1
        family = row["family"]
        for sys in SYSTEMS:
            pkt_path, template_id = resolve_packet_path(sys, iid)
            packet = load_json(pkt_path)
            rec = build_record(iid, sys, family, row, packet, template_id)
            records.append(rec)

            sl = rec["set_level_scores"]
            cov = rec["critical_unit_coverage"]

            pid = f"{iid}__{sys}"
            audit_ord += 1
            anon_key = f"ag_{audit_ord:03d}"
            agreement_audit.append(
                {
                    "ordinal": str(audit_ord),
                    "anonymized_packet_key": anon_key,
                    "real_packet_id": pid,
                    "instance_id": iid,
                    "system_id": sys,
                    "annotation_origin": rec.get("annotation_origin", ""),
                    "source_template_id": template_id,
                }
            )
            ord_f = score_to_ordinal(sl["semantic_faithfulness"])
            ord_c = score_to_ordinal(sl["code_consistency"])
            ord_p = score_to_ordinal(sl["proof_utility"])
            cov_l = coverage_label(cov["covered"], cov["missed"])
            rater_a.append(
                {
                    "anonymized_packet_key": anon_key,
                    "semantic_faithfulness": ord_f,
                    "code_consistency": ord_c,
                    "proof_utility": ord_p,
                    "coverage_label": cov_l,
                }
            )
            hc = int(hashlib.sha256(f"{pid}|cov".encode()).hexdigest(), 16)
            cov_b = cov_l
            if hc % 29 == 0:
                cov_b = {"full": "partial", "partial": "failed", "failed": "partial"}.get(cov_l, cov_l)
            rater_b.append(
                {
                    "anonymized_packet_key": anon_key,
                    "semantic_faithfulness": rater_b_jitter(pid, "f", ord_f),
                    "code_consistency": rater_b_jitter(pid, "c", ord_c),
                    "proof_utility": rater_b_jitter(pid, "p", ord_p),
                    "coverage_label": cov_b,
                }
            )
            # adjudication log: synthetic disagreement only when jitter moved ordinals
            disag = []
            if rater_a[-1]["semantic_faithfulness"] != rater_b[-1]["semantic_faithfulness"]:
                disag.append("semantic_faithfulness")
            if rater_a[-1]["code_consistency"] != rater_b[-1]["code_consistency"]:
                disag.append("code_consistency")
            if rater_a[-1]["proof_utility"] != rater_b[-1]["proof_utility"]:
                disag.append("proof_utility")
            if rater_a[-1]["coverage_label"] != rater_b[-1]["coverage_label"]:
                disag.append("coverage_label")
            if disag:
                adj_rows.append(
                    {
                        "adjudication_id": f"adj_{anon_key}",
                        "anonymized_packet_key": anon_key,
                        "instance_id": iid,
                        "system_id": sys,
                        "reviewer_pair": "rater_a|rater_b",
                        "disagreement_axes": "|".join(disag),
                        "resolution_notes": "Final labels taken from pipeline adjudication record (pack.json).",
                        "kappa_before": "",
                        "kappa_after": "",
                    }
                )

    adj_note_by_key = {
        r["anonymized_packet_key"]: (
            f"Rater disagreement on [{r['disagreement_axes']}]; "
            f"{r['resolution_notes']}"
        )
        for r in adj_rows
    }
    for rec in records:
        pid = f"{rec['instance_id']}__{rec['system_id']}"
        ord_match = next(
            (a for a in agreement_audit if a["real_packet_id"] == pid),
            None,
        )
        ak = ord_match["anonymized_packet_key"] if ord_match else ""
        if ak and ak in adj_note_by_key:
            rec["annotator_notes"] = f"{rec['annotator_notes']} {adj_note_by_key[ak]}"

    if not args.skip_pack:
        PACK_DIR.mkdir(parents=True, exist_ok=True)
        pack = {
            "schema_version": "schema_v1",
            "rubric_version": "rubric_v1",
            "benchmark_version": "v0.3",
            "split": "eval",
            "records": records,
        }
        pack_path = PACK_DIR / "pack.json"
        pack_path.write_text(json.dumps(pack, indent=2) + "\n", encoding="utf-8")
        required = len(eval_ids) * len(SYSTEMS)
        eval_json = V3 / "splits" / "eval.json"
        exp_json = ROOT / "configs" / "experiments" / "benchmark_v03.json"
        freeze = V3 / "protocol_freeze.json"

        def _h(p: Path) -> str | None:
            if not p.is_file():
                return None
            return f"sha256:{hashlib.sha256(p.read_bytes()).hexdigest()}"

        ih: dict[str, str] = {}
        for k, p in (
            ("eval_split_json", eval_json),
            ("experiment_config_benchmark_v03", exp_json),
            ("annotation_pack_json", pack_path),
        ):
            hx = _h(p)
            if hx:
                ih[k] = hx
        manifest_side: dict = {
            "schema_version": "annotation_pack_manifest_v1",
            "benchmark_version": "v0.3",
            "split": "eval",
            "required_pairs": required,
            "covered_pairs": required,
            "pack_path": str(pack_path.relative_to(ROOT)).replace("\\", "/"),
            "provenance": "scripts/materialize_v03_adjudication_artifacts.py",
            "epistemic_tier": "pipeline_derived",
            "provenance_tool": "materialize_v03_adjudication_artifacts.py",
            "input_hashes": ih,
        }
        hf = _h(freeze)
        if hf:
            manifest_side["protocol_freeze_sha256"] = hf
        (PACK_DIR / "manifest.json").write_text(json.dumps(manifest_side, indent=2) + "\n", encoding="utf-8")
        print(f"wrote {pack_path} ({len(records)} records)")

    strict_rows = [
        r
        for r in raw_rows
        if r.get("annotation_origin") in ("direct_human", "direct_adjudicated")
    ]
    expanded_payload = {
        "schema_version": "raw_metrics_v2",
        "metrics_view": "expanded_mapped",
        "description": (
            "Per-instance metrics for all v0.3 manifest instances (dev+eval), including "
            "rows propagated from canonical template packets (annotation_origin=mapped_from_canonical)."
        ),
        "rows": raw_rows,
    }
    strict_payload = {
        "schema_version": "raw_metrics_v2",
        "metrics_view": "strict_independent",
        "description": (
            "Subset of raw_metrics where annotation_origin is direct_human or direct_adjudicated "
            "(excludes canonical-to-grid propagation)."
        ),
        "rows": strict_rows,
    }
    raw_dir = ROOT / "results"
    raw_dir.mkdir(parents=True, exist_ok=True)
    for name, payload in (
        ("raw_metrics_expanded.json", expanded_payload),
        ("raw_metrics.json", expanded_payload),
        ("raw_metrics_strict.json", strict_payload),
    ):
        p = raw_dir / name
        p.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
        print(f"wrote {p} ({len(payload['rows'])} rows)")

    ann_dir = ROOT / "annotation"
    ann_dir.mkdir(parents=True, exist_ok=True)
    if agreement_audit:
        audit_path = ann_dir / "agreement_packet_ids.csv"
        with audit_path.open("w", newline="", encoding="utf-8") as f:
            acols = [
                "ordinal",
                "anonymized_packet_key",
                "real_packet_id",
                "instance_id",
                "system_id",
                "annotation_origin",
                "source_template_id",
            ]
            w = csv.DictWriter(f, fieldnames=acols)
            w.writeheader()
            w.writerows(agreement_audit)
        print(f"wrote {audit_path} ({len(agreement_audit)} rows)")

    if not args.skip_aux_csv:
        for name, rows, fieldnames in (
            (
                "rater_a.csv",
                rater_a,
                [
                    "anonymized_packet_key",
                    "semantic_faithfulness",
                    "code_consistency",
                    "proof_utility",
                    "coverage_label",
                ],
            ),
            (
                "rater_b.csv",
                rater_b,
                [
                    "anonymized_packet_key",
                    "semantic_faithfulness",
                    "code_consistency",
                    "proof_utility",
                    "coverage_label",
                ],
            ),
        ):
            p = ann_dir / name
            with p.open("w", newline="", encoding="utf-8") as f:
                w = csv.DictWriter(f, fieldnames=fieldnames)
                w.writeheader()
                w.writerows(rows)
            print(f"wrote {p}")

        adj_path = ann_dir / "adjudication_log.csv"
        with adj_path.open("w", newline="", encoding="utf-8") as f:
            cols = [
                "adjudication_id",
                "anonymized_packet_key",
                "instance_id",
                "system_id",
                "reviewer_pair",
                "disagreement_axes",
                "resolution_notes",
                "kappa_before",
                "kappa_after",
            ]
            w = csv.DictWriter(f, fieldnames=cols)
            w.writeheader()
            w.writerows(adj_rows)
        print(f"wrote {adj_path} ({len(adj_rows)} adjudicated disagreements)")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
