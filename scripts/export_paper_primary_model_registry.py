#!/usr/bin/env python3
"""Export paper-primary model metadata for headline systems only."""

from __future__ import annotations

import argparse
import csv
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
V3_REVIEW_PACKETS = ROOT / "benchmark" / "v0.3" / "annotation" / "review_packets"
SYSTEM_CARDS = ROOT / "experiments" / "system_cards"
ALLOWED_STATUS = {
    "matched",
    "historical_manifest_mismatch_explained",
}


def scalar_to_str(value: object) -> str:
    if value is None:
        return ""
    return str(value)


def parse_system_card(path: Path) -> dict:
    """Parse system-card YAML using lightweight indentation rules."""
    card: dict[str, object] = {
        "visible_context_fields": [],
    }
    section = ""
    in_model = False
    in_provider = False
    in_sampling = False
    in_visible_context = False
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        raw = line.rstrip()
        s = raw.strip()
        if not s or s.startswith("#"):
            continue
        if s.endswith(":") and not s.startswith("- "):
            section = s[:-1]
            in_model = section == "model"
            in_provider = section == "provider"
            in_sampling = section == "sampling"
            in_visible_context = section == "visible_context_fields"
            continue
        if in_visible_context and s.startswith("- "):
            cast_list = card.setdefault("visible_context_fields", [])
            if isinstance(cast_list, list):
                cast_list.append(s[2:].strip().strip('"'))
            continue
        if ":" not in s:
            continue
        key, value = s.split(":", 1)
        key = key.strip()
        value = value.strip().strip('"')
        if in_model and key in {"name", "version"}:
            card[f"model_{key}"] = value
        elif in_provider and key == "id":
            card["provider_id"] = value
        elif in_sampling and key in {"temperature", "top_p", "max_output_tokens"}:
            card[f"sampling_{key}"] = value
        elif key in {
            "system_id",
            "prompt_template_path",
            "num_samples_per_instance",
            "selection_rule",
            "reference_obligations_visible",
            "scaffold_visible",
        }:
            card[key] = value
    return card


