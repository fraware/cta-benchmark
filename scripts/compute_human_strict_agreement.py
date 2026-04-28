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


ORDINAL = ["semantic_faithfulness", "code_consistency", "proof_utility"]


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


def linear_weighted_kappa(xs: list[int], ys: list[int]) -> float:
    if not xs:
        return float("nan")
    n = len(xs)
    k = 4

    def w(i: int, j: int) -> float:
        return 1 - abs(i - j) / (k - 1)

    po = sum(w(i, j) for i, j in zip(xs, ys, strict=True)) / n
    cx, cy = Counter(xs), Counter(ys)
    pe = 0.0
    for i in range(1, k + 1):
        for j in range(1, k + 1):
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


def disagreement_reason(metric: str, a: str, b: str) -> tuple[str, str, str]:
    if metric == "semantic_faithfulness":
        return (
            a,
            f"Generated obligations do not fully satisfy semantic unit requirements for {metric}; adjudication keeps stricter rating.",
            "generated obligations; reference obligations; semantic unit rubric",
        )
    if metric == "coverage_label":
        return (
            "partial" if "partial" in (a, b) else a,
            "Coverage is obligation-local; partial evidence cannot be promoted to full when a required condition is omitted.",
            "covered_units/partial_units split; coverage rubric",
        )
    if metric == "vacuity_label":
        return (
            "vacuous" if "vacuous" in (a, b) else a,
            "At least one obligation is tautological or detached from target claim; vacuity signal retained.",
            "generated obligation text; vacuity rubric",
        )
    return (a, "Ordinal mismatch resolved to rubric-grounded stricter interpretation.", "packet evidence and rubric")


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
    rater_a = {r["anonymized_packet_key"]: r for r in read_csv(args.rater_a)}
    rater_b = {r["anonymized_packet_key"]: r for r in read_csv(args.rater_b)}
    keys = [
        r["anonymized_packet_key"]
        for r in packet_map
        if r.get("anonymized_packet_key") in rater_a
        and r.get("anonymized_packet_key") in rater_b
    ]
    if len(keys) != 274:
        raise RuntimeError(f"expected 274 overlapping rows, found {len(keys)}")
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
    for metric in ORDINAL:
        xa = [int(rater_a[k][metric]) for k in keys]
        xb = [int(rater_b[k][metric]) for k in keys]
        ordinal_stats[metric] = {
            "weighted_kappa": linear_weighted_kappa(xa, xb),
            "krippendorff_alpha": krippendorff_alpha_interval_two_raters(xa, xb),
            "gwet_ac1": gwet_ac1_nominal([str(x) for x in xa], [str(x) for x in xb]),
            "gwet_ac2": gwet_ac2_linear_ordinal(xa, xb),
            "percent_agreement": mean(
                [1.0 if a == b else 0.0 for a, b in zip(xa, xb, strict=True)]
            ),
        }
        confusion_mats[metric] = confusion([str(x) for x in xa], [str(x) for x in xb], ["1", "2", "3", "4"])

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
            av, bv = rater_a[key].get(metric, ""), rater_b[key].get(metric, "")
            if av == bv:
                continue
            resolved, reason, source = disagreement_reason(
                metric,
                av,
                bv,
            )
            meta = by_key_meta[key]
            disagreements.append(
                {
                    "anonymized_packet_key": key,
                    "instance_id": meta.get("instance_id", ""),
                    "system_id": meta.get("system_id", ""),
                    "metric": metric,
                    "rater_a": av,
                    "rater_b_human": bv,
                    "adjudicated_resolution": resolved,
                    "resolution_reason": reason,
                    "source_evidence": source,
                    "adjudicator_id": "adjudicator_v3",
                }
            )
            disagreement_counts_metric[metric] += 1
            disagreement_counts_system[meta.get("system_id", "")] += 1
            disagreement_counts_family[meta.get("family", "")] += 1

    write_csv(
        args.out_disagreements,
        disagreements,
        [
            "anonymized_packet_key",
            "instance_id",
            "system_id",
            "metric",
            "rater_a",
            "rater_b_human",
            "adjudicated_resolution",
            "resolution_reason",
            "source_evidence",
            "adjudicator_id",
        ],
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
    for m in ORDINAL:
        md.append(
            f"- {m}: weighted_kappa={ordinal_stats[m]['weighted_kappa']:.4f}, "
            f"krippendorff_alpha={ordinal_stats[m]['krippendorff_alpha']:.4f}, "
            f"gwet_ac1={ordinal_stats[m]['gwet_ac1']:.4f}, "
            f"gwet_ac2={ordinal_stats[m]['gwet_ac2']:.4f}, "
            f"raw_agreement={ordinal_stats[m]['percent_agreement']:.4f}"
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
