# Review Packet: graph_bfs_shortest_path_002 / full_method_v1

- Rubric: benchmark/v0.2/annotation/rubric_v1.md
- Manual: docs/annotation_manual.md
- Ann 01 file: benchmark\v0.2\annotation\review_packets\batch_04\full_method_v1\graph_bfs_shortest_path_002__full_method_v1__ann_01.json
- Ann 02 file: benchmark\v0.2\annotation\review_packets\batch_04\full_method_v1\graph_bfs_shortest_path_002__full_method_v1__ann_02.json
- Adjudicator file: benchmark\v0.2\annotation\review_packets\batch_04\full_method_v1\graph_bfs_shortest_path_002__full_method_v1__adjudicator.json

## Required completion steps
1. ann_01 submits independent labels.
2. ann_02 submits independent labels.
3. Adjudicator resolves disagreements in __adjudicator.json.
4. Validate each JSON with:
   - cargo run -p cta_cli --quiet -- validate file --schema annotation --path <file>

## Notes
- Packet is expected to use theorem-backed benchmark-facing obligations
  from `CTA.Benchmark.Graph.BfsShortestPathTheory` for SU1-SU5.
- SU5 must use `bfs_unreachability_iff adj source v hv` directly
  (no circular `hvalid` helper hypothesis).
- `generated_obligations` length should match obligations evaluated for this pair.
