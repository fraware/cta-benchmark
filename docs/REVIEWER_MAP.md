# Reviewer map (artifacts ↔ paper)

| Paper topic | Primary artifacts | Regenerate |
|-------------|-------------------|------------|
| Benchmark scale (instances, splits, families) | `benchmark/v0.3/benchmark_paper_summary.json` | `python scripts/export_benchmark_paper_summary.py` |
| Frozen protocol identifiers | `benchmark/v0.3/protocol_freeze.json` | `python scripts/sign_or_hash_protocol.py --benchmark-version v0.3` |
| Per-instance adjudicated metrics (expanded) | `results/raw_metrics.json`, `results/raw_metrics_expanded.json` | `python scripts/materialize_v03_adjudication_artifacts.py` (includes `mapped_from_canonical` propagation) |
| Per-instance metrics (strict independent evidence) | `results/raw_metrics_strict.json` | same materializer (`direct_human` / `direct_adjudicated` rows only) |
| Aggregate tables (per metric; preferred) | `results/system_faithfulness_summary.csv`, `results/system_consistency_summary.csv`, `results/system_vacuity_summary.csv`, `results/system_proof_utility_summary.csv`, `results/system_reliability_summary.csv`, `results/system_reliability_sensitivity.csv`, `results/instance_level.csv` | `python scripts/compute_results.py --paper` |
| Legacy faithfulness-only aliases | `results/system_summary.csv`, `results/family_summary.csv` | same; **do not** describe as full “system reliability” without naming the metric |
| Family × system (per metric) | `results/family_faithfulness_summary.csv`, `results/family_consistency_summary.csv`, `results/family_vacuity_summary.csv`, `results/family_proof_utility_summary.csv`, `results/family_reliability_summary.csv` | same as `compute_results.py --paper` |
| Bootstrap summaries | `results/system_summary_with_ci.json` | same as `compute_results.py` |
| Annotation evidence inventory (eval metrics rows; quote in paper) | `results/paper_table_annotation_evidence.csv` | `python scripts/compute_results.py --paper` |
| Agreement audit population (packet-level origins) | `results/paper_table_agreement_evidence.csv` | same (joins `annotation/agreement_packet_ids.csv`) |
| Publication-facing tables — **headline (strict)** | `results/paper_strict_*` (explicit strict layer), `results/paper_table_*.csv` (legacy compat) | `python scripts/compute_results.py --paper` → `scripts/export_paper_tables.py` |
| Appendix — **expanded mapped** | `results/paper_expanded_*` (copies promoted to `results/`), `results/appendix_mapped_evidence/paper_table_*.csv` | same pipeline (appendix pass inside `compute_results.py --paper`) |
| Failure-mode exports (manuscript-ready) | `results/paper_strict_failure_modes.csv`, `results/paper_expanded_failure_modes.csv` (`evidence_view`, `system`, `family`, `failure_mode`, `count`, `share_within_system`, `share_global`, denominator/rate columns) | same |
| Semantic correction overlay (external audit) | `annotation/external_review/semantic_corrections_v1.csv` (obligation-level overrides keyed by canonical `instance_id`, `system_id`, `obligation_index`) | update CSV then rerun `python scripts/materialize_v03_adjudication_artifacts.py` and `python scripts/compute_results.py --paper` |
| Evidence mass | `results/paper_annotation_origin_counts.csv` | same |
| Primary system-set statement | `results/paper_system_set.md`, `benchmark/v0.3/benchmark_paper_summary.json` | `python scripts/export_benchmark_paper_summary.py` |
| Per-metric winners (stacked) | `results/paper_strict_system_metrics_long.csv`, `results/paper_strict_system_*_summary.csv` | same as headline `compute_results.py --paper` |
| Repair proof-status roster | `repairs/paper_repair_status.csv` | `python scripts/export_paper_repair_status.py` (also end of `compute_results.py --paper`) |
| Repair manuscript subset (selected only) | `repairs/paper_repair_success_subset.csv` | same |
| Repair proof-facing subset (elaborated only) | `repairs/paper_repair_proof_subset.csv` (selected-budget + elaborated analysis view), `repairs/paper_proof_facing_subset.csv` (paper-facing metadata-rich view over all hotspot rows with `elaborated=true`) | same |
| Inter-rater agreement (tier declared in report audit metadata) | `annotation/agreement_report.json`, `annotation/agreement_report.md` | `python scripts/reproduce_agreement_report.py` (prefers `annotation/rater_b_human.csv` when present, else `rater_b.csv`) |
| Agreement audit trail (packet population) | `annotation/agreement_packet_ids.csv`, `annotation/rater_a.csv`, `annotation/rater_b.csv` (or `annotation/rater_b_human.csv`), `annotation/adjudication_log.csv` | `python scripts/materialize_v03_adjudication_artifacts.py` + optional human `rater_b_human.csv` import |
| Repair selection + logs | `repairs/hotspot_selection.csv`, `repairs/repair_log.jsonl` | `python scripts/materialize_repair_hotspot_artifacts.py` |
| Option-2 direct-adjudication wave plan | `benchmark/v0.3/annotation/human_wave_v03/direct_adjudication_wave_plan.csv`, `benchmark/v0.3/annotation/human_adjudicated/direct_adjudicated_pairs.csv` | `python scripts/plan_v03_direct_adjudication_wave.py` then `python scripts/materialize_v03_adjudication_artifacts.py` |
| Repair sensitivity (counterfactual proxy) | `results/repair_impact_summary.json` | `python scripts/repair_counterfactual_metrics.py` |
| Strict coverage gap disclosure | `results/paper_strict_coverage_gap.csv` (missing strict unique instances/families vs expanded view) | `python scripts/compute_results.py --paper` |
| Cost/runtime accounting | `results/paper_cost_runtime_accounting.csv` | `python scripts/export_cost_runtime_accounting.py` (also run by `compute_results.py --paper`) |
| Model metadata reconciliation | `results/paper_model_metadata_registry.csv` | `python scripts/export_model_metadata_registry.py` (also run by `compute_results.py --paper`) |
| Paper-primary model registry | `results/paper_primary_model_registry.csv` (headline-run-only model/provider metadata) | `python scripts/export_paper_primary_model_registry.py` (also run by `compute_results.py --paper`) |
| External annotation review bundle | `annotation/external_review/strict_review_queue.jsonl`, `annotation/external_review/strict_review_queue.csv`, `annotation/external_review/mapped_review_queue.jsonl`, `annotation/external_review/review_schema.md` | `python scripts/export_external_annotation_review_bundle.py` (also run by `compute_results.py --paper`) |
| Strict-gap fixed 13x4 worklist | `benchmark/v0.3/annotation/human_wave_v03/strict_gap_13x4_worklist.csv`, `benchmark/v0.3/annotation/human_wave_v03/strict_gap_13x4_completion.csv` | `python scripts/strict_gap_13x4_worklist.py` |
| One-shot audit bundle | `build/paper_build.json` (local; under `/build` in `.gitignore`) | `python scripts/paper_bundle.py` (set `PAPER_STRICT=1` to forbid `demo_synthetic` markers in scanned outputs) |

