#Requires -Version 5.1
<#
.SYNOPSIS
  Run Gitleaks on the full git history (Docker).

.DESCRIPTION
  Requires Docker. Mounts the repo at /repo and uses .gitleaks.toml at the root.
  Exit code follows gitleaks (0 = clean, 1 = findings).
#>
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

docker run --rm `
  -v "${repoRoot}:/repo" `
  zricethezav/gitleaks:latest detect `
  --source=/repo `
  --config=/repo/.gitleaks.toml `
  --verbose `
  --redact

exit $LASTEXITCODE
