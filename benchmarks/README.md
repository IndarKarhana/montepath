# Benchmark Outputs

This directory stores generated benchmark reports.

## Current Baseline

From `latest-results.json`:

- `schema_validation`: see `latest-results.json`
- `planner_overhead_auto`: see `latest-results.json`
- `planner_choice_accuracy`: `100.0%` on the internal scenario set
- `mc_cpu_european_call_rust` (`stepwise_paths`): tracked as the fair CPU baseline
- `mc_cpu_european_call_rust_antithetic` (`stepwise_paths_antithetic`): tracked as the first variance-reduced CPU path
- `mc_cpu_european_call_rust_antithetic_quality`: reports `stderr_ratio_vs_standard`
- `mc_cpu_european_call_rust_control_variate` (`stepwise_paths_control_variate`): tracked as the strongest current quality-improving CPU path
- `mc_cpu_european_call_rust_control_variate_quality`: reports `stderr_ratio_vs_standard`
- `mc_cpu_european_call_rust_terminal` (`terminal_distribution`): tracked as the specialized fast path
- `mc_cpu_european_call_rust_terminal_antithetic_quality`: reports `stderr_ratio_vs_standard`
- `mc_cpu_european_call_rust_terminal_control_variate_quality`: reports `stderr_ratio_vs_standard`
- `mc_metal_european_call_native` (`stepwise_paths_native_metal`): tracked as the first native Apple GPU execution result

From `release-results.json`:

- `mc_cpu_european_call_rust` (`stepwise_paths`): `15.129 ms` per run
- `mc_cpu_european_call_rust_antithetic` (`stepwise_paths_antithetic`): `28.199 ms` per run
- `mc_cpu_european_call_rust_antithetic_quality`: `stderr_ratio_vs_standard = 0.747`
- `mc_cpu_european_call_rust_control_variate` (`stepwise_paths_control_variate`): `14.248 ms` per run
- `mc_cpu_european_call_rust_control_variate_quality`: `stderr_ratio_vs_standard = 0.411`
- `mc_cpu_european_call_rust_terminal` (`terminal_distribution`): `0.621 ms` per run
- `mc_cpu_european_call_rust_terminal_antithetic` (`terminal_distribution_antithetic`): `1.186 ms` per run
- `mc_cpu_european_call_rust_terminal_antithetic_quality`: `stderr_ratio_vs_standard = 0.741`
- `mc_cpu_european_call_rust_terminal_control_variate`: `0.610 ms` per run
- `mc_cpu_european_call_rust_terminal_control_variate_quality`: `stderr_ratio_vs_standard = 0.412`
- `mc_metal_european_call_native` (`stepwise_paths_native_metal`): `1.252 ms` per run
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
