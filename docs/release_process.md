# Release process

This document describes how to freeze a benchmark version, add a new
version, and regenerate paper reports.

## Freezing `v0.1`

1. Confirm every instance under `benchmark/v0.1/instances/**` passes:
   - `cta validate benchmark --version v0.1 --release`
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

Paper-track note: from `v0.2` onward, release validation also enforces
held-out evaluation (`dev`/`eval` disjoint, `eval` >= 24), full annotation
coverage for any experiment that sets
`require_full_annotation_coverage: true`, and a two-reviewer gold audit
signoff (`benchmark/<version>/audit/gold_signoff.json`).

`v0.2/dev.json` is intentionally empty at this stage. Policy rationale:
for paper-track readiness we block on held-out `eval` quality first
(coverage + signoff + provider runs) and avoid mixing prompt-tuning
diagnostics into release gating. If a future cycle reintroduces active
dev-tuning, populate `dev.json` with non-overlapping instances and keep
`eval` disjoint.

Rule: never reuse an `instance_id` across versions with different
semantic content. If an instance changes meaning, give it a new id and
increment the 3-digit suffix (e.g. `arrays_binary_search_002`).

## Paper-track closure flow (`v0.2`)

Use this sequence as the authoritative release path:

1. Initialize and track annotation queue:
   - `cta annotate plan --benchmark-version v0.2 --experiment-config configs/experiments/benchmark_v1.json --out benchmark/v0.2/annotation/task_board/`
   - `cta annotate batches --benchmark-version v0.2 --missing-pairs benchmark/v0.2/annotation/task_board/missing_pairs.json --out benchmark/v0.2/annotation/task_board/batches/`
2. Sync adjudicator outputs and rebuild coverage:
   - `cta annotate sync-review-packets --benchmark-version v0.2 --from benchmark/v0.2/annotation/review_packets --out benchmark/v0.2/annotation/adjudicated_subset/`
   - `cta annotate pack --version v0.2 --from-benchmark`
   - `cta annotate coverage --benchmark-version v0.2 --experiment-config configs/experiments/benchmark_v1.json --pack benchmark/v0.2/annotation/adjudicated_subset/pack.json --out benchmark/v0.2/annotation/adjudicated_subset/`
3. Prepare audit workbook and collect human signoff:
   - `cta benchmark audit-workbook --version v0.2`
   - update `benchmark/v0.2/audit/gold_signoff.json` with real reviewer names and `approved: true`
4. Run release gate:
   - `cta validate benchmark --version v0.2 --release`
5. Run review-packet audit gate (must pass before packaging):
   - `cta annotate verify-review-packets --benchmark-version v0.2 --packets-root benchmark/v0.2/annotation/review_packets --schema schemas/review_packet.schema.json --out benchmark/v0.2/annotation/review_packets/verification_summary.signed.json`
6. Run experiment and package paper artifacts:
   - `cta experiment --config configs/experiments/benchmark_v1.json`
   - `cta reports package --benchmark-version v0.2 --canonical-run-ids <run_id_1>,<run_id_2>,...`

For automation, the fail-fast orchestrator chains these gates:

`cta benchmark paper-orchestrate --benchmark-version v0.2 --canonical-run-ids <run_ids>`

`paper-orchestrate` now enforces `annotate verify-review-packets` automatically
before `reports package`, and exits non-zero if packet verification fails.

## Code-only remediation protocol

When `code_only_v1` packets show scaffold-heavy or vacuous obligations, run
this targeted remediation loop before broad benchmark refresh:

1. Tighten prompt constraints in `configs/prompts/code_only_v1.json`:
   - benchmark-facing obligations first
   - optional auxiliary obligations second
   - no vacuous theorem forms (`True`, `P -> True`, `P ∧ True`, `∃ x, True`)
   - no off-spec promotion in benchmark-facing output
2. Tighten normalizer filtering (`crates/cta_generate/src/normalize.rs`):
   - drop vacuous obligations
   - demote off-spec extras to auxiliary
3. Regenerate only the scoped packet set with `cta generate --instances ...`.
4. Rebuild scoped packets with `cta annotate build-review-packets --pairs ...`.
5. Run packet regression and schema gates:
   - `cargo test -p cta_generate --test code_only_packet_regression`
   - `cta annotate verify-review-packets ...`

Focus-first policy: do not broaden instance scope until the targeted packet
set is clean under both regression checks and packet schema verification.

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
