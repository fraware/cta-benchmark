# Final CI Parity Evidence

- Timestamp (UTC): `2026-05-01T07:49:31.520418+00:00`
- Repo root: `C:/Users/mateo/cta-benchmark`

## `cargo fmt --all -- --check`

- Status: **PASS**

```text

```

## `cargo clippy --workspace --all-targets --no-deps`

- Status: **PASS**

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.57s
```

## `cargo test --workspace --all-targets`

- Status: **PASS**

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.58s
     Running unittests src\lib.rs (target\debug\deps\cta_annotations-0e5ed6bd505c4937.exe)

running 3 tests
test tests::prefer_adjudicator_policy_returns_adjudicator ... ok
test tests::prefer_adjudicator_errors_without_adjudicator_when_multi ... ok
test tests::majority_merges_two_annotators ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\lib.rs (target\debug\deps\cta_behavior-f8216144135323b9.exe)

running 5 tests
test pilot::tests::unknown_oracle_is_rejected ... ok
test tests::registry_reports_pilot_instances ... ok
test pilot::tests::binary_search_is_clean ... ok
test pilot::tests::dijkstra_agrees_with_bellman_ford ... ok
test pilot::tests::insertion_sort_is_clean ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\pilot_smoke.rs (target\debug\deps\pilot_smoke-4c9aa8bd7be616d9.exe)

running 1 test
test all_pilot_adapters_are_clean_on_small_trials ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running unittests src\lib.rs (target\debug\deps\cta_benchmark-24f3bb50133b211b.exe)

running 10 tests
test splits::tests::split_name_round_trip ... ok
test authoring_lint::tests::regex_matches_bare_existential ... ok
test splits::tests::missing_splits_dir_is_not_an_error ... ok
test authoring_lint::tests::regex_matches_unconditional_universal ... ok
test release_checks::tests::missing_manifest_is_error ... ok
test release_checks::tests::empty_eval_is_error ... ok
test splits::tests::load_splits_rejects_name_mismatch ... ok
test release_checks::tests::unknown_instance_in_split_is_error ... ok
test splits::tests::load_splits_rejects_version_mismatch ... ok
test splits::tests::load_splits_reads_only_canonical_stems ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests\pilot.rs (target\debug\deps\pilot-f9e76c42a76a1ad3.exe)

running 2 tests
test pilot_v0_1_loads_and_lints_clean ... ok
test pilot_v0_1_manifest_hash_is_deterministic ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running unittests src\main.rs (target\debug\deps\cta-7daedd25afb0a0b6.exe)

running 2 tests
test cmd::annotate::tests::count_admit_or_sorry_counts_benchmark_theorem_placeholders ... ok
test cmd::annotate::tests::extract_trusted_symbols_finds_axioms_and_opaques ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\lib.rs (target\debug\deps\cta_core-ce232f773727096d.exe)

running 15 tests
test enums::tests::domain_all_matches_variants ... ok
test enums::tests::domain_serializes_snake_case ... ok
test enums::tests::obligation_kind_roundtrips ... ok
test ids::tests::semantic_unit_id_accepts_canonical ... ok
test versions::tests::benchmark_version_ok ... ok
test ids::tests::system_id_accepts_canonical ... ok
test ids::tests::obligation_id_accepts_canonical ... ok
test ids::tests::system_id_rejects_bad ... ok
test ids::tests::obligation_id_rejects_bad ... ok
test ids::tests::instance_id_accepts_canonical ... ok
test ids::tests::instance_id_rejects_bad ... ok
test ids::tests::instance_id_roundtrips_through_serde ... ok
test ids::tests::instance_id_serde_rejects_invalid ... ok
test versions::tests::schema_version_ok ... ok
test ids::tests::run_id_accepts_canonical ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\lib.rs (target\debug\deps\cta_generate-8369cc796a4bb1e4.exe)

running 24 tests
test normalize::tests::confidence_is_clamped ... ok
test normalize::tests::demotes_stability_to_auxiliary ... ok
test normalize::tests::drops_implication_to_true_placeholders ... ok
test normalize::tests::empty_response_errors ... ok
test normalize::tests::drops_prop_trivial_and_placeholder_gloss ... ok
test normalize::tests::parses_bare_array ... ok
test normalize::tests::drops_vacuous_true_obligations ... ok
test normalize::tests::missing_lean_statement_errors ... ok
test prompts::tests::kind_round_trip ... ok
test normalize::tests::parses_object_form ... ok
test normalize::tests::scrapes_json_embedded_in_prose ... ok
test normalize::tests::unknown_kind_maps_to_unknown ... ok
test normalize::tests::empty_obligations_list_errors ... ok
test prompts::tests::render_leaves_unknown_placeholders ... ok
test prompts::tests::render_strict_flags_missing ... ok
test prompts::tests::render_substitutes_simple ... ok
test providers::tests::anthropic_response_parser_extracts_blocks_and_usage ... ok
test providers::tests::live_providers_refuse_without_credentials ... ok
test providers::tests::openai_request_body_shape ... ok
test providers::tests::openai_response_parser_extracts_content_model_and_usage ... ok
test providers::tests::stub_provider_returns_parseable_bundle ... ok
test pipeline::tests::build_context_code_only_errors_when_reference_rs_missing_or_empty ... ok
test pipeline::tests::generate_errors_on_unresolved_template_placeholders ... ok
test pipeline::tests::stub_bundle_roundtrips ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests\code_only_packet_regression.rs (target\debug\deps\code_only_packet_regression-a4964b10525e132c.exe)

running 1 test
test regression_target_packets_are_benchmark_aligned ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests\code_only_rust_injection.rs (target\debug\deps\code_only_rust_injection-40c6cd977d79d433.exe)

running 4 tests
test code_only_v1_prompt_contains_verbatim_reference_rs ... ok
test naive_concat_v1_prompt_resolves_rust_reference_placeholder ... ok
test stub_code_only_prompt_has_no_placeholders_after_context_build ... ok
test code_only_generation_rejects_placeholder_and_requires_code_derived_output ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests\family_packet_regression.rs (target\debug\deps\family_packet_regression-3287c1c10b963b0c.exe)

running 1 test
test family_specific_shape_guards_hold ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests\full_method_priority1_packet_regression.rs (target\debug\deps\full_method_priority1_packet_regression-b2a9cdc8f25b2f62.exe)

running 1 test
test full_method_v1_priority1_semantic_hardening_packets ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\full_method_priority2_packet_regression.rs (target\debug\deps\full_method_priority2_packet_regression-c1ca3b11ecd1e0e3.exe)

running 1 test
test full_method_v1_priority2_packets_reject_vacuity_and_tautologies ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\naive_concat_packet_regression.rs (target\debug\deps\naive_concat_packet_regression-2c9cfc0c0778865e.exe)

running 1 test
test regression_target_packets_are_benchmark_aligned ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests\normalize_proptest.rs (target\debug\deps\normalize_proptest-8690938e155da2e3.exe)

running 6 tests
test never_panics_on_arbitrary_string ... ok
test canonical_object_form_always_parses ... ok
test err_implies_typed_status ... ok
test ok_implies_well_formed_obligations ... ok
test prose_around_canonical_json_is_scraped ... ok
test never_panics_on_arbitrary_bytes ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s

     Running tests\pipeline_smoke.rs (target\debug\deps\pipeline_smoke-d34a6c9b335093f0.exe)

running 1 test
test stub_generation_produces_schema_valid_bundles_for_all_pilot_instances ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

     Running tests\prompt_snapshots.rs (target\debug\deps\prompt_snapshots-9110bbf451c304c9.exe)

running 4 tests
test snapshot_code_only_v1 ... ok
test snapshot_text_only_v1 ... ok
test snapshot_full_method_v1 ... ok
test snapshot_naive_concat_v1 ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests\review_packet_lean_lint.rs (target\debug\deps\review_packet_lean_lint-19142595ca38e8b7.exe)

running 1 test
test review_packets_benchmark_facing_lean_lints ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests\text_only_packet_regression.rs (target\debug\deps\text_only_packet_regression-991c845d7e332090.exe)

running 2 tests
test text_only_graph_dijkstra_001_generated_output_tracks_code_only_lineage ... ok
test regression_target_packets_are_benchmark_aligned ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\lib.rs (target\debug\deps\cta_lean-afbefb9269dbce09.exe)

running 7 tests
test tests::classify_error_variants ... ok
test tests::elaborate_reports_missing_file ... ok
test tests::write_generated_lean_is_deterministic ... ok
test tests::theorem_name_is_deterministic ... ok
test tests::parse_diagnostics_basic ... ok
test tests::parse_diagnostics_windows_drive_letter ... ok
test tests::extract_theorem_names_finds_all_forms ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src\lib.rs (target\debug\deps\cta_metrics-528de194976aad7c.exe)

running 12 tests
test tests::elaboration_rate_matches ... ok
test tests::metrics_version_is_v2 ... ok
test tests::rust_consistency_all_not_applicable_yields_zero ... ok
test tests::empty_input_returns_zeros ... ok
test tests::vacuity_and_consistency_exclude_not_applicable ... ok
test tests::tally_counts_labels_correctly ... ok
test tests::secondary_contradiction_on_critical_units ... ok
test tests::weighted_faithfulness_mixes_labels ... ok
test agreement::tests::raw_agreement_on_opposite_coverage_is_zero ... ok
test agreement::tests::identical_annotators_give_kappa_one ... ok
test agreement::tests::disjoint_categories_give_negative_kappa ... ok
test agreement::tests::raw_agreement_on_identical_coverage_is_one ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests\m6_pipeline.rs (target\debug\deps\m6_pipeline-3130be2898005cfa.exe)

running 1 test
test end_to_end_pipeline_produces_schema_valid_bundle ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

     Running tests\multi_annotator_pipeline.rs (target\debug\deps\multi_annotator_pipeline-1b5daae2baa7c74a.exe)

running 3 tests
test majority_policy_synthesises_from_non_adjudicator_annotators ... ok
test prefer_adjudicator_policy_selects_adjudicator_record ... ok
test agreement_metrics_flow_through_results_bundle ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

     Running unittests src\lib.rs (target\debug\deps\cta_reports-570d8db9c6dab25c.exe)

running 9 tests
test aggregate::tests::mean_of_two_runs_is_midpoint ... ok
test aggregate::tests::bootstrap_ci_is_deterministic_with_fixed_seed ... ok
test tests::csv_row_round_trips_header_column_count ... ok
test aggregate::tests::paired_deltas_are_symmetric ... ok
test aggregate::tests::paired_deltas_csv_shape ... ok
test tests::render_all_emits_four_blobs ... ok
test tests::latex_row_contains_system ... ok
test tests::markdown_contains_system_and_table ... ok
test tests::instance_csv_has_header_and_row ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\snapshots.rs (target\debug\deps\snapshots-1b2daf3c29e50a33.exe)

running 5 tests
test snapshot_instance_results_csv ... ok
test snapshot_results_latex ... ok
test snapshot_primary_metrics_csv ... ok
test snapshot_render_all_preserves_blob_ordering ... ok
test snapshot_results_markdown ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running unittests src\lib.rs (target\debug\deps\cta_rust_extract-df973731935e35b0.exe)

running 6 tests
test tests::classify_return_kind_basic ... ok
test tests::collection_tag_extraction_from_types ... ok
test tests::missing_entry_fn_is_error ... ok
test tests::helper_calls_captured ... ok
test tests::detects_direct_recursion ... ok
test tests::extracts_binary_search_shape ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\pilot_golden.rs (target\debug\deps\pilot_golden-4c214d97ce2ea46e.exe)

running 2 tests
test all_pilots_extract_cleanly ... ok
test summary_serialization_roundtrips ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src\lib.rs (target\debug\deps\cta_schema-a0c36b5e6f46e3c7.exe)

running 2 tests
test tests::rejects_missing_root ... ok
test tests::loads_all_canonical_schemas ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
```

