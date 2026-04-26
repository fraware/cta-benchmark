//! Reference implementation for `arrays_binary_search_004`.
//!
//! Pure, stable Rust. No macros. No unsafe. No external crates.

/// Binary search on a sorted nondecreasing slice.
///
/// Returns `Some(i)` with `arr[i] == target` if found, otherwise `None`.
/// If the target appears multiple times, any matching index is acceptable.
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
