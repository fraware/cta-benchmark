# Failure-mode ontology (v1)

Machine-readable list: `schemas/failure_mode_v1.json` (mirrored under
`benchmark/v0.3/failure_mode_ontology.json` for benchmark-local discovery).

`results/raw_metrics.json` field `failure_mode_label` must be either empty
(`""`, meaning *no explicit tag*) or exactly one `slug` from that file.

## CI

`python scripts/compute_results.py --paper` refuses unknown labels.

`scripts/ci_reviewer_readiness.py` (run in `.github/workflows/ci.yml`) validates
`schemas/failure_mode_v1.json` with `cta validate file --schema failure_mode_ontology`
when that file is present, then checks every non-empty `failure_mode_label` in
`results/raw_metrics.json` against the ontology `modes[].slug` set.

## Derived modes

When `failure_mode_label` is empty but `faithfulness_mean` is below the
reporting threshold, `scripts/compute_results.py` still increments
`low_faithfulness` in `failure_mode_counts.csv` so tables remain informative.
Those rows are **derived**, not additional ontology slugs.
