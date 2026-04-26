# Per-run reproducibility

## `run_manifest.json` (Rust CLI)

Every `cta generate` / `cta experiment` run writes `run_manifest.json` in the
run directory with at least:

- `run_id`, `created_at`, `repo_commit`, `benchmark_version`
- `system_id`, `seed`, `prompt_template_hash`
- `provider` block (`name`, `model`, `model_version` when known)
- `generation_parameters` (`temperature`, `max_tokens`)
- `schema_versions` pin for instances, obligations, metrics, rubric
- `toolchains` (`rust`, `lean`)

Paths to generated bundles are implied by run layout:
`generated/<system_id>/*.json` and `generated/<system_id>/raw/*.txt`.

## Sidecar schema (paper appendix)

`experiments/run_manifest.schema.json` documents optional fields for cost and
token accounting when exporting from provider logs into a single JSON file
alongside `run_manifest.json`.
