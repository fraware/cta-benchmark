//! Reference implementation for `greedy_coin_change_canonical_007`.
//!
//! Returns the greedy decomposition. Correct by preconditions (the
//! denomination system is assumed canonical).

/// Greedy coin-change decomposition for a canonical denomination system.
///
/// Preconditions:
/// - `denoms` is sorted strictly ascending with `denoms[0] == 1`
/// - the denomination system is canonical (greedy is optimal)
/// - `amount >= 0`
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
