# graph_bfs_shortest_path_007

BFS distance table (variant 7): emphasize multi-edge shortest paths where the first discovered path may be non-shortest if mis-implemented.

Lens 7 stresses first-layer discovery order must not replace shortest hop count semantics.

Derived algorithm family `graph_bfs_shortest_path`; behavioral contract matches v0.2 reference oracles.
