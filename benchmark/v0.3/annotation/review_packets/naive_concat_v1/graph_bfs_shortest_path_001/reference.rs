//! Reference implementation for `graph_bfs_shortest_path_001`.
//!
//! Pure, stable Rust. No unsafe, no macros, no external crates.

use std::collections::VecDeque;

/// Breadth-first shortest-path-length computation.
///
/// `adj[v]` is the list of out-neighbors of `v`. Returns a vector `dist` of
/// length `adj.len()` where `dist[v]` is `Some(k)` if `v` is reachable from
/// `source` in exactly `k` edges along a shortest path, or `None` if
/// unreachable.
pub fn bfs_shortest_path(adj: &[Vec<usize>], source: usize) -> Vec<Option<usize>> {
    let n = adj.len();
    let mut dist: Vec<Option<usize>> = vec![None; n];
    if source >= n {
        return dist;
    }
    dist[source] = Some(0);
    let mut q: VecDeque<usize> = VecDeque::new();
    q.push_back(source);
    while let Some(u) = q.pop_front() {
        let du = dist[u].unwrap_or(0);
        for &v in &adj[u] {
            if dist[v].is_none() {
                dist[v] = Some(du + 1);
                q.push_back(v);
            }
        }
    }
    dist
}
