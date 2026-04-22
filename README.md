# cta-benchmark

A versioned benchmark of Classical Algorithm Tasks (CTA) with a Rust-first
orchestration and evaluation stack, a Lean 4 target/checking layer for
generated obligations, and a reproducible experiment pipeline that outputs
paper-ready tables, figures, and adjudication artifacts.

## Mission

This repository produces four things, and nothing else should be allowed to
muddy the scope:

1. A versioned benchmark of classical algorithm tasks.
2. A Rust-first orchestration and evaluation stack.
3. A Lean target/checking layer for generated obligations.
4. A reproducible experiment pipeline that outputs paper-ready tables,
   figures, and adjudication artifacts.

The scientific purpose is not "general Rust verification" and not "full
theorem proving." It is a benchmark and baseline system for generating Lean
obligations from text plus code, then measuring elaboration, semantic
faithfulness, code consistency, vacuity, and proof utility.

## Non-goals (v0.1)

- No general-purpose IDE plugin
- No attempt to verify arbitrary unsafe Rust
- No full end-to-end proof agent as a required milestone
- No web app before the benchmark/eval loop is stable
- No large-scale dataset scraping
- No fancy distributed training infrastructure
- No ambiguous benchmark tasks that cannot be adjudicated cleanly

## Repository layout

```
cta-benchmark/
├── Cargo.toml              # Rust workspace
├── rust-toolchain.toml     # Pinned toolchain (1.88.0)
├── deny.toml               # cargo-deny supply-chain policy
├── configs/                # Versioned providers / prompts / experiments / metrics
├── schemas/                # JSON schemas (authoritative)
├── benchmark/
│   └── v0.1/               # Versioned benchmark artifacts (immutable)
├── lean/                   # Lean 4 project (scaffolds, generated, proofs)
├── crates/                 # Rust crates (see below)
├── runs/                   # Immutable per-run output directories
├── reports/                # Paper-ready tables and figures
├── notebooks/              # Optional Python analysis (non-core)
├── docs/                   # Architecture, spec, rubric, evaluation contract
└── tests/                  # Integration, golden, fixtures
```

### Rust crates

| Crate               | Role                                                    |
| ------------------- | ------------------------------------------------------- |
| `cta_core`          | Canonical domain types, IDs, versions, enums            |
| `cta_schema`        | JSON schema loading and validation                      |
| `cta_benchmark`     | Benchmark loading, indexing, linting, manifest building |
| `cta_rust_extract`  | Semantic cue extraction from Rust reference code        |
| `cta_generate`      | Candidate obligation generation (providers + prompts)   |
| `cta_lean`          | Lean file writer, elaboration driver, diagnostics       |
| `cta_behavior`      | Behavioral harness / falsification runner               |
| `cta_annotations`   | Annotation ingest, adjudication, aggregation            |
| `cta_metrics`       | Pure, deterministic metric computation                  |
| `cta_reports`       | Tables, figures, LaTeX / Markdown / CSV exports         |
| `cta_cli`           | Single entry point (`cta ...`)                          |

## Quickstart

### Prerequisites

- Rust `1.88.0` (pinned via `rust-toolchain.toml`).
- Lean 4 `v4.12.0` (pinned via `lean/lean-toolchain`, managed by `elan`).
- Optional: `cargo-deny`, `cargo-audit`, `cargo-insta` for local parity with
  CI supply-chain and snapshot jobs.

### Build and test

```bash
cargo build --workspace
cargo test --workspace --all-targets
cargo test --workspace --doc
```

### CLI surface

