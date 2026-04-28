# Provenance and epistemic tiers (CTA benchmark)

This document is the **single authoritative map** from repository artifacts to claims you may make in a paper. If a claim is not backed by a row here (or by a cited run manifest), treat it as **not established** by the repo alone.

## Definitions

| Tier | Meaning |
|------|---------|
| **human_gold** | Independent annotators + adjudicator; primary labels for metrics and agreement are human decisions. |
| **pipeline_derived** | Labels or scores produced deterministically from checked-in review packets, Lean/behavior hygiene, or similar automated rules—not human semantic judgments. |
| **synthetic_inter_rater** | Second rater (or jitter layer) generated for agreement statistics or stress-testing; not a human pass. |
| **automated_hygiene** | Binary or structural checks (schema validation, elaboration flags, counterexample harness)—supporting quality, not semantic gold. |

## Paper-facing provenance layers

For paper tables and reviewer communication, use these explicit layer tags:

- `human_gold`: independent human validation layer (for example, `annotation/human_pass_v3/`).
- `synthetic_stress`: synthetic/stress second-rater agreement layer.
- `adjudicated`: direct adjudicated strict benchmark layer used in headline metrics.

Machine-readable registry: `results/provenance_layer_registry.csv`.

## Artifact table

| Artifact path | Supports claim | Epistemic tier | Limitations | Regeneration |
|---------------|----------------|----------------|---------------|--------------|
| [`benchmark/v0.3/annotation/adjudicated_subset/pack.json`](../benchmark/v0.3/annotation/adjudicated_subset/pack.json) | Coverage of eval `(instance, system)` pairs; obligation-level labels used by metrics tooling | `pipeline_derived` (unless replaced by human pack) | Not crowdsourced gold; see `annotator_notes` per record | `python scripts/materialize_v03_adjudication_artifacts.py` |
| [`benchmark/v0.3/annotation/adjudicated_subset/manifest.json`](../benchmark/v0.3/annotation/adjudicated_subset/manifest.json) | Pair counts vs split; optional `epistemic_tier` / `input_hashes` | `mixed` | `cargo … annotate coverage` may refresh counts | `cargo run -p cta_cli -- annotate coverage …` or materializer |
| [`benchmark/v0.3/protocol_freeze.json`](../benchmark/v0.3/protocol_freeze.json) | Frozen protocol IDs, split, metric/rubric versions for a paper wave | N/A (registry) | Does not prove human annotation completed | `python scripts/sign_or_hash_protocol.py` |
| [`results/raw_metrics.json`](../results/raw_metrics.json) | Per-instance scalar summaries (alias of expanded view) | `pipeline_derived` | Same provenance as adjudicated pack derivation | `python scripts/materialize_v03_adjudication_artifacts.py` |
| [`results/raw_metrics_expanded.json`](../results/raw_metrics_expanded.json) | Explicit **expanded** view including `mapped_from_canonical` | `pipeline_derived` | Inflates eval-grid rows via template packets; headline tables must label view | same as above |
| [`results/raw_metrics_strict.json`](../results/raw_metrics_strict.json) | **Strict headline view** (`direct_*` origins only) | `pipeline_derived` | Smaller N; this is the independently double-annotated strict view used for headline claims | same as above |
| [`results/paper_primary_model_registry.csv`](../results/paper_primary_model_registry.csv) | Headline-run-only model/provider metadata for the four paper systems | `automated_hygiene` | Run-manifest metadata can predate prompt-template revisions; use `model_metadata_status` explicitly | `python scripts/export_paper_primary_model_registry.py` (via `compute_results.py --paper`) |
| [`annotation/external_review/strict_review_queue.jsonl`](../annotation/external_review/strict_review_queue.jsonl), [`annotation/external_review/mapped_review_queue.jsonl`](../annotation/external_review/mapped_review_queue.jsonl) | External audit queues for strict direct rows and mapped rows | `pipeline_derived` | Packaging for review workflow; not a new annotation layer | `python scripts/export_external_annotation_review_bundle.py` (via `compute_results.py --paper`) |
| [`annotation/external_review/semantic_corrections_v3.csv`](../annotation/external_review/semantic_corrections_v3.csv) (fallback: cumulative [`semantic_corrections_v1.csv`](../annotation/external_review/semantic_corrections_v1.csv) + [`semantic_corrections_v2.csv`](../annotation/external_review/semantic_corrections_v2.csv)) | Explicit correction overlay for obligation-level semantic labels keyed by canonical template, system, and obligation index | `human_gold` (external review input) applied by pipeline | Only rows explicitly listed are changed; v3 includes faithfulness/vacuity/coverage overlays, and coverage is recomputed obligation-by-obligation from corrected labels; downstream metrics must be regenerated after edits | Update CSV, then run `python scripts/materialize_v03_adjudication_artifacts.py` and `python scripts/compute_results.py --paper` |
| [`annotation/agreement_packet_ids.csv`](../annotation/agreement_packet_ids.csv) | Maps anonymized agreement keys to eval `(instance, system)` | `automated_hygiene` | Public raters use opaque keys | `materialize_v03_adjudication_artifacts.py` |
| [`annotation/adjudication_log.csv`](../annotation/adjudication_log.csv) | Adjudication outcome per agreement key (and `system_id`) | `pipeline_derived` | Same derivation tier as the adjudicated pack | same as above |
| [`results/paper_table_annotation_evidence.csv`](../results/paper_table_annotation_evidence.csv) | Row counts by `annotation_origin` for strict vs expanded **eval metric** views | `pipeline_derived` | One row per metrics view; use for paper Methods | `python scripts/compute_results.py --paper` |
| [`results/paper_table_agreement_evidence.csv`](../results/paper_table_agreement_evidence.csv) | Same origin tallies for **agreement audit packets** (`full_audit_population`, `strict_independent_only`, `strict_all_human_overlap`) | `pipeline_derived` | Separate from eval headline N; cite beside κ and strict-overlap evidence | same as above |
| [`annotation/rater_a_strict_all.csv`](../annotation/rater_a_strict_all.csv), [`annotation/human_pass_v3/rater_b_human_strict_all.csv`](../annotation/human_pass_v3/rater_b_human_strict_all.csv), [`annotation/human_pass_v3/agreement_report_human_strict_all.json`](../annotation/human_pass_v3/agreement_report_human_strict_all.json) | Independent strict-overlap human agreement layer used for headline human-agreement claims | `human_gold` (strict-overlap) | Must satisfy fixed invariants (`n_rows=274`, `n_unique_instance_ids=84`, `n_mapped_from_canonical=0`) and canonical ordinal scale `{0,1,2,3}`; confusion totals must equal 274 per ordinal metric | `python scripts/compute_human_strict_agreement.py ...` and `python scripts/ci_reviewer_readiness.py` |
| [`results/paper_table_*.csv`](../results/) | Manuscript headline wide tables (**strict** rows) | `pipeline_derived` | Expanded counterparts under `appendix_mapped_evidence/`; strict `missing_critical_semantic_unit` counts come from strict raw rows with `missing_critical_units > 0` | `python scripts/export_paper_tables.py` (via `compute_results.py --paper`) |
| [`results/appendix_mapped_evidence/`](../results/appendix_mapped_evidence/) | Same table shapes on **expanded** mapped raw metrics | `pipeline_derived` | Robustness / appendix only | same as above |
| [`annotation/rater_a.csv`](../annotation/rater_a.csv) / [`annotation/rater_b.csv`](../annotation/rater_b.csv) / [`annotation/rater_b_human.csv`](../annotation/rater_b_human.csv) | Legacy full-audit agreement inputs (compatibility layer) | `synthetic_inter_rater` for default `rater_b.csv`; `human_gold` when `rater_b_human.csv` is used | Not the strict-overlap headline agreement layer; use `human_pass_v3` artifacts for independent strict-overlap claims | `materialize_v03_adjudication_artifacts.py` + optional human pass; `reproduce_agreement_report.py` |
| [`annotation/agreement_report.json`](../annotation/agreement_report.json) | Legacy full-audit κ/AC/α agreement report | Statistics over supplied legacy raters | Compatibility/audit context only; strict-overlap headline claims must cite `agreement_report_human_strict_all.json` | `python scripts/reproduce_agreement_report.py` (or `compute_agreement_stats.py`) |
| [`annotation/agreement_report.md`](../annotation/agreement_report.md) | Human-readable agreement summary | Same as JSON | — | same as above |
| [`benchmark/v0.3/benchmark_paper_summary.json`](../benchmark/v0.3/benchmark_paper_summary.json) | Instance counts, splits, families, agreement-audit row expectations, `agreement_audit_*` provenance flags | `automated_hygiene` / bookkeeping | Includes `agreement_audit_design_note`; not a human-reliability claim | `python scripts/export_benchmark_paper_summary.py` |
| [`schemas/failure_mode_v1.json`](../schemas/failure_mode_v1.json) | Allowed `failure_mode_label` slugs | `automated_hygiene` | Sparse labels; manuscript exports canonicalize derived low-faithfulness rows to `low_semantic_faithfulness` | Curator edit + CI |
| [`results/system_summary_with_ci.json`](../results/system_summary_with_ci.json) | Bootstrap uncertainty on pooled means | `pipeline_derived` | Not per-instance CIs; see `docs/evaluation_contract.md` | `python scripts/compute_results.py --paper` |
| [`results/repair_impact_summary.json`](../results/repair_impact_summary.json) | Repair vs counterfactual proxy means | `pipeline_derived` | Proxy definition is explicit in-file | `python scripts/repair_counterfactual_metrics.py` |
| [`build/paper_build.json`](../build/paper_build.json) | One-shot rebuild index (hashes, toolchain) | `automated_hygiene` | Generated locally or in CI; not a scientific claim | `python scripts/paper_bundle.py` |
| [`repairs/hotspot_selection.csv`](../repairs/hotspot_selection.csv), [`repairs/repair_log.jsonl`](../repairs/repair_log.jsonl) | Repair study protocol and logs | `pipeline_derived` + file-backed hashes | Selection rule must match pre-spec (see repair protocol doc) | `python scripts/materialize_repair_hotspot_artifacts.py` |
| [`experiments/run_manifests/*.json`](../experiments/run_manifests/) | Provider run metadata; optional `cost_reporting: { status, reason_code?, estimated_cost_usd? }` on system cards (`schemas/run_manifest.schema.json`) | Varies per file | `unreported_with_reason` is explicit reviewer-safe semantics vs a bare null | Experiment orchestration |

