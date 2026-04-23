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
disjoint. See `docs/paper_readiness.md`.

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
- Prefer `opaque` for unknown implementations. Use `axiom` when Lean cannot
  synthesize required instances for a stub signature (for example a
  function-valued model like `bstInsert : Tree → Int → Tree`).
- Core `List` in this project does not provide `List.sum`; prefer
  `List.foldl` for summing `List Nat` inside scaffold definitions.
- Reusable vocabulary lives in `lean/CTA/Core/Checkers.lean`. Scaffolds
  re-export a handful of names under the instance's own namespace
  (`Sorted`, `IsPerm`, `IndexValid`, …) so that generated obligations
  spell properties consistently across domains while still being easy
  to trace back to the shared definition.
- Generated files never overwrite scaffold files; they live under
  `runs/<run_id>/lean_generated/...`.

## Annotation review packets (`v0.2+`)

Human adjudication for paper-track systems is staged in
`benchmark/<version>/annotation/review_packets/<system_id>/<instance_id>/`.
Those directories hold JSON review packets (`packet.json`) validated by
`schemas/review_packet.schema.json`, not benchmark instance gold files. Rubric
semantics, two-layer obligations, and `quality_summary` expectations are
documented in `docs/annotation_manual.md`; mechanical guards live in
`crates/cta_generate/tests/code_only_packet_regression.rs`,
`family_packet_regression.rs`, `naive_concat_packet_regression.rs`,
`full_method_priority1_packet_regression.rs`,
`full_method_priority2_packet_regression.rs`, and
`review_packet_lean_lint.rs`.

## Prohibited patterns

- No `unsafe` blocks in reference Rust.
- No external crate dependencies in reference Rust.
- No macro-heavy reference code; prefer explicit loops and branches.
- No scaffold that hard-codes executable reference algorithms; use `opaque`
  or `axiom` as above for declarative stubs only.
- No ambiguous edge cases whose adjudication depends on the reader.
