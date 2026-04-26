# Test fixtures

Shared static fixtures consumed by two or more crate test binaries.

Crate-local fixtures live alongside the crate that owns them and are
**not** mirrored here. This directory is reserved for fixtures that
intentionally cross crate boundaries.

Authoritative fixtures in the repository:

| Fixture location                                              | Consumers                                                              | Purpose                                                             |
| ------------------------------------------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------- |
| `benchmark/v0.1/instances/**`                                 | `cta_benchmark`, `cta_rust_extract`, `cta_behavior`, `cta_generate`    | Pilot fixture set (12 instances) used by legacy integration tests   |
| `benchmark/v0.2/instances/**`                                 | `cta_benchmark`, `cta_rust_extract`, `cta_behavior`, `cta_generate`    | Paper-track fixture set (24 instances)                              |
| `benchmark/v0.3/instances/**`                                 | `cta_benchmark`, `cta_rust_extract`, `cta_behavior`, `cta_generate`    | v0.3 family grid (84 instances; shared reference+harness per family) |
| `benchmark/v0.1/annotation/adjudicated_subset/**`             | `cta_metrics::m6_pipeline`                                              | Legacy M6 pipeline fixture                                          |
| `benchmark/v0.2/annotation/adjudicated_subset/**`             | release validation + paper pipeline                                     | Active paper-track annotation source of truth                       |
| `benchmark/v0.2/annotation/multi_annotator_fixture/**`        | `cta_metrics::multi_annotator_pipeline`                                 | Multi-annotator adjudication + agreement pathway                    |
| `benchmark/v0.2/annotation/calibration_pack/**`               | human annotators (calibration)                                          | Worked exemplars spanning the rubric                                |
| `benchmark/v0.2/annotation/review_packets/**`                 | `cta_generate` packet regressions, `cta annotate verify-review-packets` | Staged per-system review `packet.json` trees for paper-track systems |
| `configs/experiments/pilot_v1.json`                           | `cta_cli` smoke, CI                                                     | End-to-end orchestration fixture                                    |

Conventions:

- Fixtures are strictly append-only per benchmark version; renaming or
  rewriting a fixture requires a benchmark version bump.
- All JSON fixtures must validate against their declared schema at
  `schemas/*.schema.json`; CI enforces this via
  `cta validate benchmark --version v0.1 --release`,
  `cta validate benchmark --version v0.2 --release`,
  `cta validate benchmark --version v0.3 --release`, and
  `cta validate file`.
