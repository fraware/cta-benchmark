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

## Separation from raw system evaluation

Hotspot repairs are logged under `repairs/` and excluded from the primary
`system_summary.csv` scores unless a column explicitly marks `repair_subset=yes`
in `results/instance_level.csv`.
