#!/usr/bin/env bash
# Elaborate a single Lean file inside the CTA lake workspace.
#
# Usage: scripts/elaborate.sh <path-relative-to-lean/>
set -euo pipefail
cd "$(dirname "$0")/.."
TARGET="${1:?path to .lean file required}"
lake env lean "$TARGET"
