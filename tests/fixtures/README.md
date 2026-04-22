# Test fixtures

Shared static fixtures consumed by two or more crate test binaries.

Crate-local fixtures live alongside the crate that owns them and are
**not** mirrored here. This directory is reserved for fixtures that
intentionally cross crate boundaries.

Authoritative fixtures in the repository:

| Fixture location                                              | Consumers                                                              | Purpose                                                             |
| ------------------------------------------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------- |
| `benchmark/v0.1/instances/**`                                 | `cta_benchmark`, `cta_rust_extract`, `cta_behavior`, `cta_generate`    | The 12 pilot instances used by every integration test               |
| `benchmark/v0.1/annotation/adjudicated_subset/**`             | `cta_metrics::m6_pipeline`                                              | End-to-end M6 pipeline fixture                                      |
| `benchmark/v0.1/annotation/multi_annotator_fixture/**`        | `cta_metrics::multi_annotator_pipeline`                                 | Multi-annotator adjudication + agreement pathway                    |
| `benchmark/v0.1/annotation/calibration_pack/**`               | human annotators (calibration)                                          | Worked exemplars spanning the rubric                                |
| `configs/experiments/pilot_v1.json`                           | `cta_cli` smoke, CI                                                     | End-to-end orchestration fixture                                    |

Conventions:

- Fixtures are strictly append-only per benchmark version; renaming or
  rewriting a fixture requires a benchmark version bump.
- All JSON fixtures must validate against their declared schema at
  `schemas/*.schema.json`; CI enforces this via
  `cta validate benchmark --version v0.1` and `cta validate file`.
