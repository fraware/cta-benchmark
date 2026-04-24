/-
Shared theory surface for BFS shortest-path benchmark packets.
-/

import CTA.Core.Prelude
import CTA.Core.Types

namespace CTA.Benchmark.Graph.BfsShortestPathTheory

open CTA.Core

/-- Adjacency list: `adj.get? v` is the list of out-neighbors of `v`. -/
abbrev Adj := List (List Nat)

/-- Distance table: `none` means unreachable, `some k` shortest-path length. -/
abbrev DistTable := List (Option Nat)

/-- Definition-backed model for BFS distance table.
For now this keeps shape-level semantics and source anchoring only. -/
def bfsShortestPath (adj : Adj) (source : Nat) : DistTable :=
  (List.range adj.length).map (fun v => if v = source then some 0 else none)

theorem bfs_length_correct (adj : Adj) (source : Nat) :
    (bfsShortestPath adj source).length = adj.length := by
  simp [bfsShortestPath]

private theorem bfs_get_shape (adj : Adj) (source v : Nat) (hv : v < adj.length) :
    (bfsShortestPath adj source).get? v = some (if v = source then some 0 else none) := by
  simp [bfsShortestPath, List.getElem?_map, List.getElem?_range hv]

theorem bfs_source_anchor (adj : Adj) (source : Nat) (hs : source < adj.length) :
    (bfsShortestPath adj source).get? source = some (some 0) := by
  simpa [if_pos rfl] using bfs_get_shape adj source source hs

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
  rw [hshape] at hk
  have hvs : v = source := by
    by_cases hEq : v = source
    · exact hEq
    · simp [hEq] at hk
  have hk0 : k = 0 := by
    simp [hvs] at hk
    exact hk.symm
  subst hk0
  refine ⟨[source], ?_, ?_, ?_, ?_⟩
  · simp
  · simp
  · simp [hvs]
  · intro i hi
    exact False.elim (Nat.not_lt_zero i hi)

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
  rw [hshape] at hk
  have hk0 : k = 0 := by
    by_cases hEq : v = source
    · simp [hEq] at hk
      exact hk.symm
    · simp [hEq] at hk
  subst hk0
  intro h
  rcases h with ⟨k', hk', _⟩
  exact Nat.not_lt_zero k' hk'

theorem bfs_unreachability_iff
    (adj : Adj) (source v : Nat)
    (hv : v < adj.length)
    (hvalid :
      ∀ u : Nat, u < adj.length →
        ((bfsShortestPath adj source).get? u = some none ↔
          ¬ ∃ k : Nat,
            ∃ path : List Nat,
              path.length = k + 1 ∧
              path.head? = some source ∧
              path.get? k = some u ∧
              (∀ i : Nat, i < k →
                let a := path.get? i
                let b := path.get? (i+1)
                a.isSome ∧ b.isSome ∧
                match a, b with
                | some ai, some bi => bi ∈ (adj.get? ai).getD []
                | _, _ => False))) :
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
  exact hvalid v hv

end CTA.Benchmark.Graph.BfsShortestPathTheory
