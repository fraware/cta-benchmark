# Paper Readiness Playbook

This document defines the non-negotiable gates for promoting a benchmark
release from pilot plumbing to paper-reportable evidence.

## Release policy

- `v0.1` is a pilot release and may keep dev/eval overlap.
- `v0.2+` is paper-track and must satisfy held-out evaluation:
  - `eval` non-empty
  - `eval` size >= 24
  - `dev` and `eval` disjoint

These are enforced by `cta validate benchmark --release`.

## Annotation coverage policy

Any experiment config that sets:

```json
"require_full_annotation_coverage": true
```

must also provide an `annotation_pack` that covers every
`(instance_id, system_id)` pair in the experiment target split.

For paper tables this is mandatory. For pilots/smoke experiments it may be
left disabled.

## Gold audit signoff

For non-pilot releases (`v0.2+`), release validation requires:

`benchmark/<version>/audit/gold_signoff.json`

with:

- matching `benchmark_version`
- non-empty `primary_reviewer`
- non-empty `secondary_reviewer`
- `approved: true`

This encodes the two-person mathematical signoff requirement.

## Real-provider campaign protocol

Run at least one full paper-track experiment with real providers after
annotation coverage is complete.

1. Set provider credentials (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`) in the
   secure environment. For local runs, `cta` auto-loads `<workspace>/.env`.
2. Run:
   - `cta validate benchmark --version v0.2 --release`
   - `cta experiment --config configs/experiments/benchmark_v1.json`
3. Validate every emitted:
   - `run_manifest.json`
   - `results_bundle.json`
4. Build report artifacts:
   - `cta reports build --run <run_id>`
   - `cta reports aggregate --runs-root runs --out reports`
5. Archive the complete run directory and commit the paper tables generated
   from that run.

If step 2 cannot run due credentials/network, the release is not paper-ready.

## Annotation burn-down loop (operational)

Use the deterministic task-board outputs and strict per-system batches:

- `cta annotate plan --benchmark-version v0.2 --experiment-config configs/experiments/benchmark_v1.json --out benchmark/v0.2/annotation/task_board/`
- `cta annotate batches --benchmark-version v0.2 --missing-pairs benchmark/v0.2/annotation/task_board/missing_pairs.json --out benchmark/v0.2/annotation/task_board/batches/`

After each completed human-review batch:

```bash
powershell -NoProfile -File scripts/paper_release_loop.ps1 -Version v0.2 -ExperimentConfig configs/experiments/benchmark_v1_openai_only.json
```

This does three things in order:

1. Rebuilds `benchmark/v0.2/annotation/adjudicated_subset/pack.json`
   from adjudicated files.
2. Writes burn-down reports:
   - `reports/openai_campaign_2026_04_22/annotation_burndown.json`
   - `reports/openai_campaign_2026_04_22/annotation_burndown_batches.csv`
3. Runs release validation:
   - `cta validate benchmark --version v0.2 --release`

The release is green only when missing annotation pairs are zero and gold
signoff is valid.

## Human review packet workflow

For each batch CSV, generate structured review packet templates:

```bash
powershell -NoProfile -File scripts/generate_annotation_packets.ps1 -Version v0.2 -BatchCsv reports/openai_campaign_2026_04_22/annotation_batches/batch_01.csv
```

This creates `ann_01`, `ann_02`, and `adjudicator` template JSON files plus
a packet checklist under:

`benchmark/v0.2/annotation/review_packets/<batch_id>/...`

After adjudicated files are finalized in review packets, sync them into the
canonical adjudicated subset:

```bash
cargo run -p cta_cli -- annotate sync-review-packets \
  --benchmark-version v0.2 \
  --from benchmark/v0.2/annotation/review_packets \
  --out benchmark/v0.2/annotation/adjudicated_subset
```

Then validate the batch and refresh release gates:

1. `powershell -NoProfile -File scripts/validate_annotation_batch.ps1 -Version v0.2 -BatchCsv <batch_csv>`
2. `powershell -NoProfile -File scripts/paper_release_loop.ps1 -Version v0.2 -ExperimentConfig configs/experiments/benchmark_v1_openai_only.json`
3. When `packet.json` content, prompts, or normalizers changed in the same change set, run:
   - `cargo test -p cta_generate --test code_only_packet_regression`
   - `cargo test -p cta_generate --test family_packet_regression`
   - `cargo test -p cta_generate --test full_method_priority1_packet_regression`
   - `cargo test -p cta_generate --test full_method_priority2_packet_regression`
   - `cargo test -p cta_generate --test review_packet_lean_lint`
4. `cargo run -p cta_cli -- annotate verify-review-packets --benchmark-version v0.2 --packets-root benchmark/v0.2/annotation/review_packets --schema schemas/review_packet.schema.json --out benchmark/v0.2/annotation/review_packets/verification_summary.signed.json`

## Gold-audit workbook

Generate paper-track workbook CSVs directly from eval split:

```bash
cargo run -p cta_cli -- benchmark audit-workbook --version v0.2
```

This writes:

- `benchmark/v0.2/audit/evidence/per_instance_audit.csv`
- `benchmark/v0.2/audit/evidence/obligation_audit.csv`

## Paper packaging

After release validation is green and canonical run ids are selected:

```bash
cargo run -p cta_cli -- reports package \
  --benchmark-version v0.2 \
  --canonical-run-ids <run_id_1>,<run_id_2>,...
```

This materializes `reports/paper_v0.2/` and writes `paper_summary.json`.

If you use `benchmark paper-orchestrate`, the review-packet verification gate
is enforced automatically before packaging.
