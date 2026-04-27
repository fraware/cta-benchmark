# System scope for the manuscript

The repository’s **primary** paper configuration is a **four-system** study:
`text_only_v1`, `code_only_v1`, `naive_concat_v1`, and `full_method_v1` (see
`benchmark/v0.3/benchmark_paper_summary.json` field `paper_systems_ordered`).

## Headline policy

The repo contract is **`paper_headline_policy: four_system_primary_study`**:
all four systems appear in headline
`results/paper_strict_*` files produced under `compute_results.py --paper`
(legacy compatibility exports still exist under `results/paper_table_*.csv`).

## Evidence view (strict vs expanded)

Headline statistical tables are computed from **`raw_metrics_strict.json`**
(independent adjudication / direct human rows only). The expanded mapped view
is promoted to **`results/paper_expanded_*`** and also kept under
**`results/appendix_mapped_evidence/`** for appendix/sensitivity checks.
Row-count transparency is in **`results/paper_table_annotation_evidence.csv`**
and **`results/paper_annotation_origin_counts.csv`**.

## Agreement κ population vs headline eval metrics

Inter-rater agreement is keyed off **`annotation/agreement_packet_ids.csv`**
(see **`results/paper_table_agreement_evidence.csv`**). That population is the
**eval grid** (four systems × eval instances). Eval instances are variant stems
(typically ``*_004``–``*_007``) that usually **map** from ``*_001`` / ``*_002``
review packets, so every audit row can legitimately be
``mapped_from_canonical`` while headline tables still pool **strict**
`(instance, system)` rows from **`raw_metrics_strict.json`** (which includes
direct rows on dev instances and any eval pair where the template equals the
instance id). The paper should name both populations when interpreting κ versus
mean faithfulness.
