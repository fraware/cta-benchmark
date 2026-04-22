/-
Scaffold for instance `trees_bst_insert_002`.
-/

import CTA.Core.Prelude
import CTA.Core.Types

namespace CTA.Benchmark.Trees.BstInsert002

open CTA.Core

/-- Binary tree with integer keys. -/
inductive Tree where
  | nil : Tree
  | node : Tree → Int → Tree → Tree
  deriving Repr, DecidableEq

/-- In-order traversal. -/
opaque inorder : Tree → List Int

/-- BST invariant: in-order traversal is strictly ascending. -/
def IsBst (t : Tree) : Prop :=
  (inorder t).Pairwise (· < ·)

/-- Set of keys present in the tree. -/
opaque keys : Tree → List Int

/-- Declarative model of the reference `bst_insert`. -/
opaque bstInsert : Tree → Int → Tree

end CTA.Benchmark.Trees.BstInsert002
