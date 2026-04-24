# Benchmark Outputs

This directory stores generated benchmark reports.

## Current Baseline

From `latest-results.json`:

- `schema_validation`: see `latest-results.json`
- `planner_overhead_auto`: see `latest-results.json`
- `planner_choice_accuracy`: `100.0%` on the internal scenario set
- `planner_choice_accuracy_measured`: `83.3%`, tracking planner accuracy against measured local backend winners on the current machine
- `mc_cpu_european_call_rust` (`stepwise_paths`): tracked as the fair CPU baseline
- `mc_cpu_european_call_rust_antithetic` (`stepwise_paths_antithetic`): tracked as the first variance-reduced CPU path
- `mc_cpu_european_call_rust_antithetic_quality`: reports `stderr_ratio_vs_standard`
- `mc_cpu_european_call_rust_control_variate` (`stepwise_paths_control_variate`): tracked as the strongest current quality-improving CPU path
- `mc_cpu_european_call_rust_control_variate_quality`: reports `stderr_ratio_vs_standard`
- `mc_cpu_arithmetic_asian_call_rust` (`arithmetic_asian_stepwise`): tracked as the second CPU workload family
- `mc_cpu_arithmetic_asian_call_rust_control_variate`: tracks the first quality-improving arithmetic-Asian CPU path
- `mc_cpu_arithmetic_asian_call_rust_control_variate_quality`: reports `stderr_ratio_vs_standard`
- `mc_cpu_european_call_rust_terminal` (`terminal_distribution`): tracked as the specialized fast path
- `mc_cpu_european_call_rust_terminal_antithetic_quality`: reports `stderr_ratio_vs_standard`
- `mc_cpu_european_call_rust_terminal_control_variate_quality`: reports `stderr_ratio_vs_standard`
- `mc_metal_european_call_native` (`stepwise_paths_native_metal`): tracked as the first native Apple GPU execution result
- `mc_metal_european_call_native_antithetic` (`stepwise_paths_native_metal_antithetic`): tracked as the native Apple GPU antithetic path
- `mc_metal_european_call_native_antithetic_quality`: reports `stderr_ratio_vs_standard`
- `mc_metal_european_call_native_control_variate` (`stepwise_paths_native_metal_control_variate`): tracked as the native Apple GPU control-variate path
- `mc_metal_european_call_native_control_variate_quality`: reports `stderr_ratio_vs_standard`
- `mc_metal_arithmetic_asian_call_native` (`arithmetic_asian_stepwise_native_metal`): tracked as the second native Apple GPU workload family
- `mc_metal_arithmetic_asian_call_native_control_variate`: tracks the arithmetic-Asian native Apple GPU control-variate path
- `mc_metal_arithmetic_asian_call_native_control_variate_quality`: reports `stderr_ratio_vs_standard`

From `release-results.json`:

- `mc_cpu_european_call_rust` (`stepwise_paths`): `21.818 ms` per run
- `mc_cpu_european_call_rust_antithetic` (`stepwise_paths_antithetic`): `58.726 ms` per run
- `mc_cpu_european_call_rust_antithetic_quality`: `stderr_ratio_vs_standard = 0.747`
- `mc_cpu_european_call_rust_control_variate` (`stepwise_paths_control_variate`): `39.073 ms` per run
- `mc_cpu_european_call_rust_control_variate_quality`: `stderr_ratio_vs_standard = 0.411`
- `mc_cpu_arithmetic_asian_call_rust` (`arithmetic_asian_stepwise`): `31.783 ms` per run
- `mc_cpu_arithmetic_asian_call_rust_control_variate` (`arithmetic_asian_stepwise_control_variate`): `41.735 ms` per run
- `mc_cpu_arithmetic_asian_call_rust_control_variate_quality`: `stderr_ratio_vs_standard = 0.607`
- `mc_cpu_european_call_rust_terminal` (`terminal_distribution`): `7.209 ms` per run
- `mc_cpu_european_call_rust_terminal_antithetic` (`terminal_distribution_antithetic`): `2.343 ms` per run
- `mc_cpu_european_call_rust_terminal_antithetic_quality`: `stderr_ratio_vs_standard = 0.741`
- `mc_cpu_european_call_rust_terminal_control_variate`: `1.330 ms` per run
- `mc_cpu_european_call_rust_terminal_control_variate_quality`: `stderr_ratio_vs_standard = 0.412`
- `mc_metal_european_call_native` (`stepwise_paths_native_metal`): `1.457 ms` per run
- `mc_metal_european_call_native_antithetic` (`stepwise_paths_native_metal_antithetic`): `1.013 ms` per run
- `mc_metal_european_call_native_antithetic_quality`: `stderr_ratio_vs_standard = 0.746`
- `mc_metal_european_call_native_control_variate` (`stepwise_paths_native_metal_control_variate`): `1.439 ms` per run
- `mc_metal_european_call_native_control_variate_quality`: `stderr_ratio_vs_standard = 0.409`
- `mc_metal_arithmetic_asian_call_native` (`arithmetic_asian_stepwise_native_metal`): `1.751 ms` per run
- `mc_metal_arithmetic_asian_call_native_control_variate` (`arithmetic_asian_stepwise_native_metal_control_variate`): `1.520 ms` per run
- `mc_metal_arithmetic_asian_call_native_control_variate_quality`: `stderr_ratio_vs_standard = 0.609`
- `mc_cpu_european_call_numpy` (`stepwise_paths`): compare in `release-results.json`
- `mc_cpu_european_call_numba` (`stepwise_paths`): compare in `release-results.json`

## Competitiveness Output

Running benchmarks also generates:

- `benchmarks/improvement-plan.md`

That file documents whether we lead or lose against available baselines and includes an action plan when we are behind.

## Regeneration

```bash
cargo run -p mc-bench -- --output benchmarks/latest-results.json
```

Benchmark thresholds are defined in `docs/benchmark-gates.md` and enforced by `crates/mc-bench/tests/gates.rs`.
