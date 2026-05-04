#!/usr/bin/env python3
"""Upload hf_release/ to Hugging Face dataset fraware/cta-bench."""

from __future__ import annotations

import os
from pathlib import Path

from huggingface_hub import HfApi, get_token
from huggingface_hub.errors import HfHubHTTPError

ROOT = Path(__file__).resolve().parents[1]
REPO_ID = "fraware/cta-bench"
FOLDER = ROOT / "hf_release"

_AUTH_HELP = (
    "No Hugging Face credentials found.\n"
    "  • Interactive: run `hf auth login`.\n"
    "    (Legacy `huggingface-cli login` was removed upstream.)\n"
    "  • Headless / CI: set `HF_TOKEN` or `HUGGINGFACE_HUB_TOKEN` to a token with write access\n"
    "    to https://huggingface.co/datasets/fraware/cta-bench (org `fraware`).\n"
)


def _resolve_token() -> str | None:
    return (
        (os.environ.get("HF_TOKEN") or "").strip()
        or (os.environ.get("HUGGINGFACE_HUB_TOKEN") or "").strip()
        or get_token()
    )


def _http_status(exc: BaseException) -> int | None:
    resp = getattr(exc, "response", None)
    return getattr(resp, "status_code", None) if resp is not None else None


def main() -> int:
    if not FOLDER.is_dir():
        raise SystemExit(
            f"missing folder: {FOLDER} "
            "(run scripts/package_hf_dataset.py first)"
        )

    token = _resolve_token()
    if not token:
        raise SystemExit(_AUTH_HELP)

    api = HfApi(token=token)
    try:
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
    except HfHubHTTPError as e:
        code = _http_status(e)
        if code == 401:
            raise SystemExit(
                "Hugging Face returned 401 Unauthorized (invalid or expired token).\n"
                "  Re-authenticate with `hf auth login`, or refresh `HF_TOKEN`.\n"
                "  If uploads still fail, clear a stale HF CLI token cache and log in again.\n"
                f"  Details: {e}"
            ) from e
        if code == 403:
            raise SystemExit(
                "Hugging Face returned 403 Forbidden. The token cannot create or write this "
                "dataset.\n"
                "  Confirm you are in org `fraware` with write access to `fraware/cta-bench`.\n"
                f"  Details: {e}"
            ) from e
        raise

    croissant = FOLDER / "croissant.json"
    if croissant.is_file():
        try:
            api.upload_file(
                path_or_fileobj=str(croissant),
                path_in_repo="croissant.json",
                repo_id=REPO_ID,
                repo_type="dataset",
                commit_message="Add NeurIPS 2026 Croissant metadata with RAI fields",
            )
        except HfHubHTTPError as e:
            code = _http_status(e)
            if code in (401, 403):
                raise SystemExit(
                    "Upload of `croissant.json` failed after the folder upload step.\n"
                    "  Check `hf auth login`, `HF_TOKEN`, and org write access.\n"
                    f"  Details: {e}"
                ) from e
            raise

    print(f"Uploaded {FOLDER} to https://huggingface.co/datasets/{REPO_ID}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
