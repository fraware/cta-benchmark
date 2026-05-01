#!/usr/bin/env bash
# Pre-upload verification (Unix). See scripts/verify_submission_readiness.ps1.
#
# Usage:
#   bash scripts/verify_submission_readiness.sh
#   bash scripts/verify_submission_readiness.sh --full-paper-gate   # runs run_paper_readiness_gate.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

step() {
  echo ""
  echo "=== $1 ==="
  shift
  "$@"
}

if [[ "${1:-}" == "--full-paper-gate" ]]; then
  exec bash scripts/run_paper_readiness_gate.sh
fi

echo ""
echo "verify_submission_readiness: quick tier"
echo "Note: validate benchmark --release may refresh benchmark/v0.3/manifests/release_summary.json timestamps."
echo ""

step "cargo fmt" cargo fmt --all -- --check
step "cargo clippy" cargo clippy --workspace --all-targets --no-deps
step "cargo test" cargo test --workspace --all-targets
step "cargo test --doc" cargo test --workspace --doc
step "cta validate schemas" cargo run -p cta_cli -- validate schemas
step "cta benchmark manifest v0.3" cargo run -p cta_cli -- benchmark manifest --version v0.3
step "cta validate benchmark v0.3" cargo run -p cta_cli -- validate benchmark --version v0.3 --release
step "cta benchmark lint v0.3" cargo run -p cta_cli -- benchmark lint --version v0.3 --release

step "implement_evidence_hardening.py --manifest-only" python3 scripts/implement_evidence_hardening.py --manifest-only

step "lake build" bash -c 'cd lean && lake build'

step "validate_release_artifact.py" python3 scripts/validate_release_artifact.py
step "ci_reviewer_readiness.py" python3 scripts/ci_reviewer_readiness.py
step "check_paper_claim_sources.py" python3 scripts/check_paper_claim_sources.py

echo ""
echo "verify_submission_readiness: OK (quick tier)."
echo "Full pipeline: bash scripts/verify_submission_readiness.sh --full-paper-gate"
