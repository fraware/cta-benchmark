/-
Scaffold for instance `dp_knapsack_01_002`.
-/

import CTA.Core.Prelude
import CTA.Core.Types

namespace CTA.Benchmark.DP.Knapsack01_002

open CTA.Core

/-- Weight and value vectors aligned index-for-index. -/
abbrev Weights := List Nat
abbrev Values := List Nat

/-- A selection is represented as a list of distinct item indices. -/
abbrev Selection := List Nat

/-- Total weight of a selection. -/
opaque totalWeight : Weights → Selection → Nat

/-- Total value of a selection. -/
opaque totalValue : Values → Selection → Nat

/-- Predicate: `sel` is a valid subset (distinct indices, all in range). -/
def ValidSelection (weights : Weights) (sel : Selection) : Prop :=
  sel.Nodup ∧ ∀ i ∈ sel, i < weights.length

/-- Declarative model of the reference `knapsack_01`. -/
opaque knapsack01 : Weights → Values → Nat → Nat

end CTA.Benchmark.DP.Knapsack01_002
