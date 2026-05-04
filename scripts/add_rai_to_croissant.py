#!/usr/bin/env python3
"""
Merge Responsible AI fields plus provenance into HF Croissant core.

Writes `croissant.json` (core from Hub + `rai:*`, `prov:wasDerivedFrom`, `prov:wasGeneratedBy`,
`rai:syntheticDataDescription`, normalized `@context`, CC-BY-4.0 license) and
`croissant_rai_patch.json` (RAI + both `prov:*` keys and `@context` only).
"""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORE = ROOT / "hf_release" / "croissant_core.json"
OUT = ROOT / "hf_release" / "croissant.json"
PATCH_OUT = ROOT / "hf_release" / "croissant_rai_patch.json"


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
