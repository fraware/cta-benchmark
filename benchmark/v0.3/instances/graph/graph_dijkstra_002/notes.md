# Grid variant 002 (V002 paired control)

# graph_dijkstra_002

Single-source shortest paths with non-negative weights. Canonical example
where annotators must distinguish achievability and optimality as separate
semantic units.

## Design notes

- The Lean scaffold exposes `PathWeight` as an intentionally undefined
  predicate. Reference obligations reference it directly; generators must
  not redefine it, they must instantiate against the scaffold.
- The harness is reference-relational: distances are compared against a
  brute-force Bellman-Ford on small graphs (n <= 16).
- Negative weights and negative cycles are out of scope for v0.1.
