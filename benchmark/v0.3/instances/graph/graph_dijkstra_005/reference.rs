//! Reference implementation for `graph_dijkstra_005`.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Directed, non-negative-weighted edge.
#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub src: usize,
    pub dst: usize,
    pub weight: u32,
}

/// Single-source shortest paths via Dijkstra's algorithm.
///
/// Returns a vector `dist` of length `n` where `dist[v]` is `Some(d)` with
/// `d` the shortest distance from `source` to `v`, or `None` if `v` is
/// unreachable from `source`.
pub fn dijkstra(n: usize, source: usize, edges: &[Edge]) -> Vec<Option<u64>> {
    let mut adjacency: Vec<Vec<(usize, u32)>> = vec![Vec::new(); n];
    for e in edges {
        adjacency[e.src].push((e.dst, e.weight));
    }

    let mut dist: Vec<Option<u64>> = vec![None; n];
    dist[source] = Some(0);
    let mut heap: BinaryHeap<Reverse<(u64, usize)>> = BinaryHeap::new();
    heap.push(Reverse((0, source)));

    while let Some(Reverse((d, u))) = heap.pop() {
        if Some(d) != dist[u] {
            continue;
        }
        for &(v, w) in &adjacency[u] {
            let nd = d + u64::from(w);
            match dist[v] {
                None => {
                    dist[v] = Some(nd);
                    heap.push(Reverse((nd, v)));
                }
                Some(current) if nd < current => {
                    dist[v] = Some(nd);
                    heap.push(Reverse((nd, v)));
                }
                _ => {}
            }
        }
    }

    dist
}
