# Paper / reviewer readiness — exact commands

Run from the **repository root**. On Windows use **PowerShell**. On Linux or
macOS, use the **Bash** block where noted.

## 1. Toolchains

- Rust: `cargo --version` (workspace uses edition 2021; CI pins 1.88.0).
- Python: 3.11+.
- Lean: `lake --version` in `lean/` (Mathlib pin in `lean/lakefile.lean`).

## Evidence-Hardening Update (2026-04-28)

Add these commands to the paper-readiness pass:

```powershell
python scripts\implement_evidence_hardening.py
python scripts\validate_release_artifact.py
```

Required outputs now include:

- `annotation/human_pass_v3/*`
- `results/selection_robustness.csv`
- `results/prompt_token_accounting.csv`
- `results/cross_model_pilot_*.csv`
- `repairs/repair_attempts.csv`
- `artifacts/evidence_hardening_manifest.json`

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
python scripts\implement_evidence_hardening.py
python scripts\validate_release_artifact.py
python scripts\ci_reviewer_readiness.py
python scripts\export_final_ci_evidence.py
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
python3 scripts/implement_evidence_hardening.py
python3 scripts/validate_release_artifact.py
python3 scripts/ci_reviewer_readiness.py
python3 scripts/export_final_ci_evidence.py
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

If external review identifies semantic mislabeling, record corrections in
`annotation/external_review/semantic_corrections_v3.csv` (preferred) and rerun:

```powershell
python scripts\materialize_v03_adjudication_artifacts.py
python scripts\compute_results.py --paper
python scripts\export_benchmark_paper_summary.py
python scripts\ci_reviewer_readiness.py
python scripts\export_final_ci_evidence.py
```

This preserves an explicit correction audit trail instead of silent packet edits.
If `semantic_corrections_v3.csv` is absent, the materializer loads
`semantic_corrections_v1.csv` and `semantic_corrections_v2.csv` cumulatively
(in that order) for backward compatibility.

## 6. Annotation agreement (strict-overlap v3)

```powershell
python scripts\compute_human_strict_agreement.py `
  --packet-map annotation\human_pass_v3\human_strict_packet_ids.csv `
  --rater-a annotation\rater_a_strict_all.csv `
  --rater-b annotation\human_pass_v3\rater_b_human_strict_all.csv `
  --out-json annotation\human_pass_v3\agreement_report_human_strict_all.json `
  --out-md annotation\human_pass_v3\agreement_report_human_strict_all.md `
  --out-disagreements annotation\human_pass_v3\disagreement_log_strict_all.csv
```

The strict-overlap report is validity-gated by CI for:

- canonical ordinal scale `{0,1,2,3}` in both raters,
- confusion matrix totals of exactly `274` for each ordinal metric,
- coverage/missing coherence,
- and non-generic adjudication rationales.

## 7. Anonymous artifact zip (optional upload bundle)

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\build_anonymous_artifact.ps1
```

Output: `artifacts/cta-benchmark-anonymous.zip`.

Optional final parity evidence artifact:

```powershell
python scripts\export_final_ci_evidence.py
```

Output: `artifacts/final_ci_run_YYYYMMDD.md`.

## 8. Strict near-duplicate check (optional)

```powershell
python scripts\validate_benchmark.py --strict-grid-near-dup
```

## 9. Strict independent completion status

Current strict-headline coverage is
**84 unique instances / 274 strict rows** with
`n_mapped_from_canonical=0` in the strict view.
Expanded remains 84 / 336 with 114 mapped-from-canonical rows.

The primary submission posture is now the independently double-annotated strict
view with full strict overlap (no remaining strict
coverage gap required for headline evidence). Any further direct-adjudication
wave should be treated as robustness expansion, not headline completion.

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
python scripts\strict_gap_13x4_worklist.py
python scripts\materialize_v03_adjudication_artifacts.py
python scripts\compute_results.py --paper
python scripts\implement_evidence_hardening.py
python scripts\compute_human_strict_agreement.py `
  --packet-map annotation\human_pass_v3\human_strict_packet_ids.csv `
  --rater-a annotation\rater_a_strict_all.csv `
  --rater-b annotation\human_pass_v3\rater_b_human_strict_all.csv `
  --out-json annotation\human_pass_v3\agreement_report_human_strict_all.json `
  --out-md annotation\human_pass_v3\agreement_report_human_strict_all.md `
  --out-disagreements annotation\human_pass_v3\disagreement_log_strict_all.csv
python scripts\export_benchmark_paper_summary.py
python scripts\validate_release_artifact.py
python scripts\ci_reviewer_readiness.py
```

