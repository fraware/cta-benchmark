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
python scripts/compute_results.py --paper
python scripts/export_benchmark_paper_summary.py
python scripts/ci_reviewer_readiness.py
```

Paper tables and adjudication artifacts: see `docs/PAPER_READINESS.md` (Python
materializers and `compute_results.py --paper`). Canonical manuscript layer
files are `results/paper_strict_*` (headline strict evidence) and
`results/paper_expanded_*` (appendix expanded evidence), with evidence mass in
`results/paper_annotation_origin_counts.csv`, declared system set in
`results/paper_system_set.md`, and repair proof-status in
`repairs/paper_repair_status.csv` / `repairs/paper_repair_success_subset.csv` /
`repairs/paper_repair_proof_subset.csv` / `repairs/paper_proof_facing_subset.csv`.

`scripts/ci_reviewer_readiness.py` (also the **`ci.yml`** step *paper reviewer readiness*)
compares `benchmark/v0.3/benchmark_paper_summary.json` to checked-in outputs:

- `results/instance_level.csv` row count vs `expected_instance_level_rows`
- `results/raw_metrics.json` row count vs `expected_raw_metrics_rows` (when present)
- `results/raw_metrics_strict.json` row count vs `expected_raw_metrics_strict_rows` (when present)
- `annotation/agreement_packet_ids.csv` and `annotation/agreement_report.json` `n_packets`
  vs `expected_agreement_packet_audit_rows` (when those files and field exist)
- `results/paper_table_annotation_evidence.csv`: `metrics_view == strict_independent`
  row `n_eval_rows` vs `expected_raw_metrics_strict_rows` (when evidence file and field exist)
- `results/paper_table_agreement_evidence.csv`: `agreement_subset == strict_independent_only`
  row `n_packets` vs `agreement_audit_strict_independent_packet_count` (when field is set)
- `cargo run -p cta_cli -- validate file` for adjudicated subset `manifest.json`,
  `protocol_freeze.json`, and `schemas/failure_mode_v1.json` when those paths exist
- `failure_mode_label` values in `results/raw_metrics.json` vs `schemas/failure_mode_v1.json`
- Placeholder denylist scan over `annotation/` and `results/` (selected text extensions)

Badge URLs are omitted here because they encode the public GitHub repository
name; enable GitHub Actions badges in the camera-ready fork if permitted.
