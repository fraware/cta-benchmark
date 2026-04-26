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

Publication path (requires `results/raw_metrics.json`):

```powershell
python scripts/materialize_v03_adjudication_artifacts.py
python scripts/compute_results.py --paper
```

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

Materialized adjudication + `results/raw_metrics.json` + rater CSVs:

```powershell
python scripts/materialize_v03_adjudication_artifacts.py
python scripts/compute_agreement_stats.py --first annotation/rater_a.csv --second annotation/rater_b.csv
python scripts/materialize_repair_hotspot_artifacts.py
```

## Full paper experiment orchestration

Use `cargo run -p cta_cli -- experiment ...` with
`configs/experiments/benchmark_v03.json` once provider credentials are
configured under `configs/providers/`.
