# Architecture

This document describes component boundaries, artifact flow, and the hard
rules that keep the repository on-mission.

## Component diagram

```
           +------------+
           | benchmark/ |  (versioned instances, splits, annotation)
           +-----+------+
                 |
                 v
+----------+   +--+------------+   +-----------+
| schemas/ |-->| cta_schema    |   | configs/  |
+----------+   +-------+-------+   +-----+-----+
                       |                 |
                       v                 v
                +-------------+    +-------------+
                | cta_core    |<---| cta_cli     |
                +------+------+    +------+------+
                       ^                  |
                       |                  v
   +-------------------+------------------+-------------+
   | cta_benchmark  cta_rust_extract  cta_generate      |
   |   cta_lean       cta_behavior     cta_annotations   |
   |            cta_metrics        cta_reports           |
   +------------+---------------+-----------+-----------+
                                |           |
                                v           v
                           lean/ project   runs/
```

## Crate boundaries

Hard rule: no upward dependency edges. Each crate depends only on crates
above it in this list (`cta_core` depends on nothing; `cta_cli` depends on
everything).

1. `cta_core` — types, IDs, versions, enums. No business logic.
2. `cta_schema` — JSON schema loading + validation. Pure IO + jsonschema.
3. `cta_benchmark` — benchmark loader, linter, manifest builder.
4. `cta_rust_extract` — syn-based semantic cue extractor. No compiler clone.
5. `cta_generate` — provider trait, prompts, normalized output. Integration-style
   regression tests under `crates/cta_generate/tests/` pin obligation shape for
   committed `code_only_v1` / `naive_concat_v1` review packets (vacuity, layers,
   family-specific theorem guards) and must stay green alongside
   `cta annotate verify-review-packets` for paper-track releases.
6. `cta_lean` — Lean file writer, elaboration driver, diagnostics.
7. `cta_behavior` — harness executor and falsification engine.
8. `cta_annotations` — annotation ingest, adjudication, aggregation.
9. `cta_metrics` — pure deterministic metric computation.
10. `cta_reports` — CSV, LaTeX, markdown tables and figures.
11. `cta_cli` — single binary `cta`; the only user-facing entry point.

## Artifact flow

```
instance.json  ──────┐
reference.rs   ──────┤
semantic_units.json ─┤──> cta_benchmark ──> LoadedBenchmark
reference_oblig.json ┤                         │
harness.json        ─┘                         │
                                               v
                                   cta_rust_extract ──> RustSummary
                                               │
                                               v
                              cta_generate ──> GeneratedOutputBundle
                                               │
                                               v
                                    cta_lean ──> ElaborateResult
                                               │
                                               v
                                cta_behavior ──> HarnessReport
                                               │
                                               v
                            cta_annotations ──> Annotation*
                                               │
                                               v
                                cta_metrics ──> ResultsBundle
                                               │
                                               v
                                cta_reports ──> CSV / LaTeX / PDF / MD
```

For paper-track Lean proof completion (`v0.2` review packets), an additional
artifact stream is enforced:

```
review_packets/**/packet.json + scaffold.lean
    -> cta annotate verify-review-packets
    -> cta annotate refresh-lean-check --strict-m1
    -> proof_completion_dashboard.{json,csv}
       wave1_proof_worklist.{json,csv}
       global_proof_worklist.{json,csv}
       proof_execution_plan.json
```

Current repository baseline: strict refresh is fully green for `v0.2`
(`m2_ready_packets = 93 / 93`, `global_proof_worklist.count = 0`).

Definition-backed hardening baseline (`2026-04-24`):

- target families (`sorting_insertion_sort_{001,002}`,
  `sorting_merge_sort_{001,002}`, `trees_bst_insert_{001,002}`) are
  definition-backed in all four systems,
- strict refresh and packet-regression gates remain green after removing
  residual axiom/trivial obligations for those families.

## Hard mission rules

These rules are enforced by CI or by the benchmark linter:

- No benchmark instance enters v0.1 without semantic units.
- No benchmark instance enters v0.1 without a behavioral harness.
- No run is reportable without a manifest.
- No metric name may change after `metrics_v2` freeze.
- No generated Lean file is committed into benchmark gold directories.
- Canonical Lean scaffolds under `lean/CTA/Benchmark/**` must compile (`lake build`
  in `lean/`) and stay byte-identical to every checked-in `scaffold.lean` for
  the same instance (`cta benchmark lint` enforces parity per benchmark version).
- No schema-breaking change without a version bump.
- No silent prompt changes: prompt hash must change and be recorded.
- No annotation overwrite; adjudicated outputs are append-only per version.

## Experiment orchestration

`cta experiment --config configs/experiments/<id>.json` is the canonical
entry point for any paper-reportable campaign. The config declares the
Cartesian product of `systems x providers x seeds`; the orchestrator
materialises one run per combination, writes a schema-valid
`run_manifest.json`, and — when an `annotation_pack` path is supplied —
also computes `results_bundle.json` and renders CSV/Markdown/LaTeX
reports under `runs/<run_id>/reports/`.

The runner never mutates benchmark/v0.1 artifacts, preserves strict
`run_manifest` provenance (repo commit, toolchains, provider name +
model, prompt hash, seed), and halts on the first schema violation.
A cross-experiment summary is emitted at
`runs/experiments/<experiment_id>/summary.json`.

Local CI repeats the full pipeline (schemas → benchmark → experiment
run → per-artifact `cta validate file`) on every push.

## Observability

The experiment runner and `cta_metrics` emit structured `tracing` spans
and events on every invocation. The CLI initialises a
`tracing_subscriber` at `INFO` by default (override with
`RUST_LOG=cta_metrics=debug,cta_cli=debug`). Each experiment run shows
up as nested spans of the form
`run{config=..., dry_run=...}:run{run_id=..., system=..., seed=...}`
and every computed results bundle is logged with
`metrics_version`, `instances`, `elaboration_rate`,
`faithfulness_mean`, and `critical_unit_coverage` fields.

## Non-goals

- No general-purpose IDE plugin.
- No verifier for arbitrary unsafe Rust.
- No end-to-end proof agent gated on UI.
- No web app before the benchmark/eval loop is stable.
- No large-scale dataset scraping.
- No distributed training infrastructure.