## `cargo test --workspace --doc`

- Status: **PASS**

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.38s
   Doc-tests cta_annotations

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests cta_behavior

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests cta_benchmark

running 1 test
test crates\cta_benchmark\src\lib.rs - (line 13) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s

   Doc-tests cta_core

running 1 test
test crates\cta_core\src\ids.rs - ids (line 8) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s

   Doc-tests cta_generate

running 1 test
test crates\cta_generate\src\lib.rs - hash_prompt (line 168) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.15s

   Doc-tests cta_lean

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests cta_metrics

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests cta_reports

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests cta_rust_extract

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests cta_schema

running 1 test
test crates\cta_schema\src\lib.rs - (line 13) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
```

## `cargo run -p cta_cli -- validate schemas`

- Status: **PASS**

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.37s
     Running `target\debug\cta.exe validate schemas`
loaded 14 canonical schemas from C:\Users\mateo\cta-benchmark\schemas
  - Instance (instance.schema.json)
  - Obligation (obligation.schema.json)
  - Annotation (annotation.schema.json)
  - RunManifest (run_manifest.schema.json)
  - GeneratedOutput (generated_output.schema.json)
  - ResultsBundle (results_bundle.schema.json)
  - SemanticUnits (semantic_units.schema.json)
  - Harness (harness.schema.json)
  - BenchmarkManifest (benchmark_manifest.schema.json)
  - Experiment (experiment.schema.json)
  - ReviewPacket (review_packet.schema.json)
  - AnnotationPackManifest (annotation_pack_manifest.schema.json)
  - ProtocolFreeze (protocol_freeze.schema.json)
  - FailureModeOntology (failure_mode_ontology.schema.json)
```

