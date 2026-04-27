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
python scripts\ci_reviewer_readiness.py
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
python3 scripts/ci_reviewer_readiness.py
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

## 5. Reviewer readiness script (same as `ci.yml`)

After `export_benchmark_paper_summary.py`, run:

```powershell
python scripts\ci_reviewer_readiness.py
```

This mirrors the GitHub Actions *paper reviewer readiness* step: it asserts
`benchmark/v0.3/benchmark_paper_summary.json` matches row counts in
`results/instance_level.csv`, `results/raw_metrics*.json`, agreement audit
artifacts, and the strict rows in `results/paper_table_*_evidence.csv`; runs
`cta validate file` on a few frozen JSON artifacts; checks `failure_mode_label`
values against `schemas/failure_mode_v1.json`; and scans `annotation/` and
`results/` for disallowed placeholder phrasing. Full field list is summarized in
`CI_STATUS.md`.

## 6. Annotation agreement (after rater CSVs exist)

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

## 7. Anonymous artifact zip (optional upload bundle)

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\build_anonymous_artifact.ps1
```

Output: `artifacts/cta-benchmark-anonymous.zip`.

## 8. Strict near-duplicate check (optional)

```powershell
python scripts\validate_benchmark.py --strict-grid-near-dup
```

## 9. Option-2 strict coverage expansion (direct adjudication wave)

Current strict-independent coverage after the Option-2 wave is
**71 unique instances / 222 strict rows** (expanded remains 84 / 336 with
114 mapped-from-canonical rows). If you want to increase strict coverage
further, generate and execute another direct-adjudication wave:

```powershell
python scripts\plan_v03_direct_adjudication_wave.py --target-pairs 128
```

Use `docs/direct_adjudication_wave_checklist.md` for the curator batch protocol
(recommended: 4 batches x 32 pairs with acceptance gates per batch).

Curators then fill
`benchmark/v0.3/annotation/human_adjudicated/direct_adjudicated_pairs.csv`
with completed `(instance_id, system_id)` rows and
`annotation_origin=direct_adjudicated` (or `direct_human`), and rerun:

```powershell
python scripts\materialize_v03_adjudication_artifacts.py
python scripts\compute_results.py --paper
python scripts\export_benchmark_paper_summary.py
python scripts\ci_reviewer_readiness.py
```

## Tables produced for the paper

| Output | Path |
|--------|------|
| Table 1 (inventory) | `results/table1_benchmark_overview.csv`, `results/table1_family_semantic_load.csv` |
| Annotation evidence (strict vs expanded eval rows) | `results/paper_table_annotation_evidence.csv` |
| Agreement packet evidence (audit population origins) | `results/paper_table_agreement_evidence.csv` |
| Manuscript-ready aggregates — **headline (strict)** | `results/paper_table_*.csv` (legacy names) and explicit layer files: `results/paper_strict_system_summary.csv`, `results/paper_strict_family_summary.csv`, `results/paper_strict_failure_modes.csv`, `results/paper_strict_instance_level.csv`, plus per-metric aliases `results/paper_strict_system_*_summary.csv` and stacked `results/paper_strict_system_metrics_long.csv` |
| Appendix — expanded mapped robustness | `results/paper_expanded_system_summary.csv`, `results/paper_expanded_family_summary.csv`, `results/paper_expanded_failure_modes.csv` (copies promoted from `results/appendix_mapped_evidence/` after `compute_results.py --paper`) |
| Evidence mass (direct vs propagated row counts) | `results/paper_annotation_origin_counts.csv` |
| Declared primary system set (four vs three+appendix calibration) | `results/paper_system_set.md` (also `benchmark/v0.3/benchmark_paper_summary.json`) |
| Per-metric system summaries + reliability | `results/system_faithfulness_summary.csv`, `results/system_consistency_summary.csv`, `results/system_vacuity_summary.csv`, `results/system_proof_utility_summary.csv`, `results/system_reliability_summary.csv`, `results/system_reliability_sensitivity.csv` |
| Per-metric family summaries | `results/family_faithfulness_summary.csv`, `results/family_consistency_summary.csv`, `results/family_vacuity_summary.csv`, `results/family_proof_utility_summary.csv` |
| Faithfulness-only legacy alias | `results/system_summary.csv`, `results/family_summary.csv` (same pooling as faithfulness columns; not a composite “reliability” score) |
| Failure / instance / composite | `results/failure_mode_counts.csv`, `results/instance_level.csv`, `results/composite_sensitivity.csv` |
| Repair sensitivity (counterfactual proxy) | `results/repair_impact_summary.json` from `python scripts/repair_counterfactual_metrics.py` (after `compute_results.py --paper` when instance rows include repair flags) |
| Repair study proof-status export | `repairs/paper_repair_status.csv` from `python scripts/export_paper_repair_status.py` (also run automatically at end of `compute_results.py --paper`) |
| Repair manuscript subset (selected hotspots only) | `repairs/paper_repair_success_subset.csv` (`selected_for_repair_budget=true` rows with `repair_success`, `elaborated`, `admit_count`, `axiom_count`, `proof_mode`) |
| Repair proof-facing subset (Lean elaborated only) | `repairs/paper_repair_proof_subset.csv` (analysis subset) and `repairs/paper_proof_facing_subset.csv` (paper-facing minimal schema: packet/system/instance + elaboration/proof fields) |
| Bootstrap on pooled means | `results/system_summary_with_ci.json` |
| Prompt appendix | `appendix/PROMPTS_APPENDIX.md` |
| Canonical manifest | `benchmark/manifest.jsonl` |
