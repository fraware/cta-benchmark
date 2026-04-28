# Artifact Validation

- Run `python scripts/implement_evidence_hardening.py`
- Run `python scripts/validate_release_artifact.py`
- Run `python scripts/ci_reviewer_readiness.py`
- Verify `artifacts/evidence_hardening_manifest.json` has empty `missing`.
- Confirm checksums (`sha256`) are populated for required files.