## `cargo run -p cta_cli -- validate benchmark --version v0.3 --release`

- Status: **PASS**

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.38s
     Running `target\debug\cta.exe validate benchmark --version v0.3 --release`
release status: split=pass annotation_coverage=pass signoff=pass manifest=pass
release summary: wrote C:\Users\mateo\cta-benchmark\benchmark\v0.3\manifests\release_summary.json
ok: validated 84 instance(s) under C:\Users\mateo\cta-benchmark\benchmark\v0.3
```

## `cargo run -p cta_cli -- benchmark lint --version v0.3 --release`

- Status: **PASS**

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.36s
     Running `target\debug\cta.exe benchmark lint --version v0.3 --release`

summary: 0 error(s), 0 warning(s) across 84 instance(s)
```

## `C:\Users\mateo\miniconda3\python.exe scripts/materialize_v03_adjudication_artifacts.py`

- Status: **PASS**

```text
wrote C:\Users\mateo\cta-benchmark\benchmark\v0.3\annotation\adjudicated_subset\pack.json (192 records)
wrote C:\Users\mateo\cta-benchmark\results\raw_metrics_expanded.json (336 rows)
wrote C:\Users\mateo\cta-benchmark\results\raw_metrics.json (336 rows)
wrote C:\Users\mateo\cta-benchmark\results\raw_metrics_strict.json (222 rows)
wrote C:\Users\mateo\cta-benchmark\annotation\agreement_packet_ids.csv (192 rows)
wrote C:\Users\mateo\cta-benchmark\annotation\rater_a.csv
wrote C:\Users\mateo\cta-benchmark\annotation\rater_b.csv
wrote C:\Users\mateo\cta-benchmark\annotation\adjudication_log.csv (49 adjudicated disagreements)
```

