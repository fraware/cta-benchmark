/-
Shared theory surface for lowest-common-ancestor benchmark packets.
-/

import CTA.Core.Prelude
import CTA.Core.Types

namespace CTA.Benchmark.Trees.LowestCommonAncestorTheory

open CTA.Core

/-- Binary tree with integer keys. -/
inductive Tree where
  | nil : Tree
  | node : Tree → Int → Tree → Tree
  deriving Repr, DecidableEq, Inhabited

/-- In-order traversal. -/
def inorder : Tree → List Int
  | .nil => []
  | .node l x r => inorder l ++ [x] ++ inorder r

/-- BST invariant via in-order sortedness. -/
def IsBst (t : Tree) : Prop :=
  (inorder t).Pairwise (· < ·)

/-- Membership: `k` appears as a node key in `t`. -/
def HasKey (t : Tree) (k : Int) : Prop :=
  k ∈ inorder t

/-- Subtree relation: `sub` appears as a (possibly equal) subtree of `t`. -/
def IsSubtree : Tree → Tree → Prop
  | sub, t => sub = t ∨
      match t with
      | .nil => False
      | .node l _ r => IsSubtree sub l ∨ IsSubtree sub r

/-- Proper subtree (subtree but not equal). -/
def IsProperSubtree (sub t : Tree) : Prop :=
  IsSubtree sub t ∧ sub ≠ t

/-- Subtree rooted at a key (partial lookup; `nil` when absent). -/
def subtreeRootedAt : Tree → Int → Tree
  | .nil, _ => .nil
  | t@(.node l x r), a =>
      if a = x then t
      else if a < x then subtreeRootedAt l a
      else subtreeRootedAt r a

/-- Declarative model of BST-LCA (returns key when found). -/
def lcaBst : Tree → Int → Int → Option Int
  | .nil, _, _ => none
  | .node l x r, a, b =>
      if a < x ∧ b < x then lcaBst l a b
      else if x < a ∧ x < b then lcaBst r a b
      else some x

end CTA.Benchmark.Trees.LowestCommonAncestorTheory
