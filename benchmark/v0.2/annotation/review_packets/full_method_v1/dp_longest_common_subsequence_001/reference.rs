//! Reference implementation for `dp_longest_common_subsequence_001`.
//!
//! Classical O(|a| * |b|) tabular DP.

/// Length of the longest common subsequence of `a` and `b`.
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
