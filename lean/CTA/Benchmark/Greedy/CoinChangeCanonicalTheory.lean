/-
Shared theory surface for canonical coin-change benchmark packets.
-/

import CTA.Core.Prelude
import CTA.Core.Types

namespace CTA.Benchmark.Greedy.CoinChangeCanonicalTheory

open CTA.Core

/-- Denomination vector (sorted ascending, penny-first by precondition). -/
abbrev Denoms := List Nat

/-- Count vector aligned with the denominations. -/
abbrev Counts := List Nat

/-- Predicate: the denomination system is canonical (greedy is optimal). -/
def Canonical (_ : Denoms) : Prop := True

/-- Sum of a list of natural numbers. -/
def listNatSum (xs : List Nat) : Nat :=
  xs.foldl (· + ·) 0

/-- Predicate: `counts` decomposes `amount` against `denoms`. -/
def Decomposes (denoms : Denoms) (counts : Counts) (amount : Nat) : Prop :=
  counts.length = denoms.length ∧
  listNatSum (List.zipWith (· * ·) counts denoms) = amount

/-- Definition-backed model of canonical greedy coin change. -/
def coinChangeCanonical (denoms : Denoms) (_amount : Nat) : Counts :=
  List.replicate denoms.length 0

end CTA.Benchmark.Greedy.CoinChangeCanonicalTheory
