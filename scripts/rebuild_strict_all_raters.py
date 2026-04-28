#!/usr/bin/env python3
from __future__ import annotations

import csv
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SEMANTIC_CODE_TO_LABEL = {
    0: "unfaithful",
    1: "partial",
    2: "mostly_faithful",
    3: "faithful",
}
CONSISTENCY_CODE_TO_LABEL = {
    0: "inconsistent",
    1: "partially_consistent",
    2: "mostly_consistent",
    3: "consistent",
}
PROOF_CODE_TO_LABEL = {
    0: "unusable",
    1: "weak",
    2: "useful",
    3: "proof_facing",
}
ALLOWED_VACUITY = {"non_vacuous", "vacuous", "mixed"}
ALLOWED_COVERAGE = {"failed", "partial", "full"}

REQUIRED_COLUMNS = [
    "anonymized_packet_key",
    "instance_id",
    "system_id",
    "family",
    "semantic_faithfulness_label",
    "semantic_faithfulness_code",
    "code_consistency_label",
    "code_consistency_code",
    "proof_utility_label",
    "proof_utility_code",
    "vacuity_label",
    "coverage_label",
    "covered_units",
    "partial_units",
    "missing_units",
    "contradiction_signal",
    "notes",
]


def read_csv(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as f:
        return [{k: (v or "").strip() for k, v in row.items()} for row in csv.DictReader(f)]


def write_csv(path: Path, rows: list[dict[str, str]]) -> None:
    with path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=REQUIRED_COLUMNS)
        writer.writeheader()
        writer.writerows(rows)


def parse_code(row: dict[str, str], field: str) -> int:
    value = row.get(field, "").strip()
    code = int(value)
    if code not in (0, 1, 2, 3):
        raise ValueError(f"{field} out of scale: {code}")
    return code


def assert_units_coherence(row: dict[str, str], prefix: str) -> None:
    missing = row.get("missing_units", "").strip()
    coverage = row.get("coverage_label", "").strip()
    if coverage == "full" and missing:
        raise ValueError(f"{prefix} has full coverage with missing units")
    if missing and coverage == "full":
        raise ValueError(f"{prefix} has missing units but full coverage")


def rebuild_rater_rows(
    map_rows: list[dict[str, str]],
    input_rows: list[dict[str, str]],
) -> list[dict[str, str]]:
    by_key = {r["anonymized_packet_key"]: r for r in input_rows}
    out: list[dict[str, str]] = []
    for meta in map_rows:
        key = meta["anonymized_packet_key"]
        src = by_key[key]
        sem_code = parse_code(src, "semantic_faithfulness")
        cc_code = parse_code(src, "code_consistency")
        pu_code = parse_code(src, "proof_utility")
        vacuity = src.get("vacuity_label", "")
        coverage = src.get("coverage_label", "")
        if vacuity not in ALLOWED_VACUITY:
            raise ValueError(f"{key} invalid vacuity label: {vacuity!r}")
        if coverage not in ALLOWED_COVERAGE:
            raise ValueError(f"{key} invalid coverage label: {coverage!r}")
        row = {
            "anonymized_packet_key": key,
            "instance_id": meta.get("instance_id", ""),
            "system_id": meta.get("system_id", ""),
            "family": meta.get("family", ""),
            "semantic_faithfulness_label": SEMANTIC_CODE_TO_LABEL[sem_code],
            "semantic_faithfulness_code": str(sem_code),
            "code_consistency_label": CONSISTENCY_CODE_TO_LABEL[cc_code],
            "code_consistency_code": str(cc_code),
            "proof_utility_label": PROOF_CODE_TO_LABEL[pu_code],
            "proof_utility_code": str(pu_code),
            "vacuity_label": vacuity,
            "coverage_label": coverage,
            "covered_units": src.get("covered_units", ""),
            "partial_units": src.get("partial_units", ""),
            "missing_units": src.get("missing_units", ""),
            "contradiction_signal": src.get("contradiction_signal", "0"),
            "notes": src.get("notes", ""),
        }
        assert_units_coherence(row, key)
        out.append(row)
    return out


def validate_rows(rows: list[dict[str, str]], name: str) -> None:
    if len(rows) != 274:
        raise ValueError(f"{name} row_count != 274 ({len(rows)})")
    n_instances = len({r["instance_id"] for r in rows if r.get("instance_id")})
    if n_instances != 84:
        raise ValueError(f"{name} unique instance_id count != 84 ({n_instances})")


def main() -> int:
    map_path = ROOT / "annotation" / "human_pass_v3" / "human_strict_packet_ids.csv"
    rater_a_path = ROOT / "annotation" / "rater_a_strict_all.csv"
    rater_b_path = ROOT / "annotation" / "human_pass_v3" / "rater_b_human_strict_all.csv"
    map_rows = read_csv(map_path)
    rater_a_rows = read_csv(rater_a_path)
    rater_b_rows = read_csv(rater_b_path)
    out_a = rebuild_rater_rows(map_rows, rater_a_rows)
    out_b = rebuild_rater_rows(map_rows, rater_b_rows)
    validate_rows(out_a, "rater_a_strict_all.csv")
    validate_rows(out_b, "rater_b_human_strict_all.csv")
    write_csv(rater_a_path, out_a)
    write_csv(rater_b_path, out_b)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
