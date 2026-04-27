#!/usr/bin/env python3
"""Plan a direct-adjudication wave to raise strict independent coverage."""

from __future__ import annotations

import argparse
import csv
import json
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
V3 = ROOT / "benchmark" / "v0.3"


def load_manifest_rows() -> dict[str, dict]:
    out: dict[str, dict] = {}
    with (ROOT / "benchmark" / "manifest.jsonl").open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            row = json.loads(line)
            iid = str(row.get("instance_id") or "")
            prov = str(row.get("source_provenance") or "")
            if iid and "v0.3" in prov:
                out[iid] = row
    return out


def load_hotspot_index(path: Path) -> dict[tuple[str, str], dict[str, str]]:
    idx: dict[tuple[str, str], dict[str, str]] = {}
    if not path.is_file():
        return idx
    with path.open(encoding="utf-8", newline="") as f:
        for row in csv.DictReader(f):
            iid = (row.get("instance_id") or "").strip()
            sid = (row.get("system_id") or "").strip()
            if iid and sid:
                idx[(iid, sid)] = {
                    k: (v or "").strip() for k, v in row.items()
                }
    return idx


def load_raw_rows(path: Path) -> list[dict]:
    if not path.is_file():
        return []
    return json.loads(path.read_text(encoding="utf-8")).get("rows") or []


def priority_score(
    row: dict,
    hs: dict[tuple[str, str], dict[str, str]],
) -> float:
    iid = str(row.get("instance_id") or "")
    sid = str(row.get("system") or "")
    faith = float(row.get("faithfulness_mean") or 0.0)
    miss = int(row.get("missing_critical_units") or 0)
    score = (1.0 - faith) + 0.12 * miss
    hrow = hs.get((iid, sid), {})
    reason = (hrow.get("candidate_reason") or "")
    if "low_semantic_faithfulness" in reason:
        score += 0.35
    if "missing_critical_semantic_unit" in reason:
        score += 0.25
    return score


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--raw-metrics",
        type=Path,
        default=ROOT / "results" / "raw_metrics.json",
    )
    ap.add_argument(
        "--hotspot-selection",
        type=Path,
        default=ROOT / "repairs" / "hotspot_selection.csv",
    )
    ap.add_argument(
        "--target-pairs",
        type=int,
        default=96,
        help=(
            "Desired number of eval (instance, system) pairs "
            "in the next wave."
        ),
    )
    ap.add_argument(
        "--out-plan",
        type=Path,
        default=(
            V3
            / "annotation"
            / "human_wave_v03"
            / "direct_adjudication_wave_plan.csv"
        ),
    )
    ap.add_argument(
        "--out-overrides-template",
        type=Path,
        default=(
            V3
            / "annotation"
            / "human_adjudicated"
            / "direct_adjudicated_pairs.csv"
        ),
    )
    args = ap.parse_args()

    eval_ids = set(
        json.loads(
            (V3 / "splits" / "eval.json").read_text(encoding="utf-8")
        )["instance_ids"]
    )
    manifest = load_manifest_rows()
    hs = load_hotspot_index(args.hotspot_selection)
    rows = load_raw_rows(args.raw_metrics)

    mapped_eval: list[dict] = []
    for r in rows:
        iid = str(r.get("instance_id") or "")
        if iid not in eval_ids:
            continue
        if str(r.get("annotation_origin") or "") != "mapped_from_canonical":
            continue
        mapped_eval.append(r)

    # Stratified family-first interleaving by descending priority.
    by_family: dict[str, list[dict]] = defaultdict(list)
    for r in mapped_eval:
        fam = str(r.get("family") or "unknown")
        by_family[fam].append(r)
    for fam in by_family:
        by_family[fam].sort(
            key=lambda r: priority_score(r, hs),
            reverse=True,
        )

    selected: list[dict] = []
    fams = sorted(by_family.keys())
    while fams and len(selected) < args.target_pairs:
        next_fams: list[str] = []
        for fam in fams:
            bucket = by_family[fam]
            if not bucket:
                continue
            selected.append(bucket.pop(0))
            if len(selected) >= args.target_pairs:
                break
            if bucket:
                next_fams.append(fam)
        fams = next_fams

    args.out_plan.parent.mkdir(parents=True, exist_ok=True)
    with args.out_plan.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(
            [
                "wave_rank",
                "instance_id",
                "system_id",
                "family",
                "difficulty",
                "current_annotation_origin",
                "current_faithfulness_mean",
                "missing_critical_units",
                "priority_reason",
                "priority_score",
                "recommended_annotation_origin",
                "override_file",
            ]
        )
        for i, r in enumerate(selected, start=1):
            iid = str(r.get("instance_id") or "")
            sid = str(r.get("system") or "")
            mrow = manifest.get(iid, {})
            hrow = hs.get((iid, sid), {})
            reason = (hrow.get("candidate_reason") or "").strip()
            w.writerow(
                [
                    i,
                    iid,
                    sid,
                    str(r.get("family") or ""),
                    str(mrow.get("difficulty") or ""),
                    str(r.get("annotation_origin") or ""),
                    f"{float(r.get('faithfulness_mean') or 0.0):.6f}",
                    int(r.get("missing_critical_units") or 0),
                    reason
                    or "priority_by_low_faithfulness_and_missing_critical",
                    f"{priority_score(r, hs):.6f}",
                    "direct_adjudicated",
                    args.out_overrides_template.as_posix(),
                ]
            )

    # Header-only template (append completed direct-adjudicated pairs after
    # review).
    if not args.out_overrides_template.is_file():
        args.out_overrides_template.parent.mkdir(parents=True, exist_ok=True)
        with args.out_overrides_template.open(
            "w",
            newline="",
            encoding="utf-8",
        ) as f:
            w = csv.writer(f)
            w.writerow(
                [
                    "instance_id",
                    "system_id",
                    "annotation_origin",
                    "adjudication_note",
                ]
            )

    print(f"wrote {args.out_plan} ({len(selected)} planned pairs)")
    print(f"template {args.out_overrides_template}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
