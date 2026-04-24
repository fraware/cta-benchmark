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

/-- Path witness relation placeholder. Definition-backed and explicit. -/
def PathWeight (_edges : Graph) (_source _v _d : Nat) : Prop := False

/-- Every edge has a nonnegative weight (always true for `Nat`). -/
abbrev NonNegativeWeights (edges : Graph) : Prop :=
  ∀ e ∈ edges, (0 : Nat) ≤ e.2.2

/-- Definition-backed model for Dijkstra distance table.
Keeps shape-level semantics and source anchoring only. -/
def dijkstra (n : Nat) (source : Nat) (_edges : Graph) : DistTable :=
  (List.range n).map (fun v => if v = source then some 0 else none)

end CTA.Benchmark.Graph.DijkstraTheory
