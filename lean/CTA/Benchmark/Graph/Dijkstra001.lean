/-
Scaffold for instance `graph_dijkstra_001`.
-/

import CTA.Core.Prelude
import CTA.Core.Types
import CTA.Benchmark.Graph.DijkstraTheory

namespace CTA.Benchmark.Graph.Dijkstra001

open CTA.Core
open CTA.Benchmark.Graph.DijkstraTheory

abbrev Edge := DijkstraTheory.Edge
abbrev Graph := DijkstraTheory.Graph
abbrev DistTable := DijkstraTheory.DistTable
abbrev dijkstra := DijkstraTheory.dijkstra

end CTA.Benchmark.Graph.Dijkstra001
