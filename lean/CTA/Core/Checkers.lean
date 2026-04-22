/-
CTA.Core.Checkers
=================
Reusable specification predicates and small utility lemmas consumed by
benchmark scaffolds. This module is deliberately dependency-light:
everything here elaborates with only core Lean plus `CTA.Core.Prelude`
and `CTA.Core.Types`. No Mathlib imports.

Scaffolds import this module (transitively via `CTA`) so that generated
obligations can state their properties in terms of named predicates
rather than inlining bespoke definitions, which keeps the Lean surface
of the benchmark small and machine-auditable.
-/

import CTA.Core.Prelude
import CTA.Core.Types

namespace CTA.Core

/-- `IsPermutation xs ys` holds iff `ys` is a reordering of `xs`. -/
def IsPermutation (xs ys : List Int) : Prop := List.Perm xs ys

/-- `InBounds i n` is the standard zero-based index precondition
    `i < n`, expressed as a reusable predicate. -/
def InBounds (i n : Nat) : Prop := i < n

/-- `NonNegative l` holds when every element of `l` is `≥ 0`. Used by
    the graph / greedy instances to pin weight-nonnegativity preconditions. -/
def NonNegative (l : List Int) : Prop := ∀ x, x ∈ l → 0 ≤ x

/-- Multiset equality phrased via element counts. Equivalent to
    `IsPermutation` for `Int` lists and often more convenient for
    generated proofs. -/
def SameMultiset (xs ys : List Int) : Prop :=
  ∀ v : Int, xs.count v = ys.count v

/-- The empty list is trivially `SortedLE`. -/
theorem sortedLE_nil : SortedLE [] :=
  List.Pairwise.nil

/-- Reflexivity for `IsPermutation`. -/
theorem isPermutation_refl (xs : List Int) : IsPermutation xs xs :=
  List.Perm.refl xs

/-- Reflexivity for `SameMultiset`. -/
theorem sameMultiset_refl (xs : List Int) : SameMultiset xs xs :=
  fun _ => rfl

/-- Canonical sanity template used by scaffolds whose obligation set has
    not yet been exercised by a generator. Unlike the previous trivial
    `True` placeholder, this lemma references benchmark vocabulary
    (`IsPermutation`), so it fails loudly if `CTA.Core` is broken. -/
theorem scaffold_sanity : IsPermutation ([] : List Int) [] :=
  isPermutation_refl []

end CTA.Core
