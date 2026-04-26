/-
Scaffold for instance `sorting_merge_sort_005`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Core.Checkers
import CTA.Benchmark.Sorting.MergeSortTheory

namespace CTA.Benchmark.Sorting.MergeSort005

open CTA.Core
open CTA.Benchmark.Sorting.MergeSortTheory

/-- Family-level list model reused across merge-sort packets. -/
abbrev Arr := MergeSortTheory.Arr

/-- Shared merge-sort algorithm symbol. -/
abbrev mergeSort := MergeSortTheory.mergeSort

/-- Shared permutation predicate for benchmark-facing obligations. -/
abbrev IsPerm := MergeSortTheory.IsPerm

/-- Shared sortedness predicate for benchmark-facing obligations. -/
abbrev Sorted := MergeSortTheory.Sorted

end CTA.Benchmark.Sorting.MergeSort005
