#!/usr/bin/env python3
"""Export machine-readable external annotation review queues."""

from __future__ import annotations

import argparse
import csv
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def load_manifest_index(path: Path) -> dict[str, dict]:
    idx: dict[str, dict] = {}
    with path.open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            row = json.loads(line)
            iid = str(row.get("instance_id") or "")
            if iid and "v0.3" in str(row.get("source_provenance") or ""):
                idx[iid] = row
    return idx


def packet_path(system_id: str, template_id: str) -> Path:
    return (
        ROOT
        / "benchmark"
        / "v0.3"
        / "annotation"
        / "review_packets"
        / system_id
        / template_id
        / "packet.json"
    )


def instance_path(instance_id: str, family: str) -> str:
    domain = family.split("_", 1)[0]
    p = (
        Path("benchmark")
        / "v0.3"
        / "instances"
        / domain
        / instance_id
        / "instance.json"
    )
    return p.as_posix()


def pack_index(pack_records: list[dict]) -> dict[tuple[str, str], dict]:
    idx: dict[tuple[str, str], dict] = {}
    for rec in pack_records:
        iid = str(rec.get("instance_id") or "")
        sid = str(rec.get("system_id") or "")
        if iid and sid:
            idx[(iid, sid)] = rec
    return idx


def build_row(raw_row: dict, pidx: dict[tuple[str, str], dict]) -> dict:
    iid = str(raw_row.get("instance_id") or "")
    sid = str(raw_row.get("system") or "")
    family = str(raw_row.get("family") or "")
    template_id = str(raw_row.get("source_template_id") or iid)
    ann_origin = str(raw_row.get("annotation_origin") or "")
    mapped = ann_origin == "mapped_from_canonical"

    ppath = packet_path(sid, template_id)
    packet = {}
    if ppath.is_file():
        packet = load_json(ppath)
    rec = pidx.get((iid, sid), {})
    coverage = rec.get("critical_unit_coverage") or {}
    score = rec.get("set_level_scores") or {}

    sem_units = packet.get("semantic_units") or []
    critical_units = [u for u in sem_units if str(u.get("criticality") or "") == "critical"]
    ref_obl = packet.get("reference_obligations") or []
    gen_obl = packet.get("generated_obligations") or []

    out = {
        "instance_id": iid,
        "family": family,
        "system_id": sid,
        "annotation_origin": ann_origin,
        "mapped_from_canonical": mapped,
        "informal_spec": str(
            (packet.get("informal_statement") or {}).get("text")
            or ""
        ),
        "critical_semantic_units": [
            {
                "id": str(x.get("id") or ""),
                "description": str(x.get("description") or ""),
            }
            for x in critical_units
        ],
        "reference_obligations": [
            {
                "id": str(x.get("id") or ""),
                "lean_statement": str(x.get("lean_statement") or ""),
                "nl_gloss": str(x.get("nl_gloss") or ""),
            }
            for x in ref_obl
        ],
        "generated_obligations": [
            {
                "index": int(x.get("index") if x.get("index") is not None else -1),
                "lean_statement": str(x.get("lean_statement") or ""),
                "nl_gloss": str(x.get("nl_gloss") or ""),
                "linked_semantic_units": list(x.get("linked_semantic_units") or []),
                "layer": str(x.get("layer") or ""),
            }
            for x in gen_obl
        ],
        "current_labels": {
            "semantic_faithfulness": score.get("semantic_faithfulness", raw_row.get("faithfulness_mean", "")),
            "code_consistency": score.get("code_consistency", raw_row.get("code_consistency_mean", "")),
            "vacuity_rate": score.get("vacuity_rate", raw_row.get("vacuity_rate", "")),
            "proof_utility": score.get("proof_utility", raw_row.get("proof_utility_mean", "")),
            "covered_units": list(coverage.get("covered") or []),
            "missing_units": list(coverage.get("missed") or []),
            "failure_mode_label": str(raw_row.get("failure_mode_label") or ""),
        },
        "source_paths": {
            "instance": instance_path(iid, family),
            "packet": str(ppath.relative_to(ROOT)).replace("\\", "/"),
            "raw_metrics": (
                "results/raw_metrics_expanded.json" if mapped else "results/raw_metrics_strict.json"
            ),
        },
    }
    return out


def write_jsonl(path: Path, rows: list[dict]) -> None:
    with path.open("w", encoding="utf-8", newline="\n") as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=True) + "\n")


