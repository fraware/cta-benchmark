/-
Scaffold for instance `greedy_coin_change_canonical_002`.
-/

import CTA.Core.Prelude
import CTA.Core.Types

namespace CTA.Benchmark.Greedy.CoinChangeCanonical002

open CTA.Core

/-- Denomination vector (sorted ascending, penny-first by precondition). -/
abbrev Denoms := List Nat

/-- Count vector aligned with the denominations. -/
abbrev Counts := List Nat

/-- Predicate: the denomination system is canonical (greedy is optimal).
    Left opaque so annotators can specialize it per-instance in proofs. -/
opaque Canonical : Denoms → Prop

/-- Sum of a list of natural numbers (core `List` has no `List.sum` in this toolchain). -/
private def listNatSum (xs : List Nat) : Nat :=
  xs.foldl (· + ·) 0

/-- Predicate: `counts` decomposes `amount` against `denoms`. -/
def Decomposes (denoms : Denoms) (counts : Counts) (amount : Nat) : Prop :=
  counts.length = denoms.length ∧
  listNatSum (List.zipWith (· * ·) counts denoms) = amount

/-- Declarative model of the reference `coin_change_canonical`. -/
opaque coinChangeCanonical : Denoms → Nat → Counts

end CTA.Benchmark.Greedy.CoinChangeCanonical002
