# Annotation rubric (packet-level and semantic-unit-level)

This document fixes the criteria used for human annotation so the methods
section can describe them verbatim. Ordinal scales are 1–4 unless noted.

## 1. Semantic faithfulness (ordinal 1–4)

| Score | Definition |
|------:|------------|
| 4 | Every critical semantic unit is addressed by at least one generated obligation with matching quantifier structure and no scope drift. |
| 3 | All critical units covered with minor wording drift that does not change proof strength. |
| 2 | At least one critical unit missing, weakened (e.g. one-way implication instead of iff), or ambiguously scoped. |
| 1 | Multiple critical units missing or systematically mis-scoped. |

**Operational rule:** Annotators must cross-link each obligation to semantic
unit ids from the instance `semantic_units.json` when possible.

## 2. Code consistency (ordinal 1–4)

| Score | Definition |
|------:|------------|
| 4 | Generated obligations use only types and symbols consistent with `scaffold.lean` and the Rust summary; no invented accessors. |
| 3 | Mostly consistent; a single fixable naming mismatch. |
| 2 | Repeated mismatches or references to undefined symbols likely to fail elaboration. |
| 1 | Largely disconnected from the scaffold or contradicts the reference implementation. |

## 3. Proof utility (ordinal 1–4)

| Score | Definition |
|------:|------------|
| 4 | Obligations are decomposed, non-redundant, and likely to compose into a proof without major reformulation. |
| 3 | Useful but contains redundancy or missing lemmas that are obvious to add. |
| 2 | Monolithic or too coarse; would require substantial refactoring to prove. |
| 1 | Not usable as stated (e.g. false, circular, or equivalent to the full goal in one step). |

## 4. Vacuity rate (mathematical definition)

Let `O` be the multiset of **benchmark-facing** generated obligations in the
packet after normalization (the same set used for automated lint and
metrics). Let `V ⊂ O` be those obligations classified by the annotator as
**vacuous**, **tautological**, or **disconnected** from the informal statement
per the decision tree below.

\[
\text{vacuity rate} = \frac{|V|}{|O|}
\]

**Vacuous / tautological / disconnected (decision tree):**

1. **Tautological:** logically valid without using problem-specific premises
   (e.g. `True`, `∀ x, x = x`, implications with `True` consequent only).
2. **Vacuous:** antecedent impossible under well-typed scaffold inputs, or
   hypothesis forces an empty domain in a way the informal statement does not.
3. **Disconnected:** surface symbols match the scaffold but the statement does
   not correspond to any required property or semantic unit (template leakage).

Annotators record per-obligation flags; vacuity rate is recomputed from those
flags.

## 5. Double annotation and adjudication

- **Wave 1:** two independent annotators per sampled packet, blinded to
  `system_id` when feasible (present packets as `system_A`, `system_B` with
  random mapping recorded only in the adjudication log).
- **Wave 2:** adjudicator resolves disagreements on ordinal scores >1 step
  apart or any vacuity disagreement.

## 6. Agreement statistics

- **Ordinal:** weighted Cohen’s κ using linear weights for 1–4 scales.
- **Coverage labels:** Cohen’s κ or percent agreement depending on prevalence.
- **Reporting:** publish raw agreement tables plus κ with bootstrap 95% CI
  (10 000 resamples, cluster-bootstrap by instance if available).
