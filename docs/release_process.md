# Release process

This document describes how to freeze a benchmark version, add a new
version, and regenerate paper reports.

## Freezing `v0.1`

1. Confirm every instance under `benchmark/v0.1/instances/**` passes:
   - `cta validate benchmark --version v0.1`
   - `cta benchmark lint --version v0.1`
2. Regenerate the benchmark manifest:
   `cta benchmark manifest --version v0.1`.
3. Commit the manifest. `content_hash` must not change after this commit
   except by version bump.
4. Tag the repository: `git tag benchmark-v0.1 -m "freeze v0.1"`.
5. From this point on, treat every file under `benchmark/v0.1/` as
   immutable. The CI `benchmark-lint` job fails if content_hash changes
   without a version bump.

## Adding `v0.2`

1. `cp -r benchmark/v0.1 benchmark/v0.2`.
2. Update `benchmark_version` in every `instance.json` and in
   `splits/*.json`.
3. Apply changes only within `benchmark/v0.2/`.
4. Repeat the freeze process for `v0.2`.

Rule: never reuse an `instance_id` across versions with different
semantic content. If an instance changes meaning, give it a new id and
increment the 3-digit suffix (e.g. `arrays_binary_search_002`).

## Schema evolution

- Additive, non-breaking changes (new optional fields): bump the schema to
  `schema_v2` in a new file, update `cta_schema` to load both, and leave
  existing artifacts alone.
- Any breaking change requires `schema_v<n+1>` and bumping
  `benchmark_version` so the artifacts under the old schema remain valid
  in place.

## Metrics evolution

Metric names are frozen under `metrics_v2`. Never redefine an existing
metric. Introduce new metrics in `metrics_v3` and record the contract
version in every `run_manifest.json`. `metrics_v1` is retained only for
archival comparison; the current pipeline emits `metrics_v2` by default.
Schema validation allows `metrics_vN` for archival runs, while paper
aggregation should enforce the current contract version.

## Rubric evolution

Annotation rubrics are frozen under `rubric_v<n>`. Adjudicated records
under the old rubric remain authoritative for runs that used it. New
rubric versions may only be used on new annotation batches.

## Regenerating paper reports

1. Pick a committed run id under `runs/`.
2. Run `cta reports build --run <run_id>`. Reports are emitted next to
   the source bundle at `runs/<run_id>/reports/` and include a per-system
   primary metrics CSV, per-instance CSV, Markdown summary, and LaTeX
   table (override the destination with `--out <dir>`).
3. Diff the committed reports against the regenerated ones; a clean diff
   (modulo timestamps) is required before publication. The exact CSV,
   Markdown, and LaTeX shape is pinned by snapshot tests under
   `crates/cta_reports/tests/snapshots/`, so any non-trivial diff is a
   contract change and must be accompanied by a snapshot update and a
   metrics-version bump.
