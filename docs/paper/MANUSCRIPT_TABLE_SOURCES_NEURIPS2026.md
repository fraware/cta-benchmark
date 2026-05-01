# Manuscript table sources (NeurIPS 2026) — CSV-to-LaTeX checklist

Use this when building the **main** and **appendix** tables in the camera-ready
paper. Regenerate all CSVs with the canonical pipeline (`docs/PAPER_READINESS.md`
§2 or `scripts/run_paper_readiness_gate.*`) before copying numbers into LaTeX.

**Global rules**

- **Headline body / abstract:** every numeric claim must resolve to
  `results/raw_metrics_strict.json` and/or `results/paper_strict_*` and/or the
  strict row of `results/paper_table_annotation_evidence.csv`. Do **not** cite
  `results/raw_metrics_expanded.json` or `results/paper_expanded_*` in the
  main-text story without an explicit **appendix / robustness** label.
- **Mapped-from-canonical (114 rows in expanded view):** never described as
  independent duplicate-adjudication evidence. See
  `docs/paper/CLAIM_LOCK_NEURIPS2026.md` and
  `docs/paper/EVIDENCE_NOTE_NEURIPS2026.md`.
- **Composite reliability:** **secondary diagnostic** only. Lead with component
  metrics (faithfulness, consistency, vacuity, proof utility, missing critical
  units / coverage); sensitivity lives in appendix (`results/system_reliability_sensitivity.csv`, strict paper exports).
- **Naming:** in prose use **code-grounded** for regime `code_only_v1` (see
  `docs/paper/CLAIM_LOCK_NEURIPS2026.md`).
- **Verifier:** `python scripts/check_paper_claim_sources.py` and
  `docs/paper/paper_claim_sources.yaml`.

---

## Main Table 1 — Benchmark inventory

| Requirement | Source files |
|-------------|----------------|
| 84 instances; 12 families; **294** critical semantic units; seven instances per family; dev/eval split | `results/table1_benchmark_overview.csv`, `results/table1_family_semantic_load.csv` |

Re-read totals after `python scripts/export_benchmark_stats.py` (inside full gate).

---

## Main Table 2 — Evidence views

| View | Rows | Unique instances | Direct | Mapped | Role |
|------|------|------------------|--------|--------|------|
| strict direct | **274** | **84** | **274** | **0** | headline |
| expanded grid | **336** | **84** | **222** (refresh from CSV) | **114** | appendix robustness |

**Sources:** `results/paper_table_annotation_evidence.csv`,
`results/paper_annotation_origin_counts.csv`.

Do **not** hard-code 222/274/336 in LaTeX if regeneration changes counts;
paste from regenerated CSV or cite generated artifact paths.

---

## Main Table 3 — Human agreement

**Sources:** `annotation/human_pass_v3/agreement_report_human_strict_all.json`,
`results/paper_table_agreement_evidence.csv`.

Report ordinal agreement for semantic faithfulness, code consistency, proof
utility; include **coverage** and **vacuity** caveats from the same strict-overlap
layer (see agreement MD / disagreement log).

**Approved sentence:** “The strict headline view is independently double-annotated and adjudicated over **274** direct rows covering all **84** instances.”

---

## Main Table 4 — System comparison

**Sources:** `results/paper_strict_system_summary.csv`,
`results/paper_strict_system_metrics_long.csv`.

**Column order (recommended):** semantic faithfulness, code consistency,
vacuity, proof utility, critical-unit missing rate (or equivalent coverage proxy),
then **reliability composite last** (labeled secondary diagnostic).

---

## Main Table 5 — Failure modes

**Source:** `results/paper_strict_failure_modes.csv`.

Include low semantic faithfulness (`low_semantic_faithfulness`),
missing critical semantic unit (`missing_critical_semantic_unit` from strict rows with `missing_critical_units > 0`),
contradiction where exported, vacuity-related concentrations, system/family splits.

---

## Main Table 6 — Repair vs proof-facing

**Sources:** `repairs/paper_repair_status.csv`,
`repairs/paper_repair_success_subset.csv`,
`repairs/paper_proof_facing_subset.csv`,
`repairs/repair_attempts.csv` (and `repairs/paper_repair_proof_subset.csv` where used).

Clearly separate: **selected repair budget**, **attempted repairs**, **successful repair examples**, vs **Lean-elaborated proof-facing subset**. Do not treat proof-facing elaboration as universal repair success.

---

## Appendix-only tables

Expanded / mapped robustness: `results/paper_expanded_system_summary.csv`,
`results/paper_expanded_family_summary.csv`,
`results/paper_expanded_failure_modes.csv`,
`results/appendix_mapped_evidence/`.

Cross-model diagnostic slice: `results/cross_model_pilot_appendix_table.csv`,
`results/cross_model_pilot_*`, `configs/cross_model_pilot.json`, optional
`results/cross_model_pilot_external_appendix.json` (non-leaderboard wording only).

LaTeX guard: `python scripts/check_paper_claim_sources.py --scan-tex --tex-path …`
(see `docs/PAPER_READINESS.md`).
