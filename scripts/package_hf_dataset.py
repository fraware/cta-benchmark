#!/usr/bin/env python3
"""
Build hf_release/ for Hugging Face (NeurIPS 2026 E&D).

Reads frozen repo paths under benchmark/v0.3/, results/, annotation/, configs/,
appendix/, docs/, schemas/, lean/. Rebuilds hf_release/ deterministically.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import stat
import subprocess
import sys
from collections.abc import Callable
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

ARTIFACT_TREE_IGNORE = shutil.ignore_patterns(
    ".env",
    ".git",
    ".lake",
    "__pycache__",
    "*.pyc",
    ".pytest_cache",
    "target",
    ".venv",
    "venv",
    "*.egg-info",
)


def force_rmtree(target: Path) -> None:
    """Windows-friendly removal (chmod read-only files under .lake, etc.)."""

    def _onerror(func: Callable[..., object], path: str, exc_info: object) -> None:
        try:
            os.chmod(path, stat.S_IWRITE)
        except OSError:
            pass
        func(path)

    if target.exists():
        shutil.rmtree(target, onerror=_onerror)
V3 = ROOT / "benchmark" / "v0.3"
PACK_JSON = V3 / "annotation" / "adjudicated_subset" / "pack.json"
REVIEW_ROOT = V3 / "annotation" / "review_packets"
MANIFEST = ROOT / "benchmark" / "manifest.jsonl"
SYSTEM_CARDS = ROOT / "configs" / "paper_system_cards_v03.json"

PACKET_PATH_RE = re.compile(
    r"benchmark/v0\.3/annotation/review_packets/([^/\s]+)/([^/\s]+)/packet\.json"
)
PLACEHOLDER_RE = re.compile(r"\{\{([a-zA-Z0-9_]+)\}\}")


def family_from_id(instance_id: str) -> str:
    m = re.match(r"^(.*)_(\d{3})$", instance_id)
    if not m:
        return instance_id
    return m.group(1)


def load_split_by_instance() -> dict[str, str]:
    out: dict[str, str] = {}
    for name in ("dev", "eval"):
        p = V3 / "splits" / f"{name}.json"
        if not p.is_file():
            continue
        data = json.loads(p.read_text(encoding="utf-8"))
        for iid in data.get("instance_ids") or []:
            out[str(iid)] = name
    if MANIFEST.is_file():
        for line in MANIFEST.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if not line:
                continue
            row = json.loads(line)
            iid = row.get("instance_id")
            sp = row.get("split")
            if iid and sp and str(iid) not in out:
                out[str(iid)] = str(sp)
    return out


def parse_template_id(notes: str) -> tuple[str, str] | None:
    m = PACKET_PATH_RE.search(notes or "")
    if not m:
        return None
    return m.group(1), m.group(2)


def write_jsonl(path: Path, rows: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False) + "\n")


def build_instances(split_map: dict[str, str]) -> list[dict]:
    rows: list[dict] = []
    for inst in sorted(V3.glob("instances/**/instance.json")):
        data = json.loads(inst.read_text(encoding="utf-8"))
        iid = data["instance_id"]
        informal = data.get("informal_statement") or {}
        if isinstance(informal, dict):
            text = informal.get("text") or ""
            pre = informal.get("preconditions") or []
            req = informal.get("required_properties") or []
            edge = informal.get("edge_cases") or []
        else:
            text, pre, req, edge = str(informal), [], [], []
        rel = inst.parent.relative_to(V3)
        rows.append(
            {
                "instance_id": iid,
                "family": family_from_id(iid),
                "domain": data.get("domain", ""),
                "split": split_map.get(iid, ""),
                "difficulty": data.get("difficulty", ""),
                "informal_statement": text,
                "preconditions": pre,
                "required_properties": req,
                "edge_cases": edge,
                "instance_path": f"benchmark/v0.3/{rel.as_posix()}",
            }
        )
    return rows


def build_semantic_units() -> list[dict]:
    rows: list[dict] = []
    for su_path in sorted(V3.glob("instances/**/semantic_units.json")):
        data = json.loads(su_path.read_text(encoding="utf-8"))
        iid = data["instance_id"]
        for u in data.get("units") or []:
            rows.append(
                {
                    "instance_id": iid,
                    "semantic_unit_id": u.get("id", ""),
                    "description": u.get("description", ""),
                    "criticality": u.get("criticality", ""),
                    "common_failure_modes": u.get("common_failure_modes") or [],
                }
            )
    return rows


def build_reference_obligations() -> list[dict]:
    rows: list[dict] = []
    for ref_path in sorted(V3.glob("instances/**/reference_obligations.json")):
        data = json.loads(ref_path.read_text(encoding="utf-8"))
        iid = data["instance_id"]
        for o in data.get("obligations") or []:
            rows.append(
                {
                    "instance_id": iid,
                    "obligation_id": o.get("id", ""),
                    "kind": o.get("kind", ""),
                    "lean_statement": o.get("lean_statement", ""),
                    "nl_gloss": o.get("nl_gloss", ""),
                    "linked_semantic_units": o.get("linked_semantic_units") or [],
                    "importance": o.get("importance", ""),
                    "proof_relevance": o.get("proof_relevance", ""),
                }
            )
    return rows


def build_generated_packets() -> list[dict]:
    pack = json.loads(PACK_JSON.read_text(encoding="utf-8"))
    records = pack.get("records") or []
    out: list[dict] = []
    for rec in records:
        notes = rec.get("annotator_notes") or ""
        parsed = parse_template_id(notes)
        if not parsed:
            raise SystemExit(f"could not parse template packet path from notes: {notes[:200]!r}")
        system_folder, template_id = parsed
        system_id = rec.get("system_id") or ""
        if system_folder != system_id:
            raise SystemExit(
                f"system_id mismatch: notes has {system_folder!r} record has {system_id!r}"
            )
        pkt_path = REVIEW_ROOT / system_id / template_id / "packet.json"
        if not pkt_path.is_file():
            raise SystemExit(f"missing packet: {pkt_path}")
        packet = json.loads(pkt_path.read_text(encoding="utf-8"))
        gen = packet.get("generated_obligations") or []
        by_index: dict[int, dict] = {}
        for ob in gen:
            idx = int(ob.get("index", len(by_index)))
            by_index[idx] = ob
        origin = rec.get("annotation_origin", "")
        eval_iid = rec.get("instance_id", "")
        for idx in sorted(by_index):
            ob = by_index[idx]
            out.append(
                {
                    "instance_id": eval_iid,
                    "system_id": system_id,
                    "obligation_index": idx,
                    "kind": ob.get("kind", ""),
                    "lean_statement": ob.get("lean_statement", ""),
                    "nl_gloss": ob.get("nl_gloss", ""),
                    "linked_semantic_units": ob.get("linked_semantic_units") or [],
                    "raw_source": ob.get("raw_source", ""),
                    "annotation_origin": origin,
                }
            )
    return out


def visible_fields_from_prompt(path: Path) -> list[str]:
    data = json.loads(path.read_text(encoding="utf-8"))
    body = data.get("body") or ""
    names = sorted(set(PLACEHOLDER_RE.findall(body)))
    return names


def build_prompt_templates() -> list[dict]:
    systems = [
        "full_method_v1",
        "code_only_v1",
        "naive_concat_v1",
        "text_only_v1",
    ]
    rows: list[dict] = []
    for sid in systems:
        p = ROOT / "configs" / "prompts" / f"{sid}.json"
        raw = p.read_bytes()
        h = hashlib.sha256(raw).hexdigest()
        rel = p.relative_to(ROOT).as_posix()
        rows.append(
            {
                "system_id": sid,
                "template_sha256": h,
                "template_path": rel,
                "visible_context_fields": visible_fields_from_prompt(p),
            }
        )
    return rows


def write_hf_readme(out: Path) -> None:
    body = """---
