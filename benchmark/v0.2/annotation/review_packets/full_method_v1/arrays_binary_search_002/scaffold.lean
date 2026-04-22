/-
Scaffold for instance `arrays_binary_search_002`.

Keep this file minimal: declare the namespace, introduce type aliases and
function signatures that reference obligations can talk about, and nothing
else. Generated obligations import this file and reference its names.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Core.Util
import CTA.Core.Checkers

namespace CTA.Benchmark.Arrays.BinarySearch002

open CTA.Core

/-- Input slice abstracted as `List Int` (indexed by `Nat`). -/
abbrev Arr := List Int

/-- Result type of `binarySearch`. -/
abbrev SearchResult := Option Nat

/-- Declarative model of the reference `binary_search`.
    This is the *target* that faithful obligations talk about, and is left
    `noncomputable` and undefined-on-inputs so that `.lean` generated files do
    not leak executable behavior into the semantic layer. -/
opaque binarySearch : Arr → Int → SearchResult

/-- Valid index precondition for this instance, expressed via
    `CTA.Core.InBounds`. Exposed so that generated obligations and the
    behavioral harness agree on the exact index-bounds vocabulary. -/
abbrev IndexValid (i : Nat) (a : Arr) : Prop := InBounds i a.length

/-- Sortedness precondition for this instance, re-exported under a name that
    lives in the instance namespace. The underlying definition is
    `CTA.Core.SortedLE`, guaranteeing a single source of truth across the
    benchmark. -/
abbrev Sorted (a : Arr) : Prop := SortedLE a

end CTA.Benchmark.Arrays.BinarySearch002
