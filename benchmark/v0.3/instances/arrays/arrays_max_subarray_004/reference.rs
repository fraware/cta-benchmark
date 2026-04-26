//! Reference implementation for `arrays_max_subarray_004`.
//!
//! Pure, stable Rust. No macros. No unsafe. No external crates.

/// Maximum-sum contiguous non-empty subslice (Kadane's algorithm).
///
/// Precondition: `arr` is non-empty.
pub fn max_subarray(arr: &[i32]) -> i32 {
    let mut best: i32 = arr[0];
    let mut cur: i32 = arr[0];
    for &x in &arr[1..] {
        cur = std::cmp::max(x, cur + x);
        best = std::cmp::max(best, cur);
    }
    best
}
