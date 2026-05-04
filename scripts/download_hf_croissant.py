#!/usr/bin/env python3
"""Download Croissant JSON-LD from Hugging Face for fraware/cta-bench."""

from __future__ import annotations

import json
from pathlib import Path

import requests

ROOT = Path(__file__).resolve().parents[1]
REPO_ID = "fraware/cta-bench"
URL = f"https://huggingface.co/api/datasets/{REPO_ID}/croissant"


def main() -> int:
    response = requests.get(URL, timeout=120)
    response.raise_for_status()
    payload = response.json()
    if isinstance(payload, dict) and payload.get("error"):
        raise SystemExit(
            "Hugging Face Croissant API returned an error payload. "
            "Confirm the dataset repo exists and is public, then retry.\n"
            f"  URL: {URL}\n  error: {payload.get('error')}"
        )
    out = ROOT / "hf_release" / "croissant_core.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(
        json.dumps(payload, indent=2, ensure_ascii=False),
        encoding="utf-8",
    )
    print(f"Wrote {out}")
    dist = payload.get("distribution") if isinstance(payload, dict) else None
    rs = payload.get("recordSet") if isinstance(payload, dict) else None
    if not dist or (isinstance(dist, list) and len(dist) == 0):
        print(
            "note: Hub Croissant has no `distribution` yet. After the first successful "
            "upload of `hf_release/` (including `data/`), re-run this script so "
            "`distribution` / `recordSet` can populate."
        )
    if not rs or (isinstance(rs, list) and len(rs) == 0):
        print(
            "note: Hub Croissant has no `recordSet` (common for raw JSONL/CSV repos). "
            "Run `python scripts/add_rai_to_croissant.py` after download; it augments "
            "`croissant.json` with resolve/main FileObjects and RecordSets."
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
