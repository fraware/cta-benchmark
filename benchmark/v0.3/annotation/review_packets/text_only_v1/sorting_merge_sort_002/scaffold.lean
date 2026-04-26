/-
Scaffold for instance `sorting_merge_sort_002`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Core.Checkers
import CTA.Benchmark.Sorting.MergeSortTheory

namespace CTA.Benchmark.Sorting.MergeSort002

open CTA.Core
open CTA.Benchmark.Sorting.MergeSortTheory

abbrev Arr := MergeSortTheory.Arr
abbrev mergeSort := MergeSortTheory.mergeSort
abbrev IsPerm := MergeSortTheory.IsPerm
abbrev Sorted := MergeSortTheory.Sorted

end CTA.Benchmark.Sorting.MergeSort002
