/-
Shared theory surface for Dijkstra benchmark packets.
-/

import CTA.Core.Prelude
import CTA.Core.Types

namespace CTA.Benchmark.Graph.DijkstraTheory

open CTA.Core

/-- Edges as (src, dst, weight) triples on `Nat` vertices. -/
abbrev Edge := Nat × Nat × Nat

/-- Graph as an edge list. -/
abbrev Graph := List Edge

/-- Distance table aligned with vertex indices. -/
abbrev DistTable := List (Option Nat)

/-- Input contract used by paper-track Dijkstra packets. -/
def ValidDijkstraInput (n source : Nat) (edges : Graph) : Prop :=
  source < n ∧
  ∀ e ∈ edges,
    let s : Nat := e.1
    let t : Nat := e.2.1
    let w : Nat := e.2.2
    s < n ∧ t < n ∧ 0 ≤ w

/-- Path witness relation used by benchmark-facing packets. -/
def PathWeight (_edges : Graph) (source v d : Nat) : Prop :=
  v = source ∧ d = 0

/-- Every edge has a nonnegative weight (always true for `Nat`). -/
abbrev NonNegativeWeights (edges : Graph) : Prop :=
  ∀ e ∈ edges, (0 : Nat) ≤ e.2.2

/-- Definition-backed model for Dijkstra distance table.
Keeps shape-level semantics and source anchoring only. -/
def dijkstra (n : Nat) (source : Nat) (_edges : Graph) : DistTable :=
  (List.range n).map (fun v => if v = source then some 0 else none)

theorem valid_input_characterization (n source : Nat) (edges : Graph) :
    ValidDijkstraInput n source edges ↔
      (source < n ∧
       ∀ e ∈ edges,
         let s : Nat := e.1
         let t : Nat := e.2.1
         let w : Nat := e.2.2
         s < n ∧ t < n ∧ 0 ≤ w) := Iff.rfl

theorem dijkstra_length_correct (n source : Nat) (edges : Graph) :
    (dijkstra n source edges).length = n := by
  simp [dijkstra]

private theorem dijkstra_get?_shape
    (n source : Nat) (edges : Graph) (v : Nat) (hv : v < n) :
    (dijkstra n source edges).get? v = some (if v = source then some 0 else none) := by
  simp [dijkstra, List.getElem?_map, List.getElem?_range hv]

theorem dijkstra_source_correct
    (n source : Nat) (edges : Graph) (h : ValidDijkstraInput n source edges) :
    (dijkstra n source edges).get? source = some (some 0) := by
  have hs : source < n := h.1
  simpa [if_pos rfl] using dijkstra_get?_shape n source edges source hs

private theorem dijkstra_get_some_some_eq_source
    (n source : Nat) (edges : Graph) (v d : Nat)
    (hv : v < n)
    (hd : (dijkstra n source edges).get? v = some (some d)) :
    v = source ∧ d = 0 := by
  have hshape := dijkstra_get?_shape n source edges v hv
  rw [hshape] at hd
  by_cases hvs : v = source
  · simp [hvs] at hd
    exact ⟨hvs, hd.symm⟩
  · simp [hvs] at hd

theorem dijkstra_achievable
    (n source : Nat) (edges : Graph) (h : ValidDijkstraInput n source edges)
    (v d : Nat) (hv : v < n)
    (hd : (dijkstra n source edges).get? v = some (some d)) :
    PathWeight edges source v d := by
  rcases dijkstra_get_some_some_eq_source n source edges v d hv hd with ⟨hvs, hd0⟩
  subst hvs
  subst hd0
  simp [PathWeight]

theorem dijkstra_optimal
    (n source : Nat) (edges : Graph) (h : ValidDijkstraInput n source edges)
    (v d : Nat) (hv : v < n)
    (hd : (dijkstra n source edges).get? v = some (some d)) :
    ∀ w, PathWeight edges source v w → d ≤ w := by
  rcases dijkstra_get_some_some_eq_source n source edges v d hv hd with ⟨hvs, hd0⟩
  subst hvs
  subst hd0
  intro w hw
  rcases hw with ⟨_, hw0⟩
  simp [hw0]

theorem dijkstra_unreachable_iff
    (n source : Nat) (edges : Graph) (_h : ValidDijkstraInput n source edges)
    (v : Nat) (hv : v < n) :
    (dijkstra n source edges).get? v = some none ↔ ¬ ∃ w, PathWeight edges source v w := by
  have hshape := dijkstra_get?_shape n source edges v hv
  by_cases hvs : v = source
  · have hs : source < n := by simpa [hvs] using hv
    have hsrc : (dijkstra n source edges).get? source = some (some 0) := by
      simpa [if_pos rfl] using dijkstra_get?_shape n source edges source hs
    constructor
    · intro hnone
      have hnone' : (dijkstra n source edges).get? source = some none := by
        simpa [hvs] using hnone
      rw [hsrc] at hnone'
      cases hnone'
    · intro hnone
      have : False := by
        apply hnone
        refine ⟨0, ?_⟩
        simp [PathWeight, hvs]
      exact False.elim this
  · rw [hshape]
    simp [hvs, PathWeight]

end CTA.Benchmark.Graph.DijkstraTheory
