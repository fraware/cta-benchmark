# Calibration pack

Worked exemplar annotations used to calibrate new annotators against
`rubric_v1` before they touch the eval split.

Each exemplar is a schema-valid `annotation.schema.json` file produced by
the reference adjudicator. The pack intentionally spans a wide slice of
the label lattice so that a trainee who reproduces every file has
exercised every rubric decision at least once.

## Inventory

| File                                            | Domain  | Highlights                                              |
| ----------------------------------------------- | ------- | ------------------------------------------------------- |
| `arrays_binary_search_001__text_only_v1.json`   | arrays  | Partial SU coverage plus one `partial` faithfulness     |
| `sorting_insertion_sort_001__code_only_v1.json` | sorting | Fully-faithful set that demonstrates vacuity detection  |
| `graph_dijkstra_001__full_method_v1.json`       | graph   | Inconsistency on a critical SU, shows contradiction     |

## How to use

1. Read `benchmark/v0.2/annotation/rubric_v1.md` end to end.
2. For each exemplar, re-annotate the corresponding
   `(instance, system)` pair from scratch, without looking at the
   exemplar.
3. Diff your labels against the exemplar using your preferred JSON diff
   tool (or `cta annotate pack --input <your_dir>` against a copy of
   this directory to surface disagreements structurally).
4. Any mismatch triggers a calibration discussion with the adjudicator
   before you are cleared to annotate eval data.

The pack itself is immutable under `v0.2`. Additional exemplars or
rubric clarifications roll forward into the next benchmark release.
