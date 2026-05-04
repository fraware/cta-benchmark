#!/usr/bin/env python3
"""
Heuristic secret scan over hf_release/ (including artifact/).

Uses doc-safe allowlists: allows bare `HF_TOKEN`, placeholders `<...>`, and
`os.environ[...]` without literal secrets. Fails on hf_/sk- style assignments.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

TEXT_SUFFIXES = {
    ".md",
    ".py",
    ".json",
    ".csv",
    ".yml",
    ".yaml",
    ".txt",
    ".toml",
    ".ps1",
    ".sh",
    ".rs",
    ".lean",
    ".cff",
    ".html",
    ".css",
    ".js",
    ".ts",
    ".tsx",
}

MAX_FILE_BYTES = 4_000_000

# Hugging Face user tokens often start with hf_; OpenAI keys with sk-
HF_TOKEN_VALUE = re.compile(
    r"(?i)\b(?:HF_TOKEN|HUGGING_FACE_HUB_TOKEN)\s*=\s*"
    r"(?:[\"']?)([^\s#\"'<>]+)(?:[\"']?)"
)
OPENAI_SK = re.compile(r"(?i)\bOPENAI_API_KEY\s*=\s*[\"']?(sk-[a-zA-Z0-9]{20,})")
ANTHROPIC_KEY = re.compile(
    r"(?i)\bANTHROPIC_API_KEY\s*=\s*[\"']?"
    r"(sk-ant-[a-zA-Z0-9\-]{20,})"
)
AWS_SECRET = re.compile(
    r"(?i)\bAWS_SECRET_ACCESS_KEY\s*=\s*[\"']?([A-Za-z0-9/+=]{30,})"
)


def _allowed_env_value(val: str) -> bool:
    v = val.strip().strip("\"'")
    if not v:
        return True
    if v.startswith("${") or v.startswith("$("):
        return True
    if v.startswith("<") and v.endswith(">"):
        return True
    if v.upper() in {"YOUR_TOKEN_HERE", "...", "NONE", "OPTIONAL"}:
        return True
    if v.startswith("os.environ"):
        return True
    if v.startswith("process.env"):
        return True
    return False


def scan_file(path: Path) -> list[str]:
    errors: list[str] = []
    try:
        text = path.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return errors
    for i, line in enumerate(text.splitlines(), 1):
        for rx, label in (
            (OPENAI_SK, "OpenAI-style API key"),
            (ANTHROPIC_KEY, "Anthropic-style API key"),
            (AWS_SECRET, "AWS secret-like value"),
        ):
            m = rx.search(line)
            if m:
                errors.append(f"{path}:{i}: possible {label} assignment")

        m = HF_TOKEN_VALUE.search(line)
        if m:
            val = m.group(1)
            if _allowed_env_value(val):
                continue
            if val.startswith("hf_") and len(val) >= 20:
                msg = f"{path}:{i}: possible Hugging Face token assignment"
                errors.append(msg)
            elif len(val) >= 32 and not val.startswith("${"):
                msg = f"{path}:{i}: possible long secret for HF_TOKEN"
                errors.append(msg)
    return errors


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--root",
        type=Path,
        default=ROOT / "hf_release",
        help="Directory to scan (default: ./hf_release)",
    )
    args = ap.parse_args()
    root: Path = args.root
    if not root.is_dir():
        raise SystemExit(f"missing directory: {root}")

    all_err: list[str] = []
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        if path.stat().st_size > MAX_FILE_BYTES:
            continue
        if path.suffix.lower() not in TEXT_SUFFIXES:
            continue
        all_err.extend(scan_file(path))

    if all_err:
        print("hf_release secret scan FAILED:", file=sys.stderr)
        for e in all_err[:200]:
            print(e, file=sys.stderr)
        if len(all_err) > 200:
            print(f"... and {len(all_err) - 200} more", file=sys.stderr)
        return 1

    print(f"hf_release secret scan passed ({root})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
