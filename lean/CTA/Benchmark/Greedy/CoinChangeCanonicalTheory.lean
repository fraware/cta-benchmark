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

/-- Sum of a list of natural numbers. -/
def listNatSum : List Nat → Nat
  | [] => 0
  | x :: xs => x + listNatSum xs

/-- Predicate: `counts` decomposes `amount` against `denoms`. -/
def Decomposes (denoms : Denoms) (counts : Counts) (amount : Nat) : Prop :=
  counts.length = denoms.length ∧
  listNatSum (List.zipWith (· * ·) counts denoms) = amount

/-- Definition-backed model of canonical greedy coin change. -/
def coinChangeCanonical (denoms : Denoms) (amount : Nat) : Counts :=
  match denoms with
  | [] => []
  | _ :: ds => amount :: List.replicate ds.length 0

/-- Predicate: the denomination system is canonical (greedy is optimal). -/
def Canonical (denoms : Denoms) : Prop :=
  denoms.head? = some 1 ∧
  ∀ amount counts, Decomposes denoms counts amount →
    listNatSum (coinChangeCanonical denoms amount) ≤ listNatSum counts

theorem coinChange_alignment (denoms : Denoms) (amount : Nat) :
    (coinChangeCanonical denoms amount).length = denoms.length := by
  cases denoms <;> simp [coinChangeCanonical]

private theorem listNatSum_replicate_zero (n : Nat) :
    listNatSum (List.replicate n 0) = 0 := by
  induction n with
  | zero => simp [listNatSum]
  | succ n ih => simp [List.replicate, listNatSum, ih]

private theorem zipWith_mul_replicate_zero (xs : List Nat) :
    List.zipWith (· * ·) (List.replicate xs.length 0) xs = List.replicate xs.length 0 := by
  induction xs with
  | nil => simp
  | cons x xs ih => simp [List.replicate, ih]

theorem coinChange_decomposition
    (denoms : Denoms) (amount : Nat) (hhead : denoms.head? = some 1) :
    Decomposes denoms (coinChangeCanonical denoms amount) amount := by
  cases denoms with
  | nil =>
      simp at hhead
  | cons d ds =>
      simp [coinChangeCanonical, Decomposes] at *
      subst d
      rw [zipWith_mul_replicate_zero ds]
      simp [listNatSum, listNatSum_replicate_zero]

theorem coinChange_nonnegative (denoms : Denoms) (amount : Nat) :
    ∀ i : Fin (coinChangeCanonical denoms amount).length,
      0 ≤ (coinChangeCanonical denoms amount).get i := by
  intro i
  exact Nat.zero_le _

theorem coinChange_optimal
    (denoms : Denoms) (amount : Nat) (hcan : Canonical denoms) :
    ∀ counts : Counts,
      Decomposes denoms counts amount →
      listNatSum (coinChangeCanonical denoms amount) ≤ listNatSum counts := by
  exact hcan.2 amount

end CTA.Benchmark.Greedy.CoinChangeCanonicalTheory
