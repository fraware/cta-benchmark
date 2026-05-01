#!/usr/bin/env bash
# Canonical v0.3 paper-readiness gate — same order as docs/PAPER_READINESS.md §2 (Bash block).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

run() {
  echo ""
  echo "=== $1 ==="
  shift
  "$@" || exit $?
}

run materialize_benchmark_v03 python3 scripts/materialize_benchmark_v03.py --patch-grid-001-002-only
run build_v03_annotation_pack python3 scripts/build_v03_annotation_pack.py

run cargo_benchmark_manifest cargo run -p cta_cli -- benchmark manifest --version v0.3
run cargo_validate_benchmark cargo run -p cta_cli -- validate benchmark --version v0.3 --release
run cargo_benchmark_lint cargo run -p cta_cli -- benchmark lint --version v0.3 --release

run build_benchmark_manifest_jsonl python3 scripts/build_benchmark_manifest_jsonl.py --benchmark-version v0.3
run validate_benchmark python3 scripts/validate_benchmark.py
run export_benchmark_stats python3 scripts/export_benchmark_stats.py
run dump_prompts_appendix python3 scripts/dump_prompts_appendix.py

run materialize_v03_adjudication_artifacts python3 scripts/materialize_v03_adjudication_artifacts.py
run materialize_repair_hotspot_artifacts python3 scripts/materialize_repair_hotspot_artifacts.py
run reproduce_agreement_report python3 scripts/reproduce_agreement_report.py
run implement_evidence_hardening python3 scripts/implement_evidence_hardening.py
run repair_counterfactual_metrics python3 scripts/repair_counterfactual_metrics.py
run validate_release_artifact python3 scripts/validate_release_artifact.py
run ci_reviewer_readiness python3 scripts/ci_reviewer_readiness.py
run compute_human_strict_agreement python3 scripts/compute_human_strict_agreement.py \
  --packet-map annotation/human_pass_v3/human_strict_packet_ids.csv \
  --rater-a annotation/rater_a_strict_all.csv \
  --rater-b annotation/human_pass_v3/rater_b_human_strict_all.csv \
  --out-json annotation/human_pass_v3/agreement_report_human_strict_all.json \
  --out-md annotation/human_pass_v3/agreement_report_human_strict_all.md \
  --out-disagreements annotation/human_pass_v3/disagreement_log_strict_all.csv
run check_paper_claim_sources python3 scripts/check_paper_claim_sources.py

if [[ "${SKIP_EXPORT_FINAL:-}" != "1" ]]; then
  run export_final_ci_evidence python3 scripts/export_final_ci_evidence.py
fi

echo ""
echo "Paper readiness gate: OK"
