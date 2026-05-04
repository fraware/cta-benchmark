#!/usr/bin/env python3
"""Verify fraware/cta-bench lists required NeurIPS E&D paths (Hub API; no local hf_release/)."""

from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.request

from huggingface_hub import HfApi, get_token

REPO_ID = "fraware/cta-bench"
CROISSANT_API = f"https://huggingface.co/api/datasets/{REPO_ID}/croissant"

REQUIRED_FILES = frozenset(
    {
        "README.md",
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
    }
)


def _token() -> str | None:
    return (
        (os.environ.get("HF_TOKEN") or "").strip()
        or (os.environ.get("HUGGINGFACE_HUB_TOKEN") or "").strip()
        or get_token()
    )


def _hub_croissant_payload() -> dict:
    req = urllib.request.Request(
        CROISSANT_API,
        headers={"User-Agent": "cta-benchmark-check"},
    )
    with urllib.request.urlopen(req, timeout=120) as resp:
        return json.loads(resp.read().decode("utf-8"))


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--print-hub-croissant-sizes",
        action="store_true",
        help=(
            "Print Hub Croissant API distribution/recordSet lengths (informational)."
        ),
    )
    args = ap.parse_args()

    tok = _token()
    api = HfApi(token=tok) if tok else HfApi()
    try:
        files = set(api.list_repo_files(REPO_ID, repo_type="dataset"))
    except Exception as e:
        raise SystemExit(f"list_repo_files failed: {e}") from e

    missing = sorted(REQUIRED_FILES - files)
    if missing:
        joined = "\n  ".join(missing)
        raise SystemExit(f"Hub dataset {REPO_ID} is missing required paths:\n  {joined}")

    nreq = len(REQUIRED_FILES)
    print(f"Remote check OK: {len(files)} paths; all {nreq} required files present.")

    if args.print_hub_croissant_sizes:
        try:
            payload = _hub_croissant_payload()
        except Exception as e:
            print(f"warning: could not fetch Hub Croissant API: {e}", file=sys.stderr)
            return 0
        if isinstance(payload, dict) and payload.get("error"):
            err = payload.get("error")
            print(f"warning: Hub Croissant API error: {err}", file=sys.stderr)
            return 0
        dist = payload.get("distribution") or []
        rs = payload.get("recordSet") or []
        dlen = len(dist) if isinstance(dist, list) else 0
        rlen = len(rs) if isinstance(rs, list) else 0
        print(f"Hub GET /croissant: distribution={dlen} recordSet={rlen}")
        print(
            "note: Raw JSONL/CSV repos often expose an empty Hub recordSet; "
            "merged hf_release/croissant.json is authoritative after "
            "add_rai_to_croissant.py."
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
