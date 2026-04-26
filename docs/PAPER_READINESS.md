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
python scripts\compute_results.py
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
python3 scripts/compute_results.py
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
```

## 5. Annotation agreement (after rater CSVs exist)

```powershell
python scripts\compute_agreement_stats.py `
  --first annotation\rater_a.csv `
  --second annotation\rater_b.csv
```

Example inputs: `annotation/rater_a.example.csv` and
`annotation/rater_b.example.csv`.

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
| System / family / failure / instance / composite | `results/system_summary.csv`, `results/family_summary.csv`, `results/failure_mode_counts.csv`, `results/instance_level.csv`, `results/composite_sensitivity.csv` |
| Prompt appendix | `appendix/PROMPTS_APPENDIX.md` |
| Canonical manifest | `benchmark/manifest.jsonl` |
