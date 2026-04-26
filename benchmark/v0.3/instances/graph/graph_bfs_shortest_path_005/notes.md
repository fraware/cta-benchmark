# graph_bfs_shortest_path_005

BFS distance table (variant 5): emphasize disconnected components and source-out-of-range defensive behavior.

Lens 5 checks that unreachability is stated as absence of any directed walk, not bounded search depth.

Derived algorithm family `graph_bfs_shortest_path`; behavioral contract matches v0.2 reference oracles.
