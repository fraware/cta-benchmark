/-
0/1 knapsack theory: exact finite optimum over subsequences of `List.range weights.length`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import Init.Omega

namespace CTA.Benchmark.DP.KnapsackTheory

open CTA.Core

abbrev Weights := List Nat
abbrev Values := List Nat
abbrev Selection := List Nat

def sublists {α : Type} : List α → List (List α)
  | [] => [[]]
  | x :: xs =>
    let rest := sublists xs
    rest ++ rest.map (fun ys => x :: ys)

private theorem mem_sublists_iff_sublist {α : Type} {xs ys : List α} :
    xs ∈ sublists ys ↔ List.Sublist xs ys := by
  induction ys generalizing xs with
  | nil =>
      refine ⟨?_, ?_⟩
      · intro h
        simp [sublists, List.mem_singleton] at h
        subst h
        exact List.nil_sublist []
      · intro h
        rw [List.sublist_nil] at h
        subst h
        simp [sublists, List.mem_singleton]
  | cons y ys ih =>
      constructor
      · intro h
        simp [sublists, List.mem_append, List.mem_map] at h
        rcases h with h | ⟨t, ht, rfl⟩
        · exact List.Sublist.cons y (ih.mp h)
        · exact List.Sublist.cons₂ y (ih.mp ht)
      · intro h
        cases h with
        | cons _ h' =>
            simp [sublists, ih.mpr h', List.mem_append]
        | cons₂ _ h' =>
            simp [sublists, ih.mpr h', List.mem_append, List.mem_map]

private theorem mem_sublists_of_sublist_trans {α : Type} {xs ys zs : List α}
    (hxy : xs ∈ sublists ys) (hyz : List.Sublist ys zs) : xs ∈ sublists zs := by
  exact Iff.mpr mem_sublists_iff_sublist ((Iff.mp mem_sublists_iff_sublist hxy).trans hyz)

private theorem nil_mem_sublists_range (n : Nat) : [] ∈ sublists (List.range n) := by
  induction n with
  | zero => simp [sublists, List.range]
  | succ n _ =>
    simpa [List.range_succ] using (Iff.mpr mem_sublists_iff_sublist (List.nil_sublist _))

private theorem all_lt_of_not_mem_max {xs : List Nat} {n : Nat}
    (hm : ∀ i ∈ xs, i < Nat.succ n) (hni : n ∉ xs) : ∀ i ∈ xs, i < n := by
  intro i hi
  have hi' : i ≤ n := Nat.lt_succ_iff.mp (hm i hi)
  rcases Nat.lt_or_eq_of_le hi' with hlt | rfl
  · exact hlt
  · exact absurd hi hni

private theorem getLast_eq_of_mem_lt (xs : List Nat) (hne : xs ≠ []) (n : Nat)
    (hp : xs.Pairwise (· < ·)) (hn : n ∈ xs) (hm : ∀ i ∈ xs, i < Nat.succ n) :
    xs.getLast hne = n := by
  induction xs with
  | nil => cases hne rfl
  | cons x₀ xs ih =>
    cases xs with
    | nil =>
        have hn' : n = x₀ := List.mem_singleton.mp hn
        rw [hn']
        simp [List.getLast_singleton]
    | cons y ys =>
      rcases List.mem_cons.mp hn with hnx | hn'
      · -- pairwise `x₀ < y` but `hm` forces `y ≤ x₀` when `n = x₀`.
        rw [hnx] at hm hn ⊢
        have hxy : x₀ < y := List.rel_of_pairwise_cons hp (List.Mem.head _)
        have hy : y < Nat.succ x₀ := hm y (List.mem_cons_of_mem x₀ (List.Mem.head _))
        omega
      · have hne' : y :: ys ≠ [] := List.cons_ne_nil y ys
        have hp' : (y :: ys).Pairwise (· < ·) := List.Pairwise.of_cons hp
        have hm' : ∀ i ∈ y :: ys, i < Nat.succ n := fun i hi =>
          hm i (List.mem_cons_of_mem x₀ hi)
        rw [List.getLast_cons (l := y :: ys) hne']
        exact ih hne' hp' hn' hm'

private theorem mem_sublists_range_of_pairwise_lt (n : Nat) (xs : List Nat)
    (hp : xs.Pairwise (· < ·)) (hm : ∀ i ∈ xs, i < n) : xs ∈ sublists (List.range n) := by
  induction n generalizing xs with
  | zero =>
      cases xs with
      | nil => simp [sublists, List.range, nil_mem_sublists_range]
      | cons x xs => exact absurd (hm x (by simp)) (Nat.not_lt_zero _)
  | succ n ih =>
    rw [List.range_succ]
    by_cases hn : n ∈ xs
    · have hne : xs ≠ [] := List.ne_nil_of_mem hn
      have hlast : xs.getLast hne = n :=
        getLast_eq_of_mem_lt xs hne n hp hn (fun i hi => hm i hi)
      let pref := xs.dropLast
      have hxs : xs = pref ++ [n] := by
        simpa [pref, hlast] using (List.dropLast_concat_getLast hne).symm
      have hp_pre : pref.Pairwise (· < ·) :=
        List.Pairwise.sublist (by simpa [pref] using List.dropLast_sublist xs) hp
      have hap := List.pairwise_append.mp (show (pref ++ [n]).Pairwise (· < ·) by rw [← hxs]; exact hp)
      have hm_pre : ∀ i ∈ pref, i < n := fun i hi =>
        hap.2.2 i hi n (List.mem_singleton_self n)
      have hmem_pre : pref ∈ sublists (List.range n) := ih pref hp_pre hm_pre
      have hsub_pre : List.Sublist pref (List.range n) := mem_sublists_iff_sublist.mp hmem_pre
      have hsub : List.Sublist (pref ++ [n]) (List.range n ++ [n]) :=
        List.Sublist.append hsub_pre (List.Sublist.refl [n])
      have hmem' : pref ++ [n] ∈ sublists (List.range n ++ [n]) :=
        Iff.mpr mem_sublists_iff_sublist hsub
      simpa [hxs] using hmem'
    · have hm' : ∀ i ∈ xs, i < n := all_lt_of_not_mem_max hm hn
      have hmem : xs ∈ sublists (List.range n) := ih xs hp hm'
      exact mem_sublists_of_sublist_trans hmem (List.sublist_append_left (List.range n) [n])

/-! ### Totals -/

def totalWeight (weights : Weights) (sel : Selection) : Nat :=
  (sel.map (fun i => weights.getD i 0)).foldr (·+·) 0

def totalValue (values : Values) (sel : Selection) : Nat :=
  (sel.map (fun i => values.getD i 0)).foldr (·+·) 0

def ValidSelection (weights : Weights) (sel : Selection) : Prop :=
  sel.Nodup ∧ ∀ i ∈ sel, i < weights.length

instance instDecidableValidSelection (weights : Weights) (sel : Selection) :
    Decidable (ValidSelection weights sel) := by
  unfold ValidSelection
  infer_instance

private theorem foldr_add_perm {xs ys : List Nat} (h : xs.Perm ys) :
    xs.foldr (·+·) 0 = ys.foldr (·+·) 0 := by
  induction h with
  | nil => rfl
  | cons _ h ih => rw [List.foldr_cons, List.foldr_cons, ih]
  | swap x y xs => simp [List.foldr, Nat.add_assoc, Nat.add_comm, Nat.add_left_comm]
  | trans _ _ ih1 ih2 => exact Eq.trans ih1 ih2

theorem totalWeight_perm (weights : Weights) {xs ys : Selection} (h : xs.Perm ys) :
    totalWeight weights xs = totalWeight weights ys := by
  simpa [totalWeight] using foldr_add_perm (List.Perm.map (fun i => weights.getD i 0) h)

theorem totalValue_perm (values : Values) {xs ys : Selection} (h : xs.Perm ys) :
    totalValue values xs = totalValue values ys := by
  simpa [totalValue] using foldr_add_perm (List.Perm.map (fun i => values.getD i 0) h)

/-! ### Insertion sort -/

private def insertNat (x : Nat) : List Nat → List Nat
  | [] => [x]
  | y :: ys => if x ≤ y then x :: y :: ys else y :: insertNat x ys

private def insertionSortNat : List Nat → List Nat
  | [] => []
  | x :: xs => insertNat x (insertionSortNat xs)

private theorem insertNat_perm (x : Nat) (xs : List Nat) :
    (insertNat x xs).Perm (x :: xs) := by
  induction xs with
  | nil => simp [insertNat]
  | cons y ys ih =>
      by_cases hxy : x ≤ y
      · simp [insertNat, hxy]
      · simp [insertNat, hxy]
        exact List.Perm.trans (List.Perm.cons y ih) (List.Perm.swap x y ys)

private theorem mem_insertNat {x z : Nat} {xs : List Nat} (hz : z ∈ insertNat x xs) :
    z = x ∨ z ∈ xs := by
  induction xs with
  | nil => simp [insertNat] at hz; exact Or.inl hz
  | cons y ys ih =>
      by_cases hxy : x ≤ y
      · simp [insertNat, hxy] at hz
        rcases hz with rfl | hz
        · exact Or.inl rfl
        · rcases hz with rfl | hz'
          · exact Or.inr (List.Mem.head _)
          · exact Or.inr (List.Mem.tail _ hz')
      · simp [insertNat, hxy] at hz
        rcases hz with rfl | hz'
        · right; exact List.Mem.head _
        · rcases ih hz' with hzx | hys
          · left; exact hzx
          · right; exact List.Mem.tail _ hys

private theorem insertNat_sorted {x : Nat} {xs : List Nat}
    (hxs : xs.Pairwise (· ≤ ·)) : (insertNat x xs).Pairwise (· ≤ ·) := by
  induction xs with
  | nil => simp [insertNat, List.Pairwise]
  | cons y ys ih =>
      have ⟨hy, hys⟩ := List.pairwise_cons.mp hxs
      cases ys with
      | nil =>
          by_cases hxy : x ≤ y
          · simp [insertNat, hxy, List.pairwise_cons, hy, hys]
          · simpa [insertNat, hxy, List.pairwise_cons] using Nat.le_of_lt (Nat.lt_of_not_ge hxy)
      | cons v vs =>
          by_cases hxy : x ≤ y
          · have hins : insertNat x (y :: v :: vs) = x :: y :: v :: vs := by simp [insertNat, hxy]
            rw [hins]
            refine List.pairwise_cons.mpr ?_
            constructor
            · intro a ha
              rcases List.mem_cons.mp ha with rfl | ha'
              · exact hxy
              · exact Nat.le_trans hxy (hy a ha')
            · exact hxs
          · simp [insertNat, hxy, List.pairwise_cons]
            constructor
            · intro u hu
              by_cases hvx : x ≤ v
              · simp [insertNat, hvx] at hu
                rcases hu with heqx | heqv | huin
                · rw [heqx]; exact Nat.le_of_lt (Nat.lt_of_not_ge hxy)
                · rw [heqv]; exact hy v (List.Mem.head _)
                · exact hy u (List.Mem.tail v huin)
              · simp [insertNat, hvx] at hu
                rcases hu with heqv | hu'
                · rw [heqv]; exact hy v (List.Mem.head _)
                · rcases mem_insertNat hu' with hu1 | hu2
                  · rw [hu1]; exact Nat.le_of_lt (Nat.lt_of_not_ge hxy)
                  · exact hy u (List.Mem.tail v hu2)
            · exact ih hys

private theorem insertionSortNat_pairwise (xs : List Nat) :
    (insertionSortNat xs).Pairwise (· ≤ ·) := by
  induction xs with
  | nil => simp [insertionSortNat, List.Pairwise]
  | cons x xs ih =>
      simp [insertionSortNat]
      exact insertNat_sorted ih

private theorem insertionSortNat_perm (xs : List Nat) :
    (insertionSortNat xs).Perm xs := by
  induction xs with
  | nil => simp [insertionSortNat]
  | cons x xs ih =>
      simp [insertionSortNat]
      refine List.Perm.trans (insertNat_perm x (insertionSortNat xs)) ?_
      exact List.Perm.cons x ih

private def sortSel (sel : Selection) : Selection :=
  insertionSortNat sel

private theorem sortSel_perm (sel : Selection) : (sortSel sel).Perm sel :=
  insertionSortNat_perm sel

private theorem pairwise_lt_of_sorted_nodup {xs : List Nat} (hs : xs.Pairwise (· ≤ ·))
    (nd : xs.Nodup) : xs.Pairwise (· < ·) := by
  induction xs with
  | nil => simp
  | cons x xs ih =>
      rcases List.pairwise_cons.mp hs with ⟨hx, hs'⟩
      simp [List.pairwise_cons]
      constructor
      · intro y hy
        have hle := hx y hy
        have hne : x ≠ y := by
          intro heq
          subst heq
          exact (List.nodup_cons.mp nd).1 hy
        exact Nat.lt_of_le_of_ne hle hne
      · exact ih hs' (List.nodup_cons.mp nd).2

/-! ### Optimum -/

private def candidates (n : Nat) : List Selection :=
  sublists (List.range n)

private def packStep (weights : Weights) (values : Values) (capacity : Nat)
    (best : Selection × Nat) (sel : Selection) : Selection × Nat :=
  if _ : ValidSelection weights sel ∧ totalWeight weights sel ≤ capacity then
    let v := totalValue values sel
    if v ≤ best.2 then best else (sel, v)
  else best

def knapsack01Pack (weights : Weights) (values : Values) (capacity : Nat) : Selection × Nat :=
  (candidates weights.length).foldl (packStep weights values capacity) ([], 0)

def knapsack01 (weights : Weights) (values : Values) (capacity : Nat) : Nat :=
  (knapsack01Pack weights values capacity).2

private theorem valid_nil (capacity : Nat) (weights : Weights) :
    ValidSelection weights [] ∧ totalWeight weights [] ≤ capacity := by
  refine ⟨?_, by simp [totalWeight]⟩
  refine ⟨List.nodup_nil, ?_⟩
  intro i hi
  cases hi

private theorem packStep_snd_ge (weights : Weights) (values : Values) (capacity : Nat)
    (best : Selection × Nat) (sel : Selection) :
    best.2 ≤ (packStep weights values capacity best sel).2 := by
  simp [packStep]
  by_cases hfeas : ValidSelection weights sel ∧ totalWeight weights sel ≤ capacity
  · simp [hfeas]
    by_cases hle : totalValue values sel ≤ best.2
    · simp [hle]
    · simp [hle]; exact Nat.le_of_lt (Nat.lt_of_not_ge hle)
  · simp [hfeas]

private theorem foldl_packStep_snd_ge (weights : Weights) (values : Values) (capacity : Nat)
    (cands : List Selection) (best : Selection × Nat) :
    best.2 ≤ (cands.foldl (packStep weights values capacity) best).2 := by
  induction cands generalizing best with
  | nil => simp [List.foldl]
  | cons s ss ih =>
    dsimp [List.foldl]
    exact Nat.le_trans (packStep_snd_ge weights values capacity best s)
      (ih (packStep weights values capacity best s))

private theorem foldl_invariant (weights : Weights) (values : Values) (capacity : Nat)
    (cands : List Selection) (best : Selection × Nat)
    (hV : ValidSelection weights best.1) (hw : totalWeight weights best.1 ≤ capacity)
    (hv : totalValue values best.1 = best.2) :
    let r := cands.foldl (packStep weights values capacity) best
    ValidSelection weights r.1 ∧
      totalWeight weights r.1 ≤ capacity ∧
        totalValue values r.1 = r.2 ∧
          (∀ sel ∈ cands,
            ValidSelection weights sel → totalWeight weights sel ≤ capacity →
              totalValue values sel ≤ r.2) := by
  induction cands generalizing best with
  | nil =>
      refine ⟨hV, hw, hv, ?_⟩
      intro sel hmem _ _
      cases hmem
  | cons s0 as ih =>
    rw [List.foldl_cons]
    by_cases hfeas : ValidSelection weights s0 ∧ totalWeight weights s0 ≤ capacity
    · by_cases hle : totalValue values s0 ≤ best.2
      · have hstep :
            packStep weights values capacity best s0 = best := by
          simp [packStep, hfeas, hle]
        rw [hstep]
        obtain ⟨Hv, Hw, He, H4⟩ := ih best hV hw hv
        have hsnd : best.2 ≤ (as.foldl (packStep weights values capacity) best).2 :=
          foldl_packStep_snd_ge weights values capacity as best
        refine ⟨Hv, Hw, He, ?_⟩
        intro sel hmem hVs hws
        simp only [List.mem_cons] at hmem
        rcases hmem with heq | hmem
        · rw [heq]; exact Nat.le_trans hle hsnd
        · exact H4 sel hmem hVs hws
      · have hstep :
            packStep weights values capacity best s0 = (s0, totalValue values s0) := by
          simp [packStep, hfeas, hle]
        rw [hstep]
        obtain ⟨Hv, Hw, He, H4⟩ := ih (s0, totalValue values s0) hfeas.1 hfeas.2 rfl
        refine ⟨Hv, Hw, He, ?_⟩
        intro sel hmem hVs hws
        simp only [List.mem_cons] at hmem
        rcases hmem with heq | hmem
        · rw [heq]
          simpa using foldl_packStep_snd_ge weights values capacity as (s0, totalValue values s0)
        · exact H4 sel hmem hVs hws
    · have hstep : packStep weights values capacity best s0 = best := by simp [packStep, hfeas]
      rw [hstep]
      obtain ⟨Hv, Hw, He, H4⟩ := ih best hV hw hv
      refine ⟨Hv, Hw, He, ?_⟩
      intro sel hmem hVs hws
      simp only [List.mem_cons] at hmem
      rcases hmem with heq | hmem
      · subst heq; exact absurd ⟨hVs, hws⟩ hfeas
      · exact H4 sel hmem hVs hws

private theorem knapsack01Pack_spec (weights : Weights) (values : Values) (capacity : Nat) :
    ValidSelection weights (knapsack01Pack weights values capacity).1 ∧
      totalWeight weights (knapsack01Pack weights values capacity).1 ≤ capacity ∧
        totalValue values (knapsack01Pack weights values capacity).1 =
          (knapsack01Pack weights values capacity).2 ∧
          (∀ sel ∈ candidates weights.length,
            ValidSelection weights sel → totalWeight weights sel ≤ capacity →
              totalValue values sel ≤ (knapsack01Pack weights values capacity).2) := by
  simpa [knapsack01Pack, packStep] using
    foldl_invariant weights values capacity (candidates weights.length) ([], 0)
      (valid_nil capacity weights).1 (valid_nil capacity weights).2 (by simp [totalValue])

theorem knapsack01_feasible_witness (weights : Weights) (values : Values) (capacity : Nat)
    (_halign : weights.length = values.length) :
    ∃ sel : Selection,
      ValidSelection weights sel ∧
        totalWeight weights sel ≤ capacity ∧
          totalValue values sel = knapsack01 weights values capacity := by
  have h := knapsack01Pack_spec weights values capacity
  refine ⟨(knapsack01Pack weights values capacity).1, ?_, ?_, ?_⟩
  · exact h.1
  · exact h.2.1
  · simpa [knapsack01, knapsack01Pack] using h.2.2.1

theorem totalValue_le_knapsack01 (weights : Weights) (values : Values) (capacity : Nat)
    (_halign : weights.length = values.length) (sel : Selection) (hV : ValidSelection weights sel)
    (hw : totalWeight weights sel ≤ capacity) :
    totalValue values sel ≤ knapsack01 weights values capacity := by
  have hsort_perm := sortSel_perm sel
  have hV' : ValidSelection weights (sortSel sel) := by
    refine ⟨List.Perm.nodup hsort_perm.symm hV.1, ?_⟩
    intro i hi
    exact hV.2 i ((hsort_perm.mem_iff).1 hi)
  have hw' : totalWeight weights (sortSel sel) ≤ capacity := by
    simpa [totalWeight_perm weights hsort_perm] using hw
  have hsorted : (sortSel sel).Pairwise (· ≤ ·) := insertionSortNat_pairwise sel
  have hp_lt : (sortSel sel).Pairwise (· < ·) :=
    pairwise_lt_of_sorted_nodup hsorted (List.Perm.nodup hsort_perm.symm hV.1)
  have hall : ∀ i ∈ sortSel sel, i < weights.length := by
    intro i hi
    exact hV.2 i ((hsort_perm.mem_iff).1 hi)
  have hmem : sortSel sel ∈ candidates weights.length := by
    simpa [candidates, mem_sublists_iff_sublist] using
      mem_sublists_range_of_pairwise_lt weights.length (sortSel sel) hp_lt hall
  have hspec := (knapsack01Pack_spec weights values capacity).2.2.2
  simpa [knapsack01, knapsack01Pack, totalValue_perm values hsort_perm] using
    hspec (sortSel sel) hmem hV' hw'

end CTA.Benchmark.DP.KnapsackTheory
