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

theorem mem_inorder_node {l r : Tree} {k x : Int} :
    x ∈ inorder (.node l k r) ↔ x ∈ inorder l ∨ x = k ∨ x ∈ inorder r := by
  simp [inorder, List.mem_append, List.mem_cons]

theorem isBst_left_of_node {l r : Tree} {x : Int} (h : IsBst (.node l x r)) : IsBst l := by
  simp [IsBst, inorder, List.append_assoc] at h
  simpa [IsBst] using (List.pairwise_append.mp h).1

theorem isBst_right_of_node {l r : Tree} {x : Int} (h : IsBst (.node l x r)) : IsBst r := by
  simpa [IsBst, inorder, List.append_assoc, List.singleton_append] using
    List.Pairwise.sublist (List.sublist_append_right (inorder l ++ [x]) (inorder r))
      (by simpa [IsBst, inorder, List.append_assoc, List.singleton_append] using h)

private theorem forall_lt_root_left {l r : Tree} {x : Int} (h : IsBst (.node l x r)) :
    ∀ a ∈ inorder l, a < x := by
  intro a ha
  have hpw : (inorder l ++ ([x] ++ inorder r)).Pairwise (· < ·) := by
    simpa [IsBst, inorder, List.append_assoc, List.singleton_append] using h
  have trip := (List.pairwise_append.mp hpw).2.2 a ha x (by simp [List.mem_append, List.mem_cons])
  simpa using trip

private theorem forall_gt_root_right {l r : Tree} {x : Int} (h : IsBst (.node l x r)) :
    ∀ b ∈ inorder r, x < b := by
  intro b hb
  have hpw : ((inorder l ++ [x]) ++ inorder r).Pairwise (· < ·) := by
    simpa [IsBst, inorder, List.append_assoc, List.singleton_append] using h
  have hdec := List.pairwise_append.mp hpw
  have hxmem : x ∈ inorder l ++ [x] := by simp [List.mem_append, List.mem_cons]
  have trip := hdec.2.2 x hxmem b hb
  simpa using trip

theorem mem_inorder_left_of_lt {l r : Tree} {x p : Int}
    (hbst : IsBst (.node l x r)) (hp : p ∈ inorder (.node l x r)) (hlt : p < x) :
    p ∈ inorder l := by
  rw [mem_inorder_node] at hp
  rcases hp with hpL | rfl | hpR
  · exact hpL
  · exact absurd hlt (Int.lt_irrefl p)
  · have hxp := forall_gt_root_right hbst p hpR
    exact absurd (Int.lt_trans hlt hxp) (Int.lt_irrefl p)

theorem mem_inorder_right_of_gt {l r : Tree} {x p : Int}
    (hbst : IsBst (.node l x r)) (hp : p ∈ inorder (.node l x r)) (hlt : x < p) :
    p ∈ inorder r := by
  rw [mem_inorder_node] at hp
  rcases hp with hpL | rfl | hpR
  · have hxp := forall_lt_root_left hbst p hpL
    exact absurd (Int.lt_trans hxp hlt) (Int.lt_irrefl p)
  · exact absurd hlt (Int.lt_irrefl p)
  · exact hpR

theorem mem_inorder_of_subtree {sub t : Tree} (h : IsSubtree sub t) (p : Int) :
    HasKey sub p → HasKey t p := by
  intro hk
  induction t generalizing sub with
  | nil =>
      rcases h with rfl | hF
      · simp [HasKey, inorder] at hk
      · cases hF
  | node l x r ihl ihr =>
    rcases h with rfl | hL | hR
    · simpa [HasKey] using hk
    · simp [HasKey, inorder, List.mem_append, List.mem_cons]
      left
      exact ihl hL hk
    · simp [HasKey, inorder, List.mem_append, List.mem_cons]
      right; right
      exact ihr hR hk

private theorem lca_key_mem_inorder (t : Tree) : ∀ p q a, lcaBst t p q = some a → a ∈ inorder t := by
  induction t with
  | nil =>
      intro p q a h
      simp [lcaBst] at h
  | node l x r ihl ihr =>
      intro p q a h
      simp [lcaBst] at h
      by_cases hL : p < x ∧ q < x <;> simp [hL] at h
      · have ih := ihl p q a h
        simp [inorder, List.mem_append, List.mem_cons]
        left
        exact ih
      · by_cases hR : x < p ∧ x < q <;> simp [hR] at h
        · have ih := ihr p q a h
          simp [inorder, List.mem_append, List.mem_cons]
          right; right
          exact ih
        · cases h
          simp [inorder, List.mem_append, List.mem_cons]

private theorem subtreeRootedAt_node_left {l r : Tree} {x p q a : Int}
    (hbst : IsBst (.node l x r)) (ha : lcaBst l p q = some a) :
    subtreeRootedAt (.node l x r) a = subtreeRootedAt l a := by
  have hmem : a ∈ inorder l := lca_key_mem_inorder l p q a ha
  have hax : a < x := forall_lt_root_left hbst a hmem
  have hne : ¬a = x := Int.ne_of_lt hax
  simp [subtreeRootedAt, hne, show a < x by exact hax]

