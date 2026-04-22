/-
Scaffold for instance `greedy_interval_scheduling_002`.
-/

import CTA.Core.Prelude
import CTA.Core.Types

namespace CTA.Benchmark.Greedy.IntervalScheduling002

open CTA.Core

/-- A closed-open interval `[start, end)`. -/
structure Interval where
  start : Int
  stop : Int
  deriving Repr, DecidableEq

/-- A list of intervals. -/
abbrev Intervals := List Interval

/-- Two intervals are non-overlapping if one ends at or before the other
    starts. Sharing only an endpoint counts as non-overlapping. -/
def NonOverlap (a b : Interval) : Prop :=
  a.stop ≤ b.start ∨ b.stop ≤ a.start

/-- A selection of indices is pairwise non-overlapping relative to an
    interval list; reference-only relation. -/
opaque Feasible : Intervals → List Nat → Prop

/-- Declarative model of the reference `interval_scheduling`. -/
opaque intervalScheduling : Intervals → Nat

end CTA.Benchmark.Greedy.IntervalScheduling002
