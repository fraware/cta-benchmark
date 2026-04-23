# Integration tests

Cross-crate integration tests are run by `cargo test --workspace` and
live inside the crate whose public API they exercise. The canonical
list below is the source of truth; CI asserts each test binary runs
green on every push.

| Binary                                                  | Exercises                                                                             |
| ------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| `cta_benchmark/tests/pilot.rs`                          | Loader + linter + manifest hash over all 12 instances                                 |
| `cta_rust_extract/tests/golden.rs`                      | Rust semantic extractor invariants per instance                                       |
| `cta_lean/tests/elab.rs`                                | Lake-based Lean elaboration + diagnostic parsing                                      |
| `cta_behavior/tests/pilot.rs`                           | Behavioral harness oracles for all 12 reference implementations                       |
| `cta_generate/tests/pipeline_smoke.rs`                  | Full generate pipeline with the stub provider and all four prompt templates          |
| `cta_metrics/tests/m6_pipeline.rs`                      | Annotations -> adjudicated pack -> results bundle, schema-validated                   |
| `cta_metrics/tests/multi_annotator_pipeline.rs`         | Multi-annotator adjudication policies + inter-annotator agreement metrics             |
| `cta_generate/tests/code_only_packet_regression.rs`     | Curated `code_only_v1` review `packet.json` set: schema-adjacent checks, layers, vacuity, per-instance theorem hygiene |
| `cta_generate/tests/family_packet_regression.rs`        | Cross-instance template coherence (BFS adjacency, binary-search success shape, LCA, coin change, LCS)                 |
| `cta_generate/tests/naive_concat_packet_regression.rs`| Canonical `naive_concat_v1` exemplar packets                                                                            |

End-to-end CLI orchestration is additionally exercised by
`.github/workflows/ci.yml`:

1. `cta validate schemas`
2. `cta validate benchmark --version v0.1 --release`
3. `cta validate benchmark --version v0.2 --release`
4. `cta experiment --config configs/experiments/pilot_v1.json --dry-run`
5. `cta experiment --config configs/experiments/pilot_v1.json`
6. `cta validate file --schema {run_manifest, results_bundle} --path <...>`

No integration test reaches the network; every provider call goes
through `StubProvider`, and `OpenAiProvider` / `AnthropicProvider` are
behind an explicit API-key gate.
