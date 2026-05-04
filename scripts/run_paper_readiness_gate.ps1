# Canonical v0.3 paper-readiness gate (ordered steps; same sequence as REPRODUCE.md / README gate block).
#
# Usage:
#   .\scripts\run_paper_readiness_gate.ps1
#   .\scripts\run_paper_readiness_gate.ps1 -SkipExportFinal
#
param([switch]$SkipExportFinal)

$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $Root

function Assert-Step([string]$Label) {
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAILED: $Label (exit $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

python scripts\materialize_benchmark_v03.py --patch-grid-001-002-only
Assert-Step "materialize_benchmark_v03"
python scripts\build_v03_annotation_pack.py
Assert-Step "build_v03_annotation_pack"

cargo run -p cta_cli -- benchmark manifest --version v0.3
Assert-Step "cargo benchmark manifest"
cargo run -p cta_cli -- validate benchmark --version v0.3 --release
Assert-Step "cargo validate benchmark"
cargo run -p cta_cli -- benchmark lint --version v0.3 --release
Assert-Step "cargo benchmark lint"

python scripts\build_benchmark_manifest_jsonl.py --benchmark-version v0.3
Assert-Step "build_benchmark_manifest_jsonl"
python scripts\validate_benchmark.py
Assert-Step "validate_benchmark"
python scripts\export_benchmark_stats.py
Assert-Step "export_benchmark_stats"
python scripts\dump_prompts_appendix.py
Assert-Step "dump_prompts_appendix"

python scripts\materialize_v03_adjudication_artifacts.py
Assert-Step "materialize_v03_adjudication_artifacts"
python scripts\materialize_repair_hotspot_artifacts.py
Assert-Step "materialize_repair_hotspot_artifacts"
python scripts\reproduce_agreement_report.py
Assert-Step "reproduce_agreement_report"
python scripts\implement_evidence_hardening.py
Assert-Step "implement_evidence_hardening"
python scripts\repair_counterfactual_metrics.py
Assert-Step "repair_counterfactual_metrics"
python scripts\validate_release_artifact.py
Assert-Step "validate_release_artifact"
python scripts\ci_reviewer_readiness.py
Assert-Step "ci_reviewer_readiness"
python scripts\compute_human_strict_agreement.py `
  --packet-map annotation/human_pass_v3/human_strict_packet_ids.csv `
  --rater-a annotation/rater_a_strict_all.csv `
  --rater-b annotation/human_pass_v3/rater_b_human_strict_all.csv `
  --out-json annotation/human_pass_v3/agreement_report_human_strict_all.json `
  --out-md annotation/human_pass_v3/agreement_report_human_strict_all.md `
  --out-disagreements annotation/human_pass_v3/disagreement_log_strict_all.csv
Assert-Step "compute_human_strict_agreement"
python scripts\check_paper_claim_sources.py
Assert-Step "check_paper_claim_sources"

if (-not $SkipExportFinal) {
    python scripts\export_final_ci_evidence.py
    Assert-Step "export_final_ci_evidence"
}

Write-Host "Paper readiness gate: OK" -ForegroundColor Green
