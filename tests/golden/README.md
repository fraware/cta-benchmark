# Golden tests

Snapshot-style tests that pin stable-by-design outputs. Every golden
lives alongside the crate that owns the invariant so regressions surface
in the same test run that touched the code:

| Golden test                                                 | Invariant pinned                                                              |
| ----------------------------------------------------------- | ----------------------------------------------------------------------------- |
| `crates/cta_rust_extract/tests/golden.rs`                   | Extractor produces a non-empty, self-consistent `RustSummary` for all 12 instances |
| `crates/cta_benchmark/tests/pilot.rs`                       | 12-instance pilot round-trips through loader -> linter -> manifest            |
| `crates/cta_generate/tests/pipeline_smoke.rs`               | Stub provider produces schema-valid `generated_output.json` for every instance |
| `crates/cta_metrics/tests/m6_pipeline.rs`                   | `compute_results_bundle` emits schema-valid `results_bundle.json`             |
| `crates/cta_metrics/tests/multi_annotator_pipeline.rs`      | Adjudication policies and inter-annotator agreement remain deterministic      |
| `crates/cta_generate/tests/code_only_packet_regression.rs`  | Curated `code_only_v1` review packets stay schema-consistent and theorem-hygienic |
| `crates/cta_generate/tests/family_packet_regression.rs`     | Cross-instance `code_only_v1` template guards (BFS, binary search, LCA, …)        |
| `crates/cta_generate/tests/naive_concat_packet_regression.rs` | Canonical `naive_concat_v1` exemplar packet shapes                              |

Hard rules:

- A golden test never reads or writes files outside
  `benchmark/`, `schemas/`, `configs/`, or the crate's own `tests/`
  directory.
- Any golden that depends on wall-clock time, hostname, or user env
  vars is a bug; rely only on checked-in fixtures and pure functions.
- A changed golden output is a breaking change; update the expected
  value **and** bump the relevant contract version (`schema_v*`,
  `metrics_v*`, or `rubric_v*`) in the same commit.
