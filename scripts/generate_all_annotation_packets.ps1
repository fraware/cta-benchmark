param(
  [string]$WorkspaceRoot = (Resolve-Path ".").Path,
  [string]$Version = "v0.2",
  [string]$BatchDir = "reports/openai_campaign_2026_04_22/annotation_batches"
)

$ErrorActionPreference = "Stop"

function Resolve-Abs([string]$base, [string]$path) {
  if ([System.IO.Path]::IsPathRooted($path)) { return $path }
  return (Join-Path $base $path)
}

$root = (Resolve-Path $WorkspaceRoot).Path
$batchDirAbs = Resolve-Abs $root $BatchDir
if (!(Test-Path $batchDirAbs)) {
  throw "Batch directory not found: $batchDirAbs"
}

$batches = Get-ChildItem -Path $batchDirAbs -Filter "batch_*.csv" | Sort-Object Name
if ($batches.Count -eq 0) {
  throw "No batch CSV files found in $batchDirAbs"
}

$generated = 0
foreach ($b in $batches) {
  $rel = $b.FullName.Substring([Math]::Min($root.Length + 1, $b.FullName.Length))
  powershell -NoProfile -File "scripts/generate_annotation_packets.ps1" -WorkspaceRoot $root -Version $Version -BatchCsv $rel | Out-Host
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
  $generated++
}

Write-Host ""
Write-Host "Generated review packets for $generated batch file(s)."
