# Multi-annotator fixture

A small illustrative fixture that exercises the full adjudication
pipeline end-to-end against the `cta_metrics::agreement` module:

- Two non-adjudicator annotators (`ann_01`, `ann_02`) produce divergent
  judgments for a single `(instance, system)` pair
  (`arrays_binary_search_001`, `text_only_v1`).
- An `adjudicator` record resolves the disagreement and serves as the
  ground truth under the `prefer-adjudicator` policy.
- The raw three-record set is rich enough to exercise every
  inter-annotator agreement metric: weighted Cohen's kappa on the
  ordinal faithfulness scale, Cohen's kappa on the binary vacuity
  label, and raw agreement on critical-unit coverage.

Layout:

```
multi_annotator_fixture/
└── text_only_v1/
    ├── arrays_binary_search_001__ann_01.json
    ├── arrays_binary_search_001__ann_02.json
    └── arrays_binary_search_001__adjudicator.json
```

Exercise from the command line:

```bash
cta annotate pack \
  --version v0.2 \
  --input benchmark/v0.2/annotation/multi_annotator_fixture \
  --out   runs/annotation_packs/multi-annotator-demo.json \
  --policy prefer-adjudicator
```

The same directory can be fed to `cta metrics compute` via
`--raw-annotations` to populate the `inter_annotator_agreement` block of
a results bundle. Automated coverage lives in
`crates/cta_metrics/tests/multi_annotator_pipeline.rs`.

The fixture is intentionally scoped to a single `(instance, system)`
pair so that the files remain auditable at a glance. Adding further
pairs requires a benchmark version bump; see
`CONTRIBUTING.md`.