license: cc-by-4.0
language:
- en
pretty_name: CTA-Bench v0.3
task_categories:
- text-generation
tags:
- benchmark
- formal-verification
- lean
- theorem-proving
- semantic-faithfulness
- ai-evaluation
size_categories:
- n<1K
---

# CTA-Bench v0.3

CTA-Bench evaluates statement-layer semantic faithfulness in Lean-facing algorithmic correctness obligations.

## Summary

CTA-Bench v0.3 contains 84 algorithmic correctness-obligation instances across 12 classical algorithm families, 294 critical semantic units, reference obligations, code-context artifacts, generated Lean-facing obligation packets, strict and expanded result views, correction overlays, and human strict-overlap agreement reports.

## Evidence views

The strict direct view contains 274 system-instance rows over all 84 instances and excludes mapped-from-canonical rows. It is the source of headline paper claims.

The expanded grid contains 336 rows over the same 84 instances and includes 114 mapped-from-canonical rows. It is provided for appendix robustness and grid inspection only.

## What this dataset evaluates

CTA-Bench evaluates whether generated Lean-facing correctness obligations preserve the semantic contract they purport to express before proof search begins.

## What this dataset does not evaluate

- Full Rust implementation verification (CTA-Bench is not full Rust verification).
- Whole-benchmark Lean proof completion (not a proof-completion benchmark).
- General model leaderboard ranking (not a model leaderboard).
- Certification of model safety or correctness.
- A pure Rust-only ablation in v0.3.

