# Paper / reviewer readiness — exact commands

Run from the **repository root**. On Windows use **PowerShell**. On Linux or
macOS, use the **Bash** block where noted.

## 1. Toolchains

- Rust: `cargo --version` (workspace uses edition 2021; CI pins 1.88.0).
- Python: 3.11+.
- Lean: `lake --version` in `lean/` (Mathlib pin in `lean/lakefile.lean`).

## 2. v0.3 benchmark gate (schemas, manifest, splits, experiments, pack)

Gold audit: `benchmark/v0.3/audit/gold_signoff.json` defaults to
`release_gold_audit_status: "template_pending_human_review"` with
`approved: false` until humans complete `audit/evidence/*.csv` per
`benchmark/v0.3/audit/review_checklist.md`. That posture still passes
`validate benchmark --version v0.3 --release`.

**PowerShell (full local gate):**

```powershell
Set-Location path\to\cta-benchmark
.\scripts\experiment_setup.ps1
```

**Or step-by-step (PowerShell):**

```powershell
Set-Location path\to\cta-benchmark

python scripts\materialize_benchmark_v03.py --patch-grid-001-002-only
python scripts\build_v03_annotation_pack.py

cargo run -p cta_cli -- benchmark manifest --version v0.3
cargo run -p cta_cli -- validate benchmark --version v0.3 --release
cargo run -p cta_cli -- benchmark lint --version v0.3 --release

python scripts\build_benchmark_manifest_jsonl.py --benchmark-version v0.3
python scripts\validate_benchmark.py
python scripts\export_benchmark_stats.py
python scripts\dump_prompts_appendix.py
python scripts\materialize_v03_adjudication_artifacts.py
python scripts\materialize_repair_hotspot_artifacts.py
python scripts\reproduce_agreement_report.py
python scripts\compute_results.py --paper
python scripts\repair_counterfactual_metrics.py
python scripts\export_benchmark_paper_summary.py
```

**Bash (same steps):**

```bash
cd /path/to/cta-benchmark

python3 scripts/materialize_benchmark_v03.py --patch-grid-001-002-only
python3 scripts/build_v03_annotation_pack.py

cargo run -p cta_cli -- benchmark manifest --version v0.3
cargo run -p cta_cli -- validate benchmark --version v0.3 --release
cargo run -p cta_cli -- benchmark lint --version v0.3 --release

python3 scripts/build_benchmark_manifest_jsonl.py --benchmark-version v0.3
python3 scripts/validate_benchmark.py
python3 scripts/export_benchmark_stats.py
python3 scripts/dump_prompts_appendix.py
python3 scripts/materialize_v03_adjudication_artifacts.py
python3 scripts/materialize_repair_hotspot_artifacts.py
python3 scripts/reproduce_agreement_report.py
python3 scripts/compute_results.py --paper
python3 scripts/repair_counterfactual_metrics.py
python3 scripts/export_benchmark_paper_summary.py
```

**If `annotate coverage` must be skipped** (no Cargo on PATH):

```powershell
$env:CTA_SKIP_ANNOTATE_COVERAGE = "1"
python scripts\build_v03_annotation_pack.py
```

Then run coverage later when Cargo is available:

```powershell
cargo run -p cta_cli -- annotate coverage `
  --benchmark-version v0.3 `
  --experiment-config configs/experiments/benchmark_v03.json `
  --pack benchmark/v0.3/annotation/adjudicated_subset/pack.json `
  --out benchmark/v0.3/annotation/adjudicated_subset
```

## 3. Lean library

```powershell
Set-Location lean
lake build
Set-Location ..
```

## 4. Workspace tests (parity with CI)

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --no-deps
cargo test --workspace --all-targets
cargo test --workspace --doc
```

## 5. Annotation agreement (after rater CSVs exist)

```powershell
python scripts\reproduce_agreement_report.py
```

Equivalent direct invocation:

```powershell
python scripts\compute_agreement_stats.py `
  --first annotation\rater_a.csv `
  --second annotation\rater_b.csv
```

Example inputs: `annotation/rater_a.example.csv` and
`annotation/rater_b.example.csv`. Audit population: `annotation/agreement_packet_ids.csv`.

## 6. Anonymous artifact zip (optional upload bundle)

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\build_anonymous_artifact.ps1
```

Output: `artifacts/cta-benchmark-anonymous.zip`.

## 7. Strict near-duplicate check (optional)

```powershell
python scripts\validate_benchmark.py --strict-grid-near-dup
```

## Tables produced for the paper

| Output | Path |
|--------|------|
| Table 1 (inventory) | `results/table1_benchmark_overview.csv`, `results/table1_family_semantic_load.csv` |
| Manuscript-ready aggregates | `results/paper_table_systems.csv`, `results/paper_table_families.csv`, `results/paper_table_failure_modes.csv`, `results/paper_table_repairs.csv` |
| Per-metric system summaries + reliability | `results/system_faithfulness_summary.csv`, `results/system_consistency_summary.csv`, `results/system_vacuity_summary.csv`, `results/system_proof_utility_summary.csv`, `results/system_reliability_summary.csv`, `results/system_reliability_sensitivity.csv` |
| Per-metric family summaries | `results/family_faithfulness_summary.csv`, `results/family_consistency_summary.csv`, `results/family_vacuity_summary.csv`, `results/family_proof_utility_summary.csv` |
| Faithfulness-only legacy alias | `results/system_summary.csv`, `results/family_summary.csv` (same pooling as faithfulness columns; not a composite “reliability” score) |
| Failure / instance / composite | `results/failure_mode_counts.csv`, `results/instance_level.csv`, `results/composite_sensitivity.csv` |
| Repair sensitivity (counterfactual proxy) | `results/repair_impact_summary.json` from `python scripts/repair_counterfactual_metrics.py` (after `compute_results.py --paper` when instance rows include repair flags) |
| Bootstrap on pooled means | `results/system_summary_with_ci.json` |
| Prompt appendix | `appendix/PROMPTS_APPENDIX.md` |
| Canonical manifest | `benchmark/manifest.jsonl` |
