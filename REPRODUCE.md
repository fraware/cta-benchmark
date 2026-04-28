# Reproduction checklist

Commands assume repository root as working directory.

For a **single ordered checklist** (including CI parity), see
[`docs/PAPER_READINESS.md`](docs/PAPER_READINESS.md).

## Toolchain

- Rust stable (edition 2021) with `cargo`.
- Python 3.11+.
- Lean 4.12.0 with Mathlib pin from `lean/lakefile.lean` (`lake`).

## Benchmark integrity

```powershell
cargo run -p cta_cli -- benchmark manifest --version v0.3
cargo run -p cta_cli -- validate benchmark --version v0.3 --release
```

## Canonical manifest and audit scripts

```powershell
python scripts/build_benchmark_manifest_jsonl.py --benchmark-version v0.3
python scripts/validate_benchmark.py
python scripts/export_benchmark_stats.py
```

## Lean build

```powershell
cd lean
lake build
cd ..
```

## Prompt appendix

```powershell
python scripts/dump_prompts_appendix.py
```

## Results tables

Publication path (writes `raw_metrics.json` / `raw_metrics_expanded.json`,
`raw_metrics_strict.json`, agreement audit CSVs, and adjudicated pack fields
including `annotation_origin`):

```powershell
python scripts/materialize_v03_adjudication_artifacts.py
python scripts/materialize_repair_hotspot_artifacts.py
python scripts/reproduce_agreement_report.py
python scripts/compute_results.py --paper
python scripts/repair_counterfactual_metrics.py
python scripts/export_benchmark_paper_summary.py
python scripts/ci_reviewer_readiness.py
python scripts/export_final_ci_evidence.py
```

Canonical filenames for manuscript layers (also emitted by **`compute_results.py --paper`**):
**`results/paper_strict_*`** (strict independent headline), **`results/paper_expanded_*`** (expanded mapped appendix),
**`results/paper_strict_system_metrics_long.csv`**, **`results/paper_system_set.md`**,
**`results/family_reliability_summary.csv`**, **`results/paper_strict_coverage_gap.csv`**,
**`repairs/paper_repair_status.csv`**, **`repairs/paper_repair_success_subset.csv`**,
**`repairs/paper_repair_proof_subset.csv`**, and
**`repairs/paper_proof_facing_subset.csv`**; plus
**`results/paper_cost_runtime_accounting.csv`** and
**`results/paper_model_metadata_registry.csv`**,
**`results/paper_primary_model_registry.csv`**, and
**`annotation/external_review/`** review queues.
Run **`python scripts/export_benchmark_paper_summary.py`** after metric export so **`paper_system_set.md`** stays aligned with **`benchmark/v0.3/benchmark_paper_summary.json`**.
If semantic relabeling is needed, record it in
**`annotation/external_review/semantic_corrections_v3.csv`** (preferred; includes
faithfulness, vacuity, and coverage overlays). If v3 is absent, the materializer
loads **`semantic_corrections_v1.csv`** and **`semantic_corrections_v2.csv`**
cumulatively. Rerun
**`materialize_v03_adjudication_artifacts.py`** before recomputing paper outputs.

Headline paper tables from **`python scripts/compute_results.py --paper`** use
**`raw_metrics_strict.json`** only; expanded mapped summaries are written to
**`results/appendix_mapped_evidence/`**. Row-count transparency is in
**`results/paper_table_annotation_evidence.csv`**, **`results/paper_annotation_origin_counts.csv`**,
and **`results/paper_table_agreement_evidence.csv`** (agreement packet origins). For
ad-hoc analysis outside that pipeline, use **`raw_metrics_expanded.json`** (or
`raw_metrics.json`) when family-grid propagation is intended.
In `paper_strict_failure_modes.csv`, `missing_critical_semantic_unit` counts come
from strict rows with `missing_critical_units > 0` in `raw_metrics_strict.json`.

CI / quick checkout (demo fabric if `raw_metrics.json` is absent; stderr warning):

```powershell
python scripts/compute_results.py
```

## v0.3 annotation coverage gate

`configs/experiments/benchmark_v03.json` points at
`benchmark/v0.3/annotation/adjudicated_subset/pack.json`.

Skeleton pairs after split changes:

```powershell
python scripts/build_v03_annotation_pack.py
```

Materialized adjudication + raw metrics + rater CSVs + agreement audit:

```powershell
python scripts/materialize_v03_adjudication_artifacts.py
python scripts/materialize_repair_hotspot_artifacts.py
python scripts/reproduce_agreement_report.py
```

## Full paper experiment orchestration

Use `cargo run -p cta_cli -- experiment ...` with
`configs/experiments/benchmark_v03.json` once provider credentials are
configured under `configs/providers/`.
