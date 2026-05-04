#!/usr/bin/env python3
"""
Merge Responsible AI fields plus provenance into HF Croissant core.

Writes `croissant.json` (core from Hub + `rai:*`, `prov:wasDerivedFrom`, `prov:wasGeneratedBy`,
`rai:syntheticDataDescription`, normalized `@context`, CC-BY-4.0 license) and
`croissant_rai_patch.json` (RAI + both `prov:*` keys and `@context` only).
"""

from __future__ import annotations

import csv
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPO_ID = "fraware/cta-bench"
CORE = ROOT / "hf_release" / "croissant_core.json"
OUT = ROOT / "hf_release" / "croissant.json"
PATCH_OUT = ROOT / "hf_release" / "croissant_rai_patch.json"

RESOLVE_BASE = f"https://huggingface.co/datasets/{REPO_ID}/resolve/main/"

# Files published under hf_release/ that should appear in Croissant distribution.
_DIST_REL_PATHS: tuple[str, ...] = (
    "README.md",
    "LICENSE",
    "CITATION.cff",
    "rai_statement.md",
    "data/instances.jsonl",
    "data/semantic_units.jsonl",
    "data/reference_obligations.jsonl",
    "data/generated_packets.jsonl",
    "data/strict_results.csv",
    "data/expanded_results.csv",
    "data/human_agreement.json",
    "data/correction_overlays.csv",
    "data/system_cards.jsonl",
    "data/prompt_templates.jsonl",
    "data/common_cell_instances.csv",
    "data/common_cell_system_summary.csv",
)

_ENCODING = {
    ".jsonl": "application/jsonlines",
    ".json": "application/json",
    ".csv": "text/csv",
    ".md": "text/markdown",
    ".cff": "text/yaml",
}

_CSV_COLUMN_PRESET: dict[str, str] = {
    "data/strict_results.csv": "instance_id",
    "data/expanded_results.csv": "instance_id",
    "data/common_cell_instances.csv": "instance_id",
    "data/common_cell_system_summary.csv": "system",
}


def _file_object_id(rel: str) -> str:
    safe = rel.replace("/", "_").replace(".", "_")
    return f"fo_{safe}"


def _encoding_format(rel: str) -> str:
    suf = Path(rel).suffix.lower()
    return _ENCODING.get(suf, "application/octet-stream")


def _sha256_file(path: Path) -> str | None:
    if not path.is_file():
        return None
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _file_object_block(rel: str) -> dict:
    path = ROOT / "hf_release" / rel
    block: dict = {
        "@type": "cr:FileObject",
        "@id": _file_object_id(rel),
        "name": Path(rel).name,
        "description": f"Published file `{rel}` in the Hugging Face dataset revision.",
        "contentUrl": RESOLVE_BASE + rel.replace("\\", "/"),
        "encodingFormat": _encoding_format(rel),
    }
    digest = _sha256_file(path)
    if digest:
        block["sha256"] = digest
    return block


def _record_sets_empty(core: dict) -> bool:
    rs = core.get("recordSet")
    return rs is None or (isinstance(rs, list) and len(rs) == 0)


def _distribution_ids(dist: object) -> set[str]:
    if not isinstance(dist, list):
        return set()
    out: set[str] = set()
    for item in dist:
        if isinstance(item, dict) and item.get("@id"):
            out.add(str(item["@id"]))
    return out


def _infer_csv_column(rel: str) -> str:
    if rel in _CSV_COLUMN_PRESET:
        return _CSV_COLUMN_PRESET[rel]
    path = ROOT / "hf_release" / rel
    if not path.is_file():
        return "instance_id"
    with path.open(encoding="utf-8", newline="") as handle:
        header = next(csv.reader(handle))
    if "instance_id" in header:
        return "instance_id"
    return header[0]


def _jsonl_record_set(rel: str, rs_name: str) -> dict:
    fid = _file_object_id(rel)
    return {
        "@type": "cr:RecordSet",
        "@id": f"rs_{fid}",
        "name": rs_name,
        "description": f"Records from `{rel}` (one JSON object per line).",
        "field": [
            {
                "@type": "cr:Field",
                "@id": f"{fid}/line",
                "name": "line",
                "description": "Raw JSONL line as text.",
                "dataType": "sc:Text",
                "source": {
                    "fileObject": {"@id": fid},
                    "extract": {"fileProperty": "lines"},
                },
            }
        ],
    }


