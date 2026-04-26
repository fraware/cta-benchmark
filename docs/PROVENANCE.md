# Provenance and epistemic tiers (CTA benchmark)

This document is the **single authoritative map** from repository artifacts to claims you may make in a paper. If a claim is not backed by a row here (or by a cited run manifest), treat it as **not established** by the repo alone.

## Definitions

| Tier | Meaning |
|------|---------|
| **human_gold** | Independent annotators + adjudicator; primary labels for metrics and agreement are human decisions. |
| **pipeline_derived** | Labels or scores produced deterministically from checked-in review packets, Lean/behavior hygiene, or similar automated rules—not human semantic judgments. |
| **synthetic_inter_rater** | Second rater (or jitter layer) generated for agreement statistics or stress-testing; not a human pass. |
| **automated_hygiene** | Binary or structural checks (schema validation, elaboration flags, counterexample harness)—supporting quality, not semantic gold. |

## Artifact table

| Artifact path | Supports claim | Epistemic tier | Limitations | Regeneration |
|---------------|----------------|----------------|---------------|--------------|
| [`benchmark/v0.3/annotation/adjudicated_subset/pack.json`](../benchmark/v0.3/annotation/adjudicated_subset/pack.json) | Coverage of eval `(instance, system)` pairs; obligation-level labels used by metrics tooling | `pipeline_derived` (unless replaced by human pack) | Not crowdsourced gold; see `annotator_notes` per record | `python scripts/materialize_v03_adjudication_artifacts.py` |
| [`benchmark/v0.3/annotation/adjudicated_subset/manifest.json`](../benchmark/v0.3/annotation/adjudicated_subset/manifest.json) | Pair counts vs split; optional `epistemic_tier` / `input_hashes` | `mixed` | `cargo … annotate coverage` may refresh counts | `cargo run -p cta_cli -- annotate coverage …` or materializer |
| [`benchmark/v0.3/protocol_freeze.json`](../benchmark/v0.3/protocol_freeze.json) | Frozen protocol IDs, split, metric/rubric versions for a paper wave | N/A (registry) | Does not prove human annotation completed | `python scripts/sign_or_hash_protocol.py` |
| [`results/raw_metrics.json`](../results/raw_metrics.json) | Per-instance scalar summaries feeding tables | `pipeline_derived` (current default) | Same provenance as adjudicated pack derivation | `python scripts/materialize_v03_adjudication_artifacts.py` |
| [`annotation/rater_a.csv`](../annotation/rater_a.csv) / [`annotation/rater_b.csv`](../annotation/rater_b.csv) | Ordinal agreement inputs | `synthetic_inter_rater` for B unless replaced | Classical two-human κ requires real rater B export | Materializer + future human CSVs |
| [`annotation/agreement_report.json`](../annotation/agreement_report.json) | κ, AC, α, bootstrap CIs | Statistics over whatever raters supplied | Interpretation depends on rater tier | `python scripts/compute_agreement_stats.py …` |
| [`annotation/agreement_report.md`](../annotation/agreement_report.md) | Human-readable agreement summary | Same as JSON | — | same as above |
| [`benchmark/v0.3/benchmark_paper_summary.json`](../benchmark/v0.3/benchmark_paper_summary.json) | Instance counts, splits, families | `automated_hygiene` / bookkeeping | Counts only; no quality claim | `python scripts/export_benchmark_paper_summary.py` |
| [`schemas/failure_mode_v1.json`](../schemas/failure_mode_v1.json) | Allowed `failure_mode_label` slugs | `automated_hygiene` | Sparse labels; derived `low_faithfulness` still used in tables | Curator edit + CI |
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
