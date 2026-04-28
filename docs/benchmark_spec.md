# Benchmark specification

Authoritative document for benchmark content, domain policy, and the
versioning discipline that governs it.

## Instance layout

Every instance lives in its own directory:

```
benchmark/<version>/instances/<domain>/<instance_id>/
├── instance.json
├── reference.rs
├── scaffold.lean
├── reference_obligations.json
├── semantic_units.json
├── harness.json
└── notes.md
```

All seven files are mandatory. The linter rejects any instance missing one.

## Required fields

See `schemas/instance.schema.json`. Highlights:

- `schema_version`: constant `schema_v1`.
- `instance_id`: must match `^[a-z][a-z0-9]*(?:_[a-z0-9]+)*_[0-9]{3}$` and
  match the directory basename.
- `domain`: one of `arrays`, `sorting`, `graph`, `greedy`, `dp`, `trees`.
- `difficulty`: one of `easy`, `medium`, `hard`.
- `informal_statement.required_properties` must be non-empty.
- `informal_statement.edge_cases` must be non-empty.

## Domain policy

- Each domain gets its own top-level subdirectory under `instances/`.
- A single family of problems (e.g. binary search, insertion sort) gets one
  instance per canonical variant. Variants that differ only in style
  (iterative vs recursive) should be modeled with separate instance ids
  such as `arrays_binary_search_002_recursive`.
- Instances MUST expose at least one critical semantic unit whose phrasing
  would distinguish faithful from unfaithful obligations.

## Splits

Each benchmark version ships the following JSON files under `splits/`:

- `dev.json` — used while designing prompts and tuning systems; metrics on
  this split are diagnostic only.
- `eval.json` — frozen for every released benchmark version; all paper
  results are computed on this split. Must be non-empty and must list
  every instance declared by the manifest (`dev` may be a subset, a
  duplicate, or disjoint).
- `challenge.json` — *optional* stress split. A benchmark version that
  does not ship a real challenge set must omit this file; an empty
  `challenge.json` is disallowed and rejected by
  `cta benchmark lint --release`.

For `v0.1` (the pilot release), `dev.json` and `eval.json` both cover all
12 pilot instances, and `challenge.json` is absent.

For paper-track releases (`v0.2+`), release validation enforces stricter
policy: `eval` must contain at least 24 instances and `dev`/`eval` must be
disjoint. See `docs/PAPER_READINESS.md`.

### v0.3 family grid (specification stress, not new algorithms)

`v0.3` expands each v0.2 algorithm **family** to seven instance ids (`*_001`
… `*_007`) that **share** the same reference implementation and behavioral
harness. Instances differ in **informal_statement** emphasis, **semantic
unit** glosses, **notes**, and audit metadata so reviewers can stress-test
specification transfer without conflating oracle diversity with algorithmic
diversity. Variants `_001` and `_002` additionally carry distinct **grid
variant** paragraphs so paired controls are not near-duplicate prose.

## Release / versioning

- Every benchmark artifact (`v0.1`, `v0.2`, …) is **immutable once
  released**.
- Changes that would alter any instance's semantic content bump the
  benchmark version.
- The benchmark manifest (`manifests/benchmark_manifest.json`) records a
  deterministic content hash (`sha256`) and must be regenerated whenever an
  unreleased version's contents change.
- Instance ids are never reused across versions with different semantic
  content.

## Lean scaffold policy

- Each instance ships a scaffold at `scaffold.lean` (instance-local) **and**
  at the canonical Lean module path `lean/CTA/Benchmark/<Domain>/<Family><NNN>.lean`.
- The two must be byte-identical. CI enforces this via
  `cta benchmark lint --version <v>`.
- Scaffolds declare types, signatures, and instance-local aliases over the
  `CTA.Core.Checkers` predicate library (`IsPermutation`, `InBounds`,
  `SortedLE`, `NonNegative`, `SameMultiset`). They never contain
  executable reference implementations (only declarative types and
  properties).
- Prefer definition-backed family theory modules with per-instance `abbrev`
  aliases over packet-local `opaque` declarations.
