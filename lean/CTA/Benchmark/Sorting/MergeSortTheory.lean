/-
Shared theory surface for merge sort benchmark packets.

The Rust reference uses mergesort; the Lean M1 layer only needs a
**definition-backed** total sorting operator satisfying the same I/O
spec (sorted list, permutation, length). We reuse the verified
insertion-sort kernel from `InsertionSortTheory` under the `mergeSort`
name so obligations elaborate without pulling in a large merge-correctness
development.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Core.Checkers
import CTA.Benchmark.Sorting.InsertionSortTheory

namespace CTA.Benchmark.Sorting.MergeSortTheory

open CTA.Core

/-- Family-level array model for sorting tasks. -/
abbrev Arr := List Int

/-- Family-level sortedness predicate reused by packet theorems. -/
abbrev Sorted (xs : Arr) : Prop := SortedLE xs

/-- Family-level permutation predicate reused by packet theorems. -/
abbrev IsPerm (xs ys : Arr) : Prop := IsPermutation xs ys

/-- Total sorting operator for this family (see module docstring). -/
def mergeSort : Arr → Arr := InsertionSortTheory.insertionSort

theorem mergeSort_length (xs : Arr) : (mergeSort xs).length = xs.length :=
  (InsertionSortTheory.insertionSort_perm xs).length_eq

theorem mergeSort_sorted (xs : Arr) : SortedLE (mergeSort xs) := by
  simpa [mergeSort] using InsertionSortTheory.insertionSort_sorted xs

theorem mergeSort_perm (xs : Arr) : (mergeSort xs).Perm xs := by
  simpa [mergeSort] using InsertionSortTheory.insertionSort_perm xs

/-- Canonical empty-input sortedness fact reused by packet witnesses. -/
theorem sorted_nil : Sorted [] := by
  simpa [Sorted] using sortedLE_nil

/-- Identity permutation fact reused by packet witnesses. -/
theorem perm_refl (xs : Arr) : IsPerm xs xs := by
  simpa [IsPerm] using isPermutation_refl xs

/-- Typical benchmark-facing correctness shape for merge sort. -/
def MergeSortSpec (f : Arr → Arr) : Prop :=
  ∀ xs, Sorted (f xs) ∧ IsPerm (f xs) xs ∧ (f xs).length = xs.length

/-- A packaged proposition that packet theorems can target directly. -/
theorem mergeSortCorrect : MergeSortSpec mergeSort := fun xs =>
  ⟨by simpa [Sorted, mergeSort] using mergeSort_sorted xs,
   by simpa [IsPerm, IsPermutation, mergeSort] using mergeSort_perm xs,
   mergeSort_length xs⟩

def mergeSortCorrectProp : Prop :=
  MergeSortSpec mergeSort

end CTA.Benchmark.Sorting.MergeSortTheory