def _csv_record_set(rel: str, rs_name: str, column: str) -> dict:
    fid = _file_object_id(rel)
    return {
        "@type": "cr:RecordSet",
        "@id": f"rs_{fid}",
        "name": rs_name,
        "description": f"Rows from `{rel}` (column `{column}`).",
        "field": [
            {
                "@type": "cr:Field",
                "@id": f"{fid}/{column}",
                "name": column,
                "dataType": "sc:Text",
                "source": {
                    "fileObject": {"@id": fid},
                    "extract": {"column": column},
                },
            }
        ],
    }


def _whole_file_text_record_set(rel: str, rs_name: str) -> dict:
    """One logical record per file (full UTF-8 payload as text)."""
    fid = _file_object_id(rel)
    return {
        "@type": "cr:RecordSet",
        "@id": f"rs_{fid}",
        "name": rs_name,
        "description": f"Whole-file text view of `{rel}` (UTF-8).",
        "field": [
            {
                "@type": "cr:Field",
                "@id": f"{fid}/content",
                "name": "content",
                "dataType": "sc:Text",
                "source": {
                    "fileObject": {"@id": fid},
                    "extract": {"fileProperty": "content"},
                },
            }
        ],
    }


def _augment_sparse_hub_croissant(core: dict) -> None:
    """Hub Croissant often omits `recordSet` for raw JSONL/CSV repos (only a `repo` FileObject).

    NeurIPS validation requires non-empty `distribution` and `recordSet`. When the Hub
    leaves `recordSet` empty, attach resolve/main FileObjects for published paths and
    minimal RecordSets so local validation matches the on-disk `hf_release/` layout.
    """
    if not _record_sets_empty(core):
        return

    dist = core.get("distribution")
    if not isinstance(dist, list):
        dist = []
        core["distribution"] = dist

    have_ids = _distribution_ids(dist)
    for rel in _DIST_REL_PATHS:
        block = _file_object_block(rel)
        if block["@id"] not in have_ids:
            dist.append(block)
            have_ids.add(block["@id"])

    record_sets: list[dict] = []
    record_sets.append(_jsonl_record_set("data/instances.jsonl", "instances"))
    record_sets.append(_jsonl_record_set("data/semantic_units.jsonl", "semantic_units"))
    record_sets.append(
        _jsonl_record_set("data/reference_obligations.jsonl", "reference_obligations")
    )
    record_sets.append(_jsonl_record_set("data/generated_packets.jsonl", "generated_packets"))
    record_sets.append(_jsonl_record_set("data/system_cards.jsonl", "system_cards"))
    record_sets.append(_jsonl_record_set("data/prompt_templates.jsonl", "prompt_templates"))

    record_sets.append(_csv_record_set("data/strict_results.csv", "strict_results", "instance_id"))
    record_sets.append(_csv_record_set("data/expanded_results.csv", "expanded_results", "instance_id"))
    col_co = _infer_csv_column("data/correction_overlays.csv")
    record_sets.append(_csv_record_set("data/correction_overlays.csv", "correction_overlays", col_co))
    record_sets.append(
        _csv_record_set("data/common_cell_instances.csv", "common_cell_instances", "instance_id")
    )
    record_sets.append(
        _csv_record_set(
            "data/common_cell_system_summary.csv",
            "common_cell_system_summary",
            "system",
        )
    )

    record_sets.append(_whole_file_text_record_set("data/human_agreement.json", "human_agreement"))

    core["recordSet"] = record_sets


def _normalize_context(core: dict) -> dict:
    """HF sometimes returns @context as a bare URL string; mlcroissant requires a dict."""
    ctx = core.get("@context")
    if isinstance(ctx, str):
        base = ctx.strip()
        core["@context"] = {
            "@vocab": base,
            "schema": "https://schema.org/",
            "sc": "https://schema.org/",
            "cr": "http://mlcommons.org/croissant/",
            "rai": "http://mlcommons.org/croissant/RAI/",
            "prov": "http://www.w3.org/ns/prov#",
        }
    elif isinstance(ctx, dict):
        ctx.setdefault("cr", "http://mlcommons.org/croissant/")
        ctx.setdefault("sc", "https://schema.org/")
        ctx.setdefault("schema", "https://schema.org/")
        ctx.setdefault("rai", "http://mlcommons.org/croissant/RAI/")
        ctx.setdefault("prov", "http://www.w3.org/ns/prov#")
    else:
        core["@context"] = {
            "@vocab": "https://schema.org/",
            "schema": "https://schema.org/",
            "sc": "https://schema.org/",
            "cr": "http://mlcommons.org/croissant/",
            "rai": "http://mlcommons.org/croissant/RAI/",
            "prov": "http://www.w3.org/ns/prov#",
        }
    out = core["@context"]
    if not isinstance(out, dict):
        raise SystemExit("internal error: @context is not a dict after normalization")
    return out


