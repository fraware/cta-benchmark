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

1. Export provider credentials (for example `OPENAI_API_KEY`,
   `ANTHROPIC_API_KEY`) in CI or a secure runner.
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

Use the generated deterministic batches under
`reports/openai_campaign_2026_04_22/annotation_batches/` and run this loop
after each completed batch:

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

After adjudicated files are copied into
`benchmark/v0.2/annotation/adjudicated_subset/<system>/<instance>.json`,
validate the batch:

```bash
powershell -NoProfile -File scripts/validate_annotation_batch.ps1 -Version v0.2 -BatchCsv reports/openai_campaign_2026_04_22/annotation_batches/batch_01.csv
```

Then run the release loop to refresh burn-down and gate status.
