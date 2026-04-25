<#
.SYNOPSIS
  Copy canonical Lean pilot modules into v0.1 instance scaffolds (byte-identical).

.DESCRIPTION
  The v0.1 pilot (`benchmark/v0.1/instances`) requires each `scaffold.lean` to
  match `lean/CTA/Benchmark/**/<Module>.lean` exactly (see `INST_LEAN_SCAFFOLD_DIVERGENCE`
  in `crates/cta_benchmark/src/lint.rs`). After editing a canonical module, run
  this script, then regenerate the manifest:

    cargo run -p cta_cli -- benchmark manifest --version v0.1
    cargo run -p cta_cli -- validate benchmark --version v0.1 --release

.PARAMETER WhatIf
  Print copy operations without writing files.
#>
[CmdletBinding(SupportsShouldProcess = $true)]
param()

$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')

$pairs = @(
  @('lean\CTA\Benchmark\Arrays\BinarySearch001.lean', 'benchmark\v0.1\instances\arrays\arrays_binary_search_001\scaffold.lean'),
  @('lean\CTA\Benchmark\Arrays\MaxSubarray001.lean', 'benchmark\v0.1\instances\arrays\arrays_max_subarray_001\scaffold.lean'),
  @('lean\CTA\Benchmark\DP\Knapsack01_001.lean', 'benchmark\v0.1\instances\dp\dp_knapsack_01_001\scaffold.lean'),
  @('lean\CTA\Benchmark\DP\LongestCommonSubsequence001.lean', 'benchmark\v0.1\instances\dp\dp_longest_common_subsequence_001\scaffold.lean'),
  @('lean\CTA\Benchmark\Graph\BfsShortestPath001.lean', 'benchmark\v0.1\instances\graph\graph_bfs_shortest_path_001\scaffold.lean'),
  @('lean\CTA\Benchmark\Graph\Dijkstra001.lean', 'benchmark\v0.1\instances\graph\graph_dijkstra_001\scaffold.lean'),
  @('lean\CTA\Benchmark\Greedy\CoinChangeCanonical001.lean', 'benchmark\v0.1\instances\greedy\greedy_coin_change_canonical_001\scaffold.lean'),
  @('lean\CTA\Benchmark\Greedy\IntervalScheduling001.lean', 'benchmark\v0.1\instances\greedy\greedy_interval_scheduling_001\scaffold.lean'),
  @('lean\CTA\Benchmark\Sorting\InsertionSort001.lean', 'benchmark\v0.1\instances\sorting\sorting_insertion_sort_001\scaffold.lean'),
  @('lean\CTA\Benchmark\Sorting\MergeSort001.lean', 'benchmark\v0.1\instances\sorting\sorting_merge_sort_001\scaffold.lean'),
  @('lean\CTA\Benchmark\Trees\BstInsert001.lean', 'benchmark\v0.1\instances\trees\trees_bst_insert_001\scaffold.lean'),
  @('lean\CTA\Benchmark\Trees\LowestCommonAncestor001.lean', 'benchmark\v0.1\instances\trees\trees_lowest_common_ancestor_001\scaffold.lean')
)

foreach ($p in $pairs) {
  $src = Join-Path $root $p[0]
  $dst = Join-Path $root $p[1]
  if (-not (Test-Path -LiteralPath $src)) {
    throw "Source missing: $src"
  }
  $parent = Split-Path -Parent $dst
  if (-not (Test-Path -LiteralPath $parent)) {
    throw "Destination directory missing: $parent"
  }
  if ($PSCmdlet.ShouldProcess($dst, "Copy from $src")) {
    Copy-Item -LiteralPath $src -Destination $dst -Force
  }
}

Write-Host @"

Next steps (when not using -WhatIf):
  cd <workspace>
  cargo run -p cta_cli -- benchmark manifest --version v0.1
  cargo run -p cta_cli -- validate benchmark --version v0.1 --release
  cargo test -p cta_benchmark --test pilot
"@
