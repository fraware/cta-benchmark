# Paper outline

Working title: "Classical Algorithm Tasks: A Benchmark for Lean Proof
Obligation Generation from Rust Reference Code".

## Contributions

1. A versioned benchmark: pilot `v0.1` (12 tasks) and paper-track `v0.2`
   (24 eval tasks) across six domains (arrays, sorting, graph, greedy, DP,
   trees), each with
   decomposed semantic units, gold reference obligations, a deterministic
   behavioral harness, a Lean scaffold with byte-identity enforced
   between the instance-local copy and the canonical
   `lean/CTA/Benchmark/**` module, and an adjudicated annotation subset.
2. Four baseline generation systems (`text_only_v1`, `code_only_v1`,
   `naive_concat_v1`, `full_method_v1`) implemented as a reproducible
   Rust pipeline with frozen metric (`metrics_v2`), schema (`schema_v1`),
   and rubric (`rubric_v1`) contracts.
3. A metric suite decomposed into `elaboration_rate`,
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
6. Results — tables produced by `cta reports build`.
7. Analysis — failure taxonomy and qualitative cases.
8. Limitations — what our benchmark does not measure.

## Reproducibility statement

Every reported number is backed by a `run_manifest.json` capturing
commit hash, benchmark version, schema version, metrics version,
rubric version, toolchains, provider + model name, seed, and per-
instance SHA-256 prompt hashes. Every table and figure is regenerable
from a single `cta reports build --run <run_id>` invocation against the
stored run directory, and is additionally pinned by snapshot tests in
`crates/cta_reports/tests/snapshots/` so the surface shape cannot drift
without a deliberate update. Paper-track adjudication additionally relies on
`cta annotate verify-review-packets` (signed
`verification_summary.signed.json`) and the `cta_generate` packet regression
tests (`code_only_packet_regression`, `family_packet_regression`,
`naive_concat_packet_regression`, `full_method_priority1_packet_regression`,
`full_method_priority2_packet_regression`, `review_packet_lean_lint`) so
curated review obligations stay aligned with benchmark scaffolds before they
enter the canonical annotation pack.

## Rigorous status note (`2026-04-24`)

The writing should explicitly call out that the former axiom-backed target
families (`sorting_insertion_sort_{001,002}`, `sorting_merge_sort_{001,002}`,
`trees_bst_insert_{001,002}`) are now definition-backed across all four
baseline systems and pass strict refresh plus packet regression gates.
