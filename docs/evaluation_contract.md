# Evaluation contract

Authoritative definitions of every metric reported in the paper and the
acceptance conditions a run must satisfy before it is paper-reportable.

Version: `metrics_v2`. The definitions below are frozen under this version.

`metrics_v2` froze two decisions that had previously been ambiguous in the
codebase:

1. `semantic_faithfulness_mean` uses explicit per-label weights, so
   `partial` contributes `0.5` (not `0.0`) and `ambiguous` contributes `0.0`.
2. `rust_consistency_rate` excludes `not_applicable` obligations from both
   numerator and denominator.

`metrics_v1` (the prior contract) is retained only for archival comparison;
the canonical contract today is `metrics_v2` (`cta_core::METRICS_VERSION` and
`cta_metrics::METRICS_VERSION`). The JSON Schemas for `run_manifest` and
`results_bundle` accept any `metrics_vN` string that matches the version
pattern, so a manifest declaring `metrics_v1` can still *validate* as
well-formed JSON. For paper work you must not rely on that: generated runs
and fresh bundles use the current constant, and `cta reports aggregate`
skips bundles whose `aggregate_metrics.metrics_version` differs from the
current contract. Future contract changes must bump to `metrics_v3`; names
of existing metrics never change meaning once frozen.

## Primary metrics

All primary metrics are reported as scalars in `[0, 1]` unless noted.

## Rigorous proof-completion checkpoint (`2026-04-24`)

Paper-track quality gates now assume the following for the previously
axiom-backed target families:

- strict Lean refresh remains green with `--strict-m1`,
- review packets are definition-backed (`proof_mode = "definition_backed"`),
- benchmark-facing obligations are non-vacuous and avoid trivial pass-through
  wrappers.

This is an artifact-quality requirement for reportable runs and complements
metric definitions rather than replacing them.

### elaboration_rate

Fraction of instances whose generated Lean file elaborated successfully.

Denominator: number of instances in the split.
Numerator: instances with `elaborated == true`.

This metric is defined for **experiment runs** (`results_bundle` /
per-instance run artifacts). It is **not** the same field as review-packet
`packet.json` → `lean_check.elaborated`, which is maintained only under
`cta annotate refresh-lean-check` and is **required to be true** only for
`(system_id, instance_id)` pairs on the strict M1 allowlist
(`is_m1_target_packet` in `crates/cta_cli/src/cmd/annotate.rs`). Other review
packets may keep `lean_check.elaborated = false` without violating the
paper-track refresh gate. See `docs/annotation_manual.md` (“Strict M1
elaboration allowlist”).

### semantic_faithfulness_mean

Instance-average of `faithfulness_score_i / num_obligations_i` over
instances that produced at least one obligation, where
`faithfulness_score_i` is the sum of per-label weights:

| faithfulness label | weight |
|--------------------|--------|
| `faithful`         | 1.0    |
| `partial`          | 0.5    |
| `ambiguous`        | 0.0    |
| `unfaithful`       | 0.0    |

Empty-output instances are excluded from the denominator; they still
contribute to `elaboration_rate` and coverage but not to this mean.

Worked example. An instance with four generated obligations labelled
`faithful, partial, ambiguous, unfaithful` has
`faithfulness_score = 1.0 + 0.5 + 0.0 + 0.0 = 1.5` and contributes
`1.5 / 4 = 0.375` to the mean.

### critical_unit_coverage

Fraction of critical semantic units covered across the split.

Denominator: `sum_i critical_units_total_i`.
Numerator:   `sum_i critical_units_covered_i`.

### rust_consistency_rate

`total_consistent / (total_consistent + total_inconsistent)` across the
split. Obligations labelled `not_applicable` — typically structural
obligations that do not map to executable Rust behavior — are excluded
from both numerator and denominator.

If the denominator (`total_consistent + total_inconsistent`) is zero, the
metric is `0.0` and the run is not paper-eligible.

### vacuity_rate

`total_vacuous / total_obligations` across the split.

### proof_utility

Fraction of instances where at least one obligation from the generated set
was used in a (hand-written or machine-assisted) proof attempt. The proof
attempt itself is out of scope for automated pipelines; the annotator
records the boolean per instance.

## Secondary metrics

- `avg_obligations_per_instance`: total obligations / number of instances.
- `faithful_obligation_density`: `sum_i faithfulness_score_i / sum_i num_obligations_i`
  (the obligation-weighted analogue of `semantic_faithfulness_mean`, using
  the same weights).
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

The canonical adjudicated pack for a released benchmark version lives
under `benchmark/<version>/annotation/adjudicated_subset/pack.json`. It is
the single input consumed by `cta metrics compute` and `cta reports build`
when producing paper-reportable numbers. Experiment configs reference it
via a workspace-relative path.

For ad-hoc adjudication runs outside a benchmark release, `cta annotate
pack` also writes a run-local copy under
`runs/annotation_packs/<version>-adjudicated.json`; that path is never
consulted by the release pipeline.

## Acceptance criteria for a paper-reportable run

A run is paper-eligible iff **all** of the following hold:

1. Every instance in the target split validated against its schema.
2. No schema validation failure across the run's artifacts.
3. A `run_manifest.json` is present and validates against
   `schemas/run_manifest.schema.json`.
4. Lean elaboration completed (success or failure, but not timeout) for
   every instance in the **experiment run** (generated Lean artifacts under
   `runs/…`). This is independent of review-packet `lean_check.elaborated`,
   which is defined only for `benchmark/v0.2/annotation/review_packets/**`
   and is fully `true` only on the strict M1 allowlist (see above).
5. The annotation subset required by the split is complete: two
   independent annotators plus adjudication on any disagreement.
6. The metrics bundle (`results_bundle.json`) was generated successfully.
7. For `v0.2` review-packet-driven releases, strict proof-status refresh is
   green:
   - `cta annotate refresh-lean-check --benchmark-version v0.2 --packets-root benchmark/v0.2/annotation/review_packets --strict-m1`

Current baseline satisfies this gate completely (`m2_ready_packets = 94 / 94`,
empty global proof worklist).

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

## Review packets and metric inputs

Automated runs score whatever obligations the generator emits into run
artifacts. Curated human-review `packet.json` files under
`benchmark/<version>/annotation/review_packets/` are a separate staging
ground for adjudication quality; their `quality_summary` and
benchmark-facing obligation sets are regression-tested so that
`critical_unit_coverage` and `semantic_faithfulness_mean` remain meaningful
when those packets are promoted into packs and compared against model output.
See `docs/annotation_manual.md` and `README.md` (obligation quality gate).
