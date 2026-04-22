/-
CTA.Core.Types
==============
Shared benchmark-level abstractions referenced by instance scaffolds.
-/

import CTA.Core.Prelude

namespace CTA.Core

/-- A finite, zero-indexed integer array modeled as a `List Int`. -/
abbrev IntList := List Int

/-- An edge of a directed weighted graph over finite vertex labels `V`. -/
structure Edge (V : Type u) where
  src : V
  dst : V
  weight : Int
  deriving Repr

/-- Predicate: a list is sorted nondecreasingly by `≤`. -/
def SortedLE (l : List Int) : Prop :=
  l.Pairwise (· ≤ ·)

end CTA.Core
