#!/usr/bin/env python3
"""Validate hf_release/ for NeurIPS E&D hosting (files, strict/expanded invariants, Croissant keys)."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HF = ROOT / "hf_release"

REQUIRED = [
    "README.md",
    "LICENSE",
    "CITATION.cff",
    "rai_statement.md",
    "croissant.json",
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
]

CROISSANT_HF_CORE_KEYS = [
    "@context",
    "@type",
    "name",
    "url",
    "license",
    "conformsTo",
    "distribution",
    "recordSet",
]

CROISSANT_RAI_PROV_KEYS = [
    "rai:dataLimitations",
    "rai:dataBiases",
    "rai:personalSensitiveInformation",
    "rai:dataUseCases",
    "rai:dataSocialImpact",
    "rai:hasSyntheticData",
    "rai:syntheticDataDescription",
    "prov:wasDerivedFrom",
    "prov:wasGeneratedBy",
]


def main() -> int:
    try:
        import pandas as pd
    except ImportError as e:
        raise SystemExit(f"pandas required: {e}") from e

    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--allow-minimal-croissant",
        action="store_true",
        help=(
            "Allow Hugging Face Croissant core without non-empty recordSet/distribution "
            "(local smoke tests only; not submission-ready)."
        ),
    )
    args = ap.parse_args()
    allow_minimal = args.allow_minimal_croissant

    missing = [p for p in REQUIRED if not (HF / p).exists()]
    if missing:
        raise SystemExit(f"Missing required files: {missing}")

    strict = pd.read_csv(HF / "data" / "strict_results.csv")
    if len(strict) != 274:
        raise SystemExit(f"Expected 274 strict rows, got {len(strict)}")
    if not strict["annotation_origin"].eq("direct_adjudicated").all():
        bad = strict.loc[~strict["annotation_origin"].eq("direct_adjudicated"), "annotation_origin"]
        raise SystemExit(f"strict_results.csv must be all direct_adjudicated, found: {bad.unique()}")
    if strict["instance_id"].nunique() != 84:
        raise SystemExit(f"Expected 84 unique instance_id in strict, got {strict['instance_id'].nunique()}")

    expanded = pd.read_csv(HF / "data" / "expanded_results.csv")
    if len(expanded) != 336:
        raise SystemExit(f"Expected 336 expanded rows, got {len(expanded)}")

    agreement = json.loads((HF / "data" / "human_agreement.json").read_text(encoding="utf-8"))
    if agreement.get("n_rows") != 274:
        raise SystemExit(f"agreement n_rows expected 274, got {agreement.get('n_rows')}")
    if agreement.get("n_unique_instance_ids") != 84:
        raise SystemExit(
            f"agreement n_unique_instance_ids expected 84, got {agreement.get('n_unique_instance_ids')}"
        )
    if agreement.get("n_mapped_from_canonical") != 0:
        raise SystemExit(
            "agreement n_mapped_from_canonical expected 0 for strict headline overlap export, "
            f"got {agreement.get('n_mapped_from_canonical')}"
        )

    common_inst = pd.read_csv(HF / "data" / "common_cell_instances.csv")
    if len(common_inst) != 48:
        raise SystemExit(f"Expected 48 common-cell instance rows, got {len(common_inst)}")

    sysum = pd.read_csv(HF / "data" / "common_cell_system_summary.csv")
    if len(sysum) != 4:
        raise SystemExit(f"Expected 4 common-cell system summary rows, got {len(sysum)}")
    got_sys = set(sysum["system"].astype(str))
    exp_sys = {"code_only_v1", "full_method_v1", "naive_concat_v1", "text_only_v1"}
    if got_sys != exp_sys:
        raise SystemExit(f"common_cell_system_summary systems mismatch: {got_sys} vs {exp_sys}")

    croissant = json.loads((HF / "croissant.json").read_text(encoding="utf-8"))

    absent_core = [k for k in CROISSANT_HF_CORE_KEYS if k not in croissant]
    if absent_core and not allow_minimal:
        raise SystemExit(
            "Croissant missing Hugging Face core fields: "
            f"{absent_core}. "
            "The Hub Croissant API usually returns full metadata only after the dataset "
            "contains machine-readable files the viewer can convert (for example the `data/` "
            "JSONL/CSV layer) and `croissant_core.json` was re-downloaded from "
            "`https://huggingface.co/api/datasets/fraware/cta-bench/croissant`. "
            "Re-run `python scripts/upload_hf_dataset.py`, then curl-download core Croissant, then "
            "`python scripts/add_rai_to_croissant.py`. "
            "For local-only smoke tests: pass --allow-minimal-croissant."
        )

    absent_rai = [k for k in CROISSANT_RAI_PROV_KEYS if k not in croissant]
    if absent_rai:
        raise SystemExit(f"Missing required RAI / provenance Croissant keys: {absent_rai}")

    if not allow_minimal:
        rs = croissant.get("recordSet")
        dist = croissant.get("distribution")
        if not rs or (isinstance(rs, list) and len(rs) == 0):
            raise SystemExit(
                "Croissant `recordSet` is empty or absent after merge. "
                "NeurIPS requires a complete Croissant file; upload full `hf_release/data/` to Hugging Face "
                "and re-fetch core Croissant from the Hub API."
            )
        if not dist or (isinstance(dist, list) and len(dist) == 0):
            raise SystemExit(
                "Croissant `distribution` is empty or absent after merge. "
                "Upload the dataset files and re-fetch Croissant from the Hub API."
            )

    readme = (HF / "README.md").read_text(encoding="utf-8")
    rlo = readme.lower()
    if "not full rust" not in rlo:
        raise SystemExit("README.md should state CTA-Bench is not full Rust verification.")
    if "proof completion" not in rlo:
        raise SystemExit("README.md should clarify whole-benchmark proof completion is out of scope.")
    if "not a model leaderboard" not in rlo:
        raise SystemExit("README.md should state the benchmark is not a model leaderboard.")

    scan = subprocess.run(
        [sys.executable, str(ROOT / "scripts" / "scan_hf_release_secrets.py"), "--root", str(HF)],
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        timeout=600,
    )
    if scan.returncode != 0:
        raise SystemExit(
            "hf_release secret scan failed (includes artifact/). "
            f"stdout:\n{scan.stdout}\nstderr:\n{scan.stderr}"
        )

    print("NeurIPS artifact validation passed.")

    mlc = shutil.which("mlcroissant")
    if not mlc and sys.platform == "win32":
        cand = Path(sys.executable).parent / "Scripts" / "mlcroissant.exe"
        if cand.is_file():
            mlc = str(cand)
    if mlc:
        try:
            r = subprocess.run(
                [mlc, "validate", "--jsonld", str(HF / "croissant.json")],
                cwd=str(ROOT),
                capture_output=True,
                text=True,
                timeout=300,
            )
            if r.returncode != 0:
                print("warning: mlcroissant validate failed (non-fatal for local dev).")
                if r.stdout:
                    print(r.stdout)
                if r.stderr:
                    print(r.stderr, file=sys.stderr)
            else:
                print("mlcroissant validate: OK")
        except subprocess.TimeoutExpired:
            print("warning: mlcroissant validate timed out; skipped.")
    else:
        print("warning: mlcroissant CLI not on PATH; skipped optional Croissant schema validation.")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
