param(
  [string]$WorkspaceRoot = (Resolve-Path ".").Path,
  [string]$Version = "v0.2",
  [string]$BatchCsv,
  [string]$OutDir = ""
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

$batchId = [System.IO.Path]::GetFileNameWithoutExtension($batchCsvAbs)
if ([string]::IsNullOrWhiteSpace($OutDir)) {
  $outAbs = Join-Path $root ("benchmark\$Version\annotation\review_packets\" + $batchId)
} else {
  $outAbs = Resolve-Abs $root $OutDir
}
if (!(Test-Path $outAbs)) { New-Item -ItemType Directory -Path $outAbs | Out-Null }

$rows = Import-Csv $batchCsvAbs | Sort-Object instance_id, system_id

$rubricPath = "benchmark/$Version/annotation/rubric_v1.md"
$manualPath = "docs/annotation_manual.md"

foreach ($r in $rows) {
  $sysDir = Join-Path $outAbs $r.system_id
  if (!(Test-Path $sysDir)) { New-Item -ItemType Directory -Path $sysDir | Out-Null }

  $base = "$($r.instance_id)__$($r.system_id)"
  $ann1 = Join-Path $sysDir ($base + "__ann_01.json")
  $ann2 = Join-Path $sysDir ($base + "__ann_02.json")
  $adj = Join-Path $sysDir ($base + "__adjudicator.json")
  $note = Join-Path $sysDir ($base + "__packet.md")

  $templateAnn = [ordered]@{
    schema_version = "schema_v1"
    rubric_version = "rubric_v1"
    instance_id = $r.instance_id
    system_id = $r.system_id
    annotator_id = "ann_01"
    set_level_scores = [ordered]@{
      semantic_faithfulness = 0.0
      code_consistency = 0.0
      vacuity_rate = 0.0
      proof_utility = 0.0
    }
    critical_unit_coverage = [ordered]@{
      covered = @()
      missed = @()
    }
    generated_obligations = @()
    annotator_notes = "TODO"
  }
  $templateAnn2 = $templateAnn.PSObject.Copy()
  $templateAnn2.annotator_id = "ann_02"

  $templateAdj = $templateAnn.PSObject.Copy()
  $templateAdj.annotator_id = "adjudicator"

  if (!(Test-Path $ann1)) { ($templateAnn | ConvertTo-Json -Depth 8) | Set-Content $ann1 }
  if (!(Test-Path $ann2)) { ($templateAnn2 | ConvertTo-Json -Depth 8) | Set-Content $ann2 }
  if (!(Test-Path $adj)) { ($templateAdj | ConvertTo-Json -Depth 8) | Set-Content $adj }

  $md = @()
  $md += "# Review Packet: $($r.instance_id) / $($r.system_id)"
  $md += ""
  $relAnn1 = $ann1.Substring([Math]::Min($root.Length + 1, $ann1.Length))
  $relAnn2 = $ann2.Substring([Math]::Min($root.Length + 1, $ann2.Length))
  $relAdj = $adj.Substring([Math]::Min($root.Length + 1, $adj.Length))
  $md += "- Rubric: " + $rubricPath
  $md += "- Manual: " + $manualPath
  $md += "- Ann 01 file: " + $relAnn1
  $md += "- Ann 02 file: " + $relAnn2
  $md += "- Adjudicator file: " + $relAdj
  $md += ""
  $md += "## Required completion steps"
  $md += "1. ann_01 submits independent labels."
  $md += "2. ann_02 submits independent labels."
  $md += "3. Adjudicator resolves disagreements in __adjudicator.json."
  $md += "4. Validate each JSON with:"
  $md += "   - cargo run -p cta_cli --quiet -- validate file --schema annotation --path <file>"
  $md += ""
  $md += "## Notes"
  $md += "- Replace placeholder scalar values and obligations."
  $md += "- generated_obligations length should match obligations evaluated for this pair."
  Set-Content -Path $note -Value $md
}

$index = Join-Path $outAbs "README.md"
$lines = @()
$lines += "# Annotation Review Packet Index ($batchId)"
$lines += ""
$lines += "- Source batch: " + $BatchCsv
$lines += "- Total pairs: " + $rows.Count
$relOut = $outAbs.Substring([Math]::Min($root.Length + 1, $outAbs.Length))
$lines += "- Output root: " + $relOut
$lines += ""
$lines += "Each pair has three templates (ann_01, ann_02, adjudicator) plus a packet checklist markdown."
Set-Content -Path $index -Value $lines

Write-Host "Generated review packets for $($rows.Count) pair(s): $outAbs"
