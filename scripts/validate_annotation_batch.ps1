param(
  [string]$WorkspaceRoot = (Resolve-Path ".").Path,
  [string]$Version = "v0.2",
  [string]$BatchCsv,
  [switch]$RequireRawTriplet
)

$ErrorActionPreference = "Stop"
if ([string]::IsNullOrWhiteSpace($BatchCsv)) {
  throw "Provide -BatchCsv (e.g. reports/openai_campaign_2026_04_22/annotation_batches/batch_01.csv)"
}

function Resolve-Abs([string]$base, [string]$path) {
  if ([System.IO.Path]::IsPathRooted($path)) { return $path }
  return (Join-Path $base $path)
}

$root = (Resolve-Path $WorkspaceRoot).Path
$batchCsvAbs = Resolve-Abs $root $BatchCsv
if (!(Test-Path $batchCsvAbs)) { throw "Batch CSV not found: $batchCsvAbs" }

$rows = Import-Csv $batchCsvAbs | Sort-Object instance_id, system_id
$adjRoot = Join-Path $root "benchmark\$Version\annotation\adjudicated_subset"
$packetRoot = Join-Path $root "benchmark\$Version\annotation\review_packets"

$errors = @()
$validated = 0

foreach ($r in $rows) {
  $adjPath = Join-Path (Join-Path $adjRoot $r.system_id) ($r.instance_id + ".json")
  if (!(Test-Path $adjPath)) {
    $errors += "MISSING_ADJUDICATED: $($r.instance_id),$($r.system_id) -> $adjPath"
    continue
  }
  cargo run -p cta_cli --quiet -- validate file --schema annotation --path $adjPath | Out-Null
  if ($LASTEXITCODE -ne 0) {
    $errors += "INVALID_ADJUDICATED_SCHEMA: $($r.instance_id),$($r.system_id) -> $adjPath"
    continue
  }
  $validated++

  if ($RequireRawTriplet) {
    $batchId = [System.IO.Path]::GetFileNameWithoutExtension($batchCsvAbs)
    $sysPacket = Join-Path (Join-Path (Join-Path $packetRoot $batchId) $r.system_id) ($r.instance_id + "__" + $r.system_id)
    $ann1 = $sysPacket + "__ann_01.json"
    $ann2 = $sysPacket + "__ann_02.json"
    $adj = $sysPacket + "__adjudicator.json"
    foreach ($p in @($ann1, $ann2, $adj)) {
      if (!(Test-Path $p)) { $errors += "MISSING_PACKET_FILE: $p" }
    }
  }
}

Write-Host "Batch rows: $($rows.Count)"
Write-Host "Validated adjudicated files: $validated"
if ($errors.Count -gt 0) {
  Write-Host "Errors:"
  $errors | ForEach-Object { Write-Host " - $_" }
  exit 1
}
Write-Host "Batch validation passed."
