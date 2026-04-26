# System scope for the manuscript

The repository’s **primary** paper configuration is a **four-system** study:
`text_only_v1`, `code_only_v1`, `naive_concat_v1`, and `full_method_v1` (see
`benchmark/v0.3/benchmark_paper_summary.json` field `paper_systems_ordered`).

## Headline vs optional three-system scope

If the narrative should **not** treat `text_only_v1` as a first-class comparator:

1. State explicitly that `text_only_v1` is a **calibration / ablation** baseline.
2. Restrict **headline** tables and prose claims to the remaining three systems
   while keeping `text_only_v1` in **appendix** or robustness exports (filter
   columns or regenerate summaries with a three-system manifest override in
   analysis, not by silently dropping rows in the committed pack).
3. Keep `paper_systems_ordered` in the JSON for reproducibility, and add a
   sentence in the methods section naming the excluded system and the reason.

The inverse default (this repo’s contract) is **`paper_headline_policy`:
`four_system_primary_study`**: all four systems appear in headline
`paper_table_*.csv` files produced under `compute_results.py --paper`.

## Evidence view (strict vs expanded)

Headline statistical tables are computed from **`raw_metrics_strict.json`**
(independent adjudication / direct human rows only). The expanded mapped view
lives under **`results/appendix_mapped_evidence/`** for appendix or
sensitivity checks. Row-count transparency is in
**`results/paper_table_annotation_evidence.csv`**.

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
