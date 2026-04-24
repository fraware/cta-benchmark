/-
Scaffold for instance `greedy_interval_scheduling_001`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Benchmark.Greedy.IntervalSchedulingTheory

namespace CTA.Benchmark.Greedy.IntervalScheduling001

open CTA.Core
open CTA.Benchmark.Greedy.IntervalSchedulingTheory

abbrev Interval := IntervalSchedulingTheory.Interval
abbrev Intervals := IntervalSchedulingTheory.Intervals
abbrev NonOverlap := IntervalSchedulingTheory.NonOverlap
abbrev Feasible := IntervalSchedulingTheory.Feasible

/-- Shared definition-backed interval scheduling model. -/
abbrev intervalScheduling := IntervalSchedulingTheory.intervalScheduling

end CTA.Benchmark.Greedy.IntervalScheduling001
