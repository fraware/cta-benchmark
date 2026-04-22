# graph_bfs_shortest_path_002

Unweighted shortest-path lengths via BFS. Paired with `graph_dijkstra_001`
to give the benchmark two graph-traversal instances with different
complexity profiles (queue vs. priority queue) and spec shapes
(`Some(k)` = edge count vs. `Some(d)` = summed weight).

## Design notes

- The reachability obligation (`obl_003`) and minimality obligation
  (`obl_004`) must both appear; a spec that gives only reachability
  admits the vacuous "always return `Some(0)`" implementation.
- The iff in `obl_005` is deliberately bidirectional; annotators flag
  one-directional specs as a critical-unit coverage miss.
- The harness uses a Floyd-Warshall unweighted oracle as ground truth
  on small graphs (n <= 20).
