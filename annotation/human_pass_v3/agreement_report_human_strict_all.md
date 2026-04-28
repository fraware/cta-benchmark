# Human Strict Agreement Report (All Strict Rows)

- n_rows: 274
- n_unique_instance_ids: 84
- n_systems: 4
- n_mapped_from_canonical: 0

## Ordinal Metrics
- semantic_faithfulness: linear_weighted_kappa=0.8400, quadratic_weighted_kappa=0.8556, krippendorff_alpha=0.8555, gwet_ac1=0.8308, gwet_ac2=0.8399, raw_agreement=0.9124
- code_consistency: linear_weighted_kappa=0.8475, quadratic_weighted_kappa=0.8475, krippendorff_alpha=0.8474, gwet_ac1=0.8474, gwet_ac2=0.8474, raw_agreement=0.9818
- proof_utility: linear_weighted_kappa=0.8715, quadratic_weighted_kappa=0.8763, krippendorff_alpha=0.8762, gwet_ac1=0.8687, gwet_ac2=0.8767, raw_agreement=0.9453

- vacuity_agreement=0.9818, vacuity_kappa=0.0000
- coverage_agreement=0.9526, coverage_kappa=0.9235

## Disagreement Examples
- hs_013 coverage_label: A=full B=partial -> partial (Coverage derived from disjoint covered/partial/missing sets; unresolved missing unit prevents full label.)
- hs_018 semantic_faithfulness: A=1 B=2 -> 1 (Semantic-unit linkage differs across raters; adjudication keeps lower faithfulness where SU evidence is incomplete.)
- hs_021 coverage_label: A=full B=partial -> partial (Coverage derived from disjoint covered/partial/missing sets; unresolved missing unit prevents full label.)
- hs_025 vacuity_label: A=non_vacuous B=vacuous -> vacuous (At least one obligation is tautological/detached; adjudication retains vacuous flag.)
- hs_033 semantic_faithfulness: A=2 B=3 -> 2 (Semantic-unit linkage differs across raters; adjudication keeps lower faithfulness where SU evidence is incomplete.)
- hs_041 semantic_faithfulness: A=3 B=2 -> 2 (Semantic-unit linkage differs across raters; adjudication keeps lower faithfulness where SU evidence is incomplete.)
- hs_051 semantic_faithfulness: A=2 B=3 -> 2 (Semantic-unit linkage differs across raters; adjudication keeps lower faithfulness where SU evidence is incomplete.)
- hs_055 code_consistency: A=3 B=2 -> 2 (Ordinal disagreement resolved with conservative rubric interpretation tied to packet obligations.)
- hs_063 coverage_label: A=full B=partial -> partial (Coverage derived from disjoint covered/partial/missing sets; unresolved missing unit prevents full label.)
- hs_069 semantic_faithfulness: A=2 B=3 -> 2 (Semantic-unit linkage differs across raters; adjudication keeps lower faithfulness where SU evidence is incomplete.)
- hs_072 vacuity_label: A=non_vacuous B=vacuous -> vacuous (At least one obligation is tautological/detached; adjudication retains vacuous flag.)
- hs_073 vacuity_label: A=non_vacuous B=vacuous -> vacuous (At least one obligation is tautological/detached; adjudication retains vacuous flag.)
- hs_073 coverage_label: A=full B=partial -> partial (Coverage derived from disjoint covered/partial/missing sets; unresolved missing unit prevents full label.)
- hs_074 code_consistency: A=2 B=3 -> 2 (Ordinal disagreement resolved with conservative rubric interpretation tied to packet obligations.)
- hs_075 proof_utility: A=1 B=0 -> 0 (Ordinal disagreement resolved with conservative rubric interpretation tied to packet obligations.)
- hs_077 proof_utility: A=1 B=2 -> 1 (Ordinal disagreement resolved with conservative rubric interpretation tied to packet obligations.)
- hs_081 code_consistency: A=3 B=2 -> 2 (Ordinal disagreement resolved with conservative rubric interpretation tied to packet obligations.)
- hs_101 coverage_label: A=full B=partial -> partial (Coverage derived from disjoint covered/partial/missing sets; unresolved missing unit prevents full label.)
- hs_106 proof_utility: A=1 B=2 -> 1 (Ordinal disagreement resolved with conservative rubric interpretation tied to packet obligations.)
- hs_107 proof_utility: A=1 B=2 -> 1 (Ordinal disagreement resolved with conservative rubric interpretation tied to packet obligations.)
