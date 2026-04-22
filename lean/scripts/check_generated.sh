#!/usr/bin/env bash
# Elaborate every generated Lean file under the given run directory.
#
# Usage: scripts/check_generated.sh <runs/run_id>
set -euo pipefail
cd "$(dirname "$0")/.."
RUN_DIR="${1:?run directory required}"
GEN_DIR="$RUN_DIR/lean_generated"
if [ ! -d "$GEN_DIR" ]; then
  echo "no generated directory: $GEN_DIR" >&2
  exit 1
fi
FAIL=0
while IFS= read -r -d '' FILE; do
  if ! lake env lean "$FILE"; then
    echo "FAIL: $FILE" >&2
    FAIL=$((FAIL + 1))
  fi
done < <(find "$GEN_DIR" -name "*.lean" -print0)
exit $FAIL
