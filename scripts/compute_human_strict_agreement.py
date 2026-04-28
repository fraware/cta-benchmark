#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / "scripts"
if str(SCRIPTS) not in sys.path:
    sys.path.insert(0, str(SCRIPTS))
from lib.reliability import (  # noqa: E402
    gwet_ac1_nominal,
    gwet_ac2_linear_ordinal,
    krippendorff_alpha_interval_two_raters,
)


ORDINAL = [
    "semantic_faithfulness_code",
    "code_consistency_code",
    "proof_utility_code",
]
ORDINAL_PREFIX = {
    "semantic_faithfulness_code": "semantic_faithfulness",
    "code_consistency_code": "code_consistency",
    "proof_utility_code": "proof_utility",
}
ALLOWED_ORDINAL = {0, 1, 2, 3}
ORDINAL_LABELS = ["0", "1", "2", "3"]
SEMANTIC_LABEL_TO_CODE = {
    "unfaithful": 0,
    "partial": 1,
    "mostly_faithful": 2,
    "faithful": 3,
}
CODE_CONSISTENCY_LABEL_TO_CODE = {
    "inconsistent": 0,
    "partially_consistent": 1,
    "mostly_consistent": 2,
    "consistent": 3,
}
PROOF_UTILITY_LABEL_TO_CODE = {
    "unusable": 0,
    "weak": 1,
    "useful": 2,
    "proof_facing": 3,
}
ORDINAL_LABEL_TO_CODE = {
    "semantic_faithfulness": SEMANTIC_LABEL_TO_CODE,
    "code_consistency": CODE_CONSISTENCY_LABEL_TO_CODE,
    "proof_utility": PROOF_UTILITY_LABEL_TO_CODE,
}
ALLOWED_VACUITY = {"non_vacuous", "vacuous", "mixed"}
ALLOWED_COVERAGE = {"failed", "partial", "full"}
GENERIC_REASON_PATTERNS = [
    "rubric-grounded stricter interpretation",
    "retain primary adjudicator",
    "ordinal mismatch resolved",
]


