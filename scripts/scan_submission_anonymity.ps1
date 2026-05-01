# Surface likely deanonymization leaks before submission.
# Usage:
#   Expand the anonymous zip, then:
#   .\scripts\scan_submission_anonymity.ps1 -ScanRoot artifacts\_anon_scan
#
param(
    [Parameter(Mandatory = $true)]
    [string]$ScanRoot
)

$ErrorActionPreference = "Stop"
if (-not (Test-Path -LiteralPath $ScanRoot)) {
    Write-Error "ScanRoot does not exist: $ScanRoot"
}

$patterns = @(
    "fraware",
    "Mateo",
    "Petel",
    "@stanford",
    "OpenAI API key",
    "ANTHROPIC",
    "github.com/fraware"
)

$hits = @()
Get-ChildItem -LiteralPath $ScanRoot -Recurse -File | ForEach-Object {
    $path = $_.FullName
    try {
        $text = Get-Content -LiteralPath $path -Raw -ErrorAction Stop
    } catch {
        return
    }
    foreach ($p in $patterns) {
        if ($text -like "*$p*") {
            $hits += [pscustomobject]@{ Pattern = $p; File = $path }
        }
    }
}

if ($hits.Count -gt 0) {
    Write-Host "ANONYMITY SCAN FAILED — $($hits.Count) pattern hit(s):" -ForegroundColor Red
    $hits | Format-Table -AutoSize
    exit 1
}

Write-Host "ANONYMITY SCAN OK — no blocked substrings in text files under $ScanRoot" -ForegroundColor Green
exit 0