### CI-enforced summary contract

`python scripts/ci_reviewer_readiness.py` (same command as the *paper reviewer readiness*
step in `.github/workflows/ci.yml`) must pass after
`python scripts/export_benchmark_paper_summary.py`. It ties
`benchmark/v0.3/benchmark_paper_summary.json` to `results/instance_level.csv`,
`results/raw_metrics.json` / `raw_metrics_strict.json`, agreement audit CSV/JSON,
`results/paper_table_annotation_evidence.csv`, and
`results/paper_table_agreement_evidence.csv` (see `CI_STATUS.md` for the exact fields).

CI currently enforces summary/evidence-count consistency, not presence of every
publication alias file. `results/paper_strict_*`, `results/paper_expanded_*`,
`results/paper_annotation_origin_counts.csv`, `results/paper_system_set.md`,
and repair exports (`repairs/paper_repair_status.csv`,
`repairs/paper_repair_success_subset.csv`, `repairs/paper_repair_proof_subset.csv`,
`repairs/paper_proof_facing_subset.csv`) are generated in the paper pipeline and
should be checked in manuscript prep reviews.

In strict failure exports, `missing_critical_semantic_unit` is sourced from strict
raw rows with `missing_critical_units > 0` rather than hotspot proxy tags.

## Definitions and limitations

- `docs/PROVENANCE.md` — epistemic tiers.
- `docs/LIMITATIONS.md` — threats to validity.
- `docs/evaluation_contract.md` — metric definitions.
- `docs/annotator_calibration_v03.md` — human-wave calibration anchors.
- `docs/direct_adjudication_wave_checklist.md` — 128-pair wave execution + acceptance gates.
