/-
Scaffold for instance `arrays_max_subarray_004`.

Keep this file minimal: declare the namespace, introduce type aliases and
function signatures that reference obligations can talk about, and nothing
else. Generated obligations import this file and reference its names.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Core.Util
import CTA.Benchmark.Arrays.MaxSubarrayTheory

namespace CTA.Benchmark.Arrays.MaxSubarray004

open CTA.Core
open CTA.Benchmark.Arrays.MaxSubarrayTheory

abbrev Arr := MaxSubarrayTheory.Arr
abbrev maxSubarray := MaxSubarrayTheory.maxSubarray
abbrev sliceSum := MaxSubarrayTheory.sliceSum

end CTA.Benchmark.Arrays.MaxSubarray004
