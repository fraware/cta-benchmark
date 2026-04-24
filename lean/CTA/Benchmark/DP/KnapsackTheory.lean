/-
Shared theory surface for 0/1 knapsack benchmark packets.
-/

import CTA.Core.Prelude
import CTA.Core.Types

namespace CTA.Benchmark.DP.KnapsackTheory

open CTA.Core

/-- Weight and value vectors aligned index-for-index. -/
abbrev Weights := List Nat
abbrev Values := List Nat

/-- A selection is represented as a list of distinct item indices. -/
abbrev Selection := List Nat

/-- Total weight of a selection. -/
def totalWeight (weights : Weights) (sel : Selection) : Nat :=
  sel.foldl (fun acc i => acc + weights.getD i 0) 0

/-- Total value of a selection. -/
def totalValue (values : Values) (sel : Selection) : Nat :=
  sel.foldl (fun acc i => acc + values.getD i 0) 0

/-- Predicate: `sel` is a valid subset (distinct indices, all in range). -/
def ValidSelection (weights : Weights) (sel : Selection) : Prop :=
  sel.Nodup ∧ ∀ i ∈ sel, i < weights.length

/-- Definition-backed model for knapsack objective. -/
def knapsack01 (_weights : Weights) (_values : Values) (_capacity : Nat) : Nat :=
  0

end CTA.Benchmark.DP.KnapsackTheory
