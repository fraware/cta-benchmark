# Gold audit checklist (v0.2)

Reviewers should confirm the following for each instance:

- Every critical semantic unit is linked to at least one gold obligation.
- No gold obligation is vacuous.
- No gloss overstates the Lean statement.
- No precondition is encoded as a universal truth.
- No critical property is represented only weakly or ambiguously.
- Scaffold obligations and gold obligations are semantically aligned.
- The behavioral harness is compatible with intended semantics.

Signoff protocol:

1. Primary reviewer completes first pass in `evidence/per_instance_audit.csv` and `evidence/obligation_audit.csv`.
2. Secondary reviewer performs independent pass on the same rows.
3. Disagreements are resolved and disposition is recorded in `evidence/obligation_audit.csv`.
4. Final notes are captured in `evidence/signoff_notes.md`.
5. `gold_signoff.json` is updated with real reviewer names and `approved: true` only after both reviewers agree.
