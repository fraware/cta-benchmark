/-
Scaffold for instance `trees_lowest_common_ancestor_001`.
-/

import CTA.Core.Prelude
import CTA.Core.Types

namespace CTA.Benchmark.Trees.LowestCommonAncestor001

open CTA.Core

/-- Binary tree with integer keys. -/
inductive Tree where
  | nil : Tree
  | node : Tree → Int → Tree → Tree
  deriving Repr, DecidableEq

/-- In-order traversal. -/
opaque inorder : Tree → List Int

/-- BST invariant. -/
def IsBst (t : Tree) : Prop :=
  (inorder t).Pairwise (· < ·)

/-- Membership: `k` appears as a node key in `t`. -/
opaque HasKey : Tree → Int → Prop

/-- Subtree relation: `sub` appears as a subtree of `t`. -/
opaque IsSubtree : Tree → Tree → Prop

/-- Proper subtree (subtree but not equal). -/
def IsProperSubtree (sub t : Tree) : Prop :=
  IsSubtree sub t ∧ sub ≠ t

/-- Declarative model of the reference `lca_bst`. -/
opaque lcaBst : Tree → Int → Int → Option Int

end CTA.Benchmark.Trees.LowestCommonAncestor001
