#!/usr/bin/env python3
"""
Validate benchmark/manifest.jsonl: schema completeness, family counts,
semantic-unit totals, duplicate/near-duplicate informal statements.
Exit code 0 on success, 1 on blocking issues.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REQUIRED = (
    "instance_id",
    "family",
    "difficulty",
    "source_provenance",
    "informal_statement",
    "semantic_units",
    "critical_unit_count",
    "reference_obligations",
    "code_context_paths",
    "split",
    "license_status",
)


def norm_text(s: str) -> str:
    s = s.lower()
    s = re.sub(r"\s+", " ", s).strip()
    return s


def jaccard(a: set[str], b: set[str]) -> float:
    if not a and not b:
        return 1.0
    u = len(a | b)
    if u == 0:
        return 0.0
    return len(a & b) / u


def is_intentional_grid_pair_001_002(instance_id_a: str, instance_id_b: str) -> bool:
    """
    Same algorithm family with consecutive indices 001 and 002 is an
    intentional paired grid (shared reference, distinct authoring surface).
    Skip near-duplicate warnings for this pair only.
    """
    ma = re.match(r"^(.+)_(\d{3})$", instance_id_a)
    mb = re.match(r"^(.+)_(\d{3})$", instance_id_b)
    if not ma or not mb:
        return False
    if ma.group(1) != mb.group(1):
        return False
    na, nb = int(ma.group(2)), int(mb.group(2))
    return {na, nb} == {1, 2}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--manifest",
        type=Path,
        default=ROOT / "benchmark" / "manifest.jsonl",
    )
    ap.add_argument(
        "--near-dup-threshold",
        type=float,
        default=0.92,
        help="Jaccard similarity on word sets; flag pairs above threshold",
    )
    ap.add_argument(
        "--strict-grid-near-dup",
        action="store_true",
        help="Also warn on same-family _001 vs _002 (default: skip those pairs)",
    )
    args = ap.parse_args()

    if not args.manifest.is_file():
        print(f"missing {args.manifest}", file=sys.stderr)
        return 1

    rows: list[dict] = []
    with args.manifest.open(encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            rows.append(json.loads(line))

    errs: list[str] = []
    for i, r in enumerate(rows):
        for k in REQUIRED:
            if k not in r or r[k] in (None, "", [], {}):
                errs.append(f"row {i} instance {r.get('instance_id')}: missing/empty {k}")
        su = r.get("semantic_units")
        if not isinstance(su, list) or len(su) == 0:
            errs.append(f"row {i} {r.get('instance_id')}: semantic_units must be non-empty list")
        ro = r.get("reference_obligations")
        if not isinstance(ro, list) or len(ro) == 0:
            errs.append(f"row {i} {r.get('instance_id')}: reference_obligations must be non-empty list")
        cc = r.get("critical_unit_count")
        if not isinstance(cc, int) or cc < 1:
            errs.append(f"row {i} {r.get('instance_id')}: critical_unit_count must be int >= 1")

    by_family = Counter(r["family"] for r in rows)
    su_by_family: dict[str, int] = defaultdict(int)
    for r in rows:
        su_by_family[r["family"]] += len(r.get("semantic_units") or [])

    print("instances_by_family")
    for fam, c in sorted(by_family.items()):
        print(f"  {fam}: {c}")

    print("\nsemantic_units_total_by_family")
    for fam, c in sorted(su_by_family.items()):
        print(f"  {fam}: {c}")

    # Near-duplicate informal statements
    bagged: list[tuple[str, set[str]]] = []
    for r in rows:
        toks = set(norm_text(r["informal_statement"]).split())
        bagged.append((r["instance_id"], toks))

    near: list[tuple[str, str, float]] = []
    for i in range(len(bagged)):
        for j in range(i + 1, len(bagged)):
            a, b = bagged[i][1], bagged[j][1]
            jac = jaccard(a, b)
            if jac >= args.near_dup_threshold:
                id_i, id_j = bagged[i][0], bagged[j][0]
                if not args.strict_grid_near_dup and is_intentional_grid_pair_001_002(
                    id_i, id_j
                ):
                    continue
                near.append((id_i, id_j, jac))

    print(f"\nnear_duplicate_pairs (threshold={args.near_dup_threshold}): {len(near)}")
    for x, y, jac in near[:50]:
        print(f"  {x} ~ {y}  jaccard={jac:.3f}")
    if len(near) > 50:
        print(f"  ... ({len(near) - 50} more)")

    if errs:
        print("\nBLOCKING:", file=sys.stderr)
        for e in errs[:50]:
            print(e, file=sys.stderr)
        if len(errs) > 50:
            print(f"... and {len(errs) - 50} more", file=sys.stderr)
        return 1

    if near:
        print(
            "\nwarning: near-duplicate informal statements detected; "
            "review pairs above or tighten thresholds.",
            file=sys.stderr,
        )

    print(f"\nok: {len(rows)} instances validated")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
