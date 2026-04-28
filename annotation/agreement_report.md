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

- **semantic_faithfulness**: κ = 0.9115 ; bootstrap 95% CI = [0.8496, 0.9650]
- **code_consistency**: κ = 0.6164 ; bootstrap 95% CI = [0.3228, 0.8281]
- **proof_utility**: κ = 0.7803 ; bootstrap 95% CI = [0.7017, 0.8530]

### Supplemental coefficients (same ordinal columns)

Krippendorff's α (interval metric, squared distance on 1..4; two raters, pooled bootstrap):

- **semantic_faithfulness**: α_interval = 0.9267 ; bootstrap 95% CI = [0.8681, 0.9682] ; Gwet AC1 (digits treated as nominal labels) = 0.9012 ; Gwet AC2 (linear ordinal, pooled prevalence) = 0.9115
- **code_consistency**: α_interval = 0.6510 ; bootstrap 95% CI = [0.3230, 0.8517] ; Gwet AC1 (digits treated as nominal labels) = 0.5930 ; Gwet AC2 (linear ordinal, pooled prevalence) = 0.6144
- **proof_utility**: α_interval = 0.8359 ; bootstrap 95% CI = [0.7683, 0.8912] ; Gwet AC1 (digits treated as nominal labels) = 0.7369 ; Gwet AC2 (linear ordinal, pooled prevalence) = 0.7796

## Coverage labels (full / partial / failed)

- Percent agreement: **0.9635**
- Cohen's κ (unweighted nominal, full|partial|failed): **0.9233** (bootstrap 95% CI: [0.8631, 0.9703])
- Gwet's AC1 (nominal coverage labels): **0.9233**
- Pooled label prevalence (both raters): `{'full': 0.6536458333333334, 'partial': 0.3098958333333333, 'failed': 0.036458333333333336}`

## Vacuity labels

- Cohen's κ (unweighted nominal): **0.0000** (bootstrap 95% CI: [0.0000, 0.0000])
- Percent agreement: **0.9583**

## Raw agreement tables (ordinal confusion matrices)

### semantic_faithfulness

| A \ B | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| 1 | 0 | 0 | 0 | 0 |
| 2 | 0 | 8 | 0 | 0 |
| 3 | 0 | 0 | 53 | 3 |
| 4 | 0 | 0 | 6 | 122 |

### code_consistency

| A \ B | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| 1 | 0 | 0 | 0 | 0 |
| 2 | 0 | 0 | 0 | 0 |
| 3 | 0 | 1 | 5 | 0 |
| 4 | 0 | 0 | 6 | 180 |

### proof_utility

| A \ B | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| 1 | 5 | 3 | 0 | 0 |
| 2 | 11 | 107 | 10 | 0 |
| 3 | 0 | 1 | 45 | 2 |
| 4 | 0 | 0 | 0 | 8 |

