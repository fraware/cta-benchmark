#!/usr/bin/env python3
"""Upload hf_release/ to Hugging Face dataset fraware/cta-bench."""

from __future__ import annotations

from pathlib import Path

from huggingface_hub import HfApi

ROOT = Path(__file__).resolve().parents[1]
REPO_ID = "fraware/cta-bench"
FOLDER = ROOT / "hf_release"


def main() -> int:
    if not FOLDER.is_dir():
        raise SystemExit(
            f"missing folder: {FOLDER} (run scripts/package_hf_dataset.py first)"
        )

    api = HfApi()
    api.create_repo(
        repo_id=REPO_ID,
        repo_type="dataset",
        private=False,
        exist_ok=True,
    )
    api.upload_folder(
        folder_path=str(FOLDER),
        repo_id=REPO_ID,
        repo_type="dataset",
        commit_message="Release CTA-Bench v0.3 NeurIPS 2026 dataset artifact",
    )
    croissant = FOLDER / "croissant.json"
    if croissant.is_file():
        api.upload_file(
            path_or_fileobj=str(croissant),
            path_in_repo="croissant.json",
            repo_id=REPO_ID,
            repo_type="dataset",
            commit_message="Add NeurIPS 2026 Croissant metadata with RAI fields",
        )
    print(f"Uploaded {FOLDER} to https://huggingface.co/datasets/{REPO_ID}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
