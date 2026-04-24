/-
Shared theory surface for BST-insert benchmark packets.

`IsBst` uses the standard search-tree shape (all left keys strictly below
the root key and all right keys strictly above). Reference `bstInsert`
matches the Rust implementation: recurse by comparison, no structural
change on a duplicate key.
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
def inorder : Tree → List Int
  | .nil => []
  | .node l k r => inorder l ++ [k] ++ inorder r

/-- Key list used by obligations (same as in-order walk for this family). -/
abbrev keys (t : Tree) : List Int := inorder t

/-- BST invariant: left keys below `k`, right keys above `k`, inductively. -/
def IsBst : Tree → Prop
  | .nil => True
  | .node l k r =>
    IsBst l ∧ IsBst r ∧ (∀ a ∈ inorder l, a < k) ∧ (∀ b ∈ inorder r, k < b)

/-- Reference-style BST insert: recurse; duplicate key is a no-op. -/
def bstInsert : Tree → Int → Tree
  | .nil, x => .node .nil x .nil
  | .node l k r, x =>
    if x < k then .node (bstInsert l x) k r
    else if k < x then .node l k (bstInsert r x)
    else .node l k r

theorem isBst_nil : IsBst Tree.nil :=
  trivial

theorem mem_inorder_node {l r : Tree} {k x : Int} :
    x ∈ inorder (.node l k r) ↔ x ∈ inorder l ∨ x = k ∨ x ∈ inorder r := by
  simp [inorder, List.mem_append, List.mem_cons]

theorem mem_inorder_bstInsert {t : Tree} {x a : Int} :
    a ∈ inorder (bstInsert t x) → a = x ∨ a ∈ inorder t := by
  intro ha
  induction t generalizing x a with
  | nil =>
      simp [bstInsert, inorder] at ha
      exact Or.inl ha
  | node l k r ihl ihr =>
      by_cases hxlk : x < k
      · simp [bstInsert, hxlk, inorder, List.mem_append, List.mem_cons] at ha ⊢
        rcases ha with ha | rfl | ha
        · rcases ihl ha with rfl | hal
          · exact Or.inl rfl
          · exact Or.inr (Or.inl hal)
        · exact Or.inr (Or.inr (Or.inl rfl))
        · exact Or.inr (Or.inr (Or.inr ha))
      · by_cases hkx : k < x
        · simp [bstInsert, hxlk, hkx, inorder, List.mem_append, List.mem_cons] at ha ⊢
          rcases ha with ha | rfl | ha
          · exact Or.inr (Or.inl ha)
          · exact Or.inr (Or.inr (Or.inl rfl))
          · rcases ihr ha with rfl | har
            · exact Or.inl rfl
            · exact Or.inr (Or.inr (Or.inr har))
        · simpa [bstInsert, hxlk, hkx, inorder] using (Or.inr ha)

