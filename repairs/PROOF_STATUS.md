# Repaired hotspot subset: proof status vocabulary

## Definition-backed (formal contract)

A repaired packet is **definition-backed** when all of the following hold:

1. **Imported theory lemmas only:** every `sorry`-free proof step ultimately
   relies on lemmas and definitions from Mathlib or checked `CTA.*` theory
   modules that predate the packet (no ad hoc axiom blocks introduced only
   for this packet).
2. **No local wrapper theorem:** the repaired file does not introduce a
   single theorem whose statement is logically equivalent to the full user
   goal while hiding work in an opaque `opaque`/`axiom`.
3. **No goal-shaped assumptions:** there is no packet-local hypothesis whose
   statement matches (or trivially implies) the benchmark theorem goal under
   syntactic renaming only.
4. **Zero `admit` / `sorry`:** the repaired artifact elaborates with `admit_count = 0`
   and no `sorry` in tracked obligations.

## Axiom counts vs definition-backed

**Definition-backed does not mean axiom-free.** Mathlib and `CTA.*` preliminaries
may carry their own proof obligations; packet-level hygiene metrics can still
report **`axiom_count > 0`** when the checker attributes a step to a named
axiom schema from the library, or when a lemma is classified that way in the
diagnostic export. The paper should therefore distinguish:

- **Zero admits / zero sorry (repair gate):** the repaired obligation text
  contains no proof gaps (`admit` / `sorry`) in the obligations we track for the
  repair protocol.
- **Elaborated:** the file passes the configured Lean elaboration gate (for
  example strict M1 targets) without deferring the benchmark theorem to an
  unfinished shell.
- **Theory-backed:** proof steps only depend on previously published theory
  modules (Mathlib / checked `CTA.*`), not on packet-local axiom blocks invented
  to discharge the user goal.
- **Not necessarily axiom-free:** global axiom footprints can remain nonzero
  while still satisfying the three bullets above; headline claims should name
  the predicate actually checked (e.g. admit/sorry-free + elaboration) rather
  than implying `axiom_count = 0` unless that quantity is explicitly audited.

## Separation from raw system evaluation

Hotspot repairs are logged under `repairs/` and excluded from the primary
`system_summary.csv` scores unless a column explicitly marks `repair_subset=yes`
in `results/instance_level.csv`.
