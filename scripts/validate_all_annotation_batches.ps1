param(
  [string]$WorkspaceRoot = (Resolve-Path ".").Path,
  [string]$Version = "v0.2",
  [string]$BatchDir = "reports/openai_campaign_2026_04_22/annotation_batches",
  [switch]$RequireRawTriplet
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

$batchCsvs = Get-ChildItem -Path $batchDirAbs -Filter "batch_*.csv" | Sort-Object Name
if ($batchCsvs.Count -eq 0) {
  throw "No batch CSV files found in $batchDirAbs"
}

$results = @()
$totalPairs = 0
$totalDone = 0
$totalPending = 0

foreach ($b in $batchCsvs) {
  $rows = Import-Csv $b.FullName
  $pairCount = $rows.Count
  $totalPairs += $pairCount

  # Recompute done/pending exactly the same way as annotation_burndown.
  $done = 0
  $pending = 0
  foreach ($row in $rows) {
    $adjPath = Join-Path (Join-Path (Join-Path $root "benchmark\$Version\annotation\adjudicated_subset") $row.system_id) ($row.instance_id + ".json")
    if (Test-Path $adjPath) { $done++ } else { $pending++ }
  }

  $totalDone += $done
  $totalPending += $pending
  $completion = [math]::Round(($done / [math]::Max(1, $pairCount)), 4)
  $relPath = $b.FullName.Substring([Math]::Min($root.Length + 1, $b.FullName.Length))
  $results += [pscustomobject]@{
    batch_id = [System.IO.Path]::GetFileNameWithoutExtension($b.Name)
    path = $relPath
    pair_count = $pairCount
    done = $done
    pending = $pending
    completion_rate = $completion
  }
}

$ranked = $results | Sort-Object @{Expression="completion_rate";Descending=$true}, @{Expression="done";Descending=$true}, @{Expression="batch_id";Descending=$false}

$outDir = Join-Path $root "reports/openai_campaign_2026_04_22"
if (!(Test-Path $outDir)) { New-Item -ItemType Directory -Path $outDir | Out-Null }
$scoreCsv = Join-Path $outDir "annotation_batch_scoreboard.csv"
$scoreJson = Join-Path $outDir "annotation_batch_scoreboard.json"

$ranked | Export-Csv -NoTypeInformation -Path $scoreCsv
$payload = [pscustomobject]@{
  generated_at = (Get-Date).ToUniversalTime().ToString("o")
  benchmark_version = $Version
  total_batches = $ranked.Count
  total_pairs = $totalPairs
  total_done = $totalDone
  total_pending = $totalPending
  global_completion_rate = [math]::Round(($totalDone / [math]::Max(1, $totalPairs)), 4)
  ranked_batches = $ranked
}
$payload | ConvertTo-Json -Depth 8 | Set-Content $scoreJson

Write-Host "Annotation Batch Completion Scoreboard"
Write-Host "--------------------------------------"
Write-Host ("Global completion: {0}/{1} ({2}%)" -f $totalDone, $totalPairs, [math]::Round(($totalDone / [math]::Max(1, $totalPairs))*100,2))
Write-Host ""
for ($i = 0; $i -lt $ranked.Count; $i++) {
  $r = $ranked[$i]
  $pct = [math]::Round($r.completion_rate * 100, 2)
  Write-Host ("{0,2}. {1}  done={2}/{3}  pending={4}  completion={5}%" -f ($i+1), $r.batch_id, $r.done, $r.pair_count, $r.pending, $pct)
}

Write-Host ""
Write-Host "Wrote:"
Write-Host (" - " + $scoreCsv)
Write-Host (" - " + $scoreJson)

if ($RequireRawTriplet) {
  Write-Host ""
  Write-Host "Running strict per-batch raw-triplet validation..."
  foreach ($r in $ranked) {
    $args = @(
      "-NoProfile",
      "-File", "scripts/validate_annotation_batch.ps1",
      "-WorkspaceRoot", $root,
      "-Version", $Version,
      "-BatchCsv", $r.path,
      "-RequireRawTriplet"
    )
    powershell @args | Out-Host
    if ($LASTEXITCODE -ne 0) {
      Write-Host "FAILED strict validation for $($r.batch_id)"
      exit $LASTEXITCODE
    }
  }
  Write-Host "Strict validation passed for all batches."
}