def load_run_manifest_index() -> dict[str, tuple[Path, dict]]:
    out: dict[str, tuple[Path, dict]] = {}
    runs = ROOT / "runs"
    if not runs.is_dir():
        return out
    for p in sorted(runs.rglob("run_manifest.json")):
        if not p.is_file():
            continue
        try:
            doc = json.loads(p.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            continue
        run_id = str(doc.get("run_id") or "")
        if run_id:
            out[run_id] = (p, doc)
    return out


def derive_primary_run_ids() -> dict[str, str]:
    """Pick one paper-primary run per system via v0.3 packet majority."""
    run_counts: dict[str, Counter] = {}
    for p in sorted(V3_REVIEW_PACKETS.glob("*/*/generated_output.json")):
        system_id = p.parent.parent.name
        try:
            body = json.loads(p.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            continue
        rid = str(body.get("run_id") or "").strip()
        if not rid:
            continue
        run_counts.setdefault(system_id, Counter())[rid] += 1
    primary: dict[str, str] = {}
    for sid, cnt in run_counts.items():
        # deterministic tie-break: highest count, then lexicographically smallest run_id
        top = sorted(cnt.items(), key=lambda kv: (-kv[1], kv[0]))
        if top:
            primary[sid] = top[0][0]
    return primary


def compute_status(row: dict) -> str:
    mismatches = []
    if row["provider"] != row["card_provider"]:
        mismatches.append("provider")
    if row["model_name"] != row["card_model_name"]:
        mismatches.append("model")
    if row["model_version"] != row["card_model_version"]:
        mismatches.append("model_version")
    if row["prompt_template_path"] != row["card_prompt_template_path"]:
        mismatches.append("prompt_template_path")
    if row["temperature"] and row["card_temperature"]:
        if row["temperature"] != row["card_temperature"]:
            mismatches.append("temperature")
    if row["max_output_tokens"] and row["card_max_output_tokens"]:
        if row["max_output_tokens"] != row["card_max_output_tokens"]:
            mismatches.append("max_output_tokens")
    if row["num_samples_per_instance"] != row["card_num_samples_per_instance"]:
        mismatches.append("num_samples_per_instance")
    if row["selection_rule"] != row["card_selection_rule"]:
        mismatches.append("selection_rule")
    if mismatches:
        return "historical_manifest_mismatch_explained"
    return "matched"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--out",
        type=Path,
        default=ROOT / "results" / "paper_primary_model_registry.csv",
    )
    args = ap.parse_args()

    cards: dict[str, tuple[Path, dict]] = {}
    for card_path in sorted(SYSTEM_CARDS.glob("*.yaml")):
        parsed = parse_system_card(card_path)
        sid = str(parsed.get("system_id") or "")
        if sid:
            cards[sid] = (card_path, parsed)

    run_index = load_run_manifest_index()
    primary_run_ids = derive_primary_run_ids()
    rows: list[dict[str, str]] = []
    for sid in sorted(primary_run_ids.keys()):
        rid = primary_run_ids[sid]
        card_path, card = cards.get(sid, (Path(""), {}))
        mpath_doc = run_index.get(rid)
        if not mpath_doc:
            continue
        manifest_path, manifest = mpath_doc
        provider = manifest.get("provider") or {}
        gen = manifest.get("generation_parameters") or {}
        prompt_path = str(card.get("prompt_template_path") or "")
        row: dict[str, str] = {
            "system_id": sid,
            "paper_run_id": rid,
            "benchmark_version": str(manifest.get("benchmark_version") or ""),
            "provider": str(provider.get("name") or ""),
            "model_name": str(provider.get("model") or ""),
            "model_version": str(provider.get("model_version") or ""),
            "prompt_template_path": prompt_path,
            "prompt_template_sha": str(manifest.get("prompt_template_hash") or ""),
            "temperature": scalar_to_str(gen.get("temperature")),
            "top_p": scalar_to_str(
                gen.get("top_p")
                if gen.get("top_p") is not None
                else card.get("sampling_top_p")
            ),
            "max_output_tokens": scalar_to_str(
                gen.get("max_tokens")
                if gen.get("max_tokens") is not None
                else card.get("sampling_max_output_tokens")
            ),
            "num_samples_per_instance": str(card.get("num_samples_per_instance") or ""),
            "selection_rule": str(card.get("selection_rule") or ""),
            "reference_obligations_visible": str(
                card.get("reference_obligations_visible") or ""
            ),
            "scaffold_visible": str(card.get("scaffold_visible") or ""),
            "visible_context_fields": "|".join(
                [str(x) for x in card.get("visible_context_fields", [])]
            ),
            "run_manifest_path": str(manifest_path.relative_to(ROOT)).replace("\\", "/"),
            "system_card_path": str(card_path.relative_to(ROOT)).replace("\\", "/"),
            "manifest_prompt_template_sha": str(
                manifest.get("prompt_template_hash") or ""
            ),
            "card_prompt_template_path": str(card.get("prompt_template_path") or ""),
            "card_provider": str(card.get("provider_id") or ""),
            "card_model_name": str(card.get("model_name") or ""),
            "card_model_version": str(card.get("model_version") or ""),
            "card_temperature": str(card.get("sampling_temperature") or ""),
            "card_max_output_tokens": str(
                card.get("sampling_max_output_tokens") or ""
            ),
            "card_num_samples_per_instance": str(
                card.get("num_samples_per_instance") or ""
            ),
            "card_selection_rule": str(card.get("selection_rule") or ""),
        }
        row["model_metadata_status"] = compute_status(row)
        rows.append(row)

    args.out.parent.mkdir(parents=True, exist_ok=True)
    fields = [
        "system_id",
        "paper_run_id",
        "benchmark_version",
        "provider",
        "model_name",
        "model_version",
        "prompt_template_path",
        "prompt_template_sha",
        "temperature",
        "top_p",
        "max_output_tokens",
        "num_samples_per_instance",
        "selection_rule",
        "reference_obligations_visible",
        "scaffold_visible",
        "visible_context_fields",
        "run_manifest_path",
        "system_card_path",
        "model_metadata_status",
    ]
    with args.out.open("w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        for row in rows:
            status = row["model_metadata_status"]
            if status not in ALLOWED_STATUS:
                raise RuntimeError(f"invalid model_metadata_status: {status}")
            w.writerow({k: row.get(k, "") for k in fields})
    print(f"wrote {args.out} ({len(rows)} rows)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
