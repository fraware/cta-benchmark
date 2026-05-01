#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def sha256_hex(path: Path) -> str:
    h = hashlib.sha256()
    h.update(path.read_bytes())
    return h.hexdigest()


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--manifest",
        type=Path,
        default=ROOT / "artifacts" / "evidence_hardening_manifest.json",
    )
    args = ap.parse_args()
    if not args.manifest.is_file():
        print(
            f"error: missing artifact manifest {args.manifest}",
            file=sys.stderr,
        )
        return 1

    body = json.loads(args.manifest.read_text(encoding="utf-8"))
    required = [str(x) for x in (body.get("required_contents") or [])]
    missing = [rel for rel in required if not (ROOT / rel).is_file()]
    if missing:
        print(
            "error: required artifact files missing from workspace:\n"
            + "\n".join(missing),
            file=sys.stderr,
        )
        return 1

    expected_sha = body.get("sha256") or {}
    mismatches: list[str] = []
    for rel in required:
        actual = sha256_hex(ROOT / rel)
        exp = str(expected_sha.get(rel, ""))
        if exp and actual != exp:
            mismatches.append(f"{rel}: expected {exp}, got {actual}")
    if mismatches:
        print(
            "error: checksum mismatches:\n" + "\n".join(mismatches),
            file=sys.stderr,
        )
        print(
            "hint: `cargo validate benchmark --release` rewrites "
            "benchmark/v0.3/manifests/release_summary.json; refresh checksums with:\n"
            "  python scripts/implement_evidence_hardening.py --manifest-only",
            file=sys.stderr,
        )
        return 1

    if body.get("missing"):
        print(
            "error: manifest still reports missing entries:\n"
            + "\n".join(str(x) for x in body["missing"]),
            file=sys.stderr,
        )
        return 1

    print("validate_release_artifact: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
