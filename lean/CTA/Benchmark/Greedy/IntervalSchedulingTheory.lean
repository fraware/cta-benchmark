/-
Reusable definitions for interval scheduling benchmark proofs.

This module is intentionally lightweight: packet-specific files should import
it and prove instance theorems without re-defining family-level predicates.
-/

import CTA.Core.Prelude

namespace CTA.Benchmark.Greedy.IntervalSchedulingTheory

/-- Closed-open interval `[start, stop)`. -/
structure Interval where
  start : Int
  stop : Int
  deriving Repr, DecidableEq

/-- Interval collection type used by benchmark obligations. -/
abbrev Intervals := List Interval

/-- Endpoint-touching is allowed: `[a,b)` and `[b,c)` do not overlap. -/
def NonOverlap (x y : Interval) : Prop :=
  x.stop <= y.start ∨ y.stop <= x.start

/-- A selection is feasible when all chosen intervals are pairwise non-overlapping. -/
def Feasible (picked : Intervals) : Prop :=
  picked.Pairwise NonOverlap

/-- Definition-backed family model for interval scheduling output size.
This is a deterministic placeholder model used to avoid axiom-backed
algorithm symbols in packet interfaces. -/
def intervalScheduling (intervals : Intervals) : Nat :=
  intervals.length

/-- Non-overlap is symmetric. -/
theorem nonOverlap_comm {x y : Interval} : NonOverlap x y ↔ NonOverlap y x := by
  constructor
  · intro h
    rcases h with hxy | hyx
    · exact Or.inr hxy
    · exact Or.inl hyx
  · intro h
    rcases h with hyx | hxy
    · exact Or.inr hyx
    · exact Or.inl hxy

/-- Empty selections are feasible. -/
theorem feasible_nil : Feasible [] := by
  simp [Feasible]

/-- Any singleton selection is feasible. -/
theorem feasible_singleton (x : Interval) : Feasible [x] := by
  simp [Feasible]

/-- Constructor rule used by packet-level witness proofs. -/
theorem feasible_cons {x : Interval} {xs : Intervals}
    (hx : ∀ y, y ∈ xs → NonOverlap x y)
    (hxs : Feasible xs) : Feasible (x :: xs) := by
  simpa [Feasible] using And.intro hx hxs

end CTA.Benchmark.Greedy.IntervalSchedulingTheory
