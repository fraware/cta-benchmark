# Double annotation protocol (v0.3+)

This protocol satisfies paper requirements for **independent first passes**,
**adjudication**, and **agreement reporting** on human judgments, with
strict-headline evidence tied to the v3 strict-overlap set.

## Sampling

- Strict headline target: annotate **all strict rows** from
  `annotation/external_review/strict_review_queue.jsonl` and maintain
  deterministic anonymized packet keys in
  `annotation/human_pass_v3/human_strict_packet_ids.csv`.
- Expected strict overlap counters:
  - `n_rows = 274`
  - `n_unique_instance_ids = 84`
  - `n_mapped_from_canonical = 0`

## Blinding

- Present packets to annotators as `system_blind_A`, `system_blind_B`, … with
  a **random permutation** of real `system_id` values per packet.
- Store only the mapping in `annotation/adjudication_log.csv` (or a
  restricted-access sheet), not in annotator-facing CSV exports.

## Passes

1. **Pass 1:** Annotator A completes `annotator_packet_sheet_template.csv`
   rows (ordinal scores + vacuity flags).
2. **Pass 2:** Annotator B completes the same rows **without** seeing A’s
   scores.
3. **Adjudication:** For any ordinal dimension where \(|A-B| \geq 2\), or any
   vacuity disagreement, adjudicator resolves and records rationale in
  `adjudication_log.csv`.

## Agreement outputs

After both passes exist for strict-overlap (`annotation/rater_a_strict_all.csv`
and `annotation/human_pass_v3/rater_b_human_strict_all.csv`), run:

```powershell
python scripts/compute_human_strict_agreement.py `
  --packet-map annotation/human_pass_v3/human_strict_packet_ids.csv `
  --rater-a annotation/rater_a_strict_all.csv `
  --rater-b annotation/human_pass_v3/rater_b_human_strict_all.csv `
  --out-json annotation/human_pass_v3/agreement_report_human_strict_all.json `
  --out-md annotation/human_pass_v3/agreement_report_human_strict_all.md `
  --out-disagreements annotation/human_pass_v3/disagreement_log_strict_all.csv
```

This writes strict-overlap agreement/disagreement artifacts under
`annotation/human_pass_v3/` and should be followed by:

```powershell
python scripts/compute_results.py --paper
python scripts/export_benchmark_paper_summary.py
python scripts/implement_evidence_hardening.py
python scripts/ci_reviewer_readiness.py
```

**Audit trail (reviewer-facing):** `annotation/agreement_packet_ids.csv` lists the
exact packet population (including anonymized keys joined to raters);
`annotation/adjudication_log.csv` records adjudication outcomes. Both are
written by `python scripts/materialize_v03_adjudication_artifacts.py` when using
the pipeline-derived pack.

**v0.3 pipeline note:** `annotation/agreement_packet_ids.csv` remains the legacy
full-audit population view, while strict headline claims use the independently
double-annotated strict overlap represented by
`results/paper_table_agreement_evidence.csv` row
`agreement_subset == strict_all_human_overlap`.

## Rubric anchor

All dimensions are defined in `annotation/RUBRIC.md`. Vacuity rate uses the
packet’s benchmark-facing obligation multiset as denominator (see rubric).