## Main files

- `data/instances.jsonl`
- `data/semantic_units.jsonl`
- `data/reference_obligations.jsonl`
- `data/generated_packets.jsonl`
- `data/strict_results.csv`
- `data/expanded_results.csv`
- `data/human_agreement.json`
- `data/correction_overlays.csv`
- `data/system_cards.jsonl`
- `data/prompt_templates.jsonl`
- `data/common_cell_instances.csv`
- `data/common_cell_system_summary.csv`

## Full artifact

The `artifact/` directory contains the frozen benchmark, annotations, result exports, scripts, schemas, documentation, and selected Lean proof-facing subset.

## Reproduction

See the GitHub repository:

https://github.com/fraware/cta-benchmark

Recommended checks:

```bash
cargo build --workspace
cargo test --workspace --all-targets
python scripts/package_hf_dataset.py
python scripts/paper_common_cell_strict.py
cd lean && lake build
```

## License note

This Hugging Face dataset bundle is released under **CC-BY-4.0** (see `LICENSE` in this folder). The linked GitHub **software** repository remains under its own repository license (MIT at the repository root as of this release).

## Responsible AI

See `rai_statement.md` and `croissant.json`.

## Citation

See `CITATION.cff`.
"""
    out.write_text(body, encoding="utf-8")


def write_hf_citation_cff(out: Path) -> None:
    text = """cff-version: 1.2.0
title: "CTA-Bench: A Benchmark for Statement-Layer Faithfulness in Lean Correctness Obligations"
message: "If you use CTA-Bench, please cite the NeurIPS 2026 paper and this dataset."
type: dataset
authors:
  - family-names: "Anonymous"
    given-names: "Authors"
repository-code: "https://github.com/fraware/cta-benchmark"
url: "https://huggingface.co/datasets/fraware/cta-bench"
version: "0.3"
date-released: "2026-05-04"
license: "CC-BY-4.0"
"""
    out.write_text(text, encoding="utf-8")


def write_hf_rai_statement(out: Path) -> None:
    text = """# Responsible AI Statement for CTA-Bench v0.3

## Data limitations

CTA-Bench v0.3 covers 84 classical algorithmic tasks across 12 families. It should not be treated as representative of industrial verification, concurrency, distributed systems, numerical software, cryptographic implementations, or full Rust verification. The benchmark evaluates generated Lean-facing correctness obligations, not completed proofs or end-to-end verified implementations.

## Biases

The benchmark is intentionally biased toward classical textbook algorithms and English-language specifications. It overrepresents small, auditable correctness contracts and underrepresents domain-specific software, large codebases, non-English specifications, probabilistic programs, concurrent systems, and hardware/software co-verification.

## Personal and sensitive information

The dataset contains no personal, demographic, medical, political, religious, or otherwise sensitive information. Human annotators are anonymized in released files.

## Intended use cases

CTA-Bench is intended for evaluating semantic faithfulness of Lean-facing algorithmic correctness-obligation generation and for analyzing vacuity, missing critical semantic units, code consistency, proof utility, and evidence provenance.

## Out-of-scope use cases

CTA-Bench should not be used for model safety certification, proof-completion benchmarking, general verified-code synthesis, full Rust program verification, or ranking general-purpose LLMs.

## Synthetic data

