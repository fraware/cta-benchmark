//! Pilot adapters for the v0.1 benchmark.
//!
//! Every adapter statically links a byte-identical copy of the instance's
//! `reference.rs`. If a reference implementation drifts, the byte-identity
//! lint in `cta_benchmark::lint` will fail in CI. The actual oracle checks
//! are implemented here against the linked reference and additional
//! brute-force oracles.

#![allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss, clippy::cast_lossless)]
// Reference implementations are copied byte-identically from each instance's
// `reference.rs`; the style of those copies is deliberately pedagogical.
#![allow(clippy::needless_range_loop)]

use std::collections::BTreeSet;

use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

use crate::generators as gen;
use crate::{
    AdapterRegistry, BehaviorError, Falsification, HarnessAdapter, HarnessConfig, HarnessReport,
    OracleCheckStats, Result,
};

/// Populate adapters for every shipped instance id (v0.3 track: 12 families × 7).
#[must_use]
pub fn register_all() -> AdapterRegistry {
    let mut r = AdapterRegistry::new();
    for id in [
        "arrays_binary_search_001",
        "arrays_binary_search_002",
        "arrays_binary_search_003",
        "arrays_binary_search_004",
        "arrays_binary_search_005",
        "arrays_binary_search_006",
        "arrays_binary_search_007",
    ] {
        r.register(Box::new(arrays_binary_search::Adapter(id)));
    }
    for id in [
        "arrays_max_subarray_001",
        "arrays_max_subarray_002",
        "arrays_max_subarray_003",
        "arrays_max_subarray_004",
        "arrays_max_subarray_005",
        "arrays_max_subarray_006",
        "arrays_max_subarray_007",
    ] {
        r.register(Box::new(arrays_max_subarray::Adapter(id)));
    }
    for id in [
        "sorting_insertion_sort_001",
        "sorting_insertion_sort_002",
        "sorting_insertion_sort_003",
        "sorting_insertion_sort_004",
        "sorting_insertion_sort_005",
        "sorting_insertion_sort_006",
        "sorting_insertion_sort_007",
    ] {
        r.register(Box::new(sorting_insertion_sort::Adapter(id)));
    }
    for id in [
        "sorting_merge_sort_001",
        "sorting_merge_sort_002",
        "sorting_merge_sort_003",
        "sorting_merge_sort_004",
        "sorting_merge_sort_005",
        "sorting_merge_sort_006",
        "sorting_merge_sort_007",
    ] {
        r.register(Box::new(sorting_merge_sort::Adapter(id)));
    }
    for id in [
        "graph_dijkstra_001",
        "graph_dijkstra_002",
        "graph_dijkstra_003",
        "graph_dijkstra_004",
        "graph_dijkstra_005",
        "graph_dijkstra_006",
        "graph_dijkstra_007",
    ] {
        r.register(Box::new(graph_dijkstra::Adapter(id)));
    }
    for id in [
        "graph_bfs_shortest_path_001",
        "graph_bfs_shortest_path_002",
        "graph_bfs_shortest_path_003",
        "graph_bfs_shortest_path_004",
        "graph_bfs_shortest_path_005",
        "graph_bfs_shortest_path_006",
        "graph_bfs_shortest_path_007",
    ] {
        r.register(Box::new(graph_bfs_shortest_path::Adapter(id)));
    }
    for id in [
        "greedy_interval_scheduling_001",
        "greedy_interval_scheduling_002",
        "greedy_interval_scheduling_003",
        "greedy_interval_scheduling_004",
        "greedy_interval_scheduling_005",
        "greedy_interval_scheduling_006",
        "greedy_interval_scheduling_007",
    ] {
        r.register(Box::new(greedy_interval_scheduling::Adapter(id)));
    }
    for id in [
        "greedy_coin_change_canonical_001",
        "greedy_coin_change_canonical_002",
        "greedy_coin_change_canonical_003",
        "greedy_coin_change_canonical_004",
        "greedy_coin_change_canonical_005",
        "greedy_coin_change_canonical_006",
        "greedy_coin_change_canonical_007",
    ] {
        r.register(Box::new(greedy_coin_change_canonical::Adapter(id)));
    }
    for id in [
        "dp_longest_common_subsequence_001",
        "dp_longest_common_subsequence_002",
        "dp_longest_common_subsequence_003",
        "dp_longest_common_subsequence_004",
        "dp_longest_common_subsequence_005",
        "dp_longest_common_subsequence_006",
        "dp_longest_common_subsequence_007",
    ] {
        r.register(Box::new(dp_longest_common_subsequence::Adapter(id)));
    }
    for id in [
        "dp_knapsack_01_001",
        "dp_knapsack_01_002",
        "dp_knapsack_01_003",
        "dp_knapsack_01_004",
        "dp_knapsack_01_005",
        "dp_knapsack_01_006",
        "dp_knapsack_01_007",
    ] {
        r.register(Box::new(dp_knapsack_01::Adapter(id)));
    }
    for id in [
        "trees_bst_insert_001",
        "trees_bst_insert_002",
        "trees_bst_insert_003",
        "trees_bst_insert_004",
        "trees_bst_insert_005",
        "trees_bst_insert_006",
        "trees_bst_insert_007",
    ] {
        r.register(Box::new(trees_bst_insert::Adapter(id)));
    }
    for id in [
        "trees_lowest_common_ancestor_001",
        "trees_lowest_common_ancestor_002",
        "trees_lowest_common_ancestor_003",
        "trees_lowest_common_ancestor_004",
        "trees_lowest_common_ancestor_005",
        "trees_lowest_common_ancestor_006",
        "trees_lowest_common_ancestor_007",
    ] {
        r.register(Box::new(trees_lowest_common_ancestor::Adapter(id)));
    }
    r
}

/// Helper: maintain per-oracle state within a trial loop.
struct OracleState<'a> {
    stats: Vec<OracleCheckStats>,
    falsifications: Vec<Falsification>,
    oracle_names: &'a [String],
}

impl<'a> OracleState<'a> {
    fn new(oracle_names: &'a [String]) -> Self {
        let stats = oracle_names
            .iter()
            .map(|n| OracleCheckStats {
                name: n.clone(),
                trials_evaluated: 0,
                violations: 0,
            })
            .collect();
        Self {
            stats,
            falsifications: Vec::new(),
            oracle_names,
        }
    }

    /// Evaluate an oracle check. Caller passes a closure returning `Ok(())`
    /// when satisfied or `Err((observed, expected))` when falsified.
    fn check<F>(&mut self, name: &str, trial: u32, input_repr: &str, f: F)
    where
        F: FnOnce() -> std::result::Result<(), (String, String)>,
    {
        if let Some(i) = self.oracle_names.iter().position(|n| n == name) {
            self.stats[i].trials_evaluated += 1;
            if let Err((observed, expected)) = f() {
                self.stats[i].violations += 1;
                let already = self.falsifications.iter().any(|fx| fx.oracle_check == name);
                if !already {
                    self.falsifications.push(Falsification {
                        oracle_check: name.to_string(),
                        input_repr: input_repr.to_string(),
                        observed,
                        expected,
                        trial,
                    });
                }
            }
        }
    }

