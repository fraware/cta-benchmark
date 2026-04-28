# Human Independent Agreement Report (v2)

- Rows audited: **192**
- Annotator qualifications: Independent software engineer with theorem-proving annotation training (anonymized).
- Sampling: full strict direct-adjudication overlap from audit queue.

## Pre-adjudication Agreement
- semantic_faithfulness: 0.9583
- code_consistency: 0.9635
- proof_utility: 0.8594
- coverage_label: 0.9635
- vacuity_label: 0.9583

## Adjudication Procedure
Two-pass adjudication: disagreements logged, then resolved against source packet evidence and rubric.

## Confusion Matrices

### semantic_faithfulness

| rater_a \\ rater_b | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| 1 | 4 | 0 | 0 | 0 |
| 2 | 0 | 8 | 0 | 0 |
| 3 | 0 | 1 | 56 | 3 |
| 4 | 0 | 0 | 4 | 116 |

### code_consistency

| rater_a \\ rater_b | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| 1 | 0 | 0 | 0 | 0 |
| 2 | 0 | 4 | 0 | 0 |
| 3 | 0 | 1 | 5 | 0 |
| 4 | 0 | 0 | 6 | 176 |

### proof_utility

| rater_a \\ rater_b | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- |
| 1 | 8 | 4 | 0 | 0 |
| 2 | 11 | 104 | 9 | 0 |
| 3 | 0 | 1 | 45 | 2 |
| 4 | 0 | 0 | 0 | 8 |

### coverage_label

| rater_a \\ rater_b | full | partial | failed |
| --- | --- | --- | --- |
| full | 112 | 4 | 0 |
| partial | 0 | 62 | 2 |
| failed | 0 | 1 | 11 |

### vacuity_label

| rater_a \\ rater_b | non_vacuous | vacuous |
| --- | --- | --- |
| non_vacuous | 184 | 8 |
| vacuous | 0 | 0 |

## Disagreement Examples and Resolutions
- ag_008 proof_utility: A=2, B=1, resolved=2
- ag_010 semantic_faithfulness: A=4, B=3, resolved=4
- ag_012 proof_utility: A=1, B=2, resolved=1
- ag_013 proof_utility: A=2, B=1, resolved=2
- ag_024 coverage_label: A=full, B=partial, resolved=full
- ag_027 proof_utility: A=2, B=3, resolved=2
- ag_032 proof_utility: A=2, B=1, resolved=2
- ag_043 proof_utility: A=3, B=4, resolved=3
- ag_049 proof_utility: A=2, B=3, resolved=2
- ag_051 code_consistency: A=4, B=3, resolved=4
