/-
Scaffold for instance `sorting_insertion_sort_002`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Core.Checkers
import CTA.Benchmark.Sorting.InsertionSortTheory

namespace CTA.Benchmark.Sorting.InsertionSort002

open CTA.Core
open CTA.Benchmark.Sorting.InsertionSortTheory

abbrev Arr := InsertionSortTheory.Arr
abbrev insertionSort := InsertionSortTheory.insertionSort
abbrev IsPerm := InsertionSortTheory.IsPerm
abbrev Sorted := InsertionSortTheory.Sorted

end CTA.Benchmark.Sorting.InsertionSort002
