/-
Scaffold for instance `arrays_max_subarray_002`.

Keep this file minimal: declare the namespace, introduce type aliases and
function signatures that reference obligations can talk about, and nothing
else. Generated obligations import this file and reference its names.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Core.Util
import CTA.Benchmark.Arrays.MaxSubarrayTheory

namespace CTA.Benchmark.Arrays.MaxSubarray002

open CTA.Core
open CTA.Benchmark.Arrays.MaxSubarrayTheory

/-- Input slice abstracted as `List Int`. -/
abbrev Arr := MaxSubarrayTheory.Arr

/-- The declarative target: the maximum sum over all contiguous non-empty
    subslices of `arr`. Left opaque so obligations reason about it abstractly. -/
abbrev maxSubarray := MaxSubarrayTheory.maxSubarray

/-- Sum of a contiguous slice `arr.drop i |>.take (j - i)`; semantic helper
    for obligations that need to mention the witnessed subslice sum. -/
abbrev sliceSum := MaxSubarrayTheory.sliceSum

end CTA.Benchmark.Arrays.MaxSubarray002
