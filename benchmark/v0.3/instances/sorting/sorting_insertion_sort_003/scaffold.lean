/-
Scaffold for instance `sorting_insertion_sort_003`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Core.Checkers
import CTA.Benchmark.Sorting.InsertionSortTheory

namespace CTA.Benchmark.Sorting.InsertionSort003

open CTA.Core
open CTA.Benchmark.Sorting.InsertionSortTheory

/-- Family-level list model reused across insertion-sort packets. -/
abbrev Arr := InsertionSortTheory.Arr

/-- Shared insertion-sort algorithm symbol. -/
abbrev insertionSort := InsertionSortTheory.insertionSort

/-- Shared permutation predicate for benchmark-facing obligations. -/
abbrev IsPerm := InsertionSortTheory.IsPerm

/-- Shared sortedness predicate for benchmark-facing obligations. -/
abbrev Sorted := InsertionSortTheory.Sorted

end CTA.Benchmark.Sorting.InsertionSort003