def main() -> int:
    core = json.loads(CORE.read_text(encoding="utf-8"))

    ctx = _normalize_context(core)

    if not core.get("url"):
        core["url"] = f"https://huggingface.co/datasets/{REPO_ID}"
    if not core.get("name"):
        core["name"] = REPO_ID

    if core.get("@type") in (None, "Dataset", "dataset"):
        core["@type"] = "Dataset"

    if not core.get("conformsTo"):
        core["conformsTo"] = "http://mlcommons.org/croissant/1.1"

    core["license"] = "https://creativecommons.org/licenses/by/4.0/"

    core["rai:dataLimitations"] = (
        "CTA-Bench v0.3 covers 84 classical algorithmic tasks across 12 families and "
        "should not be treated as representative of industrial verification, concurrency, "
        "distributed systems, numerical software, cryptographic implementations, or full "
        "Rust verification. The benchmark evaluates generated Lean-facing correctness "
        "obligations, not completed proofs or end-to-end verified implementations."
    )

    core["rai:dataBiases"] = (
        "The benchmark is intentionally biased toward classical textbook algorithms and "
        "English-language specifications. It overrepresents small, auditable correctness "
        "contracts and underrepresents domain-specific software, large codebases, non-English "
        "specifications, probabilistic programs, concurrent systems, and hardware/software "
        "co-verification."
    )

    core["rai:personalSensitiveInformation"] = (
        "The dataset contains no personal, demographic, medical, political, religious, "
        "or otherwise sensitive information. Human annotators are anonymized in released files."
    )

    core["rai:dataUseCases"] = (
        "Established use cases: evaluating semantic faithfulness of Lean-facing algorithmic "
        "correctness-obligation generation; analyzing vacuity, missing critical semantic units, "
        "code consistency, proof utility, and evidence provenance. Not established: model safety "
        "certification, proof-completion benchmarking, general verified-code synthesis, full Rust "
        "program verification, or ranking general-purpose LLMs."
    )

    core["rai:dataSocialImpact"] = (
        "Positive impact: CTA-Bench reduces false confidence in theorem-shaped but semantically "
        "weak artifacts. Misuse risk: it could be misread as a model leaderboard or evidence of "
        "full verification capability. Mitigations include strict/expanded evidence separation, "
        "diagnostic framing, limitations, and explicit distinction between semantic faithfulness "
        "and proof completion."
    )

    core["rai:hasSyntheticData"] = True

    core["rai:syntheticDataDescription"] = (
        "Benchmark instances and semantic units are human-authored or curated. Generated obligation "
        "packets are produced by LLM systems under recorded prompts, model metadata, sampling settings, "
        "and selection rules. Generated packets are evaluation artifacts, not ground-truth labels."
    )

    core["prov:wasDerivedFrom"] = (
        "CTA-Bench is derived from human-authored classical algorithm specifications, semantic-unit "
        "inventories, reference obligations, Rust code-context artifacts, Lean-facing scaffolds, and "
        "LLM-generated obligation packets. The annotation layer includes pipeline-normalized "
        "direct-adjudication exports, correction overlays, and an independent human strict-overlap pass "
        "over 274 direct rows."
    )

    core["prov:wasGeneratedBy"] = (
        "Preprocessing and pipeline materialization: deterministic exports from frozen review packets "
        "(`benchmark/v0.3/annotation/review_packets/.../packet.json`) into adjudicated records "
        "(`benchmark/v0.3/annotation/adjudicated_subset/pack.json`), obligation hygiene scoring, "
        "semantic-correction overlay merges from `annotation/external_review/semantic_corrections_v3.csv`, "
        "and paper strict/expanded table generation. Data collection / generation: LLM-generated "
        "obligation packets under fixed provider settings (temperature 0, registered prompt templates "
        "under `configs/prompts/`, recorded run manifests). Annotation: pipeline-derived "
        "direct-adjudication labels with `annotation_origin` discipline (strict headline view excludes "
        "`mapped_from_canonical` rows), independent human strict-overlap adjudication for headline "
        "metrics (`annotation/human_pass_v3/`), and exported disagreement logs. Human annotators are "
        "represented with anonymized identifiers in released artifacts."
    )

    _augment_sparse_hub_croissant(core)

    patch_obj: dict = {"@context": ctx}
    for k, v in core.items():
        if k.startswith("rai:"):
            patch_obj[k] = v
        if k in ("prov:wasDerivedFrom", "prov:wasGeneratedBy"):
            patch_obj[k] = v

    OUT.write_text(json.dumps(core, indent=2, ensure_ascii=False), encoding="utf-8")
    PATCH_OUT.write_text(json.dumps(patch_obj, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"Wrote {OUT}")
    print(f"Wrote {PATCH_OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
