/-
Scaffold for instance `graph_dijkstra_002`.
-/

import CTA.Core.Prelude
import CTA.Core.Types

namespace CTA.Benchmark.Graph.Dijkstra002

open CTA.Core

/-- Edges as (src, dst, weight) triples on `Nat` vertices. -/
abbrev Edge := Nat × Nat × Nat

/-- Adjacency: list of outgoing edges from each vertex. -/
abbrev Graph := List Edge

/-- Distance table: `dist.get? v` is `some (some d)` if `v` is reachable at
    distance `d`, `some none` if `v` is in range but unreachable, and `none`
    if `v` is out of range. -/
abbrev DistTable := List (Option Nat)

/-- Full paper input contract: in-range source, in-range edge endpoints, nonnegative weights. -/
def ValidDijkstraInput (n source : Nat) (edges : Graph) : Prop :=
  source < n ∧
  ∀ e ∈ edges,
    let s : Nat := e.1
    let t : Nat := e.2.1
    let w : Nat := e.2.2
    s < n ∧ t < n ∧ 0 ≤ w

/-- There exists a path from `source` to `v` with total edge weight `d`. -/
opaque PathWeight : Graph → Nat → Nat → Nat → Prop

/-- Every edge weight is nonnegative (implied by `ValidDijkstraInput`). -/
abbrev NonNegativeWeights (edges : Graph) : Prop :=
  ∀ e ∈ edges, (0 : Nat) ≤ e.2.2

/-- Declarative model of the reference `dijkstra`. Parameters match the Rust
    signature: vertex count, source, edge list. -/
opaque dijkstra : (n : Nat) → (source : Nat) → Graph → DistTable

end CTA.Benchmark.Graph.Dijkstra002
