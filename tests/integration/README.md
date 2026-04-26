# Integration tests

Cross-crate integration tests are run by `cargo test --workspace` and
live inside the crate whose public API they exercise. The canonical
list below is the source of truth; CI asserts each test binary runs
green on every push.

| Binary                                                  | Exercises                                                                             |
| ------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| `cta_benchmark/tests/pilot.rs`                          | Loader + linter + manifest hash over all 12 instances                                 |
| `cta_rust_extract/tests/pilot_golden.rs`                | Rust semantic extractor invariants across pilot reference implementations              |
| `cta_lean` (unit tests in `src/lib.rs`)                 | Lean writer, `lake`-based elaboration driver, diagnostic parsing                        |
| `cta_behavior/tests/pilot_smoke.rs`                    | Behavioral harness smoke over every registered pilot adapter                           |
| `cta_generate/tests/pipeline_smoke.rs`                  | Full generate pipeline with the stub provider and all four prompt templates (v0.3 pilot registry) |
| `cta_metrics/tests/m6_pipeline.rs`                      | Annotations -> adjudicated pack -> results bundle, schema-validated                   |
| `cta_metrics/tests/multi_annotator_pipeline.rs`         | Multi-annotator adjudication policies + inter-annotator agreement metrics             |
| `cta_generate/tests/code_only_packet_regression.rs`     | Curated `code_only_v1` review `packet.json` set: schema-adjacent checks, layers, vacuity, per-instance theorem hygiene |
| `cta_generate/tests/family_packet_regression.rs`        | Cross-instance template coherence (BFS adjacency, binary-search success shape, LCA, coin change, LCS)                 |
| `cta_generate/tests/naive_concat_packet_regression.rs`| Canonical `naive_concat_v1` exemplar packets                                                                            |
| `cta_generate/tests/full_method_priority1_packet_regression.rs` | Curated `full_method_v1` high-priority packet regression set                                                    |
| `cta_generate/tests/full_method_priority2_packet_regression.rs` | Curated `full_method_v1` secondary packet regression set                                                        |
| `cta_generate/tests/review_packet_lean_lint.rs`        | Repo-wide `review_packets/**/packet.json` static checks (vacuity shells, Dijkstra `PathWeight`, coin `≤`, LCA shape on curated systems) |

End-to-end CLI orchestration is additionally exercised by
`.github/workflows/ci.yml`:

1. `cta validate schemas`
2. `cta validate benchmark --version v0.1 --release` (on every push)
3. `cta validate benchmark --version v0.2 --release` (paper track; fails with
   `GOLD_AUDIT_SIGNOFF_INVALID` until `benchmark/v0.2/audit/gold_signoff.json`
   carries two reviewer names and `approved: true` per `docs/release_process.md`)
4. `cta experiment --config configs/experiments/pilot_v1.json --dry-run`
5. `cta experiment --config configs/experiments/pilot_v1.json`
6. `cta validate file --schema {run_manifest, results_bundle} --path <...>`
7. `cta annotate refresh-lean-check --benchmark-version v0.2 --packets-root benchmark/v0.2/annotation/review_packets --strict-m1`

`.github/workflows/benchmark-lint.yml` (on `benchmark/**` changes) additionally
runs `validate benchmark --version v0.3 --release` and `benchmark lint` for v0.3;
see `docs/PAPER_READINESS.md`.

Current baseline expectation for this gate: `m2_ready_packets = 94 / 94` and
`global_proof_worklist.count = 0`.

`.github/workflows/ci.yml` runs `annotate verify-review-packets` **before**
`annotate refresh-lean-check --strict-m1`, writing the signed summary to
`/tmp/…` so CI validates the committed `packet.json` set without rewriting
`benchmark/v0.2/annotation/review_packets/verification_summary.signed.json`
on every run (local releases still regenerate that path after refresh; see
`docs/release_process.md`).

No integration test reaches the network; every provider call goes
through `StubProvider`, and `OpenAiProvider` / `AnthropicProvider` are
behind an explicit API-key gate.