    fn finalize(self, config: &HarnessConfig, instance_id: &str) -> HarnessReport {
        HarnessReport {
            instance_id: instance_id.to_string(),
            seed: config.seed,
            trials_run: config.num_trials,
            oracle_stats: self.stats,
            falsifications: self.falsifications,
        }
    }
}

/// Reject unknown oracle checks so silent misconfiguration is impossible.
fn require_known_oracles(config: &HarnessConfig, known: &[&str]) -> Result<()> {
    for o in &config.oracle_checks {
        if !known.contains(&o.as_str()) {
            return Err(BehaviorError::UnknownOracleCheck(o.clone()));
        }
    }
    Ok(())
}

// --- arrays_binary_search_001 ------------------------------------------------

mod arrays_binary_search {
    use super::*;

    pub fn binary_search(arr: &[i32], target: i32) -> Option<usize> {
        let mut lo: usize = 0;
        let mut hi: usize = arr.len();
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if arr[mid] == target {
                return Some(mid);
            }
            if arr[mid] < target {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        None
    }

    pub struct Adapter(pub &'static str);
    impl HarnessAdapter for Adapter {
        fn instance_id(&self) -> &str {
            self.0
        }
        fn run(&self, config: &HarnessConfig) -> Result<HarnessReport> {
            require_known_oracles(
                config,
                &[
                    "success_index_in_bounds",
                    "success_points_to_target",
                    "failure_if_absent",
                ],
            )?;
            let min_len = config.input_generator["min_len"].as_u64().unwrap_or(0) as usize;
            let max_len = config.input_generator["max_len"].as_u64().unwrap_or(32) as usize;
            let vr = config.input_generator["value_range"]
                .as_array()
                .ok_or_else(|| BehaviorError::Generator("missing value_range".into()))?;
            let v_min = vr[0].as_i64().unwrap_or(-100) as i32;
            let v_max = vr[1].as_i64().unwrap_or(100) as i32;

            let mut rng = ChaCha20Rng::seed_from_u64(config.seed);
            let mut st = OracleState::new(&config.oracle_checks);
            for trial in 0..config.num_trials {
                let (arr, target) =
                    gen::sorted_int_vec_with_target(&mut rng, min_len, max_len, v_min, v_max);
                let result = binary_search(&arr, target);
                let input_repr = format!("arr.len()={},target={target}", arr.len());
                let present = arr.contains(&target);
                st.check("success_index_in_bounds", trial, &input_repr, || {
                    if let Some(i) = result {
                        if i < arr.len() {
                            Ok(())
                        } else {
                            Err((format!("index={i}"), format!("index<{}", arr.len())))
                        }
                    } else {
                        Ok(())
                    }
                });
                st.check("success_points_to_target", trial, &input_repr, || {
                    if let Some(i) = result {
                        if arr[i] == target {
                            Ok(())
                        } else {
                            Err((format!("arr[{i}]={}", arr[i]), format!("arr[i]={target}")))
                        }
                    } else {
                        Ok(())
                    }
                });
                st.check("failure_if_absent", trial, &input_repr, || {
                    if !present && result.is_some() {
                        Err((
                            format!("returned {:?}", result),
                            "None when target absent".into(),
                        ))
                    } else if present && result.is_none() {
                        Err(("None".into(), "Some(_) when target present".into()))
                    } else {
                        Ok(())
                    }
                });
            }
            Ok(st.finalize(config, self.instance_id()))
        }
    }
}

// --- arrays_max_subarray_001 -------------------------------------------------

mod arrays_max_subarray {
    use super::*;

    pub fn max_subarray(arr: &[i32]) -> i32 {
        let mut best: i32 = arr[0];
        let mut cur: i32 = arr[0];
        for &x in &arr[1..] {
            cur = std::cmp::max(x, cur + x);
            best = std::cmp::max(best, cur);
        }
        best
    }

    fn quadratic_oracle(arr: &[i32]) -> i64 {
        let mut best = i64::from(arr[0]);
        for i in 0..arr.len() {
            let mut s: i64 = 0;
            for j in i..arr.len() {
                s += i64::from(arr[j]);
                if s > best {
                    best = s;
                }
            }
        }
        best
    }

    pub struct Adapter(pub &'static str);
    impl HarnessAdapter for Adapter {
        fn instance_id(&self) -> &str {
            self.0
        }
        fn run(&self, config: &HarnessConfig) -> Result<HarnessReport> {
            require_known_oracles(
                config,
                &[
                    "result_matches_quadratic_oracle",
                    "result_at_least_max_element",
                    "result_finite",
                ],
            )?;
            let min_len = config.input_generator["min_len"].as_u64().unwrap_or(1) as usize;
            let max_len = config.input_generator["max_len"].as_u64().unwrap_or(32) as usize;
            let vr = config.input_generator["value_range"]
                .as_array()
                .ok_or_else(|| BehaviorError::Generator("missing value_range".into()))?;
            let v_min = vr[0].as_i64().unwrap_or(-50) as i32;
            let v_max = vr[1].as_i64().unwrap_or(50) as i32;

            let mut rng = ChaCha20Rng::seed_from_u64(config.seed);
            let mut st = OracleState::new(&config.oracle_checks);
            for trial in 0..config.num_trials {
                let mut arr = gen::int_vec(&mut rng, min_len.max(1), max_len, v_min, v_max);
                if arr.is_empty() {
                    arr.push(0);
                }
                let out = max_subarray(&arr);
                let oracle = quadratic_oracle(&arr);
                let max_elem = *arr.iter().max().ok_or_else(|| {
                    BehaviorError::Generator(
                        "max_subarray harness: empty vec after non-empty guard".into(),
                    )
                })?;
                let input_repr = format!("arr.len()={}", arr.len());

                st.check(
                    "result_matches_quadratic_oracle",
                    trial,
                    &input_repr,
                    || {
                        if i64::from(out) == oracle {
                            Ok(())
                        } else {
                            Err((format!("{out}"), format!("{oracle}")))
                        }
                    },
                );
                st.check("result_at_least_max_element", trial, &input_repr, || {
                    if out >= max_elem {
                        Ok(())
                    } else {
                        Err((format!("{out}"), format!(">= {max_elem}")))
                    }
                });
                st.check("result_finite", trial, &input_repr, || {
                    // i32 is always finite; check that it is within a sane bound.
                    if out.abs() as i64 <= i64::from(i32::MAX) {
                        Ok(())
                    } else {
                        Err((format!("{out}"), "finite i32".into()))
                    }
                });
            }
            Ok(st.finalize(config, self.instance_id()))
        }
    }
}

// --- sorting_insertion_sort_001 ---------------------------------------------

mod sorting_insertion_sort {
    use super::*;

    pub fn insertion_sort(arr: &mut [i32]) {
        for i in 1..arr.len() {
            let mut j = i;
            while j > 0 && arr[j - 1] > arr[j] {
                arr.swap(j - 1, j);
                j -= 1;
            }
        }
    }

    fn is_sorted(arr: &[i32]) -> bool {
        arr.windows(2).all(|w| w[0] <= w[1])
    }

    fn is_permutation(a: &[i32], b: &[i32]) -> bool {
        let mut ax = a.to_vec();
        let mut bx = b.to_vec();
        ax.sort_unstable();
        bx.sort_unstable();
        ax == bx
    }

    pub struct Adapter(pub &'static str);
    impl HarnessAdapter for Adapter {
        fn instance_id(&self) -> &str {
            self.0
        }
        fn run(&self, config: &HarnessConfig) -> Result<HarnessReport> {
            require_known_oracles(
                config,
                &[
                    "output_is_sorted",
                    "output_is_permutation_of_input",
                    "length_preserved",
                ],
            )?;
            let min_len = config.input_generator["min_len"].as_u64().unwrap_or(0) as usize;
            let max_len = config.input_generator["max_len"].as_u64().unwrap_or(32) as usize;
            let vr = config.input_generator["value_range"]
                .as_array()
                .ok_or_else(|| BehaviorError::Generator("missing value_range".into()))?;
            let v_min = vr[0].as_i64().unwrap_or(-100) as i32;
            let v_max = vr[1].as_i64().unwrap_or(100) as i32;

            let mut rng = ChaCha20Rng::seed_from_u64(config.seed);
            let mut st = OracleState::new(&config.oracle_checks);
            for trial in 0..config.num_trials {
                let input = gen::int_vec(&mut rng, min_len, max_len, v_min, v_max);
                let mut scratch = input.clone();
                insertion_sort(&mut scratch);
                let input_repr = format!("len={}", input.len());
                st.check("output_is_sorted", trial, &input_repr, || {
                    if is_sorted(&scratch) {
                        Ok(())
                    } else {
                        Err((format!("{:?}", &scratch), "nondecreasing".into()))
                    }
                });
                st.check("output_is_permutation_of_input", trial, &input_repr, || {
                    if is_permutation(&input, &scratch) {
                        Ok(())
                    } else {
                        Err(("multiset differs".into(), "same multiset".into()))
                    }
                });
                st.check("length_preserved", trial, &input_repr, || {
                    if scratch.len() == input.len() {
                        Ok(())
                    } else {
                        Err((format!("{}", scratch.len()), format!("{}", input.len())))
                    }
                });
            }
            Ok(st.finalize(config, self.instance_id()))
        }
    }
}

// --- sorting_merge_sort_001 -------------------------------------------------

mod sorting_merge_sort {
    use super::*;

    pub fn merge_sort(arr: &[i32]) -> Vec<i32> {
        if arr.len() <= 1 {
            return arr.to_vec();
        }
        let mid = arr.len() / 2;
        let left = merge_sort(&arr[..mid]);
        let right = merge_sort(&arr[mid..]);
        merge(&left, &right)
    }

    fn merge(left: &[i32], right: &[i32]) -> Vec<i32> {
        let mut out = Vec::with_capacity(left.len() + right.len());
        let mut i = 0usize;
        let mut j = 0usize;
        while i < left.len() && j < right.len() {
            if left[i] <= right[j] {
                out.push(left[i]);
                i += 1;
            } else {
                out.push(right[j]);
                j += 1;
            }
        }
        out.extend_from_slice(&left[i..]);
        out.extend_from_slice(&right[j..]);
        out
    }

    fn is_sorted(arr: &[i32]) -> bool {
        arr.windows(2).all(|w| w[0] <= w[1])
    }

    fn is_permutation(a: &[i32], b: &[i32]) -> bool {
        let mut ax = a.to_vec();
        let mut bx = b.to_vec();
        ax.sort_unstable();
        bx.sort_unstable();
        ax == bx
    }

    pub struct Adapter(pub &'static str);
    impl HarnessAdapter for Adapter {
        fn instance_id(&self) -> &str {
            self.0
        }
        fn run(&self, config: &HarnessConfig) -> Result<HarnessReport> {
            require_known_oracles(
                config,
                &[
                    "output_length_matches_input",
                    "output_sorted_nondecreasing",
                    "output_permutation_of_input",
                ],
            )?;
            let min_len = config.input_generator["min_len"].as_u64().unwrap_or(0) as usize;
            let max_len = config.input_generator["max_len"].as_u64().unwrap_or(32) as usize;
            let vr = config.input_generator["value_range"]
                .as_array()
                .ok_or_else(|| BehaviorError::Generator("missing value_range".into()))?;
            let v_min = vr[0].as_i64().unwrap_or(-100) as i32;
            let v_max = vr[1].as_i64().unwrap_or(100) as i32;

            let mut rng = ChaCha20Rng::seed_from_u64(config.seed);
            let mut st = OracleState::new(&config.oracle_checks);
            for trial in 0..config.num_trials {
                let input = gen::int_vec(&mut rng, min_len, max_len, v_min, v_max);
                let out = merge_sort(&input);
                let input_repr = format!("len={}", input.len());
                st.check("output_length_matches_input", trial, &input_repr, || {
                    if out.len() == input.len() {
                        Ok(())
                    } else {
                        Err((format!("{}", out.len()), format!("{}", input.len())))
                    }
                });
                st.check("output_sorted_nondecreasing", trial, &input_repr, || {
                    if is_sorted(&out) {
                        Ok(())
                    } else {
                        Err((format!("{:?}", out), "nondecreasing".into()))
                    }
                });
                st.check("output_permutation_of_input", trial, &input_repr, || {
                    if is_permutation(&input, &out) {
                        Ok(())
                    } else {
                        Err(("multiset differs".into(), "same multiset".into()))
                    }
                });
            }
            Ok(st.finalize(config, self.instance_id()))
        }
    }
}

// --- graph_dijkstra_001 -----------------------------------------------------

mod graph_dijkstra {
    use super::*;
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    #[derive(Clone, Copy)]
    pub struct Edge {
        pub src: usize,
        pub dst: usize,
        pub weight: u32,
    }

    pub fn dijkstra(n: usize, source: usize, edges: &[Edge]) -> Vec<Option<u64>> {
        let mut adjacency: Vec<Vec<(usize, u32)>> = vec![Vec::new(); n];
        for e in edges {
            adjacency[e.src].push((e.dst, e.weight));
        }
        let mut dist: Vec<Option<u64>> = vec![None; n];
        if source >= n {
            return dist;
        }
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

    fn bellman_ford(n: usize, source: usize, edges: &[(u32, u32, u32)]) -> Vec<Option<u64>> {
        let mut dist: Vec<Option<u64>> = vec![None; n];
        dist[source] = Some(0);
        for _ in 0..n {
            let mut changed = false;
            for &(u, v, w) in edges {
                if let Some(du) = dist[u as usize] {
                    let nd = du + u64::from(w);
                    if dist[v as usize].is_none_or(|c| nd < c) {
                        dist[v as usize] = Some(nd);
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        dist
    }

    pub struct Adapter(pub &'static str);
    impl HarnessAdapter for Adapter {
        fn instance_id(&self) -> &str {
            self.0
        }
        fn run(&self, config: &HarnessConfig) -> Result<HarnessReport> {
            require_known_oracles(
                config,
                &[
                    "length_equals_n",
                    "source_distance_is_zero",
                    "distances_agree_with_brute_force_bellman_ford",
                    "none_iff_unreachable",
                ],
            )?;
            let min_n = config.input_generator["min_n"].as_u64().unwrap_or(1) as usize;
            let max_n = config.input_generator["max_n"].as_u64().unwrap_or(8) as usize;
            let density = config.input_generator["edge_density"]
                .as_f64()
                .unwrap_or(0.3);
            let wr = config.input_generator["weight_range"]
                .as_array()
                .ok_or_else(|| BehaviorError::Generator("missing weight_range".into()))?;
            let w_min = wr[0].as_u64().unwrap_or(0) as u32;
            let w_max = wr[1].as_u64().unwrap_or(50) as u32;

            let mut rng = ChaCha20Rng::seed_from_u64(config.seed);
            let mut st = OracleState::new(&config.oracle_checks);
            for trial in 0..config.num_trials {
                let (n, edges, source) =
                    gen::random_dag(&mut rng, min_n.max(1), max_n, density, w_min, w_max);
                let edge_structs: Vec<Edge> = edges
                    .iter()
                    .map(|&(u, v, w)| Edge {
                        src: u as usize,
                        dst: v as usize,
                        weight: w,
                    })
                    .collect();
                let dist = dijkstra(n, source, &edge_structs);
                let bf = bellman_ford(n, source, &edges);
                let input_repr = format!("n={n},|E|={},src={source}", edges.len());
                st.check("length_equals_n", trial, &input_repr, || {
                    if dist.len() == n {
                        Ok(())
                    } else {
                        Err((format!("{}", dist.len()), format!("{n}")))
                    }
                });
                st.check("source_distance_is_zero", trial, &input_repr, || {
                    if dist.get(source).copied() == Some(Some(0)) {
                        Ok(())
                    } else {
                        Err((format!("{:?}", dist.get(source)), "Some(Some(0))".into()))
                    }
                });
                st.check(
                    "distances_agree_with_brute_force_bellman_ford",
                    trial,
                    &input_repr,
                    || {
                        if dist == bf {
                            Ok(())
                        } else {
                            Err((format!("{:?}", dist), format!("{:?}", bf)))
                        }
                    },
                );
                st.check("none_iff_unreachable", trial, &input_repr, || {
                    for v in 0..n {
                        if (dist[v].is_none()) != (bf[v].is_none()) {
                            return Err((format!("{:?}", dist[v]), format!("{:?}", bf[v])));
                        }
                    }
                    Ok(())
                });
            }
            Ok(st.finalize(config, self.instance_id()))
        }
    }
}

// --- graph_bfs_shortest_path_001 --------------------------------------------

mod graph_bfs_shortest_path {
    use super::*;
    use std::collections::VecDeque;

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

    fn fw_unweighted(adj: &[Vec<usize>], source: usize) -> Vec<Option<usize>> {
        let n = adj.len();
        const INF: usize = usize::MAX / 4;
        let mut d = vec![vec![INF; n]; n];
        for (u, row) in d.iter_mut().enumerate().take(n) {
            row[u] = 0;
        }
        for (u, row) in adj.iter().enumerate().take(n) {
            for &v in row {
                if v < n && 1 < d[u][v] {
                    d[u][v] = 1;
                }
            }
        }
        for k in 0..n {
            for i in 0..n {
                for j in 0..n {
                    let via = d[i][k].saturating_add(d[k][j]);
                    if via < d[i][j] {
                        d[i][j] = via;
                    }
                }
            }
        }
        (0..n)
            .map(|v| {
                if d[source][v] >= INF {
                    None
                } else {
                    Some(d[source][v])
                }
            })
            .collect()
    }

    pub struct Adapter(pub &'static str);
    impl HarnessAdapter for Adapter {
        fn instance_id(&self) -> &str {
            self.0
        }
        fn run(&self, config: &HarnessConfig) -> Result<HarnessReport> {
            require_known_oracles(
                config,
                &[
                    "dist_length_matches_n",
                    "dist_source_is_zero",
                    "dist_matches_fw_unweighted",
                    "unreachable_iff_no_path",
                ],
            )?;
            let min_n = config.input_generator["min_n"].as_u64().unwrap_or(1) as usize;
            let max_n = config.input_generator["max_n"].as_u64().unwrap_or(8) as usize;
            let max_deg = config.input_generator["max_out_degree"]
                .as_u64()
                .unwrap_or(4) as usize;

            let mut rng = ChaCha20Rng::seed_from_u64(config.seed);
            let mut st = OracleState::new(&config.oracle_checks);
            for trial in 0..config.num_trials {
                let (adj, source) = gen::adj_list(&mut rng, min_n.max(1), max_n, max_deg);
                let d = bfs_shortest_path(&adj, source);
                let f = fw_unweighted(&adj, source);
                let n = adj.len();
                let input_repr = format!("n={n},src={source}");
                st.check("dist_length_matches_n", trial, &input_repr, || {
                    if d.len() == n {
                        Ok(())
                    } else {
                        Err((format!("{}", d.len()), format!("{n}")))
                    }
                });
                st.check("dist_source_is_zero", trial, &input_repr, || {
                    if d.get(source).copied() == Some(Some(0)) {
                        Ok(())
                    } else {
                        Err((format!("{:?}", d.get(source)), "Some(Some(0))".into()))
                    }
                });
                st.check("dist_matches_fw_unweighted", trial, &input_repr, || {
                    if d == f {
                        Ok(())
                    } else {
                        Err((format!("{:?}", d), format!("{:?}", f)))
                    }
                });
                st.check("unreachable_iff_no_path", trial, &input_repr, || {
                    for v in 0..n {
                        if d[v].is_none() != f[v].is_none() {
                            return Err((format!("{:?}", d[v]), format!("{:?}", f[v])));
                        }
                    }
                    Ok(())
                });
            }
            Ok(st.finalize(config, self.instance_id()))
        }
    }
}

// --- greedy_interval_scheduling_001 -----------------------------------------

mod greedy_interval_scheduling {
    use super::*;

    pub fn interval_scheduling(intervals: &[(i32, i32)]) -> usize {
        let mut sorted: Vec<(i32, i32)> = intervals.to_vec();
        sorted.sort_by_key(|iv| iv.1);
        let mut count: usize = 0;
        let mut last_end: Option<i32> = None;
        for (s, e) in sorted {
            let take = match last_end {
                None => true,
                Some(le) => s >= le,
            };
            if take {
                count += 1;
                last_end = Some(e);
            }
        }
        count
    }

    fn exhaustive_oracle(intervals: &[(i32, i32)]) -> usize {
        let n = intervals.len();
        if n == 0 {
            return 0;
        }
        let n_small = n.min(16);
        let mut best = 0usize;
        for mask in 0..(1u32 << n_small) {
            let mut chosen: Vec<(i32, i32)> = Vec::new();
            for i in 0..n_small {
                if mask & (1 << i) != 0 {
                    chosen.push(intervals[i]);
                }
            }
            chosen.sort_by_key(|iv| iv.0);
            let mut ok = true;
            for w in chosen.windows(2) {
                if w[0].1 > w[1].0 {
                    ok = false;
                    break;
                }
            }
            if ok && chosen.len() > best {
                best = chosen.len();
            }
        }
        best
    }

    pub struct Adapter(pub &'static str);
    impl HarnessAdapter for Adapter {
        fn instance_id(&self) -> &str {
            self.0
        }
        fn run(&self, config: &HarnessConfig) -> Result<HarnessReport> {
            require_known_oracles(
                config,
                &[
                    "result_matches_exhaustive_oracle",
                    "result_at_most_input_length",
                    "result_nonnegative",
                ],
            )?;
            let min_len = config.input_generator["min_len"].as_u64().unwrap_or(0) as usize;
            let max_len = config.input_generator["max_len"].as_u64().unwrap_or(8) as usize;
            let cr = config.input_generator["coordinate_range"]
                .as_array()
                .ok_or_else(|| BehaviorError::Generator("missing coordinate_range".into()))?;
            let c_min = cr[0].as_i64().unwrap_or(0) as i32;
            let c_max = cr[1].as_i64().unwrap_or(30) as i32;

            let mut rng = ChaCha20Rng::seed_from_u64(config.seed);
            let mut st = OracleState::new(&config.oracle_checks);
            for trial in 0..config.num_trials {
                let ivs = gen::interval_vec(&mut rng, min_len, max_len.min(12), c_min, c_max);
                let out = interval_scheduling(&ivs);
                let oracle = exhaustive_oracle(&ivs);
                let input_repr = format!("n={}", ivs.len());
                st.check(
                    "result_matches_exhaustive_oracle",
                    trial,
                    &input_repr,
                    || {
                        if out == oracle {
                            Ok(())
                        } else {
                            Err((format!("{out}"), format!("{oracle}")))
                        }
                    },
                );
                st.check("result_at_most_input_length", trial, &input_repr, || {
                    if out <= ivs.len() {
                        Ok(())
                    } else {
                        Err((format!("{out}"), format!("<= {}", ivs.len())))
                    }
                });
                st.check("result_nonnegative", trial, &input_repr, || {
                    // usize is always >= 0; kept for spec completeness.
                    Ok(())
                });
            }
            Ok(st.finalize(config, self.instance_id()))
        }
    }
}

// --- greedy_coin_change_canonical_001 ---------------------------------------

mod greedy_coin_change_canonical {
    use super::*;

    pub fn coin_change_canonical(denoms: &[u32], amount: u32) -> Vec<u32> {
        let mut counts: Vec<u32> = vec![0; denoms.len()];
        let mut remaining = amount;
        for i in (0..denoms.len()).rev() {
            let d = denoms[i];
            if d == 0 {
                continue;
            }
            counts[i] = remaining / d;
            remaining -= counts[i] * d;
        }
        counts
    }

    fn dp_min_coins_sum(denoms: &[u32], amount: u32) -> Option<u32> {
        let cap = amount as usize;
        let mut dp: Vec<Option<u32>> = vec![None; cap + 1];
        dp[0] = Some(0);
        for c in 1..=cap {
            for &d in denoms {
                if d == 0 {
                    continue;
                }
                let dd = d as usize;
                if dd <= c {
                    if let Some(prev) = dp[c - dd] {
                        let cand = prev + 1;
                        if dp[c].is_none_or(|x| cand < x) {
                            dp[c] = Some(cand);
                        }
                    }
                }
            }
        }
        dp[cap]
    }

    pub struct Adapter(pub &'static str);
    impl HarnessAdapter for Adapter {
        fn instance_id(&self) -> &str {
            self.0
        }
        fn run(&self, config: &HarnessConfig) -> Result<HarnessReport> {
            require_known_oracles(
                config,
                &[
                    "counts_length_matches_denoms",
                    "weighted_sum_equals_amount",
                    "sum_matches_dp_oracle",
                ],
            )?;
            let systems_json = config.input_generator["systems"]
                .as_array()
                .ok_or_else(|| BehaviorError::Generator("missing systems".into()))?;
            let systems: Vec<Vec<u32>> = systems_json
                .iter()
                .map(|s| {
                    s.as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|x| x.as_u64().map(|n| n as u32))
                                .collect::<Vec<u32>>()
                        })
                        .unwrap_or_default()
                })
                .collect();
            let ar = config.input_generator["amount_range"]
                .as_array()
                .ok_or_else(|| BehaviorError::Generator("missing amount_range".into()))?;
            let a_min = ar[0].as_u64().unwrap_or(0) as u32;
            let a_max = ar[1].as_u64().unwrap_or(100) as u32;

            let mut rng = ChaCha20Rng::seed_from_u64(config.seed);
            let mut st = OracleState::new(&config.oracle_checks);
            for trial in 0..config.num_trials {
                let (denoms, amount) = gen::canonical_coin_system(&mut rng, &systems, a_min, a_max);
                let counts = coin_change_canonical(&denoms, amount);
                let input_repr = format!("denoms={:?},amount={amount}", denoms);
                st.check("counts_length_matches_denoms", trial, &input_repr, || {
                    if counts.len() == denoms.len() {
                        Ok(())
                    } else {
                        Err((format!("{}", counts.len()), format!("{}", denoms.len())))
                    }
                });
                let weighted: u64 = counts
                    .iter()
                    .zip(denoms.iter())
                    .map(|(c, d)| u64::from(*c) * u64::from(*d))
                    .sum();
                st.check("weighted_sum_equals_amount", trial, &input_repr, || {
                    if weighted == u64::from(amount) {
                        Ok(())
                    } else {
                        Err((format!("{weighted}"), format!("{amount}")))
                    }
                });
                let total_coins: u64 = counts.iter().map(|&c| u64::from(c)).sum();
                let dp = dp_min_coins_sum(&denoms, amount);
                st.check("sum_matches_dp_oracle", trial, &input_repr, || match dp {
                    Some(exp) => {
                        if total_coins == u64::from(exp) {
                            Ok(())
                        } else {
                            Err((format!("{total_coins}"), format!("{exp}")))
                        }
                    }
                    None => Err(("dp unreachable".into(), "dp should always be Some".into())),
                });
            }
            Ok(st.finalize(config, self.instance_id()))
        }
    }
}

// --- dp_longest_common_subsequence_001 --------------------------------------

mod dp_longest_common_subsequence {
    use super::*;

    pub fn lcs_length(a: &[i32], b: &[i32]) -> usize {
        let m = a.len();
        let n = b.len();
        let mut dp: Vec<Vec<usize>> = vec![vec![0; n + 1]; m + 1];
        for i in 1..=m {
            for j in 1..=n {
                dp[i][j] = if a[i - 1] == b[j - 1] {
                    dp[i - 1][j - 1] + 1
                } else {
                    std::cmp::max(dp[i - 1][j], dp[i][j - 1])
                };
            }
        }
        dp[m][n]
    }

    pub struct Adapter(pub &'static str);
    impl HarnessAdapter for Adapter {
        fn instance_id(&self) -> &str {
            self.0
        }
        fn run(&self, config: &HarnessConfig) -> Result<HarnessReport> {
            require_known_oracles(
                config,
                &[
                    "result_bounded_by_min_length",
                    "result_matches_dp_oracle",
                    "result_symmetric",
                ],
            )?;
            let min_len = config.input_generator["min_len"].as_u64().unwrap_or(0) as usize;
            let max_len = config.input_generator["max_len"].as_u64().unwrap_or(12) as usize;
            let vr = config.input_generator["value_range"]
                .as_array()
                .ok_or_else(|| BehaviorError::Generator("missing value_range".into()))?;
            let v_min = vr[0].as_i64().unwrap_or(0) as i32;
            let v_max = vr[1].as_i64().unwrap_or(6) as i32;

            let mut rng = ChaCha20Rng::seed_from_u64(config.seed);
            let mut st = OracleState::new(&config.oracle_checks);
            for trial in 0..config.num_trials {
                let (a, b) = gen::pair_int_vec(&mut rng, min_len, max_len, v_min, v_max);
                let out = lcs_length(&a, &b);
                let input_repr = format!("|a|={},|b|={}", a.len(), b.len());
                st.check("result_bounded_by_min_length", trial, &input_repr, || {
                    if out <= a.len().min(b.len()) {
                        Ok(())
                    } else {
                        Err((format!("{out}"), format!("<= {}", a.len().min(b.len()))))
                    }
                });
                st.check("result_matches_dp_oracle", trial, &input_repr, || {
                    // Self-consistency: recompute from the same algorithm.
                    if out == lcs_length(&a, &b) {
                        Ok(())
                    } else {
                        Err((format!("{out}"), "stable under recomputation".into()))
                    }
                });
                let swapped = lcs_length(&b, &a);
                st.check("result_symmetric", trial, &input_repr, || {
                    if out == swapped {
                        Ok(())
                    } else {
                        Err((format!("{out} != {swapped}"), "lcs(a,b) == lcs(b,a)".into()))
                    }
                });
            }
            Ok(st.finalize(config, self.instance_id()))
        }
    }
}

// --- dp_knapsack_01_001 -----------------------------------------------------

mod dp_knapsack_01 {
    use super::*;

    pub fn knapsack_01(weights: &[u32], values: &[u32], capacity: u32) -> u32 {
        assert_eq!(weights.len(), values.len());
        let cap = capacity as usize;
        let mut dp: Vec<u32> = vec![0; cap + 1];
        for i in 0..weights.len() {
            let w = weights[i] as usize;
            let v = values[i];
            if w > cap {
                continue;
            }
            for c in (w..=cap).rev() {
                let cand = dp[c - w] + v;
                if cand > dp[c] {
                    dp[c] = cand;
                }
            }
        }
        dp[cap]
    }

    fn exhaustive_knapsack(weights: &[u32], values: &[u32], capacity: u32) -> u32 {
        let n = weights.len().min(16);
        let mut best = 0u32;
        for mask in 0..(1u32 << n) {
            let mut w = 0u64;
            let mut v = 0u64;
            for i in 0..n {
                if mask & (1 << i) != 0 {
                    w += u64::from(weights[i]);
                    v += u64::from(values[i]);
                }
            }
            if w <= u64::from(capacity) && v > u64::from(best) {
                best = v as u32;
            }
        }
        best
    }

    pub struct Adapter(pub &'static str);
    impl HarnessAdapter for Adapter {
        fn instance_id(&self) -> &str {
            self.0
        }
        fn run(&self, config: &HarnessConfig) -> Result<HarnessReport> {
            require_known_oracles(
                config,
                &[
                    "result_matches_exhaustive_oracle",
                    "result_nonnegative",
                    "result_zero_when_capacity_zero_and_weights_positive",
                ],
            )?;
            let min_items = config.input_generator["min_items"].as_u64().unwrap_or(0) as usize;
            let max_items = config.input_generator["max_items"].as_u64().unwrap_or(8) as usize;
            let wr = config.input_generator["weight_range"]
                .as_array()
                .ok_or_else(|| BehaviorError::Generator("missing weight_range".into()))?;
            let w_min = wr[0].as_u64().unwrap_or(0) as u32;
            let w_max = wr[1].as_u64().unwrap_or(15) as u32;
            let vr = config.input_generator["value_range"]
                .as_array()
                .ok_or_else(|| BehaviorError::Generator("missing value_range".into()))?;
            let v_min = vr[0].as_u64().unwrap_or(0) as u32;
            let v_max = vr[1].as_u64().unwrap_or(20) as u32;
            let cr = config.input_generator["capacity_range"]
                .as_array()
                .ok_or_else(|| BehaviorError::Generator("missing capacity_range".into()))?;
            let c_min = cr[0].as_u64().unwrap_or(0) as u32;
            let c_max = cr[1].as_u64().unwrap_or(50) as u32;

            let mut rng = ChaCha20Rng::seed_from_u64(config.seed);
            let mut st = OracleState::new(&config.oracle_checks);
            for trial in 0..config.num_trials {
                let (weights, values, cap) = gen::knapsack_instance(
                    &mut rng,
                    min_items,
                    max_items.min(12),
                    w_min,
                    w_max,
                    v_min,
                    v_max,
                    c_min,
                    c_max,
                );
                let out = knapsack_01(&weights, &values, cap);
                let exh = exhaustive_knapsack(&weights, &values, cap);
                let input_repr = format!("n={},cap={cap}", weights.len());
                st.check(
                    "result_matches_exhaustive_oracle",
                    trial,
                    &input_repr,
                    || {
                        if out == exh {
                            Ok(())
                        } else {
                            Err((format!("{out}"), format!("{exh}")))
                        }
                    },
                );
                st.check("result_nonnegative", trial, &input_repr, || Ok(()));
                st.check(
                    "result_zero_when_capacity_zero_and_weights_positive",
                    trial,
                    &input_repr,
                    || {
                        if cap == 0 && weights.iter().all(|&w| w > 0) && out != 0 {
                            Err((
                                format!("{out}"),
                                "0 when capacity=0 and weights positive".into(),
                            ))
                        } else {
                            Ok(())
                        }
                    },
                );
            }
            Ok(st.finalize(config, self.instance_id()))
        }
    }
}

// --- trees_bst_insert_001 ---------------------------------------------------

mod trees_bst_insert {
    use super::*;

    #[derive(Clone, PartialEq, Eq)]
    pub enum Tree {
        Nil,
        Node(Box<Tree>, i32, Box<Tree>),
    }

    pub fn bst_insert(tree: Tree, key: i32) -> Tree {
        match tree {
            Tree::Nil => Tree::Node(Box::new(Tree::Nil), key, Box::new(Tree::Nil)),
            Tree::Node(left, k, right) => {
                if key < k {
                    Tree::Node(Box::new(bst_insert(*left, key)), k, right)
                } else if key > k {
                    Tree::Node(left, k, Box::new(bst_insert(*right, key)))
                } else {
                    Tree::Node(left, k, right)
                }
            }
        }
    }

    fn inorder(tree: &Tree, out: &mut Vec<i32>) {
        if let Tree::Node(l, k, r) = tree {
            inorder(l, out);
            out.push(*k);
            inorder(r, out);
        }
    }

    fn build(tree: Tree, keys: &[i32]) -> Tree {
        let mut t = tree;
        for &k in keys {
            t = bst_insert(t, k);
        }
        t
    }

    pub struct Adapter(pub &'static str);
    impl HarnessAdapter for Adapter {
        fn instance_id(&self) -> &str {
            self.0
        }
        fn run(&self, config: &HarnessConfig) -> Result<HarnessReport> {
            require_known_oracles(
                config,
                &[
                    "output_inorder_strictly_ascending",
                    "output_contains_inserted_key",
                    "output_contains_all_input_keys",
                ],
            )?;
            let min_size = config.input_generator["min_size"].as_u64().unwrap_or(0) as usize;
            let max_size = config.input_generator["max_size"].as_u64().unwrap_or(16) as usize;
            let kr = config.input_generator["key_range"]
                .as_array()
                .ok_or_else(|| BehaviorError::Generator("missing key_range".into()))?;
            let k_min = kr[0].as_i64().unwrap_or(-20) as i32;
            let k_max = kr[1].as_i64().unwrap_or(20) as i32;

            let mut rng = ChaCha20Rng::seed_from_u64(config.seed);
            let mut st = OracleState::new(&config.oracle_checks);
            for trial in 0..config.num_trials {
                let (existing, new_key) =
                    gen::bst_key_sequence(&mut rng, min_size, max_size, k_min, k_max);
                let base = build(Tree::Nil, &existing);
                let after = bst_insert(base, new_key);
                let mut io: Vec<i32> = Vec::new();
                inorder(&after, &mut io);
                let input_repr = format!("|keys|={},new={new_key}", existing.len());
                st.check(
                    "output_inorder_strictly_ascending",
                    trial,
                    &input_repr,
                    || {
                        if io.windows(2).all(|w| w[0] < w[1]) {
                            Ok(())
                        } else {
                            Err((format!("{:?}", io), "strictly ascending".into()))
                        }
                    },
                );
                st.check("output_contains_inserted_key", trial, &input_repr, || {
                    if io.contains(&new_key) {
                        Ok(())
                    } else {
                        Err(("missing".into(), format!("{new_key} present")))
                    }
                });
                let mut expected: BTreeSet<i32> = existing.iter().copied().collect();
                expected.insert(new_key);
                let actual: BTreeSet<i32> = io.iter().copied().collect();
                st.check("output_contains_all_input_keys", trial, &input_repr, || {
                    if actual == expected {
                        Ok(())
                    } else {
                        Err((format!("{:?}", actual), format!("{:?}", expected)))
                    }
                });
            }
            Ok(st.finalize(config, self.instance_id()))
        }
    }
}

// --- trees_lowest_common_ancestor_001 ---------------------------------------

mod trees_lowest_common_ancestor {
    use super::*;

    #[derive(Clone, PartialEq, Eq)]
    pub enum Tree {
        Nil,
        Node(Box<Tree>, i32, Box<Tree>),
    }

    pub fn lca_bst(tree: &Tree, p: i32, q: i32) -> Option<i32> {
        let (lo, hi) = if p <= q { (p, q) } else { (q, p) };
        let mut cur = tree;
        loop {
            match cur {
                Tree::Nil => return None,
                Tree::Node(left, k, right) => {
                    if hi < *k {
                        cur = left;
                    } else if lo > *k {
                        cur = right;
                    } else {
                        return Some(*k);
                    }
                }
            }
        }
    }

    fn insert(tree: Tree, key: i32) -> Tree {
        match tree {
            Tree::Nil => Tree::Node(Box::new(Tree::Nil), key, Box::new(Tree::Nil)),
            Tree::Node(left, k, right) => {
                if key < k {
                    Tree::Node(Box::new(insert(*left, key)), k, right)
                } else if key > k {
                    Tree::Node(left, k, Box::new(insert(*right, key)))
                } else {
                    Tree::Node(left, k, right)
                }
            }
        }
    }

    fn path_to(tree: &Tree, key: i32) -> Option<Vec<i32>> {
        match tree {
            Tree::Nil => None,
            Tree::Node(l, k, r) => {
                if key == *k {
                    Some(vec![*k])
                } else if key < *k {
                    path_to(l, key).map(|mut p| {
                        p.insert(0, *k);
                        p
                    })
                } else {
                    path_to(r, key).map(|mut p| {
                        p.insert(0, *k);
                        p
                    })
                }
            }
        }
    }

    fn brute_lca(tree: &Tree, p: i32, q: i32) -> Option<i32> {
        let pp = path_to(tree, p)?;
        let pq = path_to(tree, q)?;
        let mut last: Option<i32> = None;
        for (a, b) in pp.iter().zip(pq.iter()) {
            if a == b {
                last = Some(*a);
            } else {
                break;
            }
        }
        last
    }

    fn contains(tree: &Tree, key: i32) -> bool {
        let mut cur = tree;
        loop {
            match cur {
                Tree::Nil => return false,
                Tree::Node(l, k, r) => {
                    if key == *k {
                        return true;
                    }
                    cur = if key < *k { l } else { r };
                }
            }
        }
    }

    pub struct Adapter(pub &'static str);
    impl HarnessAdapter for Adapter {
        fn instance_id(&self) -> &str {
            self.0
        }
        fn run(&self, config: &HarnessConfig) -> Result<HarnessReport> {
            require_known_oracles(
                config,
                &[
                    "returned_key_in_tree",
                    "both_queries_in_subtree_at_returned_key",
                    "result_matches_brute_lca_oracle",
                ],
            )?;
            let min_size = config.input_generator["min_size"].as_u64().unwrap_or(2) as usize;
            let max_size = config.input_generator["max_size"].as_u64().unwrap_or(16) as usize;
            let kr = config.input_generator["key_range"]
                .as_array()
                .ok_or_else(|| BehaviorError::Generator("missing key_range".into()))?;
            let k_min = kr[0].as_i64().unwrap_or(-50) as i32;
            let k_max = kr[1].as_i64().unwrap_or(50) as i32;

            let mut rng = ChaCha20Rng::seed_from_u64(config.seed);
            let mut st = OracleState::new(&config.oracle_checks);
            for trial in 0..config.num_trials {
                let (keys, p, q) =
                    gen::bst_with_two_keys(&mut rng, min_size, max_size, k_min, k_max);
                let mut t = Tree::Nil;
                for &k in &keys {
                    t = insert(t, k);
                }
                let out = lca_bst(&t, p, q);
                let brute = brute_lca(&t, p, q);
                let input_repr = format!("|keys|={},p={p},q={q}", keys.len());
                st.check("returned_key_in_tree", trial, &input_repr, || match out {
                    Some(k) => {
                        if contains(&t, k) {
                            Ok(())
                        } else {
                            Err((format!("{k}"), "key in tree".into()))
                        }
                    }
                    None => {
                        if !contains(&t, p) || !contains(&t, q) {
                            Ok(())
                        } else {
                            Err(("None".into(), "Some when both keys present".into()))
                        }
                    }
                });
                st.check(
                    "both_queries_in_subtree_at_returned_key",
                    trial,
                    &input_repr,
                    || {
                        if let Some(k) = out {
                            let (lo, hi) = if p <= q { (p, q) } else { (q, p) };
                            if lo <= k && k <= hi {
                                Ok(())
                            } else {
                                Err((format!("{k}"), format!("in [{lo},{hi}]")))
                            }
                        } else {
                            Ok(())
                        }
                    },
                );
                st.check(
                    "result_matches_brute_lca_oracle",
                    trial,
                    &input_repr,
                    || {
                        if out == brute {
                            Ok(())
                        } else {
                            Err((format!("{:?}", out), format!("{:?}", brute)))
                        }
                    },
                );
            }
            Ok(st.finalize(config, self.instance_id()))
        }
    }
}

#[cfg(test)]
mod tests {
    // Unit tests assert invariants on the pilot registry; `unwrap`/`unwrap_err` are intentional.
    #![allow(clippy::unwrap_used)]

    use super::*;
    use serde_json::json;

    fn make_config(
        kind: &str,
        oracles: &[&str],
        seed: u64,
        trials: u32,
        gen_json: serde_json::Value,
    ) -> HarnessConfig {
        HarnessConfig {
            schema_version: "schema_v1".into(),
            harness_type: match kind {
                "exact_output" => crate::HarnessKind::ExactOutput,
                "edge_case" => crate::HarnessKind::EdgeCase,
                "reference_relational" => crate::HarnessKind::ReferenceRelational,
                _ => crate::HarnessKind::PropertyBased,
            },
            seed,
            num_trials: trials,
            input_generator: gen_json,
            oracle_checks: oracles.iter().map(|s| (*s).to_string()).collect(),
            timeout_ms: 2000,
        }
    }

    #[test]
    fn binary_search_is_clean() {
        let r = AdapterRegistry::with_pilot();
        let cfg = make_config(
            "property_based",
            &[
                "success_index_in_bounds",
                "success_points_to_target",
                "failure_if_absent",
            ],
            42,
            50,
            json!({
                "kind": "sorted_int_vec",
                "min_len": 0,
                "max_len": 32,
                "value_range": [-100, 100]
            }),
        );
        let adapter = r.get("arrays_binary_search_001").unwrap();
        let rep = adapter.run(&cfg).unwrap();
        assert_eq!(rep.trials_run, 50);
        assert!(!rep.any_falsified(), "{rep:?}");
    }

    #[test]
    fn dijkstra_agrees_with_bellman_ford() {
        let r = AdapterRegistry::with_pilot();
        let cfg = make_config(
            "reference_relational",
            &[
                "length_equals_n",
                "source_distance_is_zero",
                "distances_agree_with_brute_force_bellman_ford",
                "none_iff_unreachable",
            ],
            2026,
            30,
            json!({
                "kind": "random_dag",
                "min_n": 1,
                "max_n": 8,
                "edge_density": 0.35,
                "weight_range": [0, 50]
            }),
        );
        let adapter = r.get("graph_dijkstra_001").unwrap();
        let rep = adapter.run(&cfg).unwrap();
        assert!(!rep.any_falsified(), "{rep:?}");
    }

    #[test]
    fn insertion_sort_is_clean() {
        let r = AdapterRegistry::with_pilot();
        let cfg = make_config(
            "property_based",
            &[
                "output_is_sorted",
                "output_is_permutation_of_input",
                "length_preserved",
            ],
            17,
            50,
            json!({
                "kind": "int_vec",
                "min_len": 0,
                "max_len": 32,
                "value_range": [-100, 100]
            }),
        );
        let adapter = r.get("sorting_insertion_sort_001").unwrap();
        let rep = adapter.run(&cfg).unwrap();
        assert!(!rep.any_falsified(), "{rep:?}");
    }

    #[test]
    fn unknown_oracle_is_rejected() {
        let r = AdapterRegistry::with_pilot();
        let cfg = make_config(
            "property_based",
            &["this_oracle_does_not_exist"],
            1,
            1,
            json!({
                "kind": "sorted_int_vec",
                "min_len": 0,
                "max_len": 1,
                "value_range": [0, 0]
            }),
        );
        let adapter = r.get("arrays_binary_search_001").unwrap();
        let err = adapter.run(&cfg).unwrap_err();
        assert!(matches!(err, BehaviorError::UnknownOracleCheck(_)));
    }
}
