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

- **semantic_faithfulness**: κ = 0.9292 ; bootstrap 95% CI = [0.8771, 0.9738]
- **code_consistency**: κ = 0.7284 ; bootstrap 95% CI = [0.4559, 0.8979]
- **proof_utility**: κ = 0.7864 ; bootstrap 95% CI = [0.7073, 0.8570]

### Supplemental coefficients (same ordinal columns)

Krippendorff's α (interval metric, squared distance on 1..4; two raters, pooled bootstrap):

- **semantic_faithfulness**: α_interval = 0.9475 ; bootstrap 95% CI = [0.9014, 0.9802] ; Gwet AC1 (digits treated as nominal labels) = 0.9166 ; Gwet AC2 (linear ordinal, pooled prevalence) = 0.9292
- **code_consistency**: α_interval = 0.8006 ; bootstrap 95% CI = [0.5192, 0.9313] ; Gwet AC1 (digits treated as nominal labels) = 0.6660 ; Gwet AC2 (linear ordinal, pooled prevalence) = 0.7274
- **proof_utility**: α_interval = 0.8420 ; bootstrap 95% CI = [0.7723, 0.8959] ; Gwet AC1 (digits treated as nominal labels) = 0.7425 ; Gwet AC2 (linear ordinal, pooled prevalence) = 0.7857

## Coverage labels (full / partial / failed)

- Percent agreement: **0.9635**
- Cohen's κ (unweighted nominal, full|partial|failed): **0.9277** (bootstrap 95% CI: [0.8697, 0.9714])
- Gwet's AC1 (nominal coverage labels): **0.9276**
- Pooled label prevalence (both raters): `{'full': 0.625, 'partial': 0.3203125, 'failed': 0.0546875}`

## Vacuity labels

- Cohen's κ (unweighted nominal): **0.0000** (bootstrap 95% CI: [0.0000, 0.0000])
- Percent agreement: **0.9583**

## Raw agreement tables (ordinal confusion matrices)

### semantic_faithfulness

| A \ B | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| 1 | 2 | 0 | 0 | 0 |
| 2 | 0 | 8 | 0 | 0 |
| 3 | 0 | 1 | 56 | 3 |
| 4 | 0 | 0 | 4 | 118 |

### code_consistency

| A \ B | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| 1 | 0 | 0 | 0 | 0 |
| 2 | 0 | 2 | 0 | 0 |
| 3 | 0 | 1 | 5 | 0 |
| 4 | 0 | 0 | 6 | 178 |

### proof_utility

| A \ B | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| 1 | 7 | 3 | 0 | 0 |
| 2 | 11 | 105 | 10 | 0 |
| 3 | 0 | 1 | 45 | 2 |
| 4 | 0 | 0 | 0 | 8 |

