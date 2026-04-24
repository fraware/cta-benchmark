/-
Shared theory surface for BST-insert benchmark packets.
-/

import CTA.Core.Prelude
import CTA.Core.Types

namespace CTA.Benchmark.Trees.BstInsertTheory

/-- Binary tree with integer keys. -/
inductive Tree where
  | nil : Tree
  | node : Tree → Int → Tree → Tree
  deriving Repr, DecidableEq, Inhabited

/-- In-order traversal of keys. -/
opaque inorder : Tree → List Int

/-- BST invariant used by packet theorems. -/
def IsBst (t : Tree) : Prop :=
  (inorder t).Pairwise (· < ·)

/-- Key multiset projection (opaque for M1 benchmark layer). -/
opaque keys : Tree → List Int

/-- Abstract insertion operator used by packet obligations. -/
opaque bstInsert : Tree → Int → Tree

/-- Base empty-tree fact exported for packet-level proofs. -/
axiom isBst_nil : IsBst Tree.nil

/-- Canonical preservation shape used by packet-facing theorems. -/
def BstInsertPreservesBst : Prop :=
  ∀ t x, IsBst t → IsBst (bstInsert t x)

/-- Canonical key-accounting shape for absent-key insertion. -/
def BstInsertAddsFreshKey : Prop :=
  ∀ t x, x ∉ keys t → (keys (bstInsert t x)).count x = (keys t).count x + 1

/-- Canonical idempotence shape for present-key insertion. -/
def BstInsertIdempotentOnPresentKey : Prop :=
  ∀ t x, x ∈ keys t → keys (bstInsert t x) = keys t

end CTA.Benchmark.Trees.BstInsertTheory
