# Evaluation contract

Authoritative definitions of every metric reported in the paper and the
acceptance conditions a run must satisfy before it is paper-reportable.

Version: `metrics_v1`. The definitions below are frozen under this version.
Changes require a new `metrics_v2` contract; old metric names never change
meaning.

## Primary metrics

All primary metrics are reported as scalars in `[0, 1]` unless noted.

### elaboration_rate

Fraction of instances whose generated Lean file elaborated successfully.

Denominator: number of instances in the split.
Numerator: instances with `elaborated == true`.

### semantic_faithfulness_mean

Instance-average of `num_faithful / num_obligations` over instances that
produced at least one obligation. Empty-output instances are excluded from
the denominator; they contribute to `elaboration_rate` and coverage but
not to this mean.

### critical_unit_coverage

Fraction of critical semantic units covered across the split.

Denominator: `sum_i critical_units_total_i`.
Numerator:   `sum_i critical_units_covered_i`.

### rust_consistency_rate

`1 - total_inconsistent / total_obligations`, summed across the split.
If `total_obligations == 0`, the metric is `0.0` and the run is not
paper-eligible.

### vacuity_rate

`total_vacuous / total_obligations` across the split.

### proof_utility

Fraction of instances where at least one obligation from the generated set
was used in a (hand-written or machine-assisted) proof attempt. The proof
attempt itself is out of scope for automated pipelines; the annotator
records the boolean per instance.

## Secondary metrics

- `avg_obligations_per_instance`: total obligations / number of instances.
- `faithful_obligation_density`: total faithful / total obligations.
- `contradiction_rate_on_critical_units`: share of obligations linked to
  at least one critical SU that were labeled `inconsistent`.
- `text_faithful_code_inconsistent_rate`: among obligations labeled
  `faithful`, the fraction also labeled `inconsistent` against the
  reference Rust.
- `code_faithful_text_incomplete_rate`: reserved for NL-gloss audits.
  Emitted as `0.0` until `metrics_v2` adds an NL semantics signal.
- `inter_annotator_agreement` (emitted only when a raw annotator set is
  supplied):
  - `weighted_kappa_faithfulness`: linearly-weighted Cohen's kappa on
    the ordinal faithfulness scale
    `unfaithful < partial < ambiguous < faithful`. Averaged across all
    annotator pairs within each `(instance, system)` group.
  - `cohen_kappa_vacuity`: Cohen's kappa on the binary vacuity label.
  - `raw_agreement_coverage`: fraction of SUs labeled identically
    (covered vs missed) by both annotators, over the union of SUs
    either annotator touched. `1.0` if neither annotator touched any
    SU.

## Adjudication policies

Two canonical policies are supported when reducing a multi-annotator
group to a single adjudicated record:

- `prefer-adjudicator` (default for paper reports): if an
  `annotator_id == "adjudicator"` record is present, it is taken
  verbatim. Otherwise the single-annotator group passes through
  unchanged, or multi-annotator groups fail loudly demanding an
  adjudicator.
- `majority`: ignore any adjudicator record and synthesise the
  canonical annotation from per-annotator majority vote (for
  categorical labels) and simple averaging (for set-level scalars).
  Ties are broken deterministically by annotator order. This policy
  is useful for sensitivity analysis.

The adjudicated pack is stored under
`runs/annotation_packs/<version>-adjudicated.json` and is the single
input consumed by `cta metrics compute` and `cta reports build`.

## Acceptance criteria for a paper-reportable run

A run is paper-eligible iff **all** of the following hold:

1. Every instance in the target split validated against its schema.
2. No schema validation failure across the run's artifacts.
3. A `run_manifest.json` is present and validates against
   `schemas/run_manifest.schema.json`.
4. Lean elaboration completed (success or failure, but not timeout) for
   every instance.
5. The annotation subset required by the split is complete: two
   independent annotators plus adjudication on any disagreement.
6. The metrics bundle (`results_bundle.json`) was generated successfully.

If any condition fails, the run is stored under `runs/` for
reproducibility but is not included in paper tables.

## End-to-end pipeline (operator's view)

```
cta experiment --config configs/experiments/<id>.json
   |
   +-- for each (system, provider, seed):
   |     cta generate        -> runs/<run_id>/generated/<system>/*.json
   |                            runs/<run_id>/generated/<system>/raw/*.txt
   |                            runs/<run_id>/run_manifest.json
   |
   +-- if config.annotation_pack is set:
   |     cta metrics compute -> runs/<run_id>/results_bundle.json
   |     cta reports build   -> runs/<run_id>/reports/primary_metrics.csv
   |                            runs/<run_id>/reports/instance_results.csv
   |                            runs/<run_id>/reports/results.md
   |                            runs/<run_id>/reports/results.tex
   |
   +-- finally:
         runs/experiments/<experiment_id>/summary.json
```

Every step writes schema-valid JSON. CI re-validates the emitted
`run_manifest.json` and `results_bundle.json` via `cta validate file`
before declaring the run green. The experiment orchestrator never
mutates benchmark artifacts and halts on the first schema violation.

Standalone equivalents of each stage are also available for running the
pipeline outside an experiment config:

- `cta generate --version <v> --split <s> --system <sys> --provider <cfg>`
- `cta annotate pack --version <v> [--policy prefer-adjudicator|majority]`
- `cta metrics compute --run <run_id> --annotations <pack.json>
  [--raw-annotations <dir>]`
- `cta reports build --run <run_id>`

The `--raw-annotations <dir>` flag on `cta metrics compute` enables
inter-annotator agreement reporting by pointing the runner at the raw
per-annotator directory (the same directory consumed by
`cta annotate pack`). Without it, the `inter_annotator_agreement` block
is elided from `results_bundle.json`.
