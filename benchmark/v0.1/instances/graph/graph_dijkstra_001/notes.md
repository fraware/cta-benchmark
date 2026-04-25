# graph_dijkstra_001

Single-source shortest paths with non-negative weights. Canonical example
where annotators must distinguish achievability and optimality as separate
semantic units.

## Design notes

- The Lean scaffold is byte-identical to `lean/CTA/Benchmark/Graph/Dijkstra001.lean`:
  it imports `DijkstraTheory` and re-exports `PathWeight` and related symbols as
  `abbrev`s. Reference obligations and generators should stay aligned with that
  theory-backed surface.
- The harness is reference-relational: distances are compared against a
  brute-force Bellman-Ford on small graphs (n <= 16).
- Negative weights and negative cycles are out of scope for v0.1.