private theorem subtreeRootedAt_node_right {l r : Tree} {x p q a : Int}
    (hbst : IsBst (.node l x r)) (ha : lcaBst r p q = some a) :
    subtreeRootedAt (.node l x r) a = subtreeRootedAt r a := by
  have hmem : a ∈ inorder r := lca_key_mem_inorder r p q a ha
  have hxa : x < a := forall_gt_root_right hbst a hmem
  have hne : ¬a = x := Ne.symm (Int.ne_of_lt hxa)
  have hnot_lt : ¬a < x := by
    intro hlt
    exact absurd (Int.lt_trans hlt hxa) (Int.lt_irrefl a)
  simp [subtreeRootedAt, hne, hnot_lt]

private theorem proper_subtree_of_node {l r : Tree} {x : Int} {sub : Tree}
    (h : IsProperSubtree sub (.node l x r)) :
    IsSubtree sub l ∨ IsSubtree sub r := by
  rcases h with ⟨hsub, hne⟩
  rcases hsub with rfl | hL | hR
  · exact absurd rfl hne
  · exact Or.inl hL
  · exact Or.inr hR

theorem lca_subtree_witness
    (t : Tree) (p q a : Int)
    (_hbst : IsBst t) (hp : HasKey t p) (hq : HasKey t q)
    (_hr : lcaBst t p q = some a) :
    ∃ sub : Tree, IsSubtree sub t ∧ HasKey sub p ∧ HasKey sub q := by
  have hst : IsSubtree t t := by
    unfold IsSubtree
    exact Or.inl rfl
  refine ⟨t, ?_⟩
  exact ⟨hst, hp, hq⟩

theorem lca_no_proper_descendant_both_keys
    (t : Tree) (p q a : Int)
    (hbst : IsBst t) (hp : HasKey t p) (hq : HasKey t q)
    (hr : lcaBst t p q = some a) :
    ∀ sub : Tree,
      IsProperSubtree sub (subtreeRootedAt t a) →
      ¬ (HasKey sub p ∧ HasKey sub q) := by
  intro sub hps ⟨hpS, hqS⟩
  induction t generalizing sub hpS hqS with
  | nil =>
      simp [lcaBst] at hr
  | node l x r ihl ihr =>
    simp [lcaBst] at hr
    by_cases h₁ : p < x ∧ q < x
    · rw [if_pos h₁] at hr
      have hbstl := isBst_left_of_node hbst
      have hpL : HasKey l p :=
        mem_inorder_left_of_lt hbst hp h₁.1
      have hqL : HasKey l q :=
        mem_inorder_left_of_lt hbst hq h₁.2
      rw [subtreeRootedAt_node_left hbst hr] at hps
      exact ihl hbstl hpL hqL hr sub hps hpS hqS
    · rw [if_neg h₁] at hr
      by_cases h₂ : x < p ∧ x < q
      · rw [if_pos h₂] at hr
        have hbstr := isBst_right_of_node hbst
        have hpR : HasKey r p :=
          mem_inorder_right_of_gt hbst hp h₂.1
        have hqR : HasKey r q :=
          mem_inorder_right_of_gt hbst hq h₂.2
        rw [subtreeRootedAt_node_right hbst hr] at hps
        exact ihr hbstr hpR hqR hr sub hps hpS hqS
      · rw [if_neg h₂] at hr
        have haEq : a = x := by cases hr; rfl
        rw [haEq] at hps
        simp [subtreeRootedAt] at hps
        rcases proper_subtree_of_node hps with hsubL | hsubR
        · have hpL : HasKey l p := mem_inorder_of_subtree hsubL p hpS
          have hqL : HasKey l q := mem_inorder_of_subtree hsubL q hqS
          have hpx : p < x := forall_lt_root_left hbst p hpL
          have hqx : q < x := forall_lt_root_left hbst q hqL
          exact h₁ ⟨hpx, hqx⟩
        · have hpR : HasKey r p := mem_inorder_of_subtree hsubR p hpS
          have hqR : HasKey r q := mem_inorder_of_subtree hsubR q hqS
          have hpx : x < p := forall_gt_root_right hbst p hpR
          have hqx : x < q := forall_gt_root_right hbst q hqR
          exact h₂ ⟨hpx, hqx⟩

theorem lca_self_key
    (t : Tree) (p : Int)
    (hbst : IsBst t) (hp : HasKey t p) :
    lcaBst t p p = some p := by
  induction t generalizing p with
  | nil =>
      simp [HasKey, inorder] at hp
  | node l x r ihl ihr =>
      rcases Int.lt_trichotomy p x with hpl | hpe | hpr
      · have hp' : HasKey l p :=
          mem_inorder_left_of_lt hbst hp hpl
        have hbstl := isBst_left_of_node hbst
        have h₁ : p < x ∧ p < x := ⟨hpl, hpl⟩
        simp [lcaBst, h₁]
        exact ihl p hbstl hp'
      · simp [lcaBst, hpe]
      · have hp' : HasKey r p :=
          mem_inorder_right_of_gt hbst hp hpr
        have hbstr := isBst_right_of_node hbst
        have hnpx : ¬ p < x := by
          intro hp_lt
          exact absurd (Int.lt_trans hp_lt hpr) (Int.lt_irrefl p)
        simp [lcaBst, if_neg hnpx, if_pos hpr]
        exact ihr p hbstr hp'

end CTA.Benchmark.Trees.LowestCommonAncestorTheory
