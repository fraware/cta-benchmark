//! Deterministic input generators used by pilot harnesses.
//!
//! Every generator consumes a `&mut ChaCha20Rng` seeded from
//! [`crate::HarnessConfig::seed`]. Generators are intentionally small and
//! defensively clamp their parameters so trials cannot blow up unbounded.

use rand::Rng;
use rand_chacha::ChaCha20Rng;

/// Clamp a length range to a sane working window.
fn clamp_len(min_len: usize, max_len: usize, hard_cap: usize) -> (usize, usize) {
    let mx = max_len.min(hard_cap);
    let mn = min_len.min(mx);
    (mn, mx)
}

/// Uniformly sample a length in `[min_len, max_len]`.
fn sample_len(rng: &mut ChaCha20Rng, min_len: usize, max_len: usize) -> usize {
    if max_len <= min_len {
        min_len
    } else {
        rng.gen_range(min_len..=max_len)
    }
}

/// Generate a non-decreasing `Vec<i32>` together with a target. The target is
/// biased toward values present in the vector (so `binary_search` success is
/// exercised about half the time).
#[must_use]
pub fn sorted_int_vec_with_target(
    rng: &mut ChaCha20Rng,
    min_len: usize,
    max_len: usize,
    value_min: i32,
    value_max: i32,
) -> (Vec<i32>, i32) {
    let (mn, mx) = clamp_len(min_len, max_len, 256);
    let len = sample_len(rng, mn, mx);
    let mut v: Vec<i32> = (0..len)
        .map(|_| rng.gen_range(value_min..=value_max))
        .collect();
    v.sort_unstable();
    let target = if !v.is_empty() && rng.gen_bool(0.5) {
        let i = rng.gen_range(0..v.len());
        v[i]
    } else {
        rng.gen_range(value_min..=value_max)
    };
    (v, target)
}

/// Generate a non-empty `Vec<i32>`.
#[must_use]
pub fn int_vec(
    rng: &mut ChaCha20Rng,
    min_len: usize,
    max_len: usize,
    value_min: i32,
    value_max: i32,
) -> Vec<i32> {
    let (mn, mx) = clamp_len(min_len, max_len, 256);
    let len = sample_len(rng, mn, mx);
    (0..len)
        .map(|_| rng.gen_range(value_min..=value_max))
        .collect()
}

/// Generate a pair of `Vec<i32>` of independent lengths.
#[must_use]
pub fn pair_int_vec(
    rng: &mut ChaCha20Rng,
    min_len: usize,
    max_len: usize,
    value_min: i32,
    value_max: i32,
) -> (Vec<i32>, Vec<i32>) {
    let a = int_vec(rng, min_len, max_len, value_min, value_max);
    let b = int_vec(rng, min_len, max_len, value_min, value_max);
    (a, b)
}

/// Generate a list of closed-open intervals `[s, e)` with `s < e`.
#[must_use]
pub fn interval_vec(
    rng: &mut ChaCha20Rng,
    min_len: usize,
    max_len: usize,
    coord_min: i32,
    coord_max: i32,
) -> Vec<(i32, i32)> {
    let (mn, mx) = clamp_len(min_len, max_len, 64);
    let len = sample_len(rng, mn, mx);
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        let a = rng.gen_range(coord_min..=coord_max);
        let b = rng.gen_range(coord_min..=coord_max);
        if a == b {
            out.push((a, a + 1));
        } else if a < b {
            out.push((a, b));
        } else {
            out.push((b, a));
        }
    }
    out
}

/// Generate a random simple adjacency list of size `n`. Self-loops and
/// duplicate edges are suppressed.
#[must_use]
pub fn adj_list(
    rng: &mut ChaCha20Rng,
    min_n: usize,
    max_n: usize,
    max_out_degree: usize,
) -> (Vec<Vec<usize>>, usize) {
    let (mn, mx) = clamp_len(min_n, max_n, 64);
    let n = sample_len(rng, mn.max(1), mx.max(1));
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (u, row) in adj.iter_mut().enumerate() {
        let k = rng.gen_range(0..=max_out_degree.min(n.saturating_sub(1)));
        let mut seen = std::collections::HashSet::new();
        seen.insert(u);
        for _ in 0..k {
            let v = rng.gen_range(0..n);
            if seen.insert(v) {
                row.push(v);
            }
        }
        row.sort_unstable();
    }
    let source = rng.gen_range(0..n);
    (adj, source)
}

