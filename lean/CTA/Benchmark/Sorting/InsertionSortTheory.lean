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

/-- Insert one element into a sorted list, preserving sortedness. -/
def insert (x : Int) : Arr → Arr
  | [] => [x]
  | y :: ys => if x ≤ y then x :: y :: ys else y :: insert x ys

/-- Insertion sort on `Arr`. -/
def insertionSort : Arr → Arr
  | [] => []
  | x :: xs => insert x (insertionSort xs)

theorem mem_insert {x z : Int} {xs : Arr} (hz : z ∈ insert x xs) : z = x ∨ z ∈ xs := by
  induction xs with
  | nil => simp [insert] at hz; left; exact hz
  | cons y ys ih =>
      by_cases hxy : x ≤ y
      · simp [insert, hxy] at hz
        rcases hz with hz1 | hz2 | hz3
        · left; exact hz1
        · subst hz2; right; exact List.Mem.head _
        · right; exact List.Mem.tail _ hz3
      · simp [insert, hxy] at hz
        rcases hz with hz' | hz'
        · subst hz'; right; exact List.Mem.head _
        · rcases ih hz' with hzx | hys
          · left; exact hzx
          · right; exact List.Mem.tail _ hys

theorem insert_perm (x : Int) (xs : Arr) : (insert x xs).Perm (x :: xs) := by
  induction xs with
  | nil => simp [insert]
  | cons y ys ih =>
      by_cases hxy : x ≤ y
      · simp [insert, hxy]
      · simp [insert, hxy]
        exact List.Perm.trans (List.Perm.cons y ih) (List.Perm.swap x y ys)

theorem insert_sorted (x : Int) (xs : Arr) (hxs : SortedLE xs) : SortedLE (insert x xs) := by
  induction xs with
  | nil => simp [insert, SortedLE, List.Pairwise]
  | cons y ys ih =>
      have hpair := hxs
      simp only [SortedLE, List.pairwise_cons] at hpair
      rcases hpair with ⟨hy, hys⟩
      by_cases hxy : x ≤ y
      · have hdef : insert x (y :: ys) = x :: y :: ys := by simp [insert, hxy]
        rw [hdef]
        simp only [SortedLE, List.pairwise_cons]
        constructor
        · intro z hz
          rw [List.mem_cons] at hz
          cases hz with
          | inl hz_eq_y =>
              rw [hz_eq_y]; exact hxy
          | inr hz_in_ys =>
              exact Int.le_trans hxy (hy z hz_in_ys)
        · simpa [SortedLE] using hxs
      · have hdef : insert x (y :: ys) = y :: insert x ys := by simp [insert, hxy]
        rw [hdef]
        have ylex : y ≤ x := Int.le_of_lt (Int.lt_of_not_ge hxy)
        simp only [SortedLE, List.pairwise_cons]
        constructor
        · intro z hz
          exact (mem_insert hz).elim (fun hzx => by subst hzx; exact ylex) (hy z)
        · exact ih hys

theorem insertionSort_sorted (xs : Arr) : SortedLE (insertionSort xs) := by
  induction xs with
  | nil => simp [insertionSort, SortedLE, List.Pairwise]
  | cons x xs ih =>
      simp [insertionSort]
      exact insert_sorted x (insertionSort xs) ih

theorem insertionSort_perm (xs : Arr) : (insertionSort xs).Perm xs := by
  induction xs with
  | nil => simp [insertionSort]
  | cons x xs ih =>
      simp [insertionSort]
      refine List.Perm.trans (insert_perm x (insertionSort xs)) ?_
      exact List.Perm.cons x ih

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
theorem insertionSortCorrect : InsertionSortSpec insertionSort := fun xs =>
  ⟨by simpa [Sorted] using insertionSort_sorted xs,
   by simpa [IsPerm, IsPermutation] using insertionSort_perm xs⟩

def insertionSortCorrectProp : Prop :=
  InsertionSortSpec insertionSort

end CTA.Benchmark.Sorting.InsertionSortTheory
