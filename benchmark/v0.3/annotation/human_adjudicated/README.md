# Human adjudicated pack (slot)

Place the canonical human adjudication export here as `pack.json` (same schema as `adjudicated_subset/pack.json`).

When this file exists, `configs/experiments/benchmark_v03.json` field `annotation_human_pack` should point to `benchmark/v0.3/annotation/human_adjudicated/pack.json` so `cta annotate plan` resolves it first. See [docs/PROVENANCE.md](../../../../docs/PROVENANCE.md).
