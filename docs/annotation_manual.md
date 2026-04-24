# Annotation manual

This manual explains how to annotate generated obligation sets against
benchmark instances. It is pinned to `rubric_v1`.

## Before you start

- Confirm the rubric version: `rubric_v1`. If the instance declares a
  different rubric version, stop and escalate.
- Skim `benchmark/<version>/annotation/rubric_v1.md` end to end at least
  once per benchmark release.
- Work through every exemplar in
  `benchmark/<version>/annotation/calibration_pack/` before touching eval
  data. The pack is authoritative and pre-adjudicated; diff your labels
  against it and escalate any mismatch with the adjudicator.

## The three tasks per obligation

For each generated obligation you will assign:

1. **Faithfulness** — one of `faithful`, `partial`, `unfaithful`,
   `ambiguous`.
2. **Rust consistency** — one of `consistent`, `inconsistent`,
   `not_applicable`.
3. **Vacuity** — boolean `is_vacuous`.

Use the rubric definitions exactly. Do not add your own intermediate
labels. When in doubt, choose `ambiguous` and add an annotator note; the
adjudicator will resolve it.

## Critical-unit coverage

For each instance, decide which critical semantic units are **covered** by
the generated set. A unit is covered iff at least one obligation was
labeled `faithful` and linked (either by the generator or by you) to that
unit.

Record covered and missed SU ids. The two lists must be disjoint and
together cover every critical SU in the instance.

## Set-level scalars

Each scalar is in `[0, 1]` and should be computed, not estimated. The
metrics layer re-derives them from the per-obligation labels; annotators
compute them as a sanity check.

- `semantic_faithfulness`: mean of per-obligation faithfulness weights,
  using the `metrics_v2` weights `faithful=1.0`, `partial=0.5`,
  `ambiguous=0.0`, `unfaithful=0.0`. If the metrics reported in a paper
  diverge from your manual computation, the metrics layer wins — report
  the discrepancy to the adjudicator so the contract stays the source of
  truth.
- `code_consistency`: share of obligations in `consistent` out of
  `consistent + inconsistent`. Obligations labelled `not_applicable` are
  excluded from both numerator and denominator (they are structural and
  do not pin down runtime behavior).
- `vacuity_rate`: share of obligations flagged vacuous.
- `proof_utility`: your subjective judgment of whether this set would
  support a hand-written proof attempt.

## Adjudication

Whenever two annotators disagree:

- `cta annotate pack` ingests the raw per-annotator records, applies the
  configured adjudication policy (`prefer-adjudicator` by default, or
  `majority` for sensitivity analyses), and emits a single pack file
  containing one `AdjudicatedRecord` per `(instance, system)` group. Each
  record pins a `per_obligation_disagreements` vector so disagreement
  counts are auditable without a separate sidecar file. For a released
  benchmark the canonical pack lives at
  `benchmark/<version>/annotation/adjudicated_subset/pack.json`; ad-hoc
  runs additionally write a copy under
  `runs/annotation_packs/<version>-adjudicated.json`.
- Under `prefer-adjudicator`, the adjudicator produces a new `Annotation`
  record with `annotator_id: "adjudicator"` and the final labels; that
  record is taken verbatim by the packer.
- Every record the packer consumes is append-only. The independent
  annotator files are preserved verbatim; they are the source of truth
  for the inter-annotator agreement metrics emitted by
  `cta metrics compute --raw-annotations <dir>`.

## Operational workflow (`v0.2`)

For paper-track batches, use this command sequence:

1. Initialize queue and batches:
   - `cta annotate plan --benchmark-version v0.2 --experiment-config configs/experiments/benchmark_v1.json --out benchmark/v0.2/annotation/task_board`
   - `cta annotate batches --benchmark-version v0.2 --missing-pairs benchmark/v0.2/annotation/task_board/missing_pairs.json --out benchmark/v0.2/annotation/task_board/batches`
2. Complete dual annotation + adjudication in review packets.
3. Sync adjudicator outputs into canonical subset:
   - `cta annotate sync-review-packets --benchmark-version v0.2 --from benchmark/v0.2/annotation/review_packets --out benchmark/v0.2/annotation/adjudicated_subset`
4. Rebuild pack and coverage summary:
   - `cta annotate pack --version v0.2 --from-benchmark`
   - `cta annotate coverage --benchmark-version v0.2 --experiment-config configs/experiments/benchmark_v1.json --pack benchmark/v0.2/annotation/adjudicated_subset/pack.json --out benchmark/v0.2/annotation/adjudicated_subset`
5. Enforce review-packet audit gate before packaging:
   - `cta annotate verify-review-packets --benchmark-version v0.2 --packets-root benchmark/v0.2/annotation/review_packets --schema schemas/review_packet.schema.json --out benchmark/v0.2/annotation/review_packets/verification_summary.signed.json`

## Rigorous status addendum (`2026-04-24`, updated)

Curated **v0.2** review packets below are **definition-backed**, pass
`cargo test -p cta_generate` (including `review_packet_lean_lint`), and—where
they appear in `is_m1_target_packet` in `crates/cta_cli/src/cmd/annotate.rs`—
also pass **strict M1** full elaboration (`lean_check.elaborated = true`,
`admit_count = 0`, real `lean_diagnostics.json`).

- **Sorting / BST:** `sorting_insertion_sort_{001,002}`,
  `sorting_merge_sort_{001,002}`, `trees_bst_insert_{001,002}` across all four
  systems (`text_only_v1`, `code_only_v1`, `naive_concat_v1`,
  `full_method_v1`).
