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
`results/paper_system_set.md`, strict-gap disclosure in
`results/paper_strict_coverage_gap.csv`, paper-primary model identity in
`results/paper_primary_model_registry.csv`, external annotation review queues in
`annotation/external_review/`, and repair proof-status in
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
- `results/paper_table_agreement_evidence.csv`: `agreement_subset == strict_all_human_overlap`
  row is expected when `annotation/human_pass_v3/human_strict_packet_ids.csv` exists
  (current strict-overlap target: 274 rows / 84 instances / 0 mapped)
- strict-overlap rater contract:
  - ordinal columns in `annotation/rater_a_strict_all.csv` and
    `annotation/human_pass_v3/rater_b_human_strict_all.csv`
    are subsets of `{0,1,2,3}`
  - coverage sets are disjoint and coherent with `coverage_label`
- `annotation/human_pass_v3/agreement_report_human_strict_all.json` confusion
  matrices must sum to `274` for each ordinal metric
- `annotation/human_pass_v3/disagreement_log_strict_all.csv` must not contain
  generic rationale templates

For manuscript claims, treat `strict_all_human_overlap` as the independently
double-annotated strict-view overlap row; `strict_independent_only` remains a
legacy audit-population compatibility row.
- `results/paper_primary_model_registry.csv`: row count (4), headline-system membership,
  and `model_metadata_status ∈ {matched, historical_manifest_mismatch_explained}`
- `annotation/external_review/strict_review_queue.jsonl` non-empty line count
  matches `results/raw_metrics_strict.json` row count when both exist
- `annotation/external_review/mapped_review_queue.jsonl` non-empty line count
  matches `mapped_from_canonical` row count in `results/raw_metrics.json` when both exist
- if present, `annotation/external_review/semantic_corrections_v3.csv` is applied
  as the explicit correction overlay; otherwise the materializer loads
  `semantic_corrections_v1.csv` + `semantic_corrections_v2.csv` cumulatively
  (not silent edits)
- `cargo run -p cta_cli -- validate file` for adjudicated subset `manifest.json`,
  `protocol_freeze.json`, and `schemas/failure_mode_v1.json` when those paths exist
- `failure_mode_label` values in `results/raw_metrics.json` vs `schemas/failure_mode_v1.json`
- Placeholder denylist scan over `annotation/` and `results/` (selected text extensions)

For manuscript interpretation, `results/paper_strict_failure_modes.csv`
`missing_critical_semantic_unit` counts are computed from strict `raw_metrics` rows
with `missing_critical_units > 0`.

Badge URLs are omitted here because they encode the public GitHub repository
name; enable GitHub Actions badges in the camera-ready fork if permitted.
