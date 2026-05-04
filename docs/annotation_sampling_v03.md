# Stratified sampling frame (v0.3 human wave)

This document defines the **intended** stratification for a human double-annotation
wave on v0.3 **eval** packets. Until real CSVs land under
`benchmark/v0.3/annotation/human_wave_v03/`, all paper metrics remain
`pipeline_derived` per `docs/reviewer_map.md` (annotation tiers and pack roles).

## Factors

Primary cells are the Cartesian product of:

- `system_id` (four baselines in `configs/experiments/benchmark_v03.json`)
- `family` (twelve families in `benchmark/v0.3/benchmark_paper_summary.json`)
- `difficulty` (`easy` | `medium` from the benchmark manifest)

## Minimum cell policy (target)

For each non-empty cell in the grid above that appears in the frozen **eval**
split (`benchmark/v0.3/splits/eval.json`), target at least **two** packets
per rater after quality filters (eligibility: packet has non-empty obligation
bundle and passes hygiene gates in `docs/annotation_manual.md`).

Cells with fewer than two eligible packets are merged upward (same `system_id`
and `difficulty`, pool families) before undersampling elsewhere, so the frame
stays feasible without inventing instances.

## Import path

Curators copy completed CSVs into `benchmark/v0.3/annotation/human_adjudicated/`
with `pack.json` and a validated `manifest.json` (see
`schemas/annotation_pack_manifest.schema.json`). The CLI resolves the pack path
from the experiment config when `annotation_human_pack` is set; see
`configs/experiments/benchmark_v03.json`.

## Direct-adjudication workflow (historical + maintenance)

For additional maintenance waves (beyond the completed strict headline set),
run:

```powershell
python scripts/plan_v03_direct_adjudication_wave.py --target-pairs 128
```

This writes:

- `benchmark/v0.3/annotation/human_wave_v03/direct_adjudication_wave_plan.csv`
  (prioritized eval `(instance_id, system_id)` worklist), and
- `benchmark/v0.3/annotation/human_adjudicated/direct_adjudicated_pairs.csv`
  (header template for completed direct adjudications).

After adjudication, append reviewed pairs to
`direct_adjudicated_pairs.csv` with `annotation_origin` set to
`direct_adjudicated` or `direct_human`, then rerun:

```powershell
python scripts/materialize_v03_adjudication_artifacts.py
python scripts/compute_results.py --paper
python scripts/export_benchmark_paper_summary.py
```

`materialize_v03_adjudication_artifacts.py` now accepts
`--direct-origin-overrides` (defaulting to that CSV) and promotes listed pairs
into `raw_metrics_strict.json` / strict summary counts.

## Evidence-Hardening Update (2026-04-28)

Human-pass strict overlap reporting now targets v3 strict-all artifacts:

- `annotation/human_pass_v3/human_strict_packet_ids.csv`
- `annotation/rater_a_strict_all.csv`
- `annotation/human_pass_v3/rater_b_human_strict_all.csv`
- `annotation/human_pass_v3/agreement_report_human_strict_all.{json,md}`
- `annotation/human_pass_v3/disagreement_log_strict_all.csv`

Paper-facing overlap accounting is reported in:

- `results/paper_table_agreement_evidence.csv` (`strict_all_human_overlap`)
