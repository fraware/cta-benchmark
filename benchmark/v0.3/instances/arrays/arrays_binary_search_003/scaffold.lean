/-
Scaffold for instance `arrays_binary_search_003`.

Keep this file minimal: declare the namespace, introduce type aliases and
function signatures that reference obligations can talk about, and nothing
else. Generated obligations import this file and reference its names.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Core.Util
import CTA.Core.Checkers
import CTA.Benchmark.Arrays.BinarySearchTheory

namespace CTA.Benchmark.Arrays.BinarySearch003

open CTA.Core
open CTA.Benchmark.Arrays.BinarySearchTheory

abbrev Arr := BinarySearchTheory.Arr
abbrev SearchResult := BinarySearchTheory.SearchResult
abbrev binarySearch := BinarySearchTheory.binarySearch
abbrev IndexValid := BinarySearchTheory.IndexValid
abbrev Sorted := BinarySearchTheory.Sorted

end CTA.Benchmark.Arrays.BinarySearch003
