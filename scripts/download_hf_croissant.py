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
    out = ROOT / "hf_release" / "croissant_core.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(response.json(), indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"Wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
