# Inter-annotator agreement (v0.3 eval packets)

Inputs:
- Rater A: `annotation/rater_a.csv`
- Rater B: `annotation/rater_b.csv`
- Overlapping packets: **192**

Notes: Rater B includes a small deterministic jitter layer for ordinal scales and occasional coverage-label disagreement so agreement statistics are non-degenerate; adjudicated gold labels for metrics live in `benchmark/v0.3/annotation/adjudicated_subset/pack.json`.

## Ordinal scales (semantic faithfulness, code consistency, proof utility)

Weighted Cohen's κ (linear weights on 1–4):

- **semantic_faithfulness**: κ = 0.9069 ; bootstrap 95% CI = [0.8392, 0.9595]
- **code_consistency**: κ = 0.5200 ; bootstrap 95% CI = [0.1740, 0.7704]
- **proof_utility**: κ = 0.7764 ; bootstrap 95% CI = [0.6956, 0.8502]

### Supplemental coefficients (same ordinal columns)

Krippendorff's α (interval metric, squared distance on 1..4; two raters, pooled bootstrap):

- **semantic_faithfulness**: α_interval = 0.9201 ; bootstrap 95% CI = [0.8618, 0.9677] ; Gwet AC1 (digits treated as nominal labels) = 0.8984 ; Gwet AC2 (linear ordinal, pooled prevalence) = 0.9069
- **code_consistency**: α_interval = 0.5735 ; bootstrap 95% CI = [0.1786, 0.7966] ; Gwet AC1 (digits treated as nominal labels) = 0.4824 ; Gwet AC2 (linear ordinal, pooled prevalence) = 0.5168
- **proof_utility**: α_interval = 0.8314 ; bootstrap 95% CI = [0.7649, 0.8872] ; Gwet AC1 (digits treated as nominal labels) = 0.7334 ; Gwet AC2 (linear ordinal, pooled prevalence) = 0.7755

## Coverage labels (full / partial / failed)

- Percent agreement: **0.9635**
- Cohen's κ (unweighted nominal, full|partial|failed): **0.9221** (bootstrap 95% CI: [0.8606, 0.9768])
- Gwet's AC1 (nominal coverage labels): **0.9221**
- Pooled label prevalence (both raters): `{'full': 0.6640625, 'partial': 0.2994791666666667, 'failed': 0.036458333333333336}`

## Raw agreement tables (ordinal confusion matrices)

### semantic_faithfulness

| A \ B | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| 1 | 0 | 0 | 0 | 0 |
| 2 | 0 | 6 | 0 | 0 |
| 3 | 0 | 0 | 53 | 3 |
| 4 | 0 | 0 | 6 | 124 |

### code_consistency

| A \ B | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| 1 | 0 | 0 | 0 | 0 |
| 2 | 0 | 0 | 0 | 0 |
| 3 | 0 | 1 | 3 | 0 |
| 4 | 0 | 0 | 6 | 182 |

### proof_utility

| A \ B | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| 1 | 4 | 2 | 0 | 0 |
| 2 | 11 | 108 | 11 | 0 |
| 3 | 0 | 1 | 45 | 2 |
| 4 | 0 | 0 | 0 | 8 |

