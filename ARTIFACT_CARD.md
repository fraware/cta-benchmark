# Artifact card (anonymous submission)

## Reviewer quick path

- **Paper framing:** benchmark + protocol for **semantic faithfulness** of **Lean-facing obligations** (not full Rust verification or general autoformalization).
- **Headline metrics:** strict layer — **84** instances, **12** families, **274** direct rows, **0** mapped-from-canonical rows (`results/raw_metrics_strict.json`, `results/paper_strict_*`).
- **Appendix:** expanded **336**-row mapped grid (`results/paper_expanded_*`) — robustness only.
- **Regenerate tables:** `docs/PAPER_READINESS.md` §2; one-shot `scripts/run_paper_readiness_gate.ps1` (Windows) or `scripts/run_paper_readiness_gate.sh` (Unix); claim discipline: `python scripts/check_paper_claim_sources.py`.
- **Pre-upload parity (no full regeneration):** `scripts/verify_submission_readiness.ps1` or `bash scripts/verify_submission_readiness.sh`.
- **Submission checklist:** `artifacts/submission_validation_matrix.md`.

| Field | Value |
|------|-------|
| Name | CTA-Benchmark (code-to-annotation benchmark) |
| Version | v0.3 canonical manifest at `benchmark/manifest.jsonl` |
| License | MIT (see `LICENSE`) |
| Benchmark instances | 84 (see `results/table1_benchmark_overview.csv`) |
| Primary data | `benchmark/v0.3/instances/**` (JSON + Rust + Lean scaffolds) |
| Evaluation software | Rust workspace (`cargo`) + Lean 4 (`lake`) |
| Randomness | Provider sampling uses fixed seeds listed per experiment JSON |
| Security / PII | No user data; prompts contain only benchmark statements |
| Citation | Use repository URL and commit hash recorded in per-run manifests |

## Anonymous packaging

1. Remove contributor-identifying paths if required by venue policy.
2. Strip git remotes or ship a `.zip` without `.git` if double-blind.
3. Keep `benchmark/manifest.jsonl` and `appendix/PROMPTS_APPENDIX.md` together.

**Automated bundle:** run `scripts/build_anonymous_artifact.ps1` to produce
`artifacts/cta-benchmark-anonymous.zip` (see `artifacts/README.md`).

## Maintenance

Regenerate manifests and tables with `scripts/experiment_setup.ps1` (Windows)
or the exact command list in `docs/PAPER_READINESS.md` (also mirrored in
`REPRODUCE.md`).
