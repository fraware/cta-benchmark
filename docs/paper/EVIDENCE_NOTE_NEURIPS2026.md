# Evidence discipline (one page) — NeurIPS 2026

## Headline vs appendix

| Layer | Rows | Unique instances | Mapped-from-canonical | Role |
|-------|------|------------------|------------------------|------|
| Strict direct | 274 | 84 | 0 | **Abstract, intro, results, conclusion** |
| Expanded mapped | 336 | 84 | 114 | **Appendix / robustness only** |

Authoritative counts: `results/paper_table_annotation_evidence.csv` and `results/paper_annotation_origin_counts.csv`. Per-row metrics: `results/raw_metrics_strict.json` (strict) vs `results/raw_metrics_expanded.json` (expanded).

## Mandatory headline files (manuscript numerics)

- `results/raw_metrics_strict.json`
- `results/paper_strict_system_summary.csv`
- `results/paper_strict_family_summary.csv`
- `results/paper_strict_failure_modes.csv`
- `results/paper_strict_instance_level.csv`
- `results/paper_strict_system_metrics_long.csv`
- `results/paper_table_annotation_evidence.csv`
- `results/paper_table_agreement_evidence.csv`
- `results/paper_annotation_origin_counts.csv`

Manuscript LaTeX construction (Tables 1–6, composite posture): [`MANUSCRIPT_TABLE_SOURCES_NEURIPS2026.md`](MANUSCRIPT_TABLE_SOURCES_NEURIPS2026.md).

## Appendix-only (never headline without labeling)

- `results/paper_expanded_system_summary.csv`
- `results/paper_expanded_family_summary.csv`
- `results/paper_expanded_failure_modes.csv`
- `results/appendix_mapped_evidence/`

## Engineering gate

Run `python scripts/check_paper_claim_sources.py` after `python scripts/ci_reviewer_readiness.py` (and after `python scripts/compute_human_strict_agreement.py …` if you are refreshing agreement artifacts outside `implement_evidence_hardening.py`). The checker asserts headline CSVs/JSON, **`results/appendix_mapped_evidence/`** presence, strict human-agreement file paths, and `docs/paper/paper_claim_sources.yaml`. Claim tiers: `docs/paper/CLAIM_LOCK_NEURIPS2026.md`.

## Human agreement (strict overlap)

Headline human-agreement claims cite **`annotation/human_pass_v3/agreement_report_human_strict_all.json`**, **`annotation/human_pass_v3/disagreement_log_strict_all.csv`**, and **`results/paper_table_agreement_evidence.csv`** (`strict_all_human_overlap`). Invariants: **274** rows, **84** instances, **0** mapped-from-canonical, ordinals in `{0,1,2,3}`. Regenerate with the exact command in `docs/PAPER_READINESS.md` and `docs/REVIEWER_MAP.md`.

**Approved wording:** “The strict headline view is independently double-annotated and adjudicated over **274** direct rows covering all **84** instances.” Do **not** imply the full **336**-row expanded grid is independently double-annotated in the same sense.

## Cross-model pilot (appendix)

`results/cross_model_pilot_*` and `results/cross_model_pilot_appendix_table.csv` support **diagnostic** wording only, for example: “A small cross-model sanity pilot shows the same qualitative split between semantic grounding and proof-facing structure.” Do **not** claim a robust cross-model ranking. The checked-in slice is derived from strict headline metrics on a **12-instance, one-per-family** selection.

The appendix table includes **primary** `code_only_v1` / `full_method_v1` slice rows plus **additional open conditioning baselines** (`naive_concat_v1`, `text_only_v1`) on the **same instances** where those systems exist in `results/raw_metrics_strict.json` (`text_only_v1` is absent on two pilot IDs; means use **10** instances—see row notes). Configuration: `configs/cross_model_pilot.json`.

**Optional external vendors:** Append summary rows (same CSV columns) via `results/cross_model_pilot_external_appendix.json` (`rows` array); rerun `python scripts/implement_evidence_hardening.py`. Keep appendix-only, non-leaderboard language (`docs/paper/CLAIM_LOCK_NEURIPS2026.md` Tier B).

## Composite reliability

Treat **`results/system_reliability_summary.csv`** / **`results/paper_strict_*`** composite columns as a **secondary diagnostic** for multi-axis failure. **Primary** claims use semantic faithfulness, coverage, code consistency, vacuity, proof utility, and failure-mode tables **separately**. See `docs/evaluation_contract.md` and `docs/LIMITATIONS.md`.
