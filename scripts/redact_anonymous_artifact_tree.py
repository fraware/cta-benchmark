#!/usr/bin/env python3
"""Walk an anonymous artifact staging tree and replace common deanonymization
substrings in text artifacts. Idempotent; safe for JSON/CSV/Markdown/TOML."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

SUFFIXES = {".md", ".txt", ".csv", ".json", ".jsonl", ".toml", ".yml", ".yaml", ".ps1", ".py", ".rs", ".lean", ".tex", ".bib"}

# (regex with re.I, replacement). Order matters for overlapping URLs vs bare tokens.
REGEX_REPLACEMENTS: list[tuple[re.Pattern[str], str]] = [
    (re.compile(r"github\.com/fraware", re.I), "github.com/REDACTED_ORG"),
    (re.compile(r"@stanford\.edu", re.I), "@REDACTED_EMAIL"),
    (re.compile(r"@stanford\b", re.I), "@REDACTED_AFFILIATION"),
    (re.compile(r"\bfraware\b", re.I), "REDACTED_ORG"),
    (re.compile(r"\bMateo\b", re.I), "AUTHOR_REDACTED"),
    (re.compile(r"\bPetel\b", re.I), "AUTHOR_REDACTED"),
]


def redact_text(s: str) -> str:
    out = s
    for pat, new in REGEX_REPLACEMENTS:
        out = pat.sub(new, out)
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("root", type=Path, help="Staging root (e.g. artifacts/cta-benchmark-anonymous)")
    args = ap.parse_args()
    root: Path = args.root.resolve()
    if not root.is_dir():
        print(f"missing staging dir: {root}", file=sys.stderr)
        return 1
    changed = 0
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        if path.suffix.lower() not in SUFFIXES:
            continue
        try:
            raw = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        new = redact_text(raw)
        if new != raw:
            path.write_text(new, encoding="utf-8", newline="")
            changed += 1
    print(f"redact_anonymous_artifact_tree: touched {changed} file(s) under {root}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