CTA-Bench includes synthetic/generated content. Benchmark instances and semantic units are human-authored or curated. Generated obligation packets are produced by LLM systems under recorded prompts, model metadata, sampling settings, and selection rules. Generated packets are evaluation artifacts, not ground-truth labels.

## Social impact

Positive impact: CTA-Bench reduces false confidence in theorem-shaped but semantically weak artifacts.

Misuse risk: it could be misread as a model leaderboard or as evidence of full verification capability.

Mitigations: strict/expanded evidence separation, diagnostic framing, explicit limitations, and distinction between semantic faithfulness and proof completion.

## Annotation provenance

CTA-Bench includes obligation-level labels, pipeline-normalized direct-adjudication exports, correction overlays, and an independent human strict-overlap pass over 274 direct rows with agreement statistics.
"""
    out.write_text(text, encoding="utf-8")


def write_hf_license_cc_by(out: Path) -> None:
    text = """Creative Commons Attribution 4.0 International Public License (CC-BY-4.0)

This dataset bundle (metadata and released tabular/JSONL exports under hf_release/data/
and companion documentation in this folder) is licensed under CC-BY-4.0.

You are free to:
- Share: copy and redistribute the material in any medium or format
- Adapt: remix, transform, and build upon the material for any purpose, even commercially

Under the following terms:
- Attribution: You must give appropriate credit, provide a link to the license,
  and indicate if changes were made. You may do so in any reasonable manner, but
  not in any way that suggests the licensor endorses you or your use.

No additional restrictions: You may not apply legal terms or technological measures
that legally restrict others from doing anything the license permits.

