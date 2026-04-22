/-
Scaffold for instance `sorting_insertion_sort_002`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Core.Checkers

namespace CTA.Benchmark.Sorting.InsertionSort002

open CTA.Core

/-- Input/output slice model. -/
abbrev Arr := List Int

/-- Declarative model of the reference `insertion_sort`: returns the sorted
    permutation of its input. Left opaque for the semantic layer. -/
opaque insertionSort : Arr → Arr

/-- Permutation predicate, exposed under this instance's namespace so that
    generated obligations spell the "output is a permutation of the input"
    property the same way across every sorting benchmark. -/
abbrev IsPerm (xs ys : Arr) : Prop := IsPermutation xs ys

/-- Sortedness predicate, re-exported so generated obligations can refer to
    `Sorted` without having to know that the underlying definition lives in
    `CTA.Core`. -/
abbrev Sorted (xs : Arr) : Prop := SortedLE xs

end CTA.Benchmark.Sorting.InsertionSort002
