/-
Scaffold for instance `greedy_interval_scheduling_007`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Benchmark.Greedy.IntervalSchedulingTheory

namespace CTA.Benchmark.Greedy.IntervalScheduling007

open CTA.Core
open CTA.Benchmark.Greedy.IntervalSchedulingTheory

/-- Family-level interval model reused across interval-scheduling packets. -/
abbrev Interval := IntervalSchedulingTheory.Interval

/-- Family-level interval collection type. -/
abbrev Intervals := IntervalSchedulingTheory.Intervals

/-- Shared non-overlap semantics (closed-open endpoint compatible). -/
abbrev NonOverlap := IntervalSchedulingTheory.NonOverlap

/-- Shared feasibility predicate used by benchmark-facing obligations. -/
abbrev Feasible := IntervalSchedulingTheory.Feasible

/-- Shared definition-backed interval scheduling model. -/
abbrev intervalScheduling := IntervalSchedulingTheory.intervalScheduling

end CTA.Benchmark.Greedy.IntervalScheduling007