Full legal text: https://creativecommons.org/licenses/by/4.0/legalcode
Human-readable summary: https://creativecommons.org/licenses/by/4.0/
"""
    out.write_text(text, encoding="utf-8")


def write_dataset_card(out: Path) -> None:
    out.write_text(
        "# Dataset card (CTA-Bench v0.3)\n\n"
        "The Hugging Face dataset card for the Hub is `README.md` in this directory "
        "(Hugging Face uses `README.md` as the dataset card).\n\n"
        "For the broader benchmark artifact and regeneration commands, see the "
        "[GitHub repository](https://github.com/fraware/cta-benchmark).\n",
        encoding="utf-8",
    )


def write_datasheet(out: Path) -> None:
    out.write_text(
        "# Datasheet (CTA-Bench v0.3 HF export)\n\n"
        "## Composition\n\n"
        "- 84 instances across 12 classical algorithm families (versioned under "
        "`benchmark/v0.3` in the GitHub repo).\n"
        "- Machine-readable tables in `data/` (JSONL and CSV) plus a full audit "
        "mirror under `artifact/`.\n\n"
        "## Collection / annotation\n\n"
        "Semantic units and reference obligations are human-authored or curated. "
        "Generated obligation packets are produced by LLM systems under recorded "
        "prompts and selection rules; see `artifact/docs/reviewer_map.md` on GitHub.\n\n"
        "## Ethics and limitations\n\n"
        "See `rai_statement.md` and `artifact/docs/reviewer_map.md` (human-gold context "
        "and metric layers; included under `artifact/docs/`).\n\n"
        "## Maintenance\n\n"
        "Prefer rebuilding `hf_release/` with `python scripts/package_hf_dataset.py` "
        "from a tagged freeze branch (for example `neurips2026-cta-freeze`) so paths "
        "and hashes stay aligned with the paper.\n",
        encoding="utf-8",
    )


def copytree_merge(
    src: Path,
    dst: Path,
    *,
    ignore: Callable[[str, list[str]], set[str]] | None = None,
) -> None:
    if not src.is_dir():
        raise SystemExit(f"missing directory: {src}")
    dst.parent.mkdir(parents=True, exist_ok=True)
    kwargs: dict = {"dirs_exist_ok": True}
    if ignore is not None:
        kwargs["ignore"] = ignore
    shutil.copytree(src, dst, **kwargs)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--out",
        type=Path,
        default=ROOT / "hf_release",
        help="Output directory (default: ./hf_release)",
    )
    ap.add_argument(
        "--no-clean",
        action="store_true",
        help="Do not delete existing output directory before build.",
    )
    args = ap.parse_args()
    out: Path = args.out

    if not PACK_JSON.is_file():
        raise SystemExit(f"missing adjudication pack: {PACK_JSON}")

    if not args.no_clean and out.exists():
        force_rmtree(out)

    data_dir = out / "data"
    artifact_dir = out / "artifact"
    data_dir.mkdir(parents=True, exist_ok=True)
    artifact_dir.mkdir(parents=True, exist_ok=True)

    split_map = load_split_by_instance()

    write_jsonl(data_dir / "instances.jsonl", build_instances(split_map))
    write_jsonl(data_dir / "semantic_units.jsonl", build_semantic_units())
    write_jsonl(data_dir / "reference_obligations.jsonl", build_reference_obligations())
    write_jsonl(data_dir / "generated_packets.jsonl", build_generated_packets())

    strict_src = ROOT / "results" / "paper_strict_instance_level.csv"
    expanded_src = ROOT / "results" / "appendix_mapped_evidence" / "instance_level.csv"
    human_src = ROOT / "annotation" / "human_pass_v3" / "agreement_report_human_strict_all.json"
    corrections_src = ROOT / "annotation" / "external_review" / "semantic_corrections_v3.csv"

    for src, name in (
        (strict_src, "strict_results.csv"),
        (expanded_src, "expanded_results.csv"),
    ):
        if not src.is_file():
            raise SystemExit(f"missing required file: {src}")
        shutil.copy2(src, data_dir / name)

    if not human_src.is_file():
        raise SystemExit(f"missing: {human_src}")
    shutil.copy2(human_src, data_dir / "human_agreement.json")

    if not corrections_src.is_file():
        raise SystemExit(f"missing: {corrections_src}")
    shutil.copy2(corrections_src, data_dir / "correction_overlays.csv")

    if not SYSTEM_CARDS.is_file():
        raise SystemExit(f"missing: {SYSTEM_CARDS}")
    cards = json.loads(SYSTEM_CARDS.read_text(encoding="utf-8"))
    with (data_dir / "system_cards.jsonl").open("w", encoding="utf-8") as f:
        for row in cards:
            f.write(json.dumps(row, ensure_ascii=False) + "\n")

    write_jsonl(data_dir / "prompt_templates.jsonl", build_prompt_templates())

    cmd = [
        sys.executable,
        str(ROOT / "scripts" / "paper_common_cell_strict.py"),
        "--strict-csv",
        str(strict_src),
        "--out-instances",
        str(data_dir / "common_cell_instances.csv"),
        "--out-system-summary",
        str(data_dir / "common_cell_system_summary.csv"),
    ]
    subprocess.run(cmd, check=True)

    write_hf_readme(out / "README.md")
    write_hf_license_cc_by(out / "LICENSE")
    write_hf_citation_cff(out / "CITATION.cff")
    write_hf_rai_statement(out / "rai_statement.md")
    write_dataset_card(out / "dataset_card.md")
    write_datasheet(out / "datasheet.md")

    copytree_merge(V3, artifact_dir / "benchmark", ignore=ARTIFACT_TREE_IGNORE)
    copytree_merge(ROOT / "annotation", artifact_dir / "annotation", ignore=ARTIFACT_TREE_IGNORE)
    copytree_merge(ROOT / "results", artifact_dir / "results", ignore=ARTIFACT_TREE_IGNORE)
    copytree_merge(ROOT / "appendix", artifact_dir / "appendix", ignore=ARTIFACT_TREE_IGNORE)
    copytree_merge(ROOT / "docs", artifact_dir / "docs", ignore=ARTIFACT_TREE_IGNORE)
    copytree_merge(ROOT / "scripts", artifact_dir / "scripts", ignore=ARTIFACT_TREE_IGNORE)
    copytree_merge(ROOT / "schemas", artifact_dir / "schemas", ignore=ARTIFACT_TREE_IGNORE)
    copytree_merge(
        ROOT / "lean",
        artifact_dir / "lean_subset",
        ignore=shutil.ignore_patterns(".lake"),
    )

    reports_dir = ROOT / "reports"
    art_reports = artifact_dir / "reports"
    art_reports.mkdir(parents=True, exist_ok=True)
    croissant_report = reports_dir / "croissant_validation_2026.md"
    if croissant_report.is_file():
        shutil.copy2(croissant_report, art_reports / croissant_report.name)

    print(f"Wrote Hugging Face release under {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
