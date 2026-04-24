/-
Shared theory surface for max-subarray benchmark packets.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Core.Util

namespace CTA.Benchmark.Arrays.MaxSubarrayTheory

open CTA.Core

/-- Input slice abstracted as `List Int`. -/
abbrev Arr := List Int

/-- Sum of contiguous slice `arr[i:j]` (half-open). -/
def sliceSum (arr : Arr) (i j : Nat) : Int :=
  (arr.drop i).take (j - i) |>.foldl (fun acc x => acc + x) 0

/-- Definition-backed placeholder model for max-subarray objective. -/
def maxSubarray (arr : Arr) : Int :=
  arr.foldl (fun acc x => max acc x) 0

end CTA.Benchmark.Arrays.MaxSubarrayTheory
