# Limitations and threats to validity

## Annotation and gold labels

- Default adjudication artifacts are **pipeline-derived** from curated review
  packets, not independent crowdsourced gold. Wording in the paper must match
  the tier named in `docs/PROVENANCE.md`.
- There are two agreement layers in-repo: legacy full-audit agreement
  (`annotation/agreement_report*.{json,md}`; may involve synthetic rater-B) and
  strict-overlap v3 human agreement
  (`annotation/human_pass_v3/agreement_report_human_strict_all.{json,md}`).
  Manuscript claims about independent human agreement must cite the strict-overlap
  v3 layer, not legacy synthetic compatibility outputs.
- Agreement packet construction remains **pipeline-keyed** to eval
  `(instance, system)` rows. Cite `results/paper_table_agreement_evidence.csv`
  and explicitly distinguish `strict_all_human_overlap` from legacy
  `strict_independent_only`.

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
