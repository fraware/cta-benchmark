# Double annotation protocol (v0.3+)

This protocol satisfies paper requirements for **independent first passes**,
**adjudication**, and **agreement reporting** on human judgments.

## Sampling

- Draw **at least 25–30%** of eval-split packets (or adjudicated rows) into
  the double-annotation pool using a **deterministic seed** recorded in the
  adjudication log.
- Stratify by `family` so graph / DP / trees are not under-represented when
  those families dominate the benchmark design.

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

After both passes exist, run:

```powershell
python scripts/compute_agreement_stats.py --first annotation/rater_a.csv --second annotation/rater_b.csv
```

This writes `annotation/agreement_report.json` (weighted κ, bootstrap CI) and
`annotation/agreement_raw_table.csv`. Paste summary statistics into
`annotation/agreement_report.md` for the appendix.

## Rubric anchor

All dimensions are defined in `annotation/RUBRIC.md`. Vacuity rate uses the
packet’s benchmark-facing obligation multiset as denominator (see rubric).