- **0/1 knapsack:** `dp_knapsack_01_{001,002}` across the same four systems.
  Canonical proofs live in `lean/CTA/Benchmark/DP/KnapsackTheory.lean`;
  benchmark-facing obligations use `knapsack01_feasible_witness`,
  `totalValue_le_knapsack01`, and `ValidSelection` → `Nodup` (`exact hV.1`).
- **Additional M1 pilots** (graph / greedy / sort / tree instances) are listed
  explicitly in `is_m1_target_packet`; only those pairs are required to
  elaborate under `--strict-m1` (details below).

Required verification sequence (run in order):

1. `cargo run -p cta_cli -- annotate refresh-lean-check --benchmark-version v0.2 --packets-root benchmark/v0.2/annotation/review_packets --strict-m1`
2. `cargo test -p cta_generate`
3. `cargo run -p cta_cli -- annotate verify-review-packets --benchmark-version v0.2 --packets-root benchmark/v0.2/annotation/review_packets --schema schemas/review_packet.schema.json --out benchmark/v0.2/annotation/review_packets/verification_summary.signed.json`

Release-facing expectation for curated paper-track packets:

- no `lean_statement` entries beginning with `axiom`,
- no `lean_check.proof_mode = "axiom_backed"` for release candidates in the
  hardened set,
- no benchmark-facing trivial wrappers (`exact hs/hp/hlen`) or vacuous
  `True` placeholders in curated obligations.

Repo-wide vacuity and wrapper checks run on **every** `packet.json` with
`layer: "benchmark_facing"` obligations. The legacy instance
`graph_dijkstra_001` is exempt from **vacuity** checks only (see
`crates/cta_generate/tests/review_packet_lean_lint.rs`); migrating that packet
removes the exemption.

## Review packet contract (`packet.json`)

Structured review artifacts live under
`benchmark/<version>/annotation/review_packets/<system_id>/<instance_id>/packet.json`
(plus sidecars such as `generated_output.json` and `raw_output.txt` where your
workflow materialises them). Treat `schemas/review_packet.schema.json` as the
authoritative field set; the points below are the paper-track conventions that
CI-style regressions enforce on top of bare schema validity.

**`generated_obligations`**: each element must include `index`, `kind`,
`lean_statement`, `nl_gloss`, `linked_semantic_units`, and `raw_source` (model
output). For `code_only_v1` and `naive_concat_v1` gold packets in their
regression rosters, every entry must also set `layer` to either
`benchmark_facing` or `auxiliary` so vacuity and off-spec checks can be scoped
correctly (`code_only_packet_regression`, `naive_concat_packet_regression`,
`text_only_packet_regression` for migrated `text_only_v1` pilots).

**`quality_summary`**: when present, it must be consistent with the
benchmark-facing theorems (for example `critical_units_only_indirectly_covered`
empty when every critical SU is directly covered, and both
`vacuous_theorems_present` and `off_spec_theorems_present` false for release
candidates). The schema permits omitting `quality_summary`; curated v0.2
`code_only_v1` packets in-repo include it and the regression tests assert final
values.

**Batch markdown** under `annotation/task_board/batches/` or similar paths is
for queueing and checklists only. It is not a substitute for a valid
`packet.json` plus `annotate verify-review-packets` and, when you change packet
shape or prompts, the `cta_generate` tests documented in `README.md` and
`docs/release_process.md` (`code_only_packet_regression`,
`family_packet_regression`, `naive_concat_packet_regression`,
`text_only_packet_regression`,
`full_method_priority1_packet_regression`,
`full_method_priority2_packet_regression`, and repo-wide
`review_packet_lean_lint`).

For Lean proof-completion tracking, each packet should carry a refreshed
`lean_check` block (`elaborated`, `admit_count`, `proof_mode`,
`axiom_dependencies`, `diagnostics_path`) maintained by:

- `cta annotate refresh-lean-check --benchmark-version v0.2 --packets-root benchmark/v0.2/annotation/review_packets --strict-m1`

At the current repository baseline, this strict gate reports complete closure
for `v0.2` review packets (`m2_ready_packets = 93 / 93`, empty global
proof worklist).

## Strict M1 elaboration allowlist

`annotate refresh-lean-check --strict-m1` applies **two** layers of rules:

1. **All packets:** refresh `lean_check` / diagnostics paths and recompute
   `proof_mode`, `admit_count`, and related fields from obligation text and
   diagnostics when present.
2. **Allowlisted pairs only:** for each `(system_id, instance_id)` that
   matches `is_m1_target_packet` in `crates/cta_cli/src/cmd/annotate.rs`, the
   CLI materialises a temporary Lean file (scaffold plus every
   `lean_statement` from `generated_obligations`), runs `lake env lean` on it,
   records structured `lean_diagnostics.json`, and **requires**
   `lean_check.elaborated = true`, `admit_count = 0`, and non-placeholder
   diagnostics. The temporary check file is deleted after the run.

Packets **not** on that list may legitimately keep `lean_check.elaborated =
false` while still being schema-valid and useful for adjudication; the
dashboard column `m1_violation_count` is the indicator for allowlisted pairs
that fail the strict contract.

## Hygiene

- Never edit a previously submitted annotation. Submit a new one.
- Never adjust a label to match a colleague's. Disagreement is signal.
- Never skip the vacuity check; vacuous obligations are the single most
  common failure mode and missing them ruins the metric.
