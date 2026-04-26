/-
Scaffold for instance `graph_bfs_shortest_path_001`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Benchmark.Graph.BfsShortestPathTheory

namespace CTA.Benchmark.Graph.BfsShortestPath001

open CTA.Core
open CTA.Benchmark.Graph.BfsShortestPathTheory

/-- Adjacency list: `adj.get? v` is the list of out-neighbors of `v`. -/
abbrev Adj := BfsShortestPathTheory.Adj

/-- Distance table aligned with `Adj`: length `n`, values `none` for
    unreachable and `some k` for "shortest path has exactly `k` edges". -/
abbrev DistTable := BfsShortestPathTheory.DistTable

/-- Declarative model of the reference `bfs_shortest_path`. Takes the
    adjacency list and the source vertex. -/
noncomputable abbrev bfsShortestPath := BfsShortestPathTheory.bfsShortestPath

end CTA.Benchmark.Graph.BfsShortestPath001
