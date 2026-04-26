# CI status summary (repository health)

Workflows under `.github/workflows/`:

| Workflow | Purpose |
|----------|---------|
| `ci.yml` | Primary Rust workspace checks |
| `benchmark-lint.yml` | Benchmark schema and lint gates |
| `nightly-evals.yml` | Scheduled heavier evaluations |
| `supply-chain.yml` | Dependency/supply-chain hygiene |

Local parity check before submission (mirror `.github/workflows/ci.yml`,
`benchmark-lint.yml`, `supply-chain.yml`, and `docs/PAPER_READINESS.md`):

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --no-deps
cargo test --workspace --all-targets
cargo test --workspace --doc
cargo run -p cta_cli -- validate schemas
cargo run -p cta_cli -- validate benchmark --version v0.1 --release
cargo run -p cta_cli -- validate benchmark --version v0.3 --release
cargo deny check
cargo audit --deny warnings
python scripts/ci_reviewer_readiness.py
```

Paper tables and adjudication artifacts: see `docs/PAPER_READINESS.md` (Python
materializers and `compute_results.py --paper`).

Badge URLs are omitted here because they encode the public GitHub repository
name; enable GitHub Actions badges in the camera-ready fork if permitted.
