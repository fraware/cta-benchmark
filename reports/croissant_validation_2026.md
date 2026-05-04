# Croissant validation (NeurIPS 2026 E&D)

## Machine verification (repository / CI)

**Mirror session (UTC):** 2026-05-04T15:16:19Z (Hugging Face mirror + report archive in this session)

**Subject file:** `hf_release/croissant.json` (merged Hub Croissant API core + RAI + augmented `distribution` / `recordSet`).

**`mlcroissant` CLI (local):** `mlcroissant validate --jsonld hf_release/croissant.json` — **passed** (warnings only: non-canonical `@context` shape vs MLCommons examples; optional `citeAs`, `datePublished`, `version`).

**`validate_neurips_artifact.py`:** **passed** (includes the same `mlcroissant` invocation when the CLI is on `PATH`).

**SHA256 (`hf_release/croissant.json`) — current `main` (local and Hub):**

`737e7fcb211905022303b1af1f2beb3658a51843d9bec49a410470a5b0380cad`

## Hugging Face dataset repo (mirrored artifact)

**Dataset:** `https://huggingface.co/datasets/fraware/cta-bench`

**Latest `main` commit SHA (Git on the Hub dataset repo), after mirror in this session:**

`0954db1fb797130e8859db446bfd805a1424638a`

Each `make hf-upload` / `make hf-upload-croissant` advances `main`. The SHA above is `HfApi().repo_info('fraware/cta-bench', repo_type='dataset').sha` immediately after the mirror cycle that published Git `6ed41dd` (freeze tag `neurips2026-cta-freeze`) to `artifact/reports/` and re-uploaded `croissant.json`. If the Hugging Face Croissant API returns a sparse core after a clean `hf_release/` rebuild, pin `croissant.json` to revision `0922551cb20e4f6f0cc51cfb2d6368c68dd72f18` (SHA256 `737e7fcb…`, Space-validated) before `make hf-upload-croissant` so `main` stays byte-identical to the checker proof.

**Byte identity (local vs Hub):** `huggingface_hub.hf_hub_download(..., filename="croissant.json", revision="main")` matches **`hf_release/croissant.json` byte-for-byte** (SHA256 above).

**`make hf-check-remote` output (expected):**

```text
Remote check OK: 2421 paths; all 14 required files present.
```

**Note on URLs:** Prefer `hf_hub_download` / Hub client for byte checks. A raw `curl` of `.../resolve/main/croissant.json` can occasionally disagree with the CLI-downloaded object (encoding, CDN, or revision edge cases); the authoritative check for “what OpenReview should cite” is the object returned for `croissant.json` on `revision=main` via the Hub API / client.

## Official Croissant checker (JoaquinVanschoren Space) — archived proof

**Checker:** [https://huggingface.co/spaces/JoaquinVanschoren/croissant-checker](https://huggingface.co/spaces/JoaquinVanschoren/croissant-checker)

**Input (Space run):** `croissant.json` byte-identical to the SHA256 above (same object as Hub `resolve/main/croissant.json` after the mirror session).

**Records generation:** All listed record sets passed; there is **no** failure on `rs_fo_data_human_agreement_json` (that RecordSet is absent by design).

The Space also prints a large JSON-LD appendix of the validated document; it is **not** duplicated here (see `hf_release/croissant.json` on `main` for bytes).

### Archived checker output (validation section)

```text
# CROISSANT VALIDATION REPORT
================================================================================
## VALIDATION RESULTS
--------------------------------------------------------------------------------
Starting validation for file: croissant.json
### JSON Format Validation
✓
The file is valid JSON.
### Croissant Schema Validation
✓
The dataset passes Croissant validation.
### Responsible AI Metadata
✓
All required Responsible AI metadata fields are present.
### Records Generation Test
✓
Record set 'rs_fo_data_instances_jsonl' passed validation.
Record set 'rs_fo_data_semantic_units_jsonl' passed validation.
Record set 'rs_fo_data_reference_obligations_jsonl' passed validation.
Record set 'rs_fo_data_generated_packets_jsonl' passed validation.
Record set 'rs_fo_data_system_cards_jsonl' passed validation.
Record set 'rs_fo_data_prompt_templates_jsonl' passed validation.
Record set 'rs_fo_data_strict_results_csv' passed validation.
Record set 'rs_fo_data_expanded_results_csv' passed validation.
Record set 'rs_fo_data_correction_overlays_csv' passed validation.
Record set 'rs_fo_data_common_cell_instances_csv' passed validation.
Record set 'rs_fo_data_common_cell_system_summary_csv' passed validation.
```

## `human_agreement.json` and optional records generation

`data/human_agreement.json` remains in Croissant **`distribution`** as FileObject `fo_data_human_agreement_json`.

The augmented RecordSet **`rs_fo_data_human_agreement_json`** (whole-file field with `extract.fileProperty: "content"`) was **removed** from `scripts/add_rai_to_croissant.py` because it triggered the official checker’s optional **Records Generation** step with `TypeError: 'str' object cannot be interpreted as an integer`. Croissant does not require every distributed file to be a RecordSet.

## Regeneration commands

See [`docs/reproducibility.md`](../docs/reproducibility.md) for the full `make hf-release` sequence and NeurIPS E&D notes.
