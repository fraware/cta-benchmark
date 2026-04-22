param(
  [string]$WorkspaceRoot = (Resolve-Path ".").Path,
  [string]$Version = "v0.2",
  [string]$BatchDir = "reports/openai_campaign_2026_04_22/annotation_batches",
  [string]$ExperimentConfig = "configs/experiments/benchmark_v1_openai_only.json",
  [switch]$Repack
)

$ErrorActionPreference = "Stop"

function Resolve-Abs([string]$base, [string]$rel) {
  if ([System.IO.Path]::IsPathRooted($rel)) { return $rel }
  return (Join-Path $base $rel)
}

$root = (Resolve-Path $WorkspaceRoot).Path
$batchDirAbs = Resolve-Abs $root $BatchDir
$expPath = Resolve-Abs $root $ExperimentConfig
$packPath = Join-Path $root "benchmark\$Version\annotation\adjudicated_subset\pack.json"
$adjudicatedRoot = Join-Path $root "benchmark\$Version\annotation\adjudicated_subset"

if ($Repack) {
  Write-Host "Repacking adjudicated subset into canonical pack..."
  cargo run -p cta_cli --quiet -- annotate pack --version $Version --from-benchmark | Out-Host
}

$exp = Get-Content -Raw $expPath | ConvertFrom-Json
$split = Get-Content -Raw (Join-Path $root "benchmark\$Version\splits\$($exp.split).json") | ConvertFrom-Json
$instances = @($split.instance_ids)
$systems = @($exp.systems)

$required = @()
foreach ($iid in $instances) {
  foreach ($sid in $systems) {
    $required += [pscustomobject]@{ instance_id = $iid; system_id = $sid }
  }
}

$available = New-Object 'System.Collections.Generic.HashSet[string]'
if (Test-Path $packPath) {
  $pack = Get-Content -Raw $packPath | ConvertFrom-Json
  foreach ($r in $pack.records) {
    [void]$available.Add("$($r.instance_id)|$($r.system_id)")
  }
}

$missing = @()
$covered = @()
foreach ($p in $required) {
  $k = "$($p.instance_id)|$($p.system_id)"
  if ($available.Contains($k)) { $covered += $p } else { $missing += $p }
}

$batchCsvs = Get-ChildItem -Path $batchDirAbs -Filter "batch_*.csv" | Sort-Object Name
$batchRows = @()
foreach ($b in $batchCsvs) {
  $rows = Import-Csv $b.FullName
  $done = 0
  $pending = 0
  foreach ($row in $rows) {
    $isDone = Test-Path (Join-Path (Join-Path $adjudicatedRoot $row.system_id) ($row.instance_id + ".json"))
    if ($isDone) { $done++ } else { $pending++ }
  }
  $batchRows += [pscustomobject]@{
    batch_id = [System.IO.Path]::GetFileNameWithoutExtension($b.Name)
    path = $b.FullName
    pair_count = $rows.Count
    done = $done
    pending = $pending
    completion_rate = [math]::Round(($done / [math]::Max(1, $rows.Count)), 4)
  }
}

$outDir = Join-Path $root "reports/openai_campaign_2026_04_22"
if (!(Test-Path $outDir)) { New-Item -ItemType Directory -Path $outDir | Out-Null }
$jsonOut = Join-Path $outDir "annotation_burndown.json"
$csvOut = Join-Path $outDir "annotation_burndown_batches.csv"
$coverageSummaryOut = Join-Path $adjudicatedRoot "coverage_summary.json"

$report = [pscustomobject]@{
  generated_at = (Get-Date).ToUniversalTime().ToString("o")
  benchmark_version = $Version
  experiment_config = $ExperimentConfig
  split = $exp.split
  required_pairs = $required.Count
  covered_pairs = $covered.Count
  missing_pairs = $missing.Count
  coverage_rate = [math]::Round(($covered.Count / [math]::Max(1, $required.Count)), 4)
  batch_status = $batchRows
  first_missing = @($missing | Select-Object -First 20)
}

$summary = [pscustomobject]@{
  generated_at = (Get-Date).ToUniversalTime().ToString("o")
  benchmark_version = $Version
  experiment_config = $ExperimentConfig
  split = $exp.split
  required_pairs = $required.Count
  covered_pairs = $covered.Count
  missing_pairs = $missing.Count
}

$report | ConvertTo-Json -Depth 8 | Set-Content $jsonOut
$batchRows | Export-Csv -NoTypeInformation -Path $csvOut
$summary | ConvertTo-Json -Depth 4 | Set-Content $coverageSummaryOut

Write-Host "Coverage: $($covered.Count)/$($required.Count) (missing=$($missing.Count))"
Write-Host "Wrote: $jsonOut"
Write-Host "Wrote: $csvOut"
Write-Host "Wrote: $coverageSummaryOut"
Write-Host ""
Write-Host "Next gate check:"
Write-Host "cargo run -p cta_cli --quiet -- validate benchmark --version $Version --release"
