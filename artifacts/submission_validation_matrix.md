# Submission validation matrix (NeurIPS 2026 E&D)

Final gate checklist before uploading code/data materials with the paper. Commands assume repository root unless noted.

| Gate | Command | Must pass? | Expected output / evidence |
|------|---------|------------|----------------------------|
| Rust build | `cargo build --workspace` | yes | Successful build log |
| Rust fmt | `cargo fmt --all -- --check` | yes | No diff |
| Rust clippy | `cargo clippy --workspace --all-targets --no-deps` | yes | Clean |
| Rust tests | `cargo test --workspace --all-targets` | yes | All tests pass |
| Rust docs | `cargo test --workspace --doc` | yes | Doctest pass |
| Benchmark manifest | `cargo run -p cta_cli -- benchmark manifest --version v0.3` | yes | Manifest emitted |
| Benchmark release validate | `cargo run -p cta_cli -- validate benchmark --version v0.3 --release` | yes | Validation pass |
| Benchmark lint | `cargo run -p cta_cli -- benchmark lint --version v0.3 --release` | yes | Lint pass |
| Lean build | `cd lean && lake build` | yes | Mathlib-linked build succeeds |
| Full paper pipeline | Exact ordered steps in `docs/PAPER_READINESS.md` (§2) | yes | Regenerated `results/paper_*`, annotations, repairs exports |
| Results paper mode | `python scripts/compute_results.py --paper` | yes | `results/paper_strict_*`, evidence CSVs |
| Human strict agreement | `python scripts/compute_human_strict_agreement.py --packet-map annotation/human_pass_v3/human_strict_packet_ids.csv --rater-a annotation/rater_a_strict_all.csv --rater-b annotation/human_pass_v3/rater_b_human_strict_all.csv --out-json annotation/human_pass_v3/agreement_report_human_strict_all.json --out-md annotation/human_pass_v3/agreement_report_human_strict_all.md --out-disagreements annotation/human_pass_v3/disagreement_log_strict_all.csv` | yes | JSON + MD + disagreement CSV |
| Evidence hardening | `python scripts/implement_evidence_hardening.py` | yes | `artifacts/evidence_hardening_manifest.json` + auxiliary results |
| Release validator | `python scripts/validate_release_artifact.py` | yes | Validator pass |
| Reviewer readiness | `python scripts/ci_reviewer_readiness.py` | yes | CI-parity checks |
| Claim-source discipline | `python scripts/check_paper_claim_sources.py` | yes | Strict vs expanded assertions |
| Final CI evidence log | `python scripts/export_final_ci_evidence.py` | recommended | `artifacts/final_ci_run_YYYYMMDD.md` |
| Anonymous zip | `.\scripts\build_anonymous_artifact.ps1` (Windows) | yes for blind packaging | `artifacts/cta-benchmark-anonymous.zip` |
| Anonymity scan | `.\scripts\scan_submission_anonymity.ps1 -ExtractPath artifacts\_anon_scan` | yes | Zero author / credential leaks |

**Credential posture:** Table reproduction for committed adjudicated artifacts must **not** require provider keys; live HTTP only when regenerating from providers.

## Main manuscript tables (CSV sources)

Regenerate via `python scripts/compute_results.py --paper` (and upstream materializers per `docs/PAPER_READINESS.md`). Before submission, every numeric cell in LaTeX must resolve to one of these paths:

| Main table | Primary CSV / JSON sources |
|------------|----------------------------|
| 1 — Benchmark inventory | `results/table1_benchmark_overview.csv`, `results/table1_family_semantic_load.csv` |
| 2 — Evidence views | `results/paper_table_annotation_evidence.csv`, `results/paper_annotation_origin_counts.csv` |
| 3 — Human agreement | `annotation/human_pass_v3/agreement_report_human_strict_all.json`, `results/paper_table_agreement_evidence.csv` |
| 4 — System comparison | `results/paper_strict_system_summary.csv`, `results/paper_strict_system_metrics_long.csv` |
| 5 — Failure modes | `results/paper_strict_failure_modes.csv` |
| 6 — Repair / proof-facing | `repairs/paper_repair_status.csv`, `repairs/paper_repair_success_subset.csv`, `repairs/paper_proof_facing_subset.csv`, `repairs/repair_attempts.csv` (also `repairs/paper_repair_proof_subset.csv` where used) |

Strict headline metrics: **`results/raw_metrics_strict.json`** and **`results/paper_strict_*`** only.
