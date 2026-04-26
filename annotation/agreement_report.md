# Inter-annotator agreement (template)

Sampling: at least 25–30% of adjudicated packets receive two independent
first-pass annotations plus adjudication on disagreements.

## Ordinal scales (semantic faithfulness, code consistency, proof utility)

- Weighted Cohen’s κ (linear weights): **fill after annotation wave**
- Bootstrap 95% CI (10 000 resamples): **fill**

## Coverage labels (full / partial / failed)

- Percent agreement: **fill**
- Cohen’s κ (unweighted): **fill**

## Raw agreement table (example layout)

| Annotator A \\ Annotator B | 1 | 2 | 3 | 4 |
|----------------------------|---|---|---|---|
| 1 | n11 | n12 | n13 | n14 |
| 2 | n21 | n22 | n23 | n24 |
| 3 | n31 | n32 | n33 | n34 |
| 4 | n41 | n42 | n43 | n44 |

Populate `annotation/agreement_report.json` from the adjudication spreadsheet
export when scores are available, or run:

`python scripts/compute_agreement_stats.py --first annotation/rater_a.csv --second annotation/rater_b.csv`

Example inputs: `annotation/rater_a.example.csv` and `annotation/rater_b.example.csv`.
