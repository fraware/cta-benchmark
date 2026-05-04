# Human annotation wave v0.3 (templates)

These CSVs are **header-only** templates for a future stratified double-annotation
wave. They intentionally contain no data rows.

## Identifier policy

- Column `anonymized_packet_key` must be an opaque token (for example a random
  UUID per packet) assigned **outside** this repository. The public benchmark
  must not ship a reversible mapping from `anonymized_packet_key` to
  `instance_id` or provider outputs.

## Columns

Rater sheets mirror the synthetic inter-rater layout (`semantic_faithfulness`,
`code_consistency`, `proof_utility` on 1–4, `coverage_label` in
`full|partial|failed`). The adjudication log records disagreements and the
resolved gold value per dimension.

When real CSVs exist, set `USE_HUMAN_ANNOTATION=1` (or the experiment config
`annotation_human_pack`) so tooling prefers `human_adjudicated/`; see
`docs/annotation_sampling_v03.md`, `docs/annotator_calibration_v03.md`, and
`docs/reviewer_map.md`.
