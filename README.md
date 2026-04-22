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

## Non-goals

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
│   ├── v0.1/               # Pilot benchmark artifacts (immutable)
│   └── v0.2/               # Paper-track benchmark artifacts (in progress)
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
cargo run -p cta_cli -- validate benchmark --version v0.2
cargo run -p cta_cli -- validate benchmark --version v0.2 --release
cargo run -p cta_cli -- validate file --schema run_manifest --path runs/<run_id>/run_manifest.json

# Benchmark loader / linter / manifest / summary
cargo run -p cta_cli -- benchmark lint     --version v0.2
cargo run -p cta_cli -- benchmark stats    --version v0.2
cargo run -p cta_cli -- benchmark manifest --version v0.2

# Semantic extractors and per-instance diagnostics
cargo run -p cta_cli -- extract rust-summary --instance arrays_binary_search_001 --version v0.2
cargo run -p cta_cli -- behavior check       --instance arrays_binary_search_001 --version v0.2
cargo run -p cta_cli -- lean check           --file benchmark/v0.2/instances/arrays/arrays_binary_search_001/scaffold.lean

# Single-system generation (stub provider is offline and deterministic)
cargo run -p cta_cli -- generate \
  --version v0.2 --split dev --system full_method_v1 \
  --provider configs/providers/local_stub.json

# Annotation pack (reads benchmark/<v>/annotation/adjudicated_subset by default).
# Add --from-benchmark to write the canonical release-grade pack back into
# the benchmark tree (benchmark/<v>/annotation/adjudicated_subset/pack.json).
cargo run -p cta_cli -- annotate pack --version v0.2 --policy prefer-adjudicator
cargo run -p cta_cli -- annotate pack --version v0.2 --from-benchmark

# Annotation closure planning / batching / coverage
cargo run -p cta_cli -- annotate plan \
  --benchmark-version v0.2 \
  --experiment-config configs/experiments/benchmark_v1.json \
  --out benchmark/v0.2/annotation/task_board
cargo run -p cta_cli -- annotate batches \
  --benchmark-version v0.2 \
  --missing-pairs benchmark/v0.2/annotation/task_board/missing_pairs.json \
  --out benchmark/v0.2/annotation/task_board/batches
cargo run -p cta_cli -- annotate coverage \
  --benchmark-version v0.2 \
  --experiment-config configs/experiments/benchmark_v1.json \
  --pack benchmark/v0.2/annotation/adjudicated_subset/pack.json \
  --out benchmark/v0.2/annotation/adjudicated_subset

# Sync adjudicator records from review packets into the canonical subset
cargo run -p cta_cli -- annotate sync-review-packets \
  --benchmark-version v0.2 \
  --from benchmark/v0.2/annotation/review_packets \
  --out benchmark/v0.2/annotation/adjudicated_subset

# Strict review-packet audit gate (schema + integrity + signed summary)
cargo run -p cta_cli -- annotate verify-review-packets \
  --benchmark-version v0.2 \
  --packets-root benchmark/v0.2/annotation/review_packets \
  --schema schemas/review_packet.schema.json \
  --out benchmark/v0.2/annotation/review_packets/verification_summary.signed.json

# Metrics + reports for a single run directory. Use the benchmark-local
# pack for paper-reportable numbers; the runs/annotation_packs/ copy is
# only for ad-hoc adjudication sessions.
cargo run -p cta_cli -- metrics compute \
  --run <run_id> \
  --annotations benchmark/v0.2/annotation/adjudicated_subset/pack.json
cargo run -p cta_cli -- reports build --run <run_id>

# Config-driven experiment orchestration
cargo run -p cta_cli -- experiment --config configs/experiments/pilot_v1.json --dry-run
cargo run -p cta_cli -- experiment --config configs/experiments/benchmark_v1_openai_only.json

# Gold-audit workbook generation for v0.2 eval instances
cargo run -p cta_cli -- benchmark audit-workbook --version v0.2

# Paper artifact package from canonical run ids
cargo run -p cta_cli -- reports package \
  --benchmark-version v0.2 \
  --canonical-run-ids <run_id_1>,<run_id_2>,...

# End-to-end fail-fast paper-track orchestrator
# (plan -> batches -> coverage -> validate --release -> verify-review-packets -> package)
cargo run -p cta_cli -- benchmark paper-orchestrate \
  --benchmark-version v0.2 \
  --canonical-run-ids <run_id_1>,<run_id_2>,...
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
- metric contract version: `metrics_v2`
- annotation rubric version: `rubric_v1`

`v0.1` is a 12-instance pilot release: 2 instances per domain across the
6 domains (`arrays`, `sorting`, `graph`, `dp`, `greedy`, `trees`). Both
`dev` and `eval` cover all 12 instances; `dev` is kept as a diagnostic
duplicate of `eval`. Adjudicated gold annotations currently cover a
3-instance subset (one per representative domain) and will expand with
subsequent releases; paper numbers are only reportable on
`(instance, system)` pairs present in the pack.

`v0.2` is the paper-track release scaffold. It enforces held-out evaluation
in release validation (disjoint `dev`/`eval`, `eval` size >= 24 for
paper-claim eligibility), full annotation coverage for experiments that opt
into `require_full_annotation_coverage`, and a two-reviewer gold-audit
signoff file under `benchmark/v0.2/audit/gold_signoff.json`.

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

`cta` auto-loads `<workspace>/.env` on startup, so local provider credentials
can be set once in the repo root for interactive runs.

## Quality gates

Every push runs:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --no-deps`
- `cargo test --workspace --all-targets`
- `cargo test --workspace --doc`
- `cta validate schemas`
- `cta validate benchmark --version v0.1 --release` (pilot checks)
- `cta validate benchmark --version v0.2 --release` (paper-track checks)
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
- `docs/paper_readiness.md` — paper-track release gates and run protocol
- `SECURITY.md` — reporting vulnerabilities and supply-chain posture
- `CONTRIBUTING.md` — development loop, coding conventions, PR checklist

## License

MIT. See `LICENSE`.
