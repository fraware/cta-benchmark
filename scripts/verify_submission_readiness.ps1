# Pre-upload verification aligned with `artifacts/submission_validation_matrix.md`.
#
# Quick tier (default): Rust parity + v0.3 CLI gates + Lean + Python artifact validators.
# Does not run Python materializers or `implement_evidence_hardening.py` (use full paper gate).
#
# Usage:
#   .\scripts\verify_submission_readiness.ps1
#   .\scripts\verify_submission_readiness.ps1 -FullPaperGate   # delegates to run_paper_readiness_gate.ps1
#
param([switch]$FullPaperGate)

$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $Root

function Assert-Step([string]$Label) {
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAILED: $Label (exit $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

if ($FullPaperGate) {
    & (Join-Path $PSScriptRoot "run_paper_readiness_gate.ps1")
    exit $LASTEXITCODE
}

Write-Host ""
Write-Host "verify_submission_readiness: quick tier (see scripts/verify_submission_readiness.ps1)" -ForegroundColor Cyan
Write-Host "Note: validate benchmark --release may refresh benchmark/v0.3/manifests/release_summary.json timestamps." -ForegroundColor DarkYellow
Write-Host ""

Write-Host "=== cargo fmt ===" -ForegroundColor Cyan
cargo fmt --all -- --check
Assert-Step "cargo fmt"

Write-Host "=== cargo clippy ===" -ForegroundColor Cyan
cargo clippy --workspace --all-targets --no-deps
Assert-Step "cargo clippy"

Write-Host "=== cargo test --workspace --all-targets ===" -ForegroundColor Cyan
cargo test --workspace --all-targets
Assert-Step "cargo test"

Write-Host "=== cargo test --workspace --doc ===" -ForegroundColor Cyan
cargo test --workspace --doc
Assert-Step "cargo test --doc"

Write-Host "=== cta validate schemas ===" -ForegroundColor Cyan
cargo run -p cta_cli -- validate schemas
Assert-Step "validate schemas"

Write-Host "=== cta benchmark manifest v0.3 ===" -ForegroundColor Cyan
cargo run -p cta_cli -- benchmark manifest --version v0.3
Assert-Step "benchmark manifest"

Write-Host "=== cta validate benchmark v0.3 --release ===" -ForegroundColor Cyan
cargo run -p cta_cli -- validate benchmark --version v0.3 --release
Assert-Step "validate benchmark v0.3"

Write-Host "=== cta benchmark lint v0.3 --release ===" -ForegroundColor Cyan
cargo run -p cta_cli -- benchmark lint --version v0.3 --release
Assert-Step "benchmark lint v0.3"

Write-Host "=== refresh evidence manifest checksums (after validate may update release_summary.json) ===" -ForegroundColor Cyan
python scripts\implement_evidence_hardening.py --manifest-only
Assert-Step "implement_evidence_hardening --manifest-only"

Write-Host "=== lake build (lean/) ===" -ForegroundColor Cyan
Push-Location (Join-Path $Root "lean")
lake build
Assert-Step "lake build"
Pop-Location

Write-Host "=== validate_release_artifact.py ===" -ForegroundColor Cyan
python scripts\validate_release_artifact.py
Assert-Step "validate_release_artifact"

Write-Host "=== ci_reviewer_readiness.py ===" -ForegroundColor Cyan
python scripts\ci_reviewer_readiness.py
Assert-Step "ci_reviewer_readiness"

Write-Host "=== check_paper_claim_sources.py ===" -ForegroundColor Cyan
python scripts\check_paper_claim_sources.py
Assert-Step "check_paper_claim_sources"

Write-Host ""
Write-Host "verify_submission_readiness: OK (quick tier)." -ForegroundColor Green
Write-Host "Full ordered pipeline: .\scripts\verify_submission_readiness.ps1 -FullPaperGate" -ForegroundColor Green
Write-Host "LaTeX guard (optional): python scripts/check_paper_claim_sources.py --scan-tex --tex-path <dir>" -ForegroundColor DarkGray
