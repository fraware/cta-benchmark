/-
Scaffold for instance `graph_bfs_shortest_path_002`.
-/

import CTA.Core.Prelude
import CTA.Core.Types

namespace CTA.Benchmark.Graph.BfsShortestPath002

open CTA.Core

/-- Adjacency list: `adj.get? v` is the list of out-neighbors of `v`. -/
abbrev Adj := List (List Nat)

/-- Distance table aligned with `Adj`: length `n`, values `none` for
    unreachable and `some k` for "shortest path has exactly `k` edges". -/
abbrev DistTable := List (Option Nat)

/-- Declarative model of the reference `bfs_shortest_path`. Takes the
    adjacency list and the source vertex. -/
opaque bfsShortestPath : Adj → Nat → DistTable

end CTA.Benchmark.Graph.BfsShortestPath002
