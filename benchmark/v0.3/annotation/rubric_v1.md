# Annotation rubric v1

Version: `rubric_v1`

This rubric governs all annotations for benchmark v0.1. Any change produces
a new rubric version (`rubric_v2`, ...). Released annotations under
`rubric_v1` are never edited.

## Per-obligation labels

For each generated obligation, assign exactly one label in each category.

### Faithfulness

- `faithful` — captures the intended meaning of a linked semantic unit
  fully and without adding or removing content.
- `partial` — captures part of the intended meaning but is materially
  incomplete.
- `unfaithful` — does not reflect the intended meaning (wrong property,
  wrong quantifier, wrong variable).
- `ambiguous` — cannot be decided by the annotator; flag for adjudication.

### Rust consistency

- `consistent` — agrees with the reference implementation's runtime
  behavior on the harness inputs.
- `inconsistent` — contradicted by at least one harness input.
- `not_applicable` — obligation is not a behaviorally testable property
  (e.g. a termination claim stated as existence of a result).

### Vacuity

- `is_vacuous: true` — the obligation is trivially true for any
  implementation of the signature (no implementation could falsify it),
  e.g. `∀ x, True`.
- `is_vacuous: false` — otherwise.

## Set-level scalars (per (instance, system))

Each in `[0, 1]`.

- `semantic_faithfulness`: fraction of obligations the annotator rates
  faithful or partial weighted by critical linkage.
- `code_consistency`: fraction of obligations marked `consistent` out of
  those that are `consistent` or `inconsistent`.
- `vacuity_rate`: fraction of obligations flagged vacuous.
- `proof_utility`: annotator judgment on whether the obligation set
  provides enough structure for a hand-written proof attempt.

## Critical-unit coverage

For each instance, record `covered` and `missed` SU ids. A unit is
`covered` iff at least one obligation was labeled `faithful` and linked to
that unit.

## Adjudication

When two independent annotators disagree on any per-obligation label or
the critical-unit coverage lists, the adjudicator produces a new
annotation with `annotator_id: "adjudicator"`. Adjudicated records are
append-only and supersede earlier records for agreement-metric purposes.
