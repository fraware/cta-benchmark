/-
Scaffold for instance `dp_knapsack_01_005`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Benchmark.DP.KnapsackTheory

namespace CTA.Benchmark.DP.Knapsack01_005

open CTA.Core
open CTA.Benchmark.DP.KnapsackTheory

abbrev Weights := KnapsackTheory.Weights
abbrev Values := KnapsackTheory.Values
abbrev Selection := KnapsackTheory.Selection
abbrev totalWeight := KnapsackTheory.totalWeight
abbrev totalValue := KnapsackTheory.totalValue
abbrev ValidSelection := KnapsackTheory.ValidSelection
abbrev knapsack01 := KnapsackTheory.knapsack01

end CTA.Benchmark.DP.Knapsack01_005