/// Generate a random DAG via a topological order and edges from earlier →
/// later vertices only. Returns `(n, edges, source)`.
#[must_use]
pub fn random_dag(
    rng: &mut ChaCha20Rng,
    min_n: usize,
    max_n: usize,
    edge_density: f64,
    weight_min: u32,
    weight_max: u32,
) -> (usize, Vec<(u32, u32, u32)>, usize) {
    let (mn, mx) = clamp_len(min_n, max_n, 64);
    let n = sample_len(rng, mn.max(1), mx.max(1));
    let density = edge_density.clamp(0.0, 1.0);
    let mut edges = Vec::new();
    for u in 0..n {
        for v in (u + 1)..n {
            if rng.gen_bool(density) {
                let w = rng.gen_range(weight_min..=weight_max);
                #[allow(clippy::cast_possible_truncation)]
                edges.push((u as u32, v as u32, w));
            }
        }
    }
    let source = rng.gen_range(0..n);
    (n, edges, source)
}

/// Pick a canonical system and a random amount.
#[must_use]
pub fn canonical_coin_system(
    rng: &mut ChaCha20Rng,
    systems: &[Vec<u32>],
    amount_min: u32,
    amount_max: u32,
) -> (Vec<u32>, u32) {
    let idx = rng.gen_range(0..systems.len());
    let denoms = systems[idx].clone();
    let amount = rng.gen_range(amount_min..=amount_max);
    (denoms, amount)
}

/// Generate a knapsack instance.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn knapsack_instance(
    rng: &mut ChaCha20Rng,
    min_items: usize,
    max_items: usize,
    w_min: u32,
    w_max: u32,
    v_min: u32,
    v_max: u32,
    cap_min: u32,
    cap_max: u32,
) -> (Vec<u32>, Vec<u32>, u32) {
    let (mn, mx) = clamp_len(min_items, max_items, 16);
    let n = sample_len(rng, mn, mx);
    let weights: Vec<u32> = (0..n).map(|_| rng.gen_range(w_min..=w_max)).collect();
    let values: Vec<u32> = (0..n).map(|_| rng.gen_range(v_min..=v_max)).collect();
    let capacity = rng.gen_range(cap_min..=cap_max);
    (weights, values, capacity)
}

/// Generate a random BST insertion sequence.
#[must_use]
pub fn bst_key_sequence(
    rng: &mut ChaCha20Rng,
    min_size: usize,
    max_size: usize,
    key_min: i32,
    key_max: i32,
) -> (Vec<i32>, i32) {
    let (mn, mx) = clamp_len(min_size, max_size, 64);
    let len = sample_len(rng, mn, mx);
    let existing: Vec<i32> = (0..len).map(|_| rng.gen_range(key_min..=key_max)).collect();
    let new_key = rng.gen_range(key_min..=key_max);
    (existing, new_key)
}

/// Generate a BST insertion sequence plus two query keys drawn from it.
#[must_use]
pub fn bst_with_two_keys(
    rng: &mut ChaCha20Rng,
    min_size: usize,
    max_size: usize,
    key_min: i32,
    key_max: i32,
) -> (Vec<i32>, i32, i32) {
    let (mn, mx) = clamp_len(min_size.max(2), max_size, 64);
    let len = sample_len(rng, mn, mx);
    // De-dup since BST-LCA assumes both query keys are in the tree (multiple
    // insertions of the same key collapse to a single node).
    let mut seen = std::collections::HashSet::new();
    let mut keys = Vec::with_capacity(len);
    let mut attempts = 0;
    while keys.len() < len && attempts < len * 8 {
        let k = rng.gen_range(key_min..=key_max);
        if seen.insert(k) {
            keys.push(k);
        }
        attempts += 1;
    }
    if keys.len() < 2 {
        // Fallback: synthesize two distinct keys.
        keys = vec![key_min, key_min.saturating_add(1)];
    }
    let p = keys[rng.gen_range(0..keys.len())];
    let q = keys[rng.gen_range(0..keys.len())];
    (keys, p, q)
}
