/-
Scaffold for instance `sorting_merge_sort_002`.
-/

import CTA.Core.Prelude
import CTA.Core.Types

namespace CTA.Benchmark.Sorting.MergeSort002

open CTA.Core

/-- Input/output slice model. -/
abbrev Arr := List Int

/-- Declarative model of the reference `merge_sort`: returns the sorted
    permutation of its input. Left opaque for the semantic layer. -/
opaque mergeSort : Arr → Arr

end CTA.Benchmark.Sorting.MergeSort002
