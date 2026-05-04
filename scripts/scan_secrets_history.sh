#!/usr/bin/env bash
set -euo pipefail
# Run Gitleaks on full git history via Docker (same as scan_secrets_history.ps1).
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
docker run --rm \
  -v "$ROOT:/repo" \
  zricethezav/gitleaks:latest detect \
  --source=/repo \
  --config=/repo/.gitleaks.toml \
  --verbose \
  --redact
