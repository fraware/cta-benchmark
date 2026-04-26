# v0.3 adjudicated subset (eval)

`pack.json` lists every `(instance_id, system_id)` pair on the **eval** split for `configs/experiments/benchmark_v03.json`.

Records are regenerated from registered review packets by:

`python scripts/materialize_v03_adjudication_artifacts.py`

See each record's `annotator_notes` for provenance (canonical `packet.json` path and mapping from eval grid variants `*_004`..`*_007` to paired `*_001` / `*_002` templates). After updating the pack, refresh coverage summaries with `cargo run -p cta_cli -- annotate coverage …` or `python scripts/build_v03_annotation_pack.py --skip-coverage-cli` when Rust refresh is deferred.
