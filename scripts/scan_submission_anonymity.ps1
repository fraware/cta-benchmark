# Surface likely deanonymization leaks before submission.
#
# Extracted anonymous tree (text scan, recursive):
#   .\scripts\scan_submission_anonymity.ps1 -ScanRoot artifacts\_anon_scan
#
# In-repo paths (NeurIPS checklist: README, REPRODUCE, docs/*,
# configs/providers/*, experiments/run_manifests/*, results/paper_*registry*.csv):
#   .\scripts\scan_submission_anonymity.ps1 -RepoRoot .
#
# Default patterns target author / home-repo identifiers (low false positives).
# Add -AggressivePatterns to also flag common provider-secret phrases (may match
# benign JSON env-var names under configs/providers/; use after redaction).
#
param(
    [Parameter(Mandatory = $true, ParameterSetName = "Extract")]
    [string]$ScanRoot,
    [Parameter(Mandatory = $true, ParameterSetName = "Repo")]
    [string]$RepoRoot,
    [Parameter(ParameterSetName = "Extract")]
    [Parameter(ParameterSetName = "Repo")]
    [switch]$AggressivePatterns
)

$ErrorActionPreference = "Stop"

$patternsCore = @(
    "fraware",
    "Mateo",
    "Petel",
    "@stanford",
    "github.com/fraware"
)
$patternsAggressive = @(
    "OpenAI API key",
    "ANTHROPIC"
)
$patterns = @() + $patternsCore
if ($AggressivePatterns) {
    $patterns += $patternsAggressive
}

$textExtensions = @(
    ".md", ".txt", ".csv", ".json", ".jsonl", ".yml", ".yaml", ".toml",
    ".ps1", ".py", ".rs", ".lean", ".tex", ".bib", ".sh"
)

function Test-FilePatterns {
    param([string]$path, [string]$text)
    $localHits = @()
    foreach ($p in $patterns) {
        if ($text -like "*$p*") {
            $localHits += [pscustomobject]@{ Pattern = $p; File = $path }
        }
    }
    return ,$localHits
}

function Scan-Tree {
    param([string]$root)
    if (-not (Test-Path -LiteralPath $root)) {
        Write-Error "Path does not exist: $root"
    }
    $hits = @()
    Get-ChildItem -LiteralPath $root -Recurse -File -ErrorAction SilentlyContinue | ForEach-Object {
        $path = $_.FullName
        try {
            $text = Get-Content -LiteralPath $path -Raw -ErrorAction Stop
        } catch {
            return
        }
        $h = Test-FilePatterns -path $path -text $text
        if ($h) { $hits += $h }
    }
    return $hits
}

function Scan-RepoChecklist {
    param([string]$root)
    $root = (Resolve-Path -LiteralPath $root).Path
    $files = New-Object System.Collections.Generic.HashSet[string]

    foreach ($rel in @("README.md", "REPRODUCE.md", "CI_STATUS.md")) {
        $p = Join-Path $root $rel
        if (Test-Path -LiteralPath $p) { [void]$files.Add((Resolve-Path -LiteralPath $p).Path) }
    }

    $docRoot = Join-Path $root "docs"
    if (Test-Path -LiteralPath $docRoot) {
        Get-ChildItem -LiteralPath $docRoot -Recurse -File -ErrorAction SilentlyContinue | ForEach-Object {
            if ($textExtensions -contains $_.Extension.ToLowerInvariant()) {
                [void]$files.Add($_.FullName)
            }
        }
    }

    $prov = Join-Path $root "configs\providers"
    if (Test-Path -LiteralPath $prov) {
        Get-ChildItem -LiteralPath $prov -Recurse -File -ErrorAction SilentlyContinue | ForEach-Object { [void]$files.Add($_.FullName) }
    }

    $manifests = Join-Path $root "experiments\run_manifests"
    if (Test-Path -LiteralPath $manifests) {
        Get-ChildItem -LiteralPath $manifests -Recurse -File -ErrorAction SilentlyContinue | ForEach-Object { [void]$files.Add($_.FullName) }
    }

    Get-ChildItem -LiteralPath (Join-Path $root "results") -Filter "paper_*registry*.csv" -File -ErrorAction SilentlyContinue |
        ForEach-Object { [void]$files.Add($_.FullName) }

    $hits = @()
    foreach ($f in $files) {
        try {
            $text = Get-Content -LiteralPath $f -Raw -ErrorAction Stop
        } catch {
            continue
        }
        $h = Test-FilePatterns -path $f -text $text
        if ($h) { $hits += $h }
    }
    return $hits
}

$hits = @()
if ($PSCmdlet.ParameterSetName -eq "Extract") {
    $hits = @(Scan-Tree -root $ScanRoot)
    $label = $ScanRoot
} else {
    $hits = @(Scan-RepoChecklist -root $RepoRoot)
    $label = (Resolve-Path -LiteralPath $RepoRoot).Path
}

if ($hits.Count -gt 0) {
    Write-Host "ANONYMITY SCAN FAILED - $($hits.Count) pattern hit(s) under $label" -ForegroundColor Red
    $hits | Format-Table -AutoSize
    exit 1
}

Write-Host "ANONYMITY SCAN OK - no blocked substrings ($label)" -ForegroundColor Green
exit 0
