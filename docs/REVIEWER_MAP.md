# Reviewer map (artifacts ↔ paper)

| Paper topic | Primary artifacts | Regenerate |
|-------------|-------------------|------------|
| Benchmark scale (instances, splits, families) | `benchmark/v0.3/benchmark_paper_summary.json` | `python scripts/export_benchmark_paper_summary.py` |
| Frozen protocol identifiers | `benchmark/v0.3/protocol_freeze.json` | `python scripts/sign_or_hash_protocol.py --benchmark-version v0.3` |
| Per-instance adjudicated metrics | `results/raw_metrics.json` | Materialize from review packets (repo-specific pipeline; see `docs/PROVENANCE.md`) |
| Aggregate tables | `results/system_summary.csv`, `results/family_summary.csv`, `results/instance_level.csv` | `python scripts/compute_results.py --paper` |
| Bootstrap summaries | `results/system_summary_with_ci.json` | same as `compute_results.py` |
| Inter-rater agreement (synthetic rater B) | `annotation/agreement_report.json`, `annotation/agreement_report.md` | `python scripts/compute_agreement_stats.py --first annotation/rater_a.csv --second annotation/rater_b.csv` |
| Repair selection + logs | `repairs/hotspot_selection.csv`, `repairs/repair_log.jsonl` | `python scripts/materialize_repair_hotspot_artifacts.py` |
| Repair sensitivity (counterfactual proxy) | `results/repair_impact_summary.json` | `python scripts/repair_counterfactual_metrics.py` |
| One-shot audit bundle | `build/paper_build.json` (local; under `/build` in `.gitignore`) | `python scripts/paper_bundle.py` (set `PAPER_STRICT=1` to forbid `demo_synthetic` markers in scanned outputs) |

## Definitions and limitations

- `docs/PROVENANCE.md` — epistemic tiers.
- `docs/LIMITATIONS.md` — threats to validity.
- `docs/evaluation_contract.md` — metric definitions.
- `docs/annotator_calibration_v03.md` — human-wave calibration anchors.
