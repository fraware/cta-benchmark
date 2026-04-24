/-
Shared theory surface for BFS shortest-path benchmark packets.
-/

import CTA.Core.Prelude
import CTA.Core.Types

namespace CTA.Benchmark.Graph.BfsShortestPathTheory

open CTA.Core

/-- Adjacency list: `adj.get? v` is the list of out-neighbors of `v`. -/
abbrev Adj := List (List Nat)

/-- Distance table: `none` means unreachable, `some k` shortest-path length. -/
abbrev DistTable := List (Option Nat)

/-- Definition-backed model for BFS distance table.
For now this keeps shape-level semantics and source anchoring only. -/
def bfsShortestPath (adj : Adj) (source : Nat) : DistTable :=
  (List.range adj.length).map (fun v => if v = source then some 0 else none)

end CTA.Benchmark.Graph.BfsShortestPathTheory
