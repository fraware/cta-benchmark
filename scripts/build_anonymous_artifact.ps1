# Build a blind-review-friendly zip under artifacts/ (no .git, no local runs/).
# Excludes Rust `target/` trees and `lean/.lake/` so the archive stays small;
# reviewers still run `cargo build` / `lake build` from the extracted tree.
$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Dest = Join-Path $Root "artifacts\cta-benchmark-anonymous"
$Zip = Join-Path $Root "artifacts\cta-benchmark-anonymous.zip"
$Log = Join-Path $Root "artifacts\build_anonymous_artifact.log"

function Write-BuildLog([string]$Message) {
    $line = "[{0:u}] {1}" -f (Get-Date).ToUniversalTime(), $Message
    Add-Content -Path $Log -Encoding utf8 -Value $line
}

New-Item -ItemType Directory -Force -Path (Split-Path $Dest) | Out-Null
Set-Content -Path $Log -Encoding utf8 -Value ""
Write-BuildLog "build_anonymous_artifact.ps1 start root=$Root"
if (Test-Path $Dest) { Remove-Item $Dest -Recurse -Force }
New-Item -ItemType Directory -Force -Path $Dest | Out-Null

function Copy-Tree {
    param(
        [Parameter(Mandatory = $true)][string]$Rel,
        [string[]]$ExcludeDirNames = @()
    )
    $src = Join-Path $Root $Rel
    $dst = Join-Path $Dest $Rel
    if (-not (Test-Path $src)) { return }
    New-Item -ItemType Directory -Force -Path (Split-Path $dst) | Out-Null
    $rcArgs = @($src, $dst, "/E", "/NFL", "/NDL", "/NJH", "/NJS", "/nc", "/ns", "/np")
    if ($ExcludeDirNames.Count -gt 0) {
        $rcArgs += "/XD"
        $rcArgs += $ExcludeDirNames
    }
    & robocopy @rcArgs | Out-Null
    $rc = $LASTEXITCODE
    Write-BuildLog "robocopy $Rel exit=$rc"
    if ($rc -ge 8) {
        Write-BuildLog "robocopy FAILURE for $Rel"
        throw "robocopy failed for $Rel (exit $rc)"
    }
}

Copy-Tree "benchmark\v0.3"
Copy-Tree "configs"
Copy-Tree "crates" -ExcludeDirNames @("target")
Copy-Tree "schemas"
Copy-Tree "scripts" -ExcludeDirNames @("__pycache__")
Copy-Tree "experiments"
Copy-Tree "lean" -ExcludeDirNames @(".lake")
Copy-Tree "annotation"
Copy-Tree "repairs"
Copy-Tree "appendix"
Copy-Tree "results"
Copy-Tree "docs"

# Root files referenced from README (and toolchain pins); keep bundle self-contained for blind review.
foreach ($f in @(
        "LICENSE",
        "Cargo.toml",
        "rust-toolchain.toml",
        "REPRODUCE.md",
        "CI_STATUS.md",
        "README.md",
        "CONTRIBUTING.md",
        "CODE_OF_CONDUCT.md",
        "SECURITY.md",
        "CITATION.cff"
    )) {
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

$pyCmd = Get-Command python -ErrorAction SilentlyContinue
if (-not $pyCmd) { $pyCmd = Get-Command python3 -ErrorAction SilentlyContinue }
if (-not $pyCmd) { throw "python not found on PATH (needed for redact_anonymous_artifact_tree.py)" }
$pyExe = $pyCmd.Path
if (-not $pyExe) { $pyExe = $pyCmd.Source }
Write-BuildLog "running redact_anonymous_artifact_tree.py"
& $pyExe (Join-Path $Root "scripts\redact_anonymous_artifact_tree.py") $Dest
$redactSelf = Join-Path $Dest "scripts\redact_anonymous_artifact_tree.py"
if (Test-Path $redactSelf) { Remove-Item $redactSelf -Force }

if (Test-Path $Zip) { Remove-Item $Zip -Force }
Write-BuildLog "Compress-Archive -> $Zip"
Compress-Archive -Path (Join-Path $Dest "*") -DestinationPath $Zip -Force
$len = (Get-Item $Zip).Length
Write-BuildLog "done zip bytes=$len"
Write-Host "wrote $Zip ($len bytes); log: $Log"
