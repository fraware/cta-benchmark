/-
Scaffold for instance `graph_dijkstra_002`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Benchmark.Graph.DijkstraTheory

namespace CTA.Benchmark.Graph.Dijkstra002

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

/-- Full paper input contract: in-range source, in-range edge endpoints, nonnegative weights. -/
abbrev ValidDijkstraInput := DijkstraTheory.ValidDijkstraInput

/-- There exists a path from `source` to `v` with total edge weight `d`. -/
abbrev PathWeight := DijkstraTheory.PathWeight

/-- Every edge weight is nonnegative (implied by `ValidDijkstraInput`). -/
abbrev NonNegativeWeights := DijkstraTheory.NonNegativeWeights

/-- Declarative model of the reference `dijkstra`. Parameters match the Rust
    signature: vertex count, source, edge list. -/
abbrev dijkstra := DijkstraTheory.dijkstra

end CTA.Benchmark.Graph.Dijkstra002
