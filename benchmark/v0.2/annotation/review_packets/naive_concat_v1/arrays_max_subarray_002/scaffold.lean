/-
Scaffold for instance `arrays_max_subarray_002`.

Keep this file minimal: declare the namespace, introduce type aliases and
function signatures that reference obligations can talk about, and nothing
else. Generated obligations import this file and reference its names.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Core.Util

namespace CTA.Benchmark.Arrays.MaxSubarray002

open CTA.Core

/-- Input slice abstracted as `List Int`. -/
abbrev Arr := List Int

/-- The declarative target: the maximum sum over all contiguous non-empty
    subslices of `arr`. Left opaque so obligations reason about it abstractly. -/
opaque maxSubarray : Arr → Int

/-- Sum of a contiguous slice `arr.drop i |>.take (j - i)`; semantic helper
    for obligations that need to mention the witnessed subslice sum. -/
opaque sliceSum : Arr → Nat → Nat → Int

end CTA.Benchmark.Arrays.MaxSubarray002
