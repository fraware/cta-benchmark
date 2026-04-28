# External annotation review schema

- `strict_review_queue.jsonl`: strict independent rows from `results/raw_metrics_strict.json`.
- `mapped_review_queue.jsonl`: mapped rows from `results/raw_metrics_expanded.json` where `annotation_origin=mapped_from_canonical`.
- `strict_review_queue.csv`: flattened strict queue for spreadsheet workflows.
- `semantic_corrections_v3.csv`: preferred explicit correction overlay for
  obligation-level faithfulness/vacuity/coverage updates (fallback behavior in
  materializer: cumulative `semantic_corrections_v1.csv` + `v2.csv` when v3 is absent).

## JSONL row fields
- `instance_id`, `family`, `system_id`: row identity.
- `annotation_origin`, `mapped_from_canonical`: provenance.
- `informal_spec`: instance natural-language contract from packet context.
- `critical_semantic_units`: critical semantic-unit ids/descriptions.
- `reference_obligations`: reference obligations from packet context.
- `generated_obligations`: generated obligations with index, statement, gloss, linked units, layer.
- `current_labels`: current adjudication/metric labels and coverage arrays.
- `source_paths`: instance/packet/raw-metrics trace paths.
