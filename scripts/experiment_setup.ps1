# Paper experiment setup (Windows). Refreshes v0.3 manifests, annotation pack gate, JSONL, and tables.
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

# Idempotent: tightens _001/_002 informal + semantic glosses for audit distinctness.
python scripts/materialize_benchmark_v03.py --patch-grid-001-002-only
python scripts/build_v03_annotation_pack.py

cargo run -p cta_cli -- benchmark manifest --version v0.3
cargo run -p cta_cli -- validate benchmark --version v0.3 --release

python scripts/build_benchmark_manifest_jsonl.py --benchmark-version v0.3
python scripts/validate_benchmark.py
python scripts/export_benchmark_stats.py
python scripts/dump_prompts_appendix.py
python scripts/compute_results.py

Write-Host "experiment_setup: ok"