```bash
# Schema + benchmark validation
cargo run -p cta_cli -- validate schemas
cargo run -p cta_cli -- validate benchmark --version v0.1
cargo run -p cta_cli -- validate file --schema run_manifest --path runs/<run_id>/run_manifest.json

# Benchmark loader / linter / manifest / summary
cargo run -p cta_cli -- benchmark lint     --version v0.1
cargo run -p cta_cli -- benchmark stats    --version v0.1
cargo run -p cta_cli -- benchmark manifest --version v0.1

# Semantic extractors and per-instance diagnostics
cargo run -p cta_cli -- extract rust-summary --instance arrays_binary_search_001 --version v0.1
cargo run -p cta_cli -- behavior check       --instance arrays_binary_search_001 --version v0.1
cargo run -p cta_cli -- lean check           --file benchmark/v0.1/instances/arrays/arrays_binary_search_001/scaffold.lean

# Single-system generation (stub provider is offline and deterministic)
cargo run -p cta_cli -- generate \
  --version v0.1 --split dev --system full_method_v1 \
  --provider configs/providers/local_stub.json

# Annotation pack (reads benchmark/<v>/annotation/adjudicated_subset by default)
cargo run -p cta_cli -- annotate pack --version v0.1 --policy prefer-adjudicator

# Metrics + reports for a single run directory
cargo run -p cta_cli -- metrics compute \
  --run <run_id> \
  --annotations runs/annotation_packs/v0.1-adjudicated.json
cargo run -p cta_cli -- reports build --run <run_id>

# Config-driven experiment orchestration
cargo run -p cta_cli -- experiment --config configs/experiments/pilot_v1.json --dry-run
cargo run -p cta_cli -- experiment --config configs/experiments/pilot_v1.json
```

### Lean

```bash
cd lean
lake build
```

## Versioning discipline

Everything benchmark-related is versioned explicitly:

- benchmark version: `v0.1`, `v0.2`, ...
- schema version: `schema_v1`
- metric contract version: `metrics_v1`
- annotation rubric version: `rubric_v1`

Released benchmark instances are **never mutated in place**. Add a new
version instead. The benchmark linter additionally enforces byte-identity
between each instance scaffold and its canonical Lean module under
`lean/CTA/Benchmark/**`.

## Reproducibility

Every run emits a `run_manifest.json` (see `schemas/run_manifest.schema.json`)
capturing commit hash, benchmark / schema / metrics / rubric versions,
system and prompt identifiers, prompt SHA-256 hashes per instance, seeds,
toolchain versions, timestamp, and provider metadata. **No result is
accepted into the paper without a manifest.**

The generation pipeline is build-pure: no network calls happen during
`cargo build`. Live providers (`OpenAiProvider`, `AnthropicProvider`) only
issue HTTP requests at runtime, and only when their respective credential
environment variables are present; otherwise they refuse to run and surface
a typed `GenerateError::Provider`.

## Quality gates

Every push runs:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --no-deps`
- `cargo test --workspace --all-targets`
- `cargo test --workspace --doc`
- `cta validate schemas`
- `cta validate benchmark --version v0.1`
- `cta experiment --config configs/experiments/pilot_v1.json --dry-run` and
  the full run against the stub provider, with schema validation of every
  emitted `run_manifest.json` and `results_bundle.json`.
- `cargo-deny check --all-features` and `cargo audit --deny warnings`
  (see `.github/workflows/supply-chain.yml`).
- `lake build` over the Lean project.

Snapshot tests in `crates/cta_reports` and `crates/cta_generate` pin the
exact CSV/Markdown/LaTeX shape of every report artifact and the rendered
form of every prompt template. Property-based tests in
`crates/cta_generate/tests/normalize_proptest.rs` assert that the
LLM-output normalizer never panics and only accepts well-formed
obligations.

## Documentation

- `docs/architecture.md` — components, crate boundaries, artifact flow
- `docs/benchmark_spec.md` — instance schema, domain/split policy
- `docs/annotation_manual.md` — rubric, adjudication, calibration
- `docs/evaluation_contract.md` — metric definitions, acceptance criteria
- `docs/release_process.md` — how to freeze / bump versions
- `SECURITY.md` — reporting vulnerabilities and supply-chain posture
- `CONTRIBUTING.md` — development loop, coding conventions, PR checklist

## License

MIT. See `LICENSE`.
