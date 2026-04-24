/-
Shared theory surface for binary-search benchmark packets.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Core.Util
import CTA.Core.Checkers

namespace CTA.Benchmark.Arrays.BinarySearchTheory

open CTA.Core

/-- Input slice abstracted as `List Int` (indexed by `Nat`). -/
abbrev Arr := List Int

/-- Result type of `binarySearch`. -/
abbrev SearchResult := Option Nat

/-- Definition-backed model for binary search.
Returns the first index whose value equals `target`, if present. -/
def binarySearch (a : Arr) (target : Int) : SearchResult :=
  (a.enum.find? (fun p => p.2 == target)).map (fun p => p.1)

/-- Valid index precondition for this family. -/
abbrev IndexValid (i : Nat) (a : Arr) : Prop := InBounds i a.length

/-- Sortedness precondition for this family. -/
abbrev Sorted (a : Arr) : Prop := SortedLE a

end CTA.Benchmark.Arrays.BinarySearchTheory