## `C:\Users\mateo\miniconda3\python.exe scripts/materialize_repair_hotspot_artifacts.py`

- Status: **PASS**

```text
wrote C:\Users\mateo\cta-benchmark\repairs\hotspot_selection.csv (192 rows), C:\Users\mateo\cta-benchmark\repairs\repair_log.jsonl (192 records)
```

## `C:\Users\mateo\miniconda3\python.exe scripts/reproduce_agreement_report.py`

- Status: **PASS**

```text
reproduce_agreement_report: C:\Users\mateo\miniconda3\python.exe C:\Users\mateo\cta-benchmark\scripts\compute_agreement_stats.py --first C:\Users\mateo\cta-benchmark\annotation\rater_a.csv --second C:\Users\mateo\cta-benchmark\annotation\rater_b.csv
wrote C:\Users\mateo\cta-benchmark\annotation\agreement_report.json, C:\Users\mateo\cta-benchmark\annotation\agreement_raw_table.csv, C:\Users\mateo\cta-benchmark\annotation\agreement_report.md (192 packets)
```

## `C:\Users\mateo\miniconda3\python.exe scripts/implement_evidence_hardening.py`

- Status: **PASS**

```text
wrote C:\Users\mateo\cta-benchmark\annotation\human_pass_v3\agreement_report_human_strict_all.json, C:\Users\mateo\cta-benchmark\annotation\human_pass_v3\agreement_report_human_strict_all.md, C:\Users\mateo\cta-benchmark\annotation\human_pass_v3\disagreement_log_strict_all.csv
wrote C:\Users\mateo\cta-benchmark\results\paper_table_systems.csv, C:\Users\mateo\cta-benchmark\results\paper_table_families.csv, C:\Users\mateo\cta-benchmark\results\paper_table_failure_modes.csv, C:\Users\mateo\cta-benchmark\results\paper_table_repairs.csv
wrote C:\Users\mateo\cta-benchmark\results\appendix_mapped_evidence\paper_table_systems.csv, C:\Users\mateo\cta-benchmark\results\appendix_mapped_evidence\paper_table_families.csv, C:\Users\mateo\cta-benchmark\results\appendix_mapped_evidence\paper_table_failure_modes.csv, C:\Users\mateo\cta-benchmark\results\appendix_mapped_evidence\paper_table_repairs.csv
wrote C:\Users\mateo\cta-benchmark\results\appendix_mapped_evidence\system_summary.csv, C:\Users\mateo\cta-benchmark\results\appendix_mapped_evidence\family_summary.csv, C:\Users\mateo\cta-benchmark\results\appendix_mapped_evidence\failure_mode_counts.csv, C:\Users\mateo\cta-benchmark\results\appendix_mapped_evidence\instance_level.csv, C:\Users\mateo\cta-benchmark\results\appendix_mapped_evidence\composite_sensitivity.csv, C:\Users\mateo\cta-benchmark\results\appendix_mapped_evidence\system_summary_with_ci.json, C:\Users\mateo\cta-benchmark\results\appendix_mapped_evidence\system_faithfulness_summary.csv, C:\Users\mateo\cta-benchmark\results\appendix_mapped_evidence\system_consistency_summary.csv, C:\Users\mateo\cta-benchmark\results\appendix_mapped_evidence\system_vacuity_summary.csv, C:\Users\mateo\cta-benchmark\results\appendix_mapped_evidence\system_proof_utility_summary.csv, C:\Users\mateo\cta-benchmark\results\appendix_mapped_evidence\system_reliability_summary.csv, C:\Users\mateo\cta-benchmark\results\appendix_mapped_evidence\system_reliability_sensitivity.csv, C:\Users\mateo\cta-benchmark\results\appendix_mapped_evidence\family_reliability_summary.csv, paper_table_*.csv, paper_table_annotation_evidence.csv, paper_table_agreement_evidence.csv
wrote C:\Users\mateo\cta-benchmark\repairs\paper_repair_status.csv (192 rows)
wrote C:\Users\mateo\cta-benchmark\repairs\paper_repair_success_subset.csv (12 rows)
wrote C:\Users\mateo\cta-benchmark\repairs\paper_repair_proof_subset.csv (0 rows)
wrote C:\Users\mateo\cta-benchmark\repairs\paper_proof_facing_subset.csv (56 rows)
wrote C:\Users\mateo\cta-benchmark\results\paper_cost_runtime_accounting.csv (110 rows)
wrote C:\Users\mateo\cta-benchmark\results\paper_model_metadata_registry.csv (110 rows)
wrote C:\Users\mateo\cta-benchmark\results\paper_primary_model_registry.csv (4 rows)
wrote C:\Users\mateo\cta-benchmark\annotation\external_review\strict_review_queue.jsonl (274 rows)
wrote C:\Users\mateo\cta-benchmark\annotation\external_review\strict_review_queue.csv (274 rows)
wrote C:\Users\mateo\cta-benchmark\annotation\external_review\mapped_review_queue.jsonl (114 rows)
wrote C:\Users\mateo\cta-benchmark\annotation\external_review\review_schema.md
wrote C:\Users\mateo\cta-benchmark\benchmark\v0.3\annotation\human_wave_v03\strict_gap_13x4_worklist.csv (52 rows)
wrote C:\Users\mateo\cta-benchmark\benchmark\v0.3\annotation\human_wave_v03\strict_gap_13x4_completion.csv
wrote C:\Users\mateo\cta-benchmark\results\system_summary.csv, C:\Users\mateo\cta-benchmark\results\family_summary.csv, C:\Users\mateo\cta-benchmark\results\failure_mode_counts.csv, C:\Users\mateo\cta-benchmark\results\instance_level.csv, C:\Users\mateo\cta-benchmark\results\composite_sensitivity.csv, C:\Users\mateo\cta-benchmark\results\system_summary_with_ci.json, C:\Users\mateo\cta-benchmark\results\system_faithfulness_summary.csv, C:\Users\mateo\cta-benchmark\results\system_consistency_summary.csv, C:\Users\mateo\cta-benchmark\results\system_vacuity_summary.csv, C:\Users\mateo\cta-benchmark\results\system_proof_utility_summary.csv, C:\Users\mateo\cta-benchmark\results\system_reliability_summary.csv, C:\Users\mateo\cta-benchmark\results\system_reliability_sensitivity.csv, C:\Users\mateo\cta-benchmark\results\family_reliability_summary.csv, paper_table_*.csv, paper_table_annotation_evidence.csv, paper_table_agreement_evidence.csv, appendix_mapped_evidence/
wrote C:\Users\mateo\cta-benchmark\benchmark\v0.3\benchmark_paper_summary.json
wrote C:\Users\mateo\cta-benchmark\results\paper_system_set.md
implemented evidence-hardening outputs
```

