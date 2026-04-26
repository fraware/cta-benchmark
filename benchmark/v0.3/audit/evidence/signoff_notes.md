# Gold audit signoff notes (v0.3)

## Current repository state

The `v0.3` benchmark ships in **template** gold-audit posture (see
`../gold_signoff.json`: `release_gold_audit_status` is
`template_pending_human_review` and `approved` is `false`).

Skeleton rows for the **eval** split live in:

- `per_instance_audit.csv`
- `obligation_audit.csv`

They were produced with `cta benchmark audit-workbook --version v0.3` so every
eval instance and every reference obligation has a row awaiting human verdicts.

This file is intentionally **not** a closing argument yet: once both reviewers
finish the dual pass, replace this section with:

- A short narrative of scope (which families received extra scrutiny, if any).
- Any benchmark-wide concerns discovered during gold review.
- A one-paragraph statement that supports setting `approved: true` in
  `gold_signoff.json` and moving `release_gold_audit_status` to
  `human_audit_complete` (or removing the field).

## Blind / anonymized submissions

If reviewer identities must stay off disk until after acceptance, keep
`approved: false` and the template status until de-anonymization is allowed, then
record legal names in `gold_signoff.json` and flip the posture fields as above.
