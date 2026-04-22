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

/-- Declarative model of the reference `dijkstra`. Parameters match the Rust
    signature: vertex count, source, edge list. -/
opaque dijkstra : (n : Nat) → (source : Nat) → Graph → DistTable

end CTA.Benchmark.Graph.Dijkstra002
