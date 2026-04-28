# Selection Robustness Summary

- Baseline reliability mean (`current_selector`): 0.6925
- all_three_samples_best_case_optimistic: mean=0.7107, delta_vs_baseline=+0.0182
- all_three_samples_mean: mean=0.6925, delta_vs_baseline=+0.0000
- all_three_samples_worst_case: mean=0.6533, delta_vs_baseline=-0.0392
- current_selector: mean=0.6925, delta_vs_baseline=+0.0000
- first_parseable_only: mean=0.6826, delta_vs_baseline=-0.0099

## Sensitivity of key caveat metrics
- all_three_samples_best_case_optimistic: contradiction_rate=0.0292, missing_critical_units_mean=0.9635
- all_three_samples_mean: contradiction_rate=0.0292, missing_critical_units_mean=0.9635
- all_three_samples_worst_case: contradiction_rate=0.0292, missing_critical_units_mean=0.9635
- current_selector: contradiction_rate=0.0292, missing_critical_units_mean=0.9635
- first_parseable_only: contradiction_rate=0.0292, missing_critical_units_mean=0.9635

Best-case selector is optimistic and should not be treated as a conservative estimate.