- Use `axiom` only as a transitional interface mechanism and track those
  packets through `lean_check.proof_mode = "axiom_backed"` in review packets.
- Core `List` in this project does not provide `List.sum`; prefer
  `List.foldl` for summing `List Nat` inside scaffold definitions.
- Reusable vocabulary lives in `lean/CTA/Core/Checkers.lean`. Scaffolds
  re-export a handful of names under the instance's own namespace
  (`Sorted`, `IsPerm`, `IndexValid`, …) so that generated obligations
  spell properties consistently across domains while still being easy
  to trace back to the shared definition.
- Generated files never overwrite scaffold files; they live under
  `runs/<run_id>/lean_generated/...`.

Current repository baseline: `v0.2` review packets are fully proof-complete
under strict refresh (`m2_ready_packets = 94 / 94`).

Additional hardening baseline (`2026-04-24`, updated):

- For target families `sorting_insertion_sort_{001,002}`,
  `sorting_merge_sort_{001,002}`, and `trees_bst_insert_{001,002}` across all
  review-packet systems, no curated obligation uses `axiom` declarations and no
  such packet reports `proof_mode: "axiom_backed"`.
- For **0/1 knapsack** (`dp_knapsack_01_{001,002}`), all four systems reuse
  `KnapsackTheory` (`lean/CTA/Benchmark/DP/KnapsackTheory.lean`) and carry three
  aligned `benchmark_facing` theorems in `packet.json`.
- Curated packet obligations for those families avoid benchmark-facing vacuous
  placeholders and tautological wrappers.
- **Strict M1** elaboration (`lean_check.elaborated = true` after
  `annotate refresh-lean-check --strict-m1`) applies only to `(system_id,
  instance_id)` pairs in `is_m1_target_packet` in `crates/cta_cli/src/cmd/annotate.rs`;
  other packets may still show `elaborated = false` without failing the gate.

## Annotation review packets (`v0.2+`)

Human adjudication for paper-track systems is staged in
`benchmark/<version>/annotation/review_packets/<system_id>/<instance_id>/`.
Those directories hold JSON review packets (`packet.json`) validated by
`schemas/review_packet.schema.json`, not benchmark instance gold files. Rubric
semantics, two-layer obligations, and `quality_summary` expectations are
documented in `docs/annotation_manual.md`; mechanical guards live in
`crates/cta_generate/tests/code_only_packet_regression.rs`,
`family_packet_regression.rs`, `naive_concat_packet_regression.rs`,
`text_only_packet_regression.rs`,
`full_method_priority1_packet_regression.rs`,
`full_method_priority2_packet_regression.rs`, and
`review_packet_lean_lint.rs`.

## Prohibited patterns

- No `unsafe` blocks in reference Rust.
- No external crate dependencies in reference Rust.
- No macro-heavy reference code; prefer explicit loops and branches.
- No scaffold that hard-codes executable reference algorithms; use `opaque`
  only when intentionally preserving an abstract interface; the default is
  centralized definition-backed family theory.
- No ambiguous edge cases whose adjudication depends on the reader.

## Roadmap: `v0.4` and beyond

`v0.3` is the current paper-track benchmark slice. **New instance families or
material changes to task semantics** ship only under a new `benchmark/v0.4/`
tree after an explicit milestone: a stable `protocol_freeze.json` for v0.3,
completed release gates in `docs/release_process.md`, and a version bump in
manifest `source_provenance`.

Deprecation policy: frozen `benchmark/v0.N/` directories remain immutable except
for documented hygiene repairs; consumers should pin `benchmark_version` in
experiment configs. When `v0.4` exists, `v0.3` remains addressable for
reproduction of earlier papers without silent content drift.

## Evidence-Hardening Update (2026-04-28)

The benchmark release surface now includes paper-hardening outputs:

- provenance layer registry: `results/provenance_layer_registry.csv`
- strict-gap closure invariant checks in CI (`strict_unique_instances = 84`,
  no strict mapped headline rows)
- artifact package manifest + checksums:
  `artifacts/evidence_hardening_manifest.json`
