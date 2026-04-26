//! Reference implementation for `dp_knapsack_01_001`.
//!
//! Classical O(n * capacity) 0/1 knapsack using a 1-D rolling table.

/// Maximum total value over subsets of items with total weight <= `capacity`.
///
/// Precondition: `weights.len() == values.len()`.
pub fn knapsack_01(weights: &[u32], values: &[u32], capacity: u32) -> u32 {
    assert_eq!(weights.len(), values.len(), "weights and values must align");
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
