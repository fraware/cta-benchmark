/-
Scaffold for instance `graph_dijkstra_001`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Benchmark.Graph.DijkstraTheory

namespace CTA.Benchmark.Graph.Dijkstra001

open CTA.Core
open CTA.Benchmark.Graph.DijkstraTheory

/-- Edges as (src, dst, weight) triples on `Nat` vertices. -/
abbrev Edge := DijkstraTheory.Edge

/-- Adjacency: list of outgoing edges from each vertex. -/
abbrev Graph := DijkstraTheory.Graph

/-- Distance table: `dist.get? v` is `some (some d)` if `v` is reachable at
    distance `d`, `some none` if `v` is in range but unreachable, and `none`
    if `v` is out of range. -/
abbrev DistTable := DijkstraTheory.DistTable

/-- Declarative model of the reference `dijkstra`. Parameters match the Rust
    signature: vertex count, source, edge list. -/
abbrev dijkstra := DijkstraTheory.dijkstra

end CTA.Benchmark.Graph.Dijkstra001
