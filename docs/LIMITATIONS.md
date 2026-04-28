# Limitations and threats to validity

## Annotation and gold labels

- Default adjudication artifacts are **pipeline-derived** from curated review
  packets, not independent crowdsourced gold. Wording in the paper must match
  the tier named in `docs/PROVENANCE.md`.
- Inter-rater agreement defaults to a **deterministic synthetic rater B**
  layer unless `annotation/rater_b_human.csv` is provided and used to
  regenerate `annotation/agreement_report*.{json,md}`. Treat synthetic-mode
  agreement as an audit/stress-test artifact, not independent two-human
  reliability. The agreement
  packet population is still **pipeline-keyed** to eval `(instance, system)`
  rows; cite `annotation/agreement_packet_ids.csv` and distinguish headline
  **strict** metrics rows from agreement table construction in the text.
  `results/paper_table_agreement_evidence.csv` quantifies how many agreement
  packets fall under each `annotation_origin` (the strict packet subset may be
  empty while the headline eval table remains strict-backed).

## Metrics

- Primary scalars in `results/raw_metrics.json` (expanded view) are **point
  estimates** per instance and system; some rows arise from
  `mapped_from_canonical` propagation. Conservative claims should cite
  `results/raw_metrics_strict.json` and the smaller effective N.
- Aggregate CSVs pool instances; bootstrap intervals in
  `results/system_summary_with_ci.json` apply to those pooled means only (see
  `docs/evaluation_contract.md`). Do not describe `system_summary.csv` as a full
  reliability headline without also citing the per-metric and reliability CSVs.
- Failure-mode labels are sparse; empty labels with low faithfulness are folded
  into derived counts and manuscript exports canonicalize the label as
  `low_semantic_faithfulness` (`docs/failure_mode_ontology.md`).

## Repairs

- Hotspot repair is **budgeted** and documented; it is not a universal fix for
  all low-scoring pairs. Counterfactual summaries use a conservative proxy
  described in `results/repair_impact_summary.json`.

## External validity

- Instances are classical algorithmic tasks with reference Rust; systems are
  four fixed baselines. Performance on other domains, languages, or provider
  models may not transfer.

## Reproducibility

- `build/paper_build.json` records command hashes for a full rebuild. Provider
  keys for non-stub runs remain outside the repository.

## Evidence-Hardening Update (2026-04-28)

- Independent strict-overlap agreement artifacts now live under
  `annotation/human_pass_v3/` (plus `annotation/rater_a_strict_all.csv`) and are
  summarized in `results/paper_table_agreement_evidence.csv` via
  `strict_all_human_overlap`.
- Selector sensitivity and context-budget confounds are now explicitly exported
  in `results/selection_robustness.csv` and
  `results/prompt_token_accounting.csv`.
- Cross-model sanity and repair-denominator transparency are now documented in
  `results/cross_model_pilot_*` and `repairs/repair_attempts.csv`.
