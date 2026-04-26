# Limitations and threats to validity

## Annotation and gold labels

- Default adjudication artifacts are **pipeline-derived** from curated review
  packets, not independent crowdsourced gold. Wording in the paper must match
  the tier named in `docs/PROVENANCE.md`.
- Inter-rater agreement CSVs include a **deterministic synthetic rater B**
  layer so coefficients are numerically stable; treat them as methodology
  demos, not human reliability from two independent experts.

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
  into derived `low_faithfulness` counts for tables (`docs/failure_mode_ontology.md`).

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
