# dp_longest_common_subsequence_003

LCS length (variant 3): emphasize symmetry lcs(a,b)=lcs(b,a) as a cross-check on substructure recurrence.

Lens 3 catches asymmetric indexing in the DP table that still looks plausible in prose.

Derived algorithm family `dp_longest_common_subsequence`; behavioral contract matches v0.2 reference oracles.
