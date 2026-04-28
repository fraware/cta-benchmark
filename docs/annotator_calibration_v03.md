# Annotator calibration (v0.3)

This note complements `docs/annotation_manual.md` for any **human** wave on
v0.3 eval packets. Until real ratings exist under
`benchmark/v0.3/annotation/human_wave_v03/`, it is a checklist only.

## Gold anchors

Before scoring eval packets, each annotator must work the curated exemplars
in `benchmark/v0.3/annotation/calibration_pack/` (same discipline as the
manual: rubric `rubric_v1`, diff against pack adjudication, escalate
disagreements).

Anchor packet IDs for **spot checks** after the first wave (curator-defined;
extend this list when the wave is scheduled):

- `arrays_binary_search_001` — `text_only_v1` review packet (vacuity + partial
  faithfulness stress).
- `greedy_interval_scheduling_001` — `full_method_v1` (definition-backed proof
  mode reference).

## Pass / fail rubric checks

Annotators pass calibration when, on a closed set of obligations from the
anchor packets:

- every faithfulness label matches the adjudicated pack **or** the deviation
  is documented with an `ambiguous` + note for adjudication;
- vacuity is not skipped on any obligation;
- critical-unit links reference real SU ids from `semantic_units.json`.

## Drift between waves (optional)

If a second wave runs, re-score the same anchor subset and compare weighted
agreement to wave 1. A large drop triggers manual review of instructions, not
silent relabeling of historical rows.

## Evidence-Hardening Update (2026-04-28)

Calibration completion should now be reflected in:

- `annotation/human_pass_v2/agreement_report_human.json`
- `annotation/human_pass_v2/agreement_report_human.md`
- `results/paper_table_human_agreement.csv`

These files are materialized through
`python scripts/implement_evidence_hardening.py`.
