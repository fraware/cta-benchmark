//! Reference implementation for `greedy_interval_scheduling_002`.
//!
//! Classical earliest-finish-time greedy. Intervals are `[start, end)`;
//! intervals sharing only a single endpoint are non-overlapping.

/// Maximum-cardinality set of mutually non-overlapping intervals.
pub fn interval_scheduling(intervals: &[(i32, i32)]) -> usize {
    let mut sorted: Vec<(i32, i32)> = intervals.to_vec();
    sorted.sort_by_key(|iv| iv.1);
    let mut count: usize = 0;
    let mut last_end: Option<i32> = None;
    for (s, e) in sorted {
        if last_end.is_none() || s >= last_end.unwrap() {
            count += 1;
            last_end = Some(e);
        }
    }
    count
}
