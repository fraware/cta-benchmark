# Contributing to cta-benchmark

Thanks for your interest in improving `cta-benchmark`. This repository
hosts a reproducible research benchmark, so our contribution rules are
stricter than a typical Rust library: every change is subject to the
same determinism and versioning guarantees that the benchmark itself
promises its consumers.

This project follows the [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md). By
participating, you agree to uphold it. Unless stated otherwise in a
signed agreement, contributions are accepted under the same **MIT**
license as the repository (see root `LICENSE`).

## Ground rules

1. **Benchmark artifacts are immutable per tagged version.** Do not
   edit files under `benchmark/v0.1/**` in a way that changes their
   meaning; instead, open a PR that adds a new benchmark version
   (`benchmark/v0.2/…`) and updates the manifest.
2. **All JSON artifacts must validate against their schema.** CI
   enforces this via `cta validate benchmark --version <v>` and
   `cta validate file`.
3. **Metrics contracts are versioned.** Any change to
   `crates/cta_metrics` that affects a scored quantity requires a
   corresponding bump of `METRICS_VERSION` in `crates/cta_core`.
4. **No network calls at build time.** `cta_generate` is build-pure;
   live providers are runtime-only and gated on credential env vars.
5. **No placeholders or stub comments.** `todo!()`, `unimplemented!()`,
   TODO/FIXME comments, and "implemented in milestone N" prose are
   rejected in review. Clippy lints enforce the first two. CI also runs
   `scripts/ci_reviewer_readiness.py`, which denylists common placeholder
   phrases in committed `annotation/` and `results/` text artifacts.

## Development loop