## `C:\Users\mateo\miniconda3\python.exe scripts/repair_counterfactual_metrics.py`

- Status: **PASS**

```text
wrote C:\Users\mateo\cta-benchmark\results\repair_impact_summary.json
```

## `C:\Users\mateo\miniconda3\python.exe scripts/validate_release_artifact.py`

- Status: **PASS**

```text
validate_release_artifact: ok
```

## `C:\Users\mateo\miniconda3\python.exe scripts/ci_reviewer_readiness.py`

- Status: **PASS**

```text
ok: C:\Users\mateo\cta-benchmark\benchmark\v0.3\annotation\adjudicated_subset\manifest.json validates against AnnotationPackManifest
ok: C:\Users\mateo\cta-benchmark\benchmark\v0.3\protocol_freeze.json validates against ProtocolFreeze
ok: C:\Users\mateo\cta-benchmark\schemas\failure_mode_v1.json validates against FailureModeOntology
validate_release_artifact: ok
ci_reviewer_readiness: ok
```

## `C:\Users\mateo\miniconda3\python.exe scripts/compute_human_strict_agreement.py --packet-map annotation/human_pass_v3/human_strict_packet_ids.csv --rater-a annotation/rater_a_strict_all.csv --rater-b annotation/human_pass_v3/rater_b_human_strict_all.csv --out-json annotation/human_pass_v3/agreement_report_human_strict_all.json --out-md annotation/human_pass_v3/agreement_report_human_strict_all.md --out-disagreements annotation/human_pass_v3/disagreement_log_strict_all.csv`

- Status: **PASS**

```text
wrote annotation\human_pass_v3\agreement_report_human_strict_all.json, annotation\human_pass_v3\agreement_report_human_strict_all.md, annotation\human_pass_v3\disagreement_log_strict_all.csv
```

## `C:\Users\mateo\miniconda3\python.exe scripts/check_paper_claim_sources.py`

- Status: **PASS**

```text
check_paper_claim_sources: OK - strict headline discipline, yaml manifest, and mandatory paths verified.
```

## `cd lean && lake build`

- Status: **PASS**

```text
Build completed successfully.
```