/-- Inserting a key that already occurs leaves the tree unchanged. -/
theorem bstInsert_eq_self_of_mem (t : Tree) (x : Int) (hb : IsBst t) (hx : x ∈ keys t) :
    bstInsert t x = t := by
  induction t generalizing x with
  | nil =>
      simp [keys, inorder] at hx
  | node l k r ihl ihr =>
      rcases hb with ⟨hbl, hbr, hall, hallr⟩
      have hx' : x ∈ inorder (.node l k r) := by simpa [keys] using hx
      rw [mem_inorder_node] at hx'
      rcases hx' with hxl | rfl | hxr
      · have hxlk : x < k := hall x hxl
        have hkx : ¬ k < x := by
          intro hkx'
          exact Int.not_le_of_gt hxlk (Int.le_of_lt hkx')
        have hEqL : bstInsert l x = l := ihl x hbl (by simpa [keys] using hxl)
        simp [bstInsert, hxlk, hkx, hEqL]
      · simp [bstInsert, Int.lt_irrefl]
      · have hkx : k < x := hallr x hxr
        have hxlk : ¬ x < k := by
          intro hxlk'
          exact Int.not_le_of_gt hkx (Int.le_of_lt hxlk')
        have hEqR : bstInsert r x = r := ihr x hbr (by simpa [keys] using hxr)
        simp [bstInsert, hxlk, hkx, hEqR]

theorem keys_bstInsert_eq_of_mem (t : Tree) (x : Int) (hb : IsBst t) (hx : x ∈ keys t) :
    keys (bstInsert t x) = keys t := by
  simp [keys, bstInsert_eq_self_of_mem t x hb hx]

/-- Insertion preserves the BST invariant. -/
theorem bstInsert_preserves_bst (t : Tree) (x : Int) (hb : IsBst t) : IsBst (bstInsert t x) := by
  induction t generalizing x with
  | nil =>
      simp [bstInsert, IsBst, inorder]
  | node l k r ihl ihr =>
      rcases hb with ⟨hbl, hbr, hall, hallr⟩
      by_cases hxlk : x < k
      · simp [bstInsert, hxlk, IsBst]
        refine ⟨ihl x hbl, hbr, ?_, hallr⟩
        intro a ha
        rcases mem_inorder_bstInsert ha with rfl | hal
        · exact hxlk
        · exact hall a hal
      · by_cases hkx : k < x
        · simp [bstInsert, hxlk, hkx, IsBst]
          refine ⟨hbl, ihr x hbr, hall, ?_⟩
          intro a ha
          rcases mem_inorder_bstInsert ha with rfl | har
          · exact hkx
          · exact hallr a har
        · simpa [bstInsert, hxlk, hkx, IsBst] using
            (And.intro hbl (And.intro hbr (And.intro hall hallr)))

/-- Fresh insert only permutes keys (multiset / `List.Perm` viewpoint). -/
theorem keys_bstInsert_perm_of_not_mem (t : Tree) (x : Int) (hb : IsBst t) (hx : x ∉ keys t) :
    (keys (bstInsert t x)).Perm (x :: keys t) := by
  induction t generalizing x with
  | nil =>
      simp [keys, inorder, bstInsert]
  | node l k r ihl ihr =>
      rcases hb with ⟨hbl, hbr, _, _⟩
      have hx_not_l : x ∉ inorder l := by
        intro hmem
        apply hx
        simpa [keys, mem_inorder_node] using Or.inl hmem
      have hx_not_k : x ≠ k := by
        intro hEq
        apply hx
        have : x ∈ keys (.node l k r) := by
          simp [keys, mem_inorder_node, hEq]
        exact this
      have hx_not_r : x ∉ inorder r := by
        intro hmem
        apply hx
        simpa [keys, mem_inorder_node] using Or.inr (Or.inr hmem)
      rcases Int.lt_or_gt_of_ne hx_not_k with hxlk | hkx
      · have hkx_not : ¬ k < x := by
          intro hkx'
          exact Int.not_le_of_gt hxlk (Int.le_of_lt hkx')
        have hpermL : (inorder (bstInsert l x)).Perm (x :: inorder l) :=
          ihl x hbl (by simpa [keys] using hx_not_l)
        simpa [keys, inorder, bstInsert, hxlk, hkx_not, List.append_assoc] using
          (hpermL.append_right ([k] ++ inorder r))
      · have hxlk_not : ¬ x < k := by
          intro hxlk'
          exact Int.not_le_of_gt hkx (Int.le_of_lt hxlk')
        have hpermR : (inorder (bstInsert r x)).Perm (x :: inorder r) :=
          ihr x hbr (by simpa [keys] using hx_not_r)
        have happend :
            (inorder l ++ [k] ++ inorder (bstInsert r x)).Perm
              (inorder l ++ [k] ++ (x :: inorder r)) := by
          simpa [List.append_assoc] using hpermR.append_left (inorder l ++ [k])
        have hmove :
            (inorder l ++ [k] ++ (x :: inorder r)).Perm
              (x :: (inorder l ++ [k] ++ inorder r)) := by
          simpa [List.append_assoc] using
            (List.perm_middle (a := x) (l₁ := inorder l ++ [k]) (l₂ := inorder r))
        exact (List.Perm.trans
          (by simpa [keys, inorder, bstInsert, hxlk_not, hkx, List.append_assoc] using happend)
          (by simpa [keys, inorder, List.append_assoc] using hmove))

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