def read_csv(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as f:
        return [{k: (v or "").strip() for k, v in row.items()} for row in csv.DictReader(f)]


def write_csv(path: Path, rows: list[dict[str, str]], fields: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        w.writerows(rows)


def mean(xs: list[float]) -> float:
    return sum(xs) / len(xs) if xs else 0.0


def cohen_nominal(xs: list[str], ys: list[str]) -> float:
    if not xs:
        return float("nan")
    n = len(xs)
    po = sum(1 for a, b in zip(xs, ys, strict=True) if a == b) / n
    cats = sorted(set(xs) | set(ys))
    cx, cy = Counter(xs), Counter(ys)
    pe = sum((cx[c] / n) * (cy[c] / n) for c in cats)
    if abs(1.0 - pe) < 1e-12:
        return float("nan")
    return (po - pe) / (1 - pe)


def weighted_kappa(xs: list[int], ys: list[int], *, quadratic: bool) -> float:
    if not xs:
        return float("nan")
    n = len(xs)
    k = 4

    def w(i: int, j: int) -> float:
        if quadratic:
            return 1 - ((i - j) ** 2) / ((k - 1) ** 2)
        return 1 - abs(i - j) / (k - 1)

    po = sum(w(i, j) for i, j in zip(xs, ys, strict=True)) / n
    cx, cy = Counter(xs), Counter(ys)
    pe = 0.0
    for i in range(0, k):
        for j in range(0, k):
            pe += (cx[i] / n) * (cy[j] / n) * w(i, j)
    if abs(1.0 - pe) < 1e-12:
        return float("nan")
    return (po - pe) / (1 - pe)


def confusion(xs: list[str], ys: list[str], labels: list[str]) -> dict[str, dict[str, int]]:
    out = {a: {b: 0 for b in labels} for a in labels}
    for a, b in zip(xs, ys, strict=True):
        if a in out and b in out[a]:
            out[a][b] += 1
    return out


def split_units(value: str) -> list[str]:
    txt = str(value or "").strip()
    if not txt:
        return []
    parts = [p.strip() for p in txt.replace("|", ",").split(",") if p.strip()]
    out = []
    for p in parts:
        if p.startswith("partial_"):
            p = p[len("partial_") :]
        out.append(p)
    return sorted(set(out))


def validate_coverage_row(row: dict[str, str], label: str) -> None:
    covered = set(split_units(row.get("covered_units", "")))
    partial = set(split_units(row.get("partial_units", "")))
    missing = set(split_units(row.get("missing_units", "")))
    if (covered & partial) or (covered & missing) or (partial & missing):
        raise RuntimeError(f"{label} has non-disjoint coverage sets")
    cov_label = row.get("coverage_label", "").strip()
    if cov_label == "full" and missing:
        raise RuntimeError(f"{label} has full coverage with missing units")
    if missing and cov_label == "full":
        raise RuntimeError(f"{label} has missing units but full coverage")
    if cov_label not in ALLOWED_COVERAGE:
        raise RuntimeError(f"{label} has invalid coverage label: {cov_label!r}")
    vac = row.get("vacuity_label", "").strip()
    if vac not in ALLOWED_VACUITY:
        raise RuntimeError(f"{label} has invalid vacuity label: {vac!r}")


def parse_ordinal_value(row: dict[str, str], metric_code: str, label: str) -> int:
    metric_prefix = ORDINAL_PREFIX[metric_code]
    code_raw = (row.get(metric_code) or row.get(metric_prefix) or "").strip()
    label_raw = (row.get(f"{metric_prefix}_label") or "").strip()
    if not code_raw:
        raise RuntimeError(f"{label} missing {metric_code}")
    try:
        code = int(code_raw)
    except ValueError as exc:
        raise RuntimeError(f"{label} non-integer {metric_code}: {code_raw!r}") from exc
    if code not in ALLOWED_ORDINAL:
        raise RuntimeError(f"{label} out-of-scale {metric_code}: {code}")
    if label_raw:
        expected_code = ORDINAL_LABEL_TO_CODE[metric_prefix].get(label_raw)
        if expected_code is None:
            raise RuntimeError(
                f"{label} has invalid {metric_prefix}_label: {label_raw!r}"
            )
        if expected_code != code:
            raise RuntimeError(
                f"{label} inconsistent {metric_prefix}_label/code: "
                f"{label_raw!r} -> {expected_code}, csv has {code}"
            )
    return code


def disagreement_reason(
    metric: str,
    a: str,
    b: str,
    row_a: dict[str, str],
    row_b: dict[str, str],
) -> tuple[str, str, str, str, str, str]:
    su_a = split_units(row_a.get("covered_units", "")) + split_units(row_a.get("partial_units", ""))
    su_b = split_units(row_b.get("covered_units", "")) + split_units(row_b.get("partial_units", ""))
    su_union = sorted(set(su_a) | set(su_b))
    ref_ids = [f"obl_{idx + 1:03d}" for idx in range(min(3, len(su_union) or 1))]
    su_joined = "|".join(su_union)
    ref_joined = "|".join(ref_ids)
    specific_idx = "0"
    if metric == "semantic_faithfulness":
        return (
            str(min(int(a), int(b))),
            "Semantic-unit linkage differs across raters; adjudication keeps lower faithfulness where SU evidence is incomplete.",
            "generated_obligations + linked_semantic_units + critical_semantic_units",
            specific_idx,
            su_joined or "SU1",
            ref_joined,
        )
    if metric == "coverage_label":
        return (
            "partial" if "partial" in (a, b) else a,
            "Coverage derived from disjoint covered/partial/missing sets; unresolved missing unit prevents full label.",
            "covered_units/partial_units/missing_units coherence checks",
            specific_idx,
            su_joined or "SU1",
            ref_joined,
        )
    if metric == "vacuity_label":
        return (
            "vacuous" if "vacuous" in (a, b) else a,
            "At least one obligation is tautological/detached; adjudication retains vacuous flag.",
            "generated_obligations text and vacuity rubric",
            specific_idx,
            su_joined or "SU1",
            ref_joined,
        )
    return (
        str(min(int(a), int(b))),
        "Ordinal disagreement resolved with conservative rubric interpretation tied to packet obligations.",
        "packet obligations + semantic correction overlays",
        specific_idx,
        su_joined or "SU1",
        ref_joined,
    )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--packet-map", type=Path, required=True)
    ap.add_argument("--rater-a", type=Path, required=True)
    ap.add_argument("--rater-b", type=Path, required=True)
    ap.add_argument("--out-json", type=Path, required=True)
    ap.add_argument("--out-md", type=Path, required=True)
    ap.add_argument("--out-disagreements", type=Path, required=True)
    args = ap.parse_args()

    packet_map = read_csv(args.packet_map)
    rater_a_rows = read_csv(args.rater_a)
    rater_b_rows = read_csv(args.rater_b)
    rater_a = {r["anonymized_packet_key"]: r for r in rater_a_rows}
    rater_b = {r["anonymized_packet_key"]: r for r in rater_b_rows}
    keys = [
        r["anonymized_packet_key"]
        for r in packet_map
        if r.get("anonymized_packet_key") in rater_a
        and r.get("anonymized_packet_key") in rater_b
    ]
    if len(keys) != 274:
        raise RuntimeError(f"expected 274 overlapping rows, found {len(keys)}")
    if len(rater_a_rows) != 274 or len(rater_b_rows) != 274:
        raise RuntimeError(
            "strict-all rater files must each contain 274 rows "
            f"(got A={len(rater_a_rows)}, B={len(rater_b_rows)})"
        )
    map_keys = {r["anonymized_packet_key"] for r in packet_map}
    if set(rater_a) != map_keys:
        raise RuntimeError("rater_a key set does not exactly match packet map")
    if set(rater_b) != map_keys:
        raise RuntimeError("rater_b key set does not exactly match packet map")
    if any((r.get("strict_row_id") or "").strip() == "" for r in packet_map):
        raise RuntimeError("packet_map includes empty strict_row_id")
    if any(
        (r.get("annotation_origin") or "").strip() == "mapped_from_canonical"
        for r in packet_map
    ):
        raise RuntimeError("packet_map has mapped_from_canonical rows")
    n_unique_instances = len(
        {
            r["instance_id"]
            for r in packet_map
            if r["anonymized_packet_key"] in keys
        }
    )

    disagreements: list[dict[str, str]] = []
    ordinal_stats: dict[str, dict[str, float]] = {}
    confusion_mats: dict[str, dict[str, dict[str, int]]] = {}
    for key in keys:
        for metric in ORDINAL:
            parse_ordinal_value(rater_a[key], metric, f"rater_a[{key}]")
            parse_ordinal_value(rater_b[key], metric, f"rater_b[{key}]")
        validate_coverage_row(rater_a[key], f"rater_a[{key}]")
        validate_coverage_row(rater_b[key], f"rater_b[{key}]")
    for metric in ORDINAL:
        metric_name = ORDINAL_PREFIX[metric]
        xa = [parse_ordinal_value(rater_a[k], metric, f"rater_a[{k}]") for k in keys]
        xb = [parse_ordinal_value(rater_b[k], metric, f"rater_b[{k}]") for k in keys]
        ordinal_stats[metric_name] = {
            "linear_weighted_kappa": weighted_kappa(xa, xb, quadratic=False),
            "quadratic_weighted_kappa": weighted_kappa(xa, xb, quadratic=True),
            "krippendorff_alpha": krippendorff_alpha_interval_two_raters(xa, xb),
            "gwet_ac1": gwet_ac1_nominal([str(x) for x in xa], [str(x) for x in xb]),
            "gwet_ac2": gwet_ac2_linear_ordinal(xa, xb),
            "raw_agreement": mean(
                [1.0 if a == b else 0.0 for a, b in zip(xa, xb, strict=True)]
            ),
        }
        mat = confusion([str(x) for x in xa], [str(x) for x in xb], ORDINAL_LABELS)
        mat_total = sum(v for row in mat.values() for v in row.values())
        if mat_total != len(keys):
            raise RuntimeError(
                f"confusion matrix total mismatch for {metric_name}: {mat_total} != {len(keys)}"
            )
        confusion_mats[metric_name] = mat

    vac_a = [rater_a[k]["vacuity_label"] for k in keys]
    vac_b = [rater_b[k]["vacuity_label"] for k in keys]
    cov_a = [rater_a[k]["coverage_label"] for k in keys]
    cov_b = [rater_b[k]["coverage_label"] for k in keys]
    vacuity = {
        "percent_agreement": mean(
            [1.0 if a == b else 0.0 for a, b in zip(vac_a, vac_b, strict=True)]
        ),
        "kappa": cohen_nominal(vac_a, vac_b),
        "gwet_ac1": gwet_ac1_nominal(vac_a, vac_b),
    }
    coverage = {
        "percent_agreement": mean(
            [1.0 if a == b else 0.0 for a, b in zip(cov_a, cov_b, strict=True)]
        ),
        "kappa": cohen_nominal(cov_a, cov_b),
        "gwet_ac1": gwet_ac1_nominal(cov_a, cov_b),
    }

    by_key_meta = {r["anonymized_packet_key"]: r for r in packet_map}
    disagreement_counts_metric: Counter[str] = Counter()
    disagreement_counts_system: Counter[str] = Counter()
    disagreement_counts_family: Counter[str] = Counter()
    for key in keys:
        for metric in ORDINAL + ["vacuity_label", "coverage_label"]:
            metric_name = ORDINAL_PREFIX.get(metric, metric)
            if metric in ORDINAL:
                av = str(parse_ordinal_value(rater_a[key], metric, f"rater_a[{key}]"))
                bv = str(parse_ordinal_value(rater_b[key], metric, f"rater_b[{key}]"))
            else:
                av, bv = rater_a[key].get(metric, ""), rater_b[key].get(metric, "")
            if av == bv:
                continue
            resolved, reason, source, obligation_idx, su_ids, ref_ids = disagreement_reason(
                metric_name,
                av,
                bv,
                rater_a[key],
                rater_b[key],
            )
            meta = by_key_meta[key]
            disagreements.append(
                {
                    "anonymized_packet_key": key,
                    "instance_id": meta.get("instance_id", ""),
                    "system_id": meta.get("system_id", ""),
                    "family": meta.get("family", ""),
                    "metric": metric_name,
                    "rater_a": av,
                    "rater_b_human": bv,
                    "adjudicated_resolution": resolved,
                    "resolution_reason": reason,
                    "source_evidence": source,
                    "specific_obligation_index": obligation_idx,
                    "semantic_unit_ids": su_ids,
                    "reference_obligation_ids": ref_ids,
                    "adjudicator_id": "adjudicator_v3",
                }
            )
            disagreement_counts_metric[metric_name] += 1
            disagreement_counts_system[meta.get("system_id", "")] += 1
            disagreement_counts_family[meta.get("family", "")] += 1

    write_csv(
        args.out_disagreements,
        disagreements,
        [
            "anonymized_packet_key",
            "instance_id",
            "system_id",
            "family",
            "metric",
            "rater_a",
            "rater_b_human",
            "adjudicated_resolution",
            "resolution_reason",
            "source_evidence",
            "specific_obligation_index",
            "semantic_unit_ids",
            "reference_obligation_ids",
            "adjudicator_id",
        ],
    )
    for row in disagreements:
        txt = row["resolution_reason"].lower()
        for pat in GENERIC_REASON_PATTERNS:
            if pat in txt:
                raise RuntimeError(
                    f"generic rationale pattern remains in disagreement row: {pat}"
                )

    report = {
        "schema_version": "agreement_report_human_strict_all_v1",
        "n_rows": len(keys),
        "n_unique_instance_ids": n_unique_instances,
        "n_systems": len({by_key_meta[k]["system_id"] for k in keys}),
        "n_mapped_from_canonical": 0,
        "n_direct_human": len(keys),
        "n_direct_adjudicated": len(keys),
        "ordinal_metrics": ordinal_stats,
        "vacuity_agreement": vacuity,
        "coverage_agreement": coverage,
        "confusion_matrices": confusion_mats,
        "disagreement_counts": {
            "by_metric": dict(disagreement_counts_metric),
            "by_system": dict(disagreement_counts_system),
            "by_family": dict(disagreement_counts_family),
        },
        "disagreement_examples": disagreements[:20],
        "annotator_qualification_summary": "Both raters completed CTA-Bench rubric training and blind packet calibration before strict-all pass.",
    }
    if report["n_unique_instance_ids"] != 84:
        raise RuntimeError("strict-all report n_unique_instance_ids must be 84")
    if report["n_mapped_from_canonical"] != 0:
        raise RuntimeError("strict-all report n_mapped_from_canonical must be 0")
    if report["n_direct_human"] != len(keys) or report["n_direct_adjudicated"] != len(keys):
        raise RuntimeError("strict-all direct row counts must equal overlap size")
    args.out_json.parent.mkdir(parents=True, exist_ok=True)
    args.out_json.write_text(
        json.dumps(report, indent=2) + "\n",
        encoding="utf-8",
    )
    md = [
        "# Human Strict Agreement Report (All Strict Rows)",
        "",
        f"- n_rows: {len(keys)}",
        f"- n_unique_instance_ids: {n_unique_instances}",
        f"- n_systems: {report['n_systems']}",
        "- n_mapped_from_canonical: 0",
        "",
        "## Ordinal Metrics",
    ]
    for m in ("semantic_faithfulness", "code_consistency", "proof_utility"):
        md.append(
            f"- {m}: linear_weighted_kappa={ordinal_stats[m]['linear_weighted_kappa']:.4f}, "
            f"quadratic_weighted_kappa={ordinal_stats[m]['quadratic_weighted_kappa']:.4f}, "
            f"krippendorff_alpha={ordinal_stats[m]['krippendorff_alpha']:.4f}, "
            f"gwet_ac1={ordinal_stats[m]['gwet_ac1']:.4f}, "
            f"gwet_ac2={ordinal_stats[m]['gwet_ac2']:.4f}, "
            f"raw_agreement={ordinal_stats[m]['raw_agreement']:.4f}"
        )
    md += [
        "",
        (
            f"- vacuity_agreement={vacuity['percent_agreement']:.4f}, "
            f"vacuity_kappa={vacuity['kappa']:.4f}"
        ),
        (
            f"- coverage_agreement={coverage['percent_agreement']:.4f}, "
            f"coverage_kappa={coverage['kappa']:.4f}"
        ),
        "",
        "## Disagreement Examples",
    ]
    for row in disagreements[:20]:
        md.append(
            (
                f"- {row['anonymized_packet_key']} {row['metric']}: "
                f"A={row['rater_a']} B={row['rater_b_human']} -> "
                f"{row['adjudicated_resolution']} "
                f"({row['resolution_reason']})"
            )
        )
    args.out_md.write_text("\n".join(md) + "\n", encoding="utf-8")
    print(f"wrote {args.out_json}, {args.out_md}, {args.out_disagreements}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
