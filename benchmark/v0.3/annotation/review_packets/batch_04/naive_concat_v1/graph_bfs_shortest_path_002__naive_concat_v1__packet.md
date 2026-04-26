# Review Packet: graph_bfs_shortest_path_002 / naive_concat_v1

- Rubric: benchmark/v0.2/annotation/rubric_v1.md
- Manual: docs/annotation_manual.md
- Ann 01 file: benchmark\v0.2\annotation\review_packets\batch_04\naive_concat_v1\graph_bfs_shortest_path_002__naive_concat_v1__ann_01.json
- Ann 02 file: benchmark\v0.2\annotation\review_packets\batch_04\naive_concat_v1\graph_bfs_shortest_path_002__naive_concat_v1__ann_02.json
- Adjudicator file: benchmark\v0.2\annotation\review_packets\batch_04\naive_concat_v1\graph_bfs_shortest_path_002__naive_concat_v1__adjudicator.json

## Required completion steps
1. ann_01 submits independent labels.
2. ann_02 submits independent labels.
3. Adjudicator resolves disagreements in __adjudicator.json.
4. Validate each JSON with:
   - cargo run -p cta_cli --quiet -- validate file --schema annotation --path <file>

## Notes
- Use theorem-backed benchmark-facing obligations whenever the packet
  has moved to theory-backed Lean proofs.
- For SU5 in theory-backed variants, use
  `bfs_unreachability_iff adj source v hv` directly (no `hvalid` helper).
- `generated_obligations` length should match obligations evaluated for this pair.
