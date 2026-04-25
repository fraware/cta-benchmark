/-
Shared theory surface for BFS shortest-path benchmark packets.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import Mathlib.Data.Nat.Find

namespace CTA.Benchmark.Graph.BfsShortestPathTheory

open CTA.Core
open Classical

/-- Adjacency list: `adj.get? v` is the list of out-neighbors of `v`. -/
abbrev Adj := List (List Nat)

/-- Distance table: `none` means unreachable, `some k` shortest-path length. -/
abbrev DistTable := List (Option Nat)

/-- Path witness for an exact hop count `k`. -/
def PathAtDist (adj : Adj) (source v k : Nat) : Prop :=
  ∃ path : List Nat,
    path.length = k + 1 ∧
    path.head? = some source ∧
    path.get? k = some v ∧
    (∀ i : Nat, i < k →
      let a := path.get? i
      let b := path.get? (i + 1)
      a.isSome ∧ b.isSome ∧
      match a, b with
      | some ai, some bi => bi ∈ (adj.get? ai).getD []
      | _, _ => False)

/-- Reachability from `source` to `v`. -/
def Reachable (adj : Adj) (source v : Nat) : Prop :=
  ∃ k : Nat, PathAtDist adj source v k

/-- Shortest hop count if reachable, otherwise `none`. -/
noncomputable def shortestDist (adj : Adj) (source v : Nat) : Option Nat :=
  if h : Reachable adj source v then some (Nat.find h) else none

/-- Definition-backed model: pointwise shortest distance over all vertices. -/
noncomputable def bfsShortestPath (adj : Adj) (source : Nat) : DistTable :=
  (List.range adj.length).map (fun v => shortestDist adj source v)

theorem bfs_length_correct (adj : Adj) (source : Nat) :
    (bfsShortestPath adj source).length = adj.length := by
  simp [bfsShortestPath]

private theorem bfs_get_shape (adj : Adj) (source v : Nat) (hv : v < adj.length) :
    (bfsShortestPath adj source).get? v = some (shortestDist adj source v) := by
  simp [bfsShortestPath, List.getElem?_map, List.getElem?_range hv]

private theorem source_reachable (adj : Adj) (source : Nat) :
    Reachable adj source source := by
  refine ⟨0, ?_⟩
  refine ⟨[source], ?_, ?_, ?_, ?_⟩
  · simp
  · simp
  · simp
  · intro i hi
    exact False.elim (Nat.not_lt_zero i hi)

theorem bfs_source_anchor (adj : Adj) (source : Nat) (hs : source < adj.length) :
    (bfsShortestPath adj source).get? source = some (some 0) := by
  have hshape := bfs_get_shape adj source source hs
  have hreach : Reachable adj source source := source_reachable adj source
  have hfind_le_zero : Nat.find hreach ≤ 0 := by
    exact Nat.find_min' hreach ⟨[source], by simp, by simp, by simp, by
      intro i hi
      exact False.elim (Nat.not_lt_zero i hi)⟩
  have hfind_zero : Nat.find hreach = 0 := Nat.le_zero.mp hfind_le_zero
  have hsd : shortestDist adj source source = some 0 := by
    unfold shortestDist
    simp [hreach, hfind_zero]
  simpa [hsd] using hshape

theorem bfs_reachability_witness
    (adj : Adj) (source v k : Nat)
    (hv : v < adj.length)
    (hk : (bfsShortestPath adj source).get? v = some (some k)) :
    ∃ path : List Nat,
      path.length = k + 1 ∧
      path.head? = some source ∧
      path.get? k = some v ∧
      (∀ i : Nat, i < k →
        let a := path.get? i
        let b := path.get? (i+1)
        a.isSome ∧ b.isSome ∧
        match a, b with
        | some ai, some bi => bi ∈ (adj.get? ai).getD []
        | _, _ => False) := by
  have hshape := bfs_get_shape adj source v hv
  have hsd : shortestDist adj source v = some k := by
    rw [hshape] at hk
    injection hk with hsk
  by_cases hreach : Reachable adj source v
  · have hfind : Nat.find hreach = k := by
      unfold shortestDist at hsd
      simpa [hreach] using hsd
    have hw : PathAtDist adj source v (Nat.find hreach) := Nat.find_spec hreach
    rcases hw with ⟨path, hlen, hhead, hlast, hstep⟩
    refine ⟨path, ?_, hhead, ?_, ?_⟩
    · simpa [hfind] using hlen
    · simpa [hfind] using hlast
    · simpa [hfind] using hstep
  · unfold shortestDist at hsd
    simp [hreach] at hsd

theorem bfs_minimality
    (adj : Adj) (source v k : Nat)
    (hv : v < adj.length)
    (hk : (bfsShortestPath adj source).get? v = some (some k)) :
    ¬ ∃ k' : Nat,
      k' < k ∧
      (∃ path : List Nat,
        path.length = k' + 1 ∧
        path.head? = some source ∧
        path.get? k' = some v ∧
        (∀ i : Nat, i < k' →
          let a := path.get? i
          let b := path.get? (i+1)
          a.isSome ∧ b.isSome ∧
          match a, b with
          | some ai, some bi => bi ∈ (adj.get? ai).getD []
          | _, _ => False)) := by
  have hshape := bfs_get_shape adj source v hv
  have hsd : shortestDist adj source v = some k := by
    rw [hshape] at hk
    injection hk with hsk
  by_cases hreach : Reachable adj source v
  · have hfind : Nat.find hreach = k := by
      unfold shortestDist at hsd
      simpa [hreach] using hsd
    intro hcontra
    rcases hcontra with ⟨k', hk', hpath⟩
    have hmin : Nat.find hreach ≤ k' := Nat.find_min' hreach hpath
    have : ¬ k' < Nat.find hreach := Nat.not_lt_of_ge hmin
    exact this (by simpa [hfind] using hk')
  · unfold shortestDist at hsd
    simp [hreach] at hsd

theorem bfs_unreachability_iff
    (adj : Adj) (source v : Nat)
    (hv : v < adj.length) :
    (bfsShortestPath adj source).get? v = some none ↔
      ¬ ∃ k : Nat,
        ∃ path : List Nat,
          path.length = k + 1 ∧
          path.head? = some source ∧
          path.get? k = some v ∧
          (∀ i : Nat, i < k →
            let a := path.get? i
            let b := path.get? (i+1)
            a.isSome ∧ b.isSome ∧
            match a, b with
            | some ai, some bi => bi ∈ (adj.get? ai).getD []
            | _, _ => False) := by
  have hshape := bfs_get_shape adj source v hv
  constructor
  · intro hnone
    intro hreach
    have hreachR : Reachable adj source v := by
      simpa [Reachable, PathAtDist] using hreach
    have hsd : shortestDist adj source v = none := by
      rw [hshape] at hnone
      injection hnone with hsd
    have hne : shortestDist adj source v ≠ none := by
      unfold shortestDist
      simp [hreachR]
    exact hne hsd
  · intro hnone
    have hnotReach : ¬ Reachable adj source v := by
      simpa [Reachable, PathAtDist] using hnone
    have hsd : shortestDist adj source v = none := by
      unfold shortestDist
      simp [hnotReach]
    rw [hshape]
    simp [hsd]

end CTA.Benchmark.Graph.BfsShortestPathTheory
