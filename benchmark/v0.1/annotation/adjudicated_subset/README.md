# Adjudicated subset

Final, immutable adjudicated annotations for benchmark `v0.1`. Every file
here is the authoritative answer for a `(instance, system)` pair under
`rubric_v1` and is consumed directly by `cta annotate pack` (when invoked
without `--input`) and by `cta metrics compute --annotations`.

Layout:

```
adjudicated_subset/
└── <system_id>/
    └── <instance_id>.json   # schema: annotation.schema.json
```

Rules:

- Files are append-only. Never edit or delete an adjudicated record; if a
  defect is found, issue a new benchmark version (`v0.2`).
- Every file must validate against `schemas/annotation.schema.json`. CI
  enforces this via `cta validate benchmark --version v0.1 --release` and the
  per-artifact `cta validate file` gate.
- `annotator_id` must be `"adjudicator"` for every record in this
  subset; the unadjudicated raw annotations live in the per-version raw
  directory and must not be placed here.
