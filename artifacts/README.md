# Build outputs

`scripts/build_anonymous_artifact.ps1` writes `cta-benchmark-anonymous.zip` here.

Zip contents exclude `.git`, `target`, and `runs/` to reduce leakage of local
paths. For double-blind venues, scrub any remaining institution strings in
`annotation/` or `benchmark/` before upload.
