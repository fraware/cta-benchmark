# Croissant validation (NeurIPS 2026 E&D)

## Machine verification (repository / CI)

**UTC timestamp:** 2026-05-04T14:28:36Z (recorded at report write)

**Subject file:** `hf_release/croissant.json` (merged Hub core + RAI + augmented `distribution` / `recordSet`).

**`mlcroissant` CLI (local):** `mlcroissant validate --jsonld hf_release/croissant.json` — **passed** (warnings only: non-canonical `@context` shape vs MLCommons examples; optional `citeAs`, `datePublished`, `version`).

**`validate_neurips_artifact.py`:** **passed** (includes the same `mlcroissant` invocation when the CLI is on `PATH`).

**SHA256 (`hf_release/croissant.json`):**

`9389472747ed82892486a815df7c4f2f6ff29c7bd7f45c9bc66204c596c304a7`

## Hugging Face dataset repo (after upload)

**Dataset:** `https://huggingface.co/datasets/fraware/cta-bench`

**Latest `main` commit SHA (Git on the Hub dataset repo), after the uploads in this run:**

`29b5ad8965738e7a5e68576eff43af0c6ff8cb54`

**Byte identity (local vs Hub):** `huggingface_hub.hf_hub_download(..., filename="croissant.json", revision="main")` matches **`hf_release/croissant.json` byte-for-byte** (same SHA256 as above). That is the file that cleared `mlcroissant validate` before upload.

**`make hf-check-remote` output:**

```text
Remote check OK: 2421 paths; all 14 required files present.
```

**Note on URLs:** Prefer `hf_hub_download` / Hub client for byte checks. A raw `curl` of `.../resolve/main/croissant.json` can occasionally disagree with the CLI-downloaded object (encoding, CDN, or revision edge cases); the authoritative check for “what OpenReview should cite” is the object returned for `croissant.json` on `revision=main` via the Hub API / client.

## Official online checker (human gate — still required for “validated” wording)

The **hosted** Croissant checker Space is **not** executed from this repository’s automation:

[https://huggingface.co/spaces/JoaquinVanschoren/croissant-checker](https://huggingface.co/spaces/JoaquinVanschoren/croissant-checker)

**Action for authors:** download `croissant.json` from `resolve/main` (or use local `hf_release/croissant.json` after `make hf-croissant`), validate that **exact** JSON in the Space, then either:

- save **`reports/croissant_validation_2026.png`** (screenshot), or  
- append the Space log / URL + timestamp under this file,

and re-run `make hf-package` (and the usual Hub upload sequence) so the proof is mirrored under `hf_release/artifact/reports/` when present.

Until that Space run is recorded here (or in the PNG), treat pipeline status as **“CLI + Hub bytes verified”**, not **“official Space validated”**.

## Regeneration commands

See [`docs/reproducibility.md`](../docs/reproducibility.md) for the full `make hf-release` sequence and NeurIPS E&D notes.
