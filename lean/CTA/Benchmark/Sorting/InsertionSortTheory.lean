/-
Shared theory surface for insertion sort benchmark packets.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Core.Checkers

namespace CTA.Benchmark.Sorting.InsertionSortTheory

open CTA.Core

/-- Family-level array model for sorting tasks. -/
abbrev Arr := List Int

/-- Family-level sortedness predicate reused by packet theorems. -/
abbrev Sorted (xs : Arr) : Prop := SortedLE xs

/-- Family-level permutation predicate reused by packet theorems. -/
abbrev IsPerm (xs ys : Arr) : Prop := IsPermutation xs ys

/-- Abstract algorithm symbol for theorem-surface obligations. -/
opaque insertionSort : Arr → Arr

/-- Canonical empty-input sortedness fact reused by packet witnesses. -/
theorem sorted_nil : Sorted [] := by
  simpa [Sorted] using sortedLE_nil

/-- Identity permutation fact reused by packet witnesses. -/
theorem perm_refl (xs : Arr) : IsPerm xs xs := by
  simpa [IsPerm] using isPermutation_refl xs

/-- Typical benchmark-facing correctness shape for insertion sort. -/
def InsertionSortSpec (f : Arr → Arr) : Prop :=
  ∀ xs, Sorted (f xs) ∧ IsPerm (f xs) xs

/-- A packaged proposition that packet theorems can target directly. -/
def insertionSortCorrect : Prop :=
  InsertionSortSpec insertionSort

end CTA.Benchmark.Sorting.InsertionSortTheory
