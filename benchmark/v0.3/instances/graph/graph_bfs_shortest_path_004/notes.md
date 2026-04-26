# graph_bfs_shortest_path_004

BFS distance table (variant 4): emphasize handling of parallel edges and duplicate enqueues without breaking shortest counts.

Lens 4 targets specifications that accidentally allow counting non-simple walks or double-counting hop length.

Derived algorithm family `graph_bfs_shortest_path`; behavioral contract matches v0.2 reference oracles.
