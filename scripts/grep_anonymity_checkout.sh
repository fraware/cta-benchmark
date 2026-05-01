#!/usr/bin/env bash
# NeurIPS-style anonymity greps on an extracted artifact tree (§7 supplement).
# Usage:
#   bash scripts/grep_anonymity_checkout.sh /path/to/extracted/cta-contents
#
# Uses grep -rI (skip binary). Exit 0 if no hits; exit 1 if any pattern matches.
set -euo pipefail

SCAN_ROOT="${1:?usage: $0 /path/to/extracted-artifact-root}"

if [[ ! -d "$SCAN_ROOT" ]]; then
  echo "error: not a directory: $SCAN_ROOT" >&2
  exit 2
fi

patterns=(
  "fraware"
  "Mateo"
  "Petel"
  "@stanford"
  "OpenAI API key"
  "ANTHROPIC"
  "github.com/fraware"
)

failed=0
for p in "${patterns[@]}"; do
  # grep returns 0 when matches exist
  if grep -rIl "$p" "$SCAN_ROOT" >/dev/null 2>&1; then
    echo "ANONYMITY GREP HIT: $p" >&2
    grep -rIn "$p" "$SCAN_ROOT" 2>/dev/null | head -50 >&2 || true
    failed=1
  fi
done

if [[ "$failed" -ne 0 ]]; then
  echo "grep_anonymity_checkout: FAILED (see hits above)" >&2
  exit 1
fi

echo "grep_anonymity_checkout: OK - no pattern hits under $SCAN_ROOT"
