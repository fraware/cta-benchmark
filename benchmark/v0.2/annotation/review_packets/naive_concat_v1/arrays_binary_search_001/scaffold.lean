/-
Scaffold for instance `arrays_binary_search_001`.

Keep this file minimal: declare the namespace, introduce type aliases and
function signatures that reference obligations can talk about, and nothing
else. Generated obligations import this file and reference its names.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Core.Util
import CTA.Core.Checkers
import CTA.Benchmark.Arrays.BinarySearchTheory

namespace CTA.Benchmark.Arrays.BinarySearch001

open CTA.Core
open CTA.Benchmark.Arrays.BinarySearchTheory

/-- Input slice abstracted as `List Int` (indexed by `Nat`). -/
abbrev Arr := BinarySearchTheory.Arr

/-- Result type of `binarySearch`. -/
abbrev SearchResult := BinarySearchTheory.SearchResult

/-- Declarative model of the reference `binary_search`.
    This is the *target* that faithful obligations talk about, and is left
    `noncomputable` and undefined-on-inputs so that `.lean` generated files do
    not leak executable behavior into the semantic layer. -/
abbrev binarySearch := BinarySearchTheory.binarySearch

/-- Valid index precondition for this instance, expressed via
    `CTA.Core.InBounds`. Exposed so that generated obligations and the
    behavioral harness agree on the exact index-bounds vocabulary. -/
abbrev IndexValid := BinarySearchTheory.IndexValid

/-- Sortedness precondition for this instance, re-exported under a name that
    lives in the instance namespace. The underlying definition is
    `CTA.Core.SortedLE`, guaranteeing a single source of truth across the
    benchmark. -/
abbrev Sorted := BinarySearchTheory.Sorted

end CTA.Benchmark.Arrays.BinarySearch001
