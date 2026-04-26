# Artifact card (anonymous submission)

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
