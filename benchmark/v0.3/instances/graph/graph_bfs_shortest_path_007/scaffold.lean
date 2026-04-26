/-
Scaffold for instance `graph_bfs_shortest_path_007`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Benchmark.Graph.BfsShortestPathTheory

namespace CTA.Benchmark.Graph.BfsShortestPath007

open CTA.Core
open CTA.Benchmark.Graph.BfsShortestPathTheory

abbrev Adj := BfsShortestPathTheory.Adj
abbrev DistTable := BfsShortestPathTheory.DistTable
noncomputable abbrev bfsShortestPath := BfsShortestPathTheory.bfsShortestPath

end CTA.Benchmark.Graph.BfsShortestPath007
