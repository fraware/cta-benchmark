# Repair protocol (v0.3)

## Scope

Repairs documented here apply to **eval** instances only. They are hygiene and
scaffold-alignment edits tracked in `repairs/hotspot_selection.csv` and
`repairs/repair_log.jsonl`, not changes to benchmark semantics.

## Inclusion

A `(instance_id, system_id)` pair becomes a **repair candidate** when at least
one of the following holds on adjudicated metrics from `results/raw_metrics.json`:

- low semantic faithfulness (materializer threshold near `0.55`),
- contradiction signal,
- missing critical semantic units.

The **selection rule** `hotspot_low_faithfulness_top12` (see
`benchmark/v0.3/protocol_freeze.json`) ranks candidates by faithfulness and
selects up to twelve unique pairs for logged repair attempts.

## Stopping rules

- Selection stops at the configured budget even if more candidates remain.
- No repair may change `instance_id`, obligations’ informal meaning, or harness
  contracts; only packet-local Lean imports, obligation bundling, and
  diagnostics-aligned scaffolding may move.

## ITT vs per-protocol metrics

- **ITT-style reporting (default tables):** use `results/instance_level.csv` as
  emitted: repaired rows keep post-repair scores.
- **Counterfactual summary:** `scripts/repair_counterfactual_metrics.py` writes
  `results/repair_impact_summary.json` with a transparent proxy described in
  that file’s `counterfactual_definition` field.

## Success criteria

A repair attempt is **successful** when `repair_log.jsonl` records
`outcome_summary` consistent with elaboration hygiene (see selection script) and
the corresponding row in `hotspot_selection.csv` shows `outcome` not equal to
`not_selected` for selected rows.
