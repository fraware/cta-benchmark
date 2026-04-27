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

After both passes exist (or after the repo materializer has written
`annotation/rater_a.csv` and `annotation/rater_b.csv` from adjudicated packets),
run the reproducible wrapper:

```powershell
python scripts/reproduce_agreement_report.py
```

Equivalent direct invocation (human second pass):

```powershell
python scripts/compute_agreement_stats.py --first annotation/rater_a.csv --second annotation/rater_b_human.csv
```

Fallback (synthetic second pass):

```powershell
python scripts/compute_agreement_stats.py --first annotation/rater_a.csv --second annotation/rater_b.csv
```

This writes `annotation/agreement_report.json` (weighted κ, bootstrap CI) and
`annotation/agreement_raw_table.csv`, and refreshes `annotation/agreement_report.md`.

**Audit trail (reviewer-facing):** `annotation/agreement_packet_ids.csv` lists the
exact packet population (including anonymized keys joined to raters);
`annotation/adjudication_log.csv` records adjudication outcomes. Both are
written by `python scripts/materialize_v03_adjudication_artifacts.py` when using
the pipeline-derived pack.

**v0.3 pipeline note:** the materialized audit list is the **full eval grid**
(four systems × eval instances). Those instances usually load canonical
`*_001`/`*_002` packets, so `annotation_origin` in `agreement_packet_ids.csv` is
often entirely `mapped_from_canonical`. That is distinct from headline **eval
metrics**, which (under `compute_results.py --paper`) pool **strict**
independent rows from `results/raw_metrics_strict.json`. See
`results/paper_table_agreement_evidence.csv`,
`results/paper_table_annotation_evidence.csv`, and
[`docs/paper/system_scope.md`](../docs/paper/system_scope.md).

## Rubric anchor

All dimensions are defined in `annotation/RUBRIC.md`. Vacuity rate uses the
packet’s benchmark-facing obligation multiset as denominator (see rubric).
