param(
  [string]$Version = "v0.2",
  [string]$ExperimentConfig = "configs/experiments/benchmark_v1_openai_only.json"
)

$ErrorActionPreference = "Stop"

Write-Host "== Step 1: Repack adjudicated subset =="
cargo run -p cta_cli --quiet -- annotate pack --version $Version --from-benchmark | Out-Host
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host ""
Write-Host "== Step 2: Coverage burndown =="
powershell -NoProfile -File "scripts/annotation_burndown.ps1" -Version $Version -ExperimentConfig $ExperimentConfig | Out-Host
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host ""
Write-Host "== Step 3: Release validation gate =="
cargo run -p cta_cli --quiet -- validate benchmark --version $Version --release | Out-Host
exit $LASTEXITCODE
