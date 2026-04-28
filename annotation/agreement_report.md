# Inter-annotator agreement (v0.3 eval packets)

Inputs:
- Join key: **`anonymized_packet_key`**
- Rater A: `annotation/rater_a.csv`
- Rater B: `annotation/rater_b.csv`
- Audit mapping: `annotation/agreement_packet_ids.csv`
- Overlapping packets: **192**
- Row-count breakdown (eval audit vs strict subset): `results/paper_table_agreement_evidence.csv`
- Population note: Audit rows are eval-split (instance, system) pairs; canonical template packets yield mapped_from_canonical unless instance_id equals the template stem.

Notes: Rater-B provenance tier is `synthetic_inter_rater`; adjudicated gold labels for metrics live in `benchmark/v0.3/annotation/adjudicated_subset/pack.json`.

## Ordinal scales (semantic faithfulness, code consistency, proof utility)

Weighted Cohen's κ (linear weights on 1–4):

- **semantic_faithfulness**: κ = 0.9341 ; bootstrap 95% CI = [0.8842, 0.9756]
- **code_consistency**: κ = 0.7887 ; bootstrap 95% CI = [0.5766, 0.9195]
- **proof_utility**: κ = 0.7898 ; bootstrap 95% CI = [0.7123, 0.8599]

### Supplemental coefficients (same ordinal columns)

Krippendorff's α (interval metric, squared distance on 1..4; two raters, pooled bootstrap):

- **semantic_faithfulness**: α_interval = 0.9549 ; bootstrap 95% CI = [0.9147, 0.9833] ; Gwet AC1 (digits treated as nominal labels) = 0.9187 ; Gwet AC2 (linear ordinal, pooled prevalence) = 0.9341
- **code_consistency**: α_interval = 0.8595 ; bootstrap 95% CI = [0.6635, 0.9542] ; Gwet AC1 (digits treated as nominal labels) = 0.7159 ; Gwet AC2 (linear ordinal, pooled prevalence) = 0.7881
- **proof_utility**: α_interval = 0.8459 ; bootstrap 95% CI = [0.7790, 0.8988] ; Gwet AC1 (digits treated as nominal labels) = 0.7455 ; Gwet AC2 (linear ordinal, pooled prevalence) = 0.7893

## Coverage labels (full / partial / failed)

- Percent agreement: **0.9635**
- Cohen's κ (unweighted nominal, full|partial|failed): **0.9308** (bootstrap 95% CI: [0.8765, 0.9726])
- Gwet's AC1 (nominal coverage labels): **0.9308**
- Pooled label prevalence (both raters): `{'full': 0.59375, 'partial': 0.3411458333333333, 'failed': 0.06510416666666667}`

## Vacuity labels

- Cohen's κ (unweighted nominal): **0.0000** (bootstrap 95% CI: [0.0000, 0.0000])
- Percent agreement: **0.9583**

## Raw agreement tables (ordinal confusion matrices)

### semantic_faithfulness

| A \ B | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| 1 | 4 | 0 | 0 | 0 |
| 2 | 0 | 8 | 0 | 0 |
| 3 | 0 | 1 | 56 | 3 |
| 4 | 0 | 0 | 4 | 116 |

### code_consistency

| A \ B | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| 1 | 0 | 0 | 0 | 0 |
| 2 | 0 | 4 | 0 | 0 |
| 3 | 0 | 1 | 5 | 0 |
| 4 | 0 | 0 | 6 | 176 |

### proof_utility

| A \ B | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| 1 | 8 | 4 | 0 | 0 |
| 2 | 11 | 104 | 9 | 0 |
| 3 | 0 | 1 | 45 | 2 |
| 4 | 0 | 0 | 0 | 8 |

