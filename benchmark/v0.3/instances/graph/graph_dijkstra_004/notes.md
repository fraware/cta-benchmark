# graph_dijkstra_004

Single-source shortest paths (variant 4): emphasize zero-weight edges and tie-breaking stability of distances.

Lens 4 targets vacuous distance updates that forget to propagate through weight-0 stacks.

Derived algorithm family `graph_dijkstra`; behavioral contract matches v0.2 reference oracles.