## Tables produced for the paper

| Output | Path |
|--------|------|
| Table 1 (inventory) | `results/table1_benchmark_overview.csv`, `results/table1_family_semantic_load.csv` |
| Annotation evidence (strict vs expanded eval rows) | `results/paper_table_annotation_evidence.csv` |
| Agreement packet evidence (audit population origins) | `results/paper_table_agreement_evidence.csv` |
| Manuscript-ready aggregates — **headline (strict)** | `results/paper_table_*.csv` (legacy names) and explicit layer files: `results/paper_strict_system_summary.csv`, `results/paper_strict_family_summary.csv`, `results/paper_strict_failure_modes.csv`, `results/paper_strict_instance_level.csv`, plus per-metric aliases `results/paper_strict_system_*_summary.csv` and stacked `results/paper_strict_system_metrics_long.csv` (`missing_critical_semantic_unit` in strict failure tables is sourced from strict rows where `missing_critical_units > 0`) |
| Appendix — expanded mapped robustness | `results/paper_expanded_system_summary.csv`, `results/paper_expanded_family_summary.csv`, `results/paper_expanded_failure_modes.csv` (copies promoted from `results/appendix_mapped_evidence/` after `compute_results.py --paper`) |
| Evidence mass (direct vs propagated row counts) | `results/paper_annotation_origin_counts.csv` |
| Declared primary system set (four-system primary study) | `results/paper_system_set.md` (also `benchmark/v0.3/benchmark_paper_summary.json`) |
| Per-metric system summaries + reliability | `results/system_faithfulness_summary.csv`, `results/system_consistency_summary.csv`, `results/system_vacuity_summary.csv`, `results/system_proof_utility_summary.csv`, `results/system_reliability_summary.csv`, `results/system_reliability_sensitivity.csv` |
| Per-metric family summaries | `results/family_faithfulness_summary.csv`, `results/family_consistency_summary.csv`, `results/family_vacuity_summary.csv`, `results/family_proof_utility_summary.csv` |
| Family reliability summary | `results/family_reliability_summary.csv`, `results/paper_strict_family_reliability_summary.csv`, `results/paper_expanded_family_reliability_summary.csv` |
| Faithfulness-only legacy alias | `results/system_summary.csv`, `results/family_summary.csv` (same pooling as faithfulness columns; not a composite “reliability” score) |
| Failure / instance / composite | `results/failure_mode_counts.csv`, `results/instance_level.csv`, `results/composite_sensitivity.csv` |
| Strict coverage-gap disclosure | `results/paper_strict_coverage_gap.csv` (missing strict unique instances/families relative to expanded) |
| Repair sensitivity (counterfactual proxy) | `results/repair_impact_summary.json` from `python scripts/repair_counterfactual_metrics.py` (after `compute_results.py --paper` when instance rows include repair flags) |
| Repair study proof-status export | `repairs/paper_repair_status.csv` from `python scripts/export_paper_repair_status.py` (also run automatically at end of `compute_results.py --paper`) |
| Repair manuscript subset (selected hotspots only) | `repairs/paper_repair_success_subset.csv` (`selected_for_repair_budget=true` rows with `repair_success`, `elaborated`, `admit_count`, `axiom_count`, `proof_mode`) |
| Repair proof-facing subset (Lean elaborated only) | `repairs/paper_repair_proof_subset.csv` (selected-budget + elaborated analysis subset) and `repairs/paper_proof_facing_subset.csv` (paper-facing metadata-rich subset over all hotspot packets with `elaborated=true`) |
| Cost/runtime accounting | `results/paper_cost_runtime_accounting.csv` (tokens/time/cost/runner metadata when present in run manifests) |
| Model/run metadata registry | `results/paper_model_metadata_registry.csv` (system card vs run-manifest model metadata reconciliation) |
| Paper-primary model registry (headline runs only) | `results/paper_primary_model_registry.csv` (one row per headline system with `model_metadata_status`) |
| External annotation audit bundle | `annotation/external_review/strict_review_queue.jsonl`, `annotation/external_review/strict_review_queue.csv`, `annotation/external_review/mapped_review_queue.jsonl`, `annotation/external_review/review_schema.md` |
| Bootstrap on pooled means | `results/system_summary_with_ci.json` |
| Prompt appendix | `appendix/PROMPTS_APPENDIX.md` |
| Canonical manifest | `benchmark/manifest.jsonl` |
