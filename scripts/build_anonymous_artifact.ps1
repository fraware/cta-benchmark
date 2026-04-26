# Build a blind-review-friendly zip under artifacts/ (no .git, no target, no local runs).
$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Dest = Join-Path $Root "artifacts\cta-benchmark-anonymous"
$Zip = Join-Path $Root "artifacts\cta-benchmark-anonymous.zip"

New-Item -ItemType Directory -Force -Path (Split-Path $Dest) | Out-Null
if (Test-Path $Dest) { Remove-Item $Dest -Recurse -Force }
New-Item -ItemType Directory -Force -Path $Dest | Out-Null

function Copy-Tree($Rel) {
    $src = Join-Path $Root $Rel
    $dst = Join-Path $Dest $Rel
    if (-not (Test-Path $src)) { return }
    New-Item -ItemType Directory -Force -Path (Split-Path $dst) | Out-Null
    robocopy $src $dst /E /NFL /NDL /NJH /NJS /nc /ns /np | Out-Null
    if ($LASTEXITCODE -ge 8) {
        throw "robocopy failed for $Rel (exit $LASTEXITCODE)"
    }
}

Copy-Tree "benchmark\v0.3"
Copy-Tree "configs"
Copy-Tree "crates"
Copy-Tree "schemas"
Copy-Tree "scripts"
Copy-Tree "experiments"
Copy-Tree "lean"
Copy-Tree "annotation"
Copy-Tree "repairs"
Copy-Tree "appendix"
Copy-Tree "results"
Copy-Tree "docs"

foreach ($f in @("LICENSE", "Cargo.toml", "ARTIFACT_CARD.md", "REPRODUCE.md", "CI_STATUS.md", "README.md")) {
    $p = Join-Path $Root $f
    if (Test-Path $p) {
        Copy-Item $p (Join-Path $Dest $f) -Force
    }
}

$mj = Join-Path $Root "benchmark\manifest.jsonl"
if (Test-Path $mj) {
    $bd = Join-Path $Dest "benchmark"
    New-Item -ItemType Directory -Force -Path $bd | Out-Null
    Copy-Item $mj (Join-Path $bd "manifest.jsonl") -Force
}

if (Test-Path $Zip) { Remove-Item $Zip -Force }
Compress-Archive -Path (Join-Path $Dest "*") -DestinationPath $Zip -Force
Write-Host "wrote $Zip"
