# Paper outline

Working title: "Classical Algorithm Tasks: A Benchmark for Lean Proof
Obligation Generation from Rust Reference Code".

## Contributions

1. A versioned benchmark: pilot `v0.1` (12 tasks), paper-track `v0.2`
   (24 eval tasks), and current paper-track `v0.3` with **84** instances
   (authoritative counts in `benchmark/v0.3/benchmark_paper_summary.json`)
   across six domains (arrays, sorting, graph, greedy, DP, trees), each with
   decomposed semantic units, gold reference obligations, a deterministic
   behavioral harness, a Lean scaffold with byte-identity enforced
   between the instance-local copy and the canonical
   `lean/CTA/Benchmark/**` module, and an adjudicated annotation subset.
2. **System scope (default: four-system primary study):** baseline generators
   `text_only_v1`, `code_only_v1`, `naive_concat_v1`, and `full_method_v1` are
   first-class comparators unless the manuscript explicitly adopts the optional
   **calibration-only** scope for `text_only_v1` (see `docs/paper/system_scope.md`
   and `benchmark_paper_summary.json` fields `paper_headline_policy` /
   `paper_alternate_scope_note`). **Headline** metrics tables use the strict
   independent evidence view; expanded mapped propagation is appendix-only
   (`results/appendix_mapped_evidence/`, `paper_table_annotation_evidence.csv`).
3. A reproducible Rust pipeline with frozen metric (`metrics_v2`), schema
   (`schema_v1`), and rubric (`rubric_v1`) contracts.
4. A metric suite decomposed into `elaboration_rate`,
   `semantic_faithfulness_mean`, `critical_unit_coverage`,
   `rust_consistency_rate`, `vacuity_rate`, `proof_utility`, plus
   secondary metrics and inter-annotator agreement (linearly-weighted
   Cohen's kappa on faithfulness, Cohen's kappa on vacuity, raw
   agreement on critical-unit coverage).

## Section structure

1. Introduction — mission, scope, and what is explicitly out of scope.
2. Related work — benchmarks for autoformalization, specification
   mining, verification-in-the-loop, Lean obligation generation.
3. Benchmark design — domain selection, semantic units, scaffolding
   (including the `CTA.Core.Checkers` reusable predicate library),
   harness philosophy.
4. Systems — prompt templates, structured inputs, output schema,
   provider abstraction (stub, OpenAI, Anthropic).
5. Metrics — primary and secondary definitions with acceptance
   criteria, including inter-annotator agreement methodology.
6. Results — primary tables from `python scripts/compute_results.py --paper`
   (`results/paper_table_*.csv`, per-metric summaries, reliability) plus run
   bundles from `cta reports build` where model generations are reported.
7. Analysis — failure taxonomy and qualitative cases.
8. Limitations — what our benchmark does not measure.

Reviewer-facing indexes: `docs/REVIEWER_MAP.md`, `docs/LIMITATIONS.md`, and
`docs/PROVENANCE.md` map each section to committed artifacts and regeneration
commands.

## Reproducibility statement

Every reported number is backed by a `run_manifest.json` capturing
commit hash, benchmark version, schema version, metrics version,
rubric version, toolchains, provider + model name, seed, and per-
instance SHA-256 prompt hashes. Run-scoped tables and figures are regenerable
from `cta reports build --run <run_id>` against the stored run directory, with
snapshot tests in `crates/cta_reports/tests/snapshots/` pinning export shape.

v0.3 headline aggregates and evidence tables come from the Python paper track
(`python scripts/compute_results.py --paper`,
`python scripts/export_benchmark_paper_summary.py`), checked into `results/`
and `benchmark/v0.3/benchmark_paper_summary.json`, and guarded by
`python scripts/ci_reviewer_readiness.py` in CI (row-count and label contracts).

Paper-track adjudication additionally relies on `cta annotate verify-review-packets`
(signed `verification_summary.signed.json`) and the `cta_generate` packet
regression tests (`code_only_packet_regression`, `family_packet_regression`,
`naive_concat_packet_regression`, `text_only_packet_regression`,
`full_method_priority1_packet_regression`,
`full_method_priority2_packet_regression`, `review_packet_lean_lint`) so
curated review obligations stay aligned with benchmark scaffolds before they
enter the canonical annotation pack.

## Rigorous status note (`2026-04-24`, updated)

The writing should explicitly call out that the former axiom-backed target
families (`sorting_insertion_sort_{001,002}`, `sorting_merge_sort_{001,002}`,
`trees_bst_insert_{001,002}`) are now definition-backed across all four
baseline systems, and that **0/1 knapsack** (`dp_knapsack_01_{001,002}`) is
similarly definition-backed in all four systems via `KnapsackTheory`, with
strict M1 elaboration enforced for the allowlisted `(system, instance)` pairs
documented in `docs/annotation_manual.md`. Distinguish experiment-level
`elaboration_rate` (`docs/evaluation_contract.md`) from review-packet
`lean_check.elaborated` when discussing metrics vs packet hygiene.