## Human-gold slot (empty until populated)

When a human wave completes, place:

- `benchmark/v0.3/annotation/human_adjudicated/pack.json`
- Optional CSVs under `benchmark/v0.3/annotation/human_wave_v03/`

Set experiment field `annotation_human_pack` in [`configs/experiments/benchmark_v03.json`](../configs/experiments/benchmark_v03.json) to that path. The CLI resolves this path **when the file exists** for `annotate plan` default pack resolution; release checks still validate primary `annotation_pack` unless you switch the canonical path deliberately.

## Related docs

- [`docs/REVIEWER_MAP.md`](REVIEWER_MAP.md) — section → artifact → command.
- [`docs/LIMITATIONS.md`](LIMITATIONS.md) — threats to validity.
- [`docs/evaluation_contract.md`](evaluation_contract.md) — metric definitions.
- [`docs/annotation_sampling_v03.md`](annotation_sampling_v03.md) — human wave stratification.
- [`docs/repair_protocol_v03.md`](repair_protocol_v03.md) — repair inclusion and counterfactual reporting.
- [`docs/failure_mode_ontology.md`](failure_mode_ontology.md) — failure labels.
- [`docs/annotator_calibration_v03.md`](annotator_calibration_v03.md) — human-wave calibration anchors.

## Evidence-Hardening Update (2026-04-28)

Packaging and integrity are now validated against
`artifacts/evidence_hardening_manifest.json` via
`python scripts/validate_release_artifact.py`. CI invokes this validator through
`python scripts/ci_reviewer_readiness.py` when the manifest is present.
