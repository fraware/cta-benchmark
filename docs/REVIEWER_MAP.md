# Reviewer map (artifacts ↔ paper)

| Paper topic | Primary artifacts | Regenerate |
|-------------|-------------------|------------|
| Benchmark scale (instances, splits, families) | `benchmark/v0.3/benchmark_paper_summary.json` | `python scripts/export_benchmark_paper_summary.py` |
| Frozen protocol identifiers | `benchmark/v0.3/protocol_freeze.json` | `python scripts/sign_or_hash_protocol.py --benchmark-version v0.3` |
| Per-instance adjudicated metrics (expanded) | `results/raw_metrics.json`, `results/raw_metrics_expanded.json` | `python scripts/materialize_v03_adjudication_artifacts.py` (includes `mapped_from_canonical` propagation) |
| Per-instance metrics (strict independent evidence) | `results/raw_metrics_strict.json` | same materializer (`direct_human` / `direct_adjudicated` rows only) |
| Aggregate tables (per metric; preferred) | `results/system_faithfulness_summary.csv`, `results/system_consistency_summary.csv`, `results/system_vacuity_summary.csv`, `results/system_proof_utility_summary.csv`, `results/system_reliability_summary.csv`, `results/system_reliability_sensitivity.csv`, `results/instance_level.csv` | `python scripts/compute_results.py --paper` |
| Legacy faithfulness-only aliases | `results/system_summary.csv`, `results/family_summary.csv` | same; **do not** describe as full “system reliability” without naming the metric |
| Family × system (per metric) | `results/family_faithfulness_summary.csv`, `results/family_consistency_summary.csv`, `results/family_vacuity_summary.csv`, `results/family_proof_utility_summary.csv` | same as `compute_results.py --paper` |
| Bootstrap summaries | `results/system_summary_with_ci.json` | same as `compute_results.py` |
| Annotation evidence inventory (eval metrics rows; quote in paper) | `results/paper_table_annotation_evidence.csv` | `python scripts/compute_results.py --paper` |
| Agreement audit population (packet-level origins) | `results/paper_table_agreement_evidence.csv` | same (joins `annotation/agreement_packet_ids.csv`) |
| Publication-facing tables — **headline (strict)** | `results/paper_table_systems.csv`, `results/paper_table_families.csv`, `results/paper_table_failure_modes.csv`, `results/paper_table_repairs.csv` | same (strict `raw_metrics_strict.json` pipeline) |
| Appendix — **expanded mapped** | `results/appendix_mapped_evidence/paper_table_*.csv` and sibling summaries | same command (second pass inside `compute_results.py --paper`) |
| Inter-rater agreement (synthetic rater B) | `annotation/agreement_report.json`, `annotation/agreement_report.md` | `python scripts/reproduce_agreement_report.py` (wraps `compute_agreement_stats.py`) |
| Agreement audit trail (packet population) | `annotation/agreement_packet_ids.csv`, `annotation/rater_a.csv`, `annotation/rater_b.csv`, `annotation/adjudication_log.csv` | `python scripts/materialize_v03_adjudication_artifacts.py` |
| Repair selection + logs | `repairs/hotspot_selection.csv`, `repairs/repair_log.jsonl` | `python scripts/materialize_repair_hotspot_artifacts.py` |
| Repair sensitivity (counterfactual proxy) | `results/repair_impact_summary.json` | `python scripts/repair_counterfactual_metrics.py` |
| One-shot audit bundle | `build/paper_build.json` (local; under `/build` in `.gitignore`) | `python scripts/paper_bundle.py` (set `PAPER_STRICT=1` to forbid `demo_synthetic` markers in scanned outputs) |

### CI-enforced summary contract

`python scripts/ci_reviewer_readiness.py` (same command as the *paper reviewer readiness*
step in `.github/workflows/ci.yml`) must pass after
`python scripts/export_benchmark_paper_summary.py`. It ties
`benchmark/v0.3/benchmark_paper_summary.json` to `results/instance_level.csv`,
`results/raw_metrics.json` / `raw_metrics_strict.json`, agreement audit CSV/JSON,
`results/paper_table_annotation_evidence.csv`, and
`results/paper_table_agreement_evidence.csv` (see `CI_STATUS.md` for the exact fields).

## Definitions and limitations

- `docs/PROVENANCE.md` — epistemic tiers.
- `docs/LIMITATIONS.md` — threats to validity.
- `docs/evaluation_contract.md` — metric definitions.
- `docs/annotator_calibration_v03.md` — human-wave calibration anchors.
