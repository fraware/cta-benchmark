#!/usr/bin/env python3
"""Export model metadata reconciliation for paper-reportable runs."""

from __future__ import annotations

import argparse
import csv
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def scan_run_manifests() -> list[Path]:
    runs = ROOT / "runs"
    if not runs.is_dir():
        return []
    return sorted(p for p in runs.rglob("run_manifest.json") if p.is_file())


def parse_system_card(path: Path) -> dict[str, str]:
    out: dict[str, str] = {}
    if not path.is_file():
        return out
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    for ln in lines:
        s = ln.strip()
        if s.startswith("system_id:"):
            out["system_id"] = s.split(":", 1)[1].strip().strip('"')
        elif s.startswith("name:") and "card_model_name" not in out:
            out["card_model_name"] = s.split(":", 1)[1].strip().strip('"')
        elif s.startswith("version:") and "card_model_version" not in out:
            out["card_model_version"] = s.split(":", 1)[1].strip().strip('"')
        elif s.startswith("prompt_template_path:"):
            out["prompt_template_path"] = s.split(":", 1)[1].strip().strip('"')
        elif s.startswith("num_samples_per_instance:"):
            out["num_samples_per_instance"] = s.split(":", 1)[1].strip().strip('"')
        elif s.startswith("selection_rule:"):
            out["selection_rule"] = s.split(":", 1)[1].strip().strip('"')
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--out",
        type=Path,
        default=ROOT / "results" / "paper_model_metadata_registry.csv",
    )
    args = ap.parse_args()

    card_dir = ROOT / "experiments" / "system_cards"
    cards: dict[str, dict[str, str]] = {}
    for p in sorted(card_dir.glob("*.yaml")):
        c = parse_system_card(p)
        sid = c.get("system_id")
        if sid:
            cards[sid] = c

    rows: list[dict[str, str]] = []
    for p in scan_run_manifests():
        try:
            doc = json.loads(p.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            continue
        sid = str(doc.get("system_id", ""))
        provider = doc.get("provider") or {}
        gen = doc.get("generation_parameters") or {}
        card = cards.get(sid, {})
        run_model = str(provider.get("model", ""))
        card_model = card.get("card_model_name", "")
        rows.append(
            {
                "run_id": str(doc.get("run_id", "")),
                "benchmark_version": str(doc.get("benchmark_version", "")),
                "system_id": sid,
                "provider": str(provider.get("name", "")),
                "model_name": run_model,
                "model_version": str(provider.get("model_version", "")),
                "prompt_template_sha": str(
                    doc.get("prompt_template_hash", "")
                ),
                "prompt_template_path": card.get("prompt_template_path", ""),
                "num_samples_per_instance": card.get(
                    "num_samples_per_instance", ""
                ),
                "selection_rule": card.get("selection_rule", ""),
                "temperature": str(gen.get("temperature", "")),
                "top_p": str(gen.get("top_p", "")),
                "max_output_tokens": str(gen.get("max_tokens", "")),
                "card_model_name": card_model,
                "card_model_version": card.get("card_model_version", ""),
                "model_matches_system_card": "true"
                if card_model and run_model == card_model
                else "false",
                "manifest_path": str(p.relative_to(ROOT)).replace("\\", "/"),
            }
        )

    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("w", newline="", encoding="utf-8") as f:
        fields = [
            "run_id",
            "benchmark_version",
            "system_id",
            "provider",
            "model_name",
            "model_version",
            "prompt_template_sha",
            "prompt_template_path",
            "num_samples_per_instance",
            "selection_rule",
            "temperature",
            "top_p",
            "max_output_tokens",
            "card_model_name",
            "card_model_version",
            "model_matches_system_card",
            "manifest_path",
        ]
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        w.writerows(rows)
    print(f"wrote {args.out} ({len(rows)} rows)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