```bash
# 1. format + lints
cargo fmt --all
cargo clippy --workspace --all-targets --no-deps

# 2. unit + integration tests + doctests
cargo test --workspace --all-targets
cargo test --workspace --doc

# 3. schema + benchmark validation
cargo run -p cta_cli -- validate schemas
cargo run -p cta_cli -- validate benchmark --version v0.1 --release
cargo run -p cta_cli -- validate benchmark --version v0.2 --release
cargo run -p cta_cli -- validate benchmark --version v0.3 --release

# 3b. paper-track annotation/packaging flow (v0.2)
cargo run -p cta_cli -- annotate plan --benchmark-version v0.2 --experiment-config configs/experiments/benchmark_v1.json --out benchmark/v0.2/annotation/task_board
cargo run -p cta_cli -- annotate batches --benchmark-version v0.2 --missing-pairs benchmark/v0.2/annotation/task_board/missing_pairs.json --out benchmark/v0.2/annotation/task_board/batches
cargo run -p cta_cli -- annotate coverage --benchmark-version v0.2 --experiment-config configs/experiments/benchmark_v1.json --pack benchmark/v0.2/annotation/adjudicated_subset/pack.json --out benchmark/v0.2/annotation/adjudicated_subset
cargo run -p cta_cli -- benchmark audit-workbook --version v0.2
cargo run -p cta_cli -- reports package --benchmark-version v0.2 --canonical-run-ids <run_ids>

# 3c. review-packet regressions (when touching prompts, normalize.rs, or review packet JSON)
cargo test -p cta_generate --test code_only_packet_regression
cargo test -p cta_generate --test family_packet_regression
cargo test -p cta_generate --test naive_concat_packet_regression
cargo test -p cta_generate --test text_only_packet_regression
cargo test -p cta_generate --test full_method_priority1_packet_regression
cargo test -p cta_generate --test full_method_priority2_packet_regression
cargo test -p cta_generate --test review_packet_lean_lint

# 3d. Lean scaffolds (when touching lean/CTA/Benchmark/** or any scaffold.lean)
cd lean
lake build
cd ..
# v0.1 pilot: instance scaffolds must stay byte-identical to canonical modules.
# After editing any `lean/CTA/Benchmark/**` file listed for the twelve v0.1 pilots:
#   powershell -NoProfile -File scripts/sync_v0_1_pilot_scaffolds.ps1
#   cargo run -p cta_cli -- benchmark manifest --version v0.1
#   cargo run -p cta_cli -- validate benchmark --version v0.1 --release
cargo run -p cta_cli -- benchmark lint --version v0.1
cargo run -p cta_cli -- benchmark lint --version v0.2
cargo run -p cta_cli -- benchmark lint --version v0.3 --release

# 3e. v0.3 gold audit workbook (regenerate skeleton CSVs from eval split)
cargo run -p cta_cli -- benchmark audit-workbook --version v0.3

# 3f. review-packet Lean proof status (when touching review `packet.json`,
#     `scaffold.lean` copies, `lean/CTA/Benchmark/**` used by packets, or
#     `is_m1_target_packet` in `crates/cta_cli/src/cmd/annotate.rs`)
cargo run -p cta_cli -- annotate refresh-lean-check \
  --benchmark-version v0.2 \
  --packets-root benchmark/v0.2/annotation/review_packets \
  --strict-m1
# Commit refreshed `packet.json` lean_check blocks, `lean_diagnostics.json`,
# and aggregate outputs (`proof_completion_*`, worklists) if this command
# changes them. See `docs/annotation_manual.md` (Strict M1 elaboration allowlist).

# 4. end-to-end experiment smoke test (stub provider, offline)
cargo run -p cta_cli -- experiment --config configs/experiments/pilot_v1.json --dry-run

# 5. supply-chain gates (requires cargo-deny + cargo-audit installed)
cargo deny check --all-features
cargo audit --deny warnings
```

## Coding conventions

- **Rust edition 2021, toolchain `1.88.0`** (pinned in
  `rust-toolchain.toml`). Lockfile is checked in and must stay frozen on
  CI.
- **No `unwrap()` / `expect()` outside tests.** Clippy is configured to
  deny both in non-test code; tests may use them, but prefer
  `anyhow::Context` where it clarifies failures.
- **Error types are domain-specific.** Each crate exposes its own
  `Error` enum via `thiserror`; do not return `anyhow::Error` from
  library functions.
- **Public APIs must have rustdoc.** `#![deny(missing_docs)]` is set at
  every crate root. Doctests for top-level public APIs are encouraged
  and run by `cargo test --doc`.
- **Tracing, not `println!`.** Use the `tracing` crate for any output
  from library code; CLI commands may `println!` for human-facing
  summaries.
- **Never commit secrets.** Provider credentials are loaded from
  `<workspace>/.env` at CLI startup for local runs; `.env` is private and
  must not be committed.

## Adding a benchmark instance

1. Place the seven canonical files under
   `benchmark/<version>/instances/<id>/` (see an existing instance for
   the template).
2. Mirror the Lean scaffold under `lean/CTA/Benchmark/<Id>.lean` and
   reference it from `lean/CTA/Benchmark.lean`. The benchmark linter
   enforces byte-identity between the two copies.
3. Add the instance id to the relevant split file under
   `benchmark/<version>/splits/`.
4. Add a reference implementation and oracle to `crates/cta_behavior`
   and wire it into the per-instance harness test.
5. Run `cta benchmark lint --version <v>` and
   `cta benchmark manifest --version <v>`.

## Pull request checklist

- [ ] `cargo fmt --all --check` clean.
- [ ] `cargo clippy --workspace --all-targets --no-deps` zero errors.
- [ ] `cargo test --workspace --all-targets` and
      `cargo test --workspace --doc` green.
- [ ] `cta validate schemas` and
      `cta validate benchmark --version <v> --release` for every benchmark
      version your PR touches (for example `v0.1`, `v0.2`, and/or `v0.3`).
      If validation reports `MANIFEST_CONTENT_HASH_STALE`, regenerate with
      `cta benchmark manifest --version <v>` and commit the updated
      `benchmark/<version>/manifests/benchmark_manifest.json`.
- [ ] Supply-chain gates (`cargo deny check`, `cargo audit`) unchanged
      or improved.
- [ ] If the PR touches v0.3 adjudication, `results/raw_metrics*.json`,
      `annotation/agreement_*`, or `benchmark/v0.3/benchmark_paper_summary.json`:
      run `python scripts/compute_results.py --paper`,
      `python scripts/export_benchmark_paper_summary.py`, and
      `python scripts/ci_reviewer_readiness.py`.
- [ ] No placeholder prose; every comment describes intent, trade-off,
      or constraint — never "what the code does".
- [ ] If adding or changing an evaluated quantity, bumped the relevant
      contract version.
- [ ] If the PR touches `configs/prompts/code_only_v1.json`,
      `configs/prompts/naive_concat_v1.json`, `configs/prompts/text_only_v1.json`,
      `configs/prompts/full_method_v1.json`, `crates/cta_generate/src/normalize.rs`,
      `crates/cta_cli/src/cmd/annotate.rs` (especially `is_m1_target_packet`),
      `lean/CTA/Benchmark/**` modules referenced by review scaffolds, or any
      `benchmark/v0.2/annotation/review_packets/**/packet.json`:
      run `cargo test -p cta_generate --test code_only_packet_regression`,
      `cargo test -p cta_generate --test family_packet_regression`,
      (when `naive_concat_v1` packets change)
      `cargo test -p cta_generate --test naive_concat_packet_regression`,
      `cargo test -p cta_generate --test text_only_packet_regression`,
      `cargo test -p cta_generate --test full_method_priority1_packet_regression`,
      `cargo test -p cta_generate --test full_method_priority2_packet_regression`, and
      `cargo test -p cta_generate --test review_packet_lean_lint`, then run
      `cta annotate refresh-lean-check --benchmark-version v0.2 --packets-root benchmark/v0.2/annotation/review_packets --strict-m1`
      and commit any regenerated `lean_check` / `lean_diagnostics.json` /
      `proof_completion_*` / worklist artifacts. Re-run
      `cta annotate verify-review-packets` for `v0.2` afterward and commit
      `benchmark/v0.2/annotation/review_packets/verification_summary.signed.json`
      when it is part of your change set so signed `packet_hashes` match the
      final `packet.json` bytes (refresh mutates `lean_check` and changes
      hashes). New theory-backed pairs that must always pass full elaboration
      belong in `is_m1_target_packet`; see `docs/annotation_manual.md`.
