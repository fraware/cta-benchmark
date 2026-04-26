# Review Packet: sorting_merge_sort_001 / naive_concat_v1

- Rubric: benchmark/v0.2/annotation/rubric_v1.md
- Manual: docs/annotation_manual.md
- Ann 01 file: benchmark\v0.2\annotation\review_packets\batch_08\naive_concat_v1\sorting_merge_sort_001__naive_concat_v1__ann_01.json
- Ann 02 file: benchmark\v0.2\annotation\review_packets\batch_08\naive_concat_v1\sorting_merge_sort_001__naive_concat_v1__ann_02.json
- Adjudicator file: benchmark\v0.2\annotation\review_packets\batch_08\naive_concat_v1\sorting_merge_sort_001__naive_concat_v1__adjudicator.json

## Required completion steps
1. ann_01 submits independent labels.
2. ann_02 submits independent labels.
3. Adjudicator resolves disagreements in __adjudicator.json.
4. Validate each JSON with:
   - cargo run -p cta_cli --quiet -- validate file --schema annotation --path <file>

## Notes
- Replace placeholder scalar values and obligations.
- generated_obligations length should match obligations evaluated for this pair.