def write_strict_csv(path: Path, rows: list[dict]) -> None:
    fields = [
        "instance_id",
        "family",
        "system_id",
        "annotation_origin",
        "mapped_from_canonical",
        "semantic_faithfulness",
        "code_consistency",
        "vacuity_rate",
        "proof_utility",
        "failure_mode_label",
        "covered_units",
        "missing_units",
        "instance_path",
        "packet_path",
        "raw_metrics_path",
    ]
    with path.open("w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        for row in rows:
            labels = row.get("current_labels") or {}
            sp = row.get("source_paths") or {}
            w.writerow(
                {
                    "instance_id": row.get("instance_id", ""),
                    "family": row.get("family", ""),
                    "system_id": row.get("system_id", ""),
                    "annotation_origin": row.get("annotation_origin", ""),
                    "mapped_from_canonical": row.get("mapped_from_canonical", False),
                    "semantic_faithfulness": labels.get("semantic_faithfulness", ""),
                    "code_consistency": labels.get("code_consistency", ""),
                    "vacuity_rate": labels.get("vacuity_rate", ""),
                    "proof_utility": labels.get("proof_utility", ""),
                    "failure_mode_label": labels.get("failure_mode_label", ""),
                    "covered_units": "|".join(labels.get("covered_units", [])),
                    "missing_units": "|".join(labels.get("missing_units", [])),
                    "instance_path": sp.get("instance", ""),
                    "packet_path": sp.get("packet", ""),
                    "raw_metrics_path": sp.get("raw_metrics", ""),
                }
            )


def write_schema_md(path: Path) -> None:
    path.write_text(
        "# External annotation review schema\n\n"
        "- `strict_review_queue.jsonl`: strict independent rows from `results/raw_metrics_strict.json`.\n"
        "- `mapped_review_queue.jsonl`: mapped rows from `results/raw_metrics_expanded.json` where `annotation_origin=mapped_from_canonical`.\n"
        "- `strict_review_queue.csv`: flattened strict queue for spreadsheet workflows.\n\n"
        "## JSONL row fields\n"
        "- `instance_id`, `family`, `system_id`: row identity.\n"
        "- `annotation_origin`, `mapped_from_canonical`: provenance.\n"
        "- `informal_spec`: instance natural-language contract from packet context.\n"
        "- `critical_semantic_units`: critical semantic-unit ids/descriptions.\n"
        "- `reference_obligations`: reference obligations from packet context.\n"
        "- `generated_obligations`: generated obligations with index, statement, gloss, linked units, layer.\n"
        "- `current_labels`: current adjudication/metric labels and coverage arrays.\n"
        "- `source_paths`: instance/packet/raw-metrics trace paths.\n",
        encoding="utf-8",
    )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--strict-raw",
        type=Path,
        default=ROOT / "results" / "raw_metrics_strict.json",
    )
    ap.add_argument(
        "--expanded-raw",
        type=Path,
        default=ROOT / "results" / "raw_metrics_expanded.json",
    )
    ap.add_argument(
        "--pack",
        type=Path,
        default=ROOT / "benchmark" / "v0.3" / "annotation" / "adjudicated_subset" / "pack.json",
    )
    ap.add_argument(
        "--manifest",
        type=Path,
        default=ROOT / "benchmark" / "manifest.jsonl",
    )
    ap.add_argument(
        "--out-dir",
        type=Path,
        default=ROOT / "annotation" / "external_review",
    )
    args = ap.parse_args()

    strict_raw = (load_json(args.strict_raw).get("rows") or [])
    expanded_raw = (load_json(args.expanded_raw).get("rows") or [])
    load_manifest_index(args.manifest)
    pack = load_json(args.pack).get("records") or []
    pidx = pack_index(pack)

    strict_rows = [build_row(r, pidx) for r in strict_raw]
    mapped_rows = [
        build_row(r, pidx)
        for r in expanded_raw
        if str(r.get("annotation_origin") or "") == "mapped_from_canonical"
    ]

    args.out_dir.mkdir(parents=True, exist_ok=True)
    strict_jsonl = args.out_dir / "strict_review_queue.jsonl"
    strict_csv = args.out_dir / "strict_review_queue.csv"
    mapped_jsonl = args.out_dir / "mapped_review_queue.jsonl"
    schema_md = args.out_dir / "review_schema.md"
    write_jsonl(strict_jsonl, strict_rows)
    write_strict_csv(strict_csv, strict_rows)
    write_jsonl(mapped_jsonl, mapped_rows)
    write_schema_md(schema_md)

    print(f"wrote {strict_jsonl} ({len(strict_rows)} rows)")
    print(f"wrote {strict_csv} ({len(strict_rows)} rows)")
    print(f"wrote {mapped_jsonl} ({len(mapped_rows)} rows)")
    print(f"wrote {schema_md}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
