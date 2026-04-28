# Benchmark Outputs

This directory stores generated benchmark reports.

## Current Baseline

From `latest-results.json`:

- `schema_validation`: see `latest-results.json`
- `planner_overhead_auto`: see `latest-results.json`
- `planner_choice_accuracy`: `100.0%` on the internal scenario set
- `planner_choice_accuracy_measured`: `87.5%`, tracking planner accuracy against measured local backend winners on the current machine
- `mc_cpu_european_call_rust` (`stepwise_paths`): tracked as the fair CPU European baseline
- `mc_cpu_european_call_rust_antithetic` (`stepwise_paths_antithetic`): tracked as the first variance-reduced CPU European path
- `mc_cpu_european_call_rust_control_variate` (`stepwise_paths_control_variate`): tracked as the strongest current CPU European quality-improving path
- `mc_cpu_arithmetic_asian_call_rust` (`arithmetic_asian_stepwise`): tracked as the second CPU workload family
- `mc_cpu_arithmetic_asian_call_rust_control_variate`: tracks the arithmetic-Asian CPU control-variate path
- `mc_cpu_arithmetic_asian_call_rust_mlmc`: tracks the first arithmetic-Asian CPU MLMC path
- `mc_cpu_arithmetic_asian_call_rust_mlqmc`: tracks the first arithmetic-Asian CPU MLQMC path
- `mc_cpu_european_call_rust_randomized_halton` (`stepwise_paths_randomized_halton`): tracks the first randomized-QMC CPU path
- `mc_cpu_european_call_rust_randomized_halton_control_variate_quality`: reports `stderr_ratio_vs_standard`
- `mc_cpu_european_call_rust_latin_hypercube` (`stepwise_paths_latin_hypercube`): tracks the first Latin-hypercube CPU path
- `mc_cpu_european_call_rust_latin_hypercube_control_variate_quality`: reports `stderr_ratio_vs_standard`
- `mc_cpu_european_call_rust_scrambled_sobol` and `mc_cpu_european_call_rust_scrambled_sobol_brownian_bridge`: track scrambled Sobol CPU paths
- `mc_cpu_qmc_realized_error_european_*`: tracks European QMC absolute-error ratios against the Black-Scholes analytic reference
- `mc_cpu_qmc_rust_*_generation` and `mc_cpu_qmc_scipy_qmc_*_generation`: track direct Rust-vs-SciPy QMC normal generation timing and sample-mean sanity metrics
- `mc_cpu_european_call_quantlib` or `mc_cpu_european_call_quantlib_unavailable`: tracks the selected QuantLib-Python `MCEuropeanEngine` competitor lane for the overlapping European workload
- `mc_cpu_qmc_quality_*`: tracks structured-pricing stderr ratios against pseudorandom baselines across European, arithmetic Asian, down-and-out, and basket workloads
- `mc_cpu_gaussian_uncertainty_rust_*`: tracks a non-option Gaussian uncertainty-propagation benchmark with analytic-mean error
- structured-sampling benchmarks now cover European, arithmetic Asian, down-and-out, and two-asset basket CPU workload families
- `mc_cpu_down_and_out_call_rust` (`down_and_out_stepwise`): tracks the third CPU workload family
- `mc_cpu_down_and_out_call_rust_control_variate`: tracks the down-and-out CPU control-variate path
- `mc_cpu_down_and_out_call_rust_control_variate_quality`: reports `stderr_ratio_vs_standard`
- `mc_cpu_basket_call_rust*`: tracks the two-asset terminal basket-call CPU workload across pseudorandom, randomized Halton, Latin hypercube, and scrambled Sobol sampling
- `mc_cpu_qmc_quality_basket_*`: reports basket `stderr_ratio_vs_pseudorandom`
- `mc_cpu_european_call_rust_terminal` (`terminal_distribution`): tracked as the specialized fast path
- `mc_metal_european_call_native` (`stepwise_paths_native_metal`): tracked as the first native Apple GPU execution result
- `mc_metal_european_call_native_antithetic`: tracks the native Apple GPU antithetic path
- `mc_metal_european_call_native_control_variate`: tracks the native Apple GPU European control-variate path
- `mc_metal_arithmetic_asian_call_native` (`arithmetic_asian_stepwise_native_metal`): tracked as the second native Apple GPU workload family
- `mc_metal_arithmetic_asian_call_native_control_variate`: tracks the arithmetic-Asian native Apple GPU control-variate path
- `mc_metal_down_and_out_call_native` (`down_and_out_stepwise_native_metal`): tracked as the third native Apple GPU workload family
- `mc_metal_down_and_out_call_native_control_variate`: tracks the native Apple GPU down-and-out control-variate path

From `release-results.json`:

- `planner_choice_accuracy_measured`: `87.5%` on the measured local backend-winner suite
- `mc_cpu_european_call_rust` (`stepwise_paths`): `11.445 ms` per run, price `9.430456`
- `mc_cpu_european_call_rust_control_variate` (`stepwise_paths_control_variate`): `11.429 ms` per run, `stderr_ratio_vs_standard = 0.411`
- `mc_cpu_european_call_rust_terminal` (`terminal_distribution`): `0.500 ms` per run
- `mc_cpu_arithmetic_asian_call_rust` (`arithmetic_asian_stepwise`): `16.130 ms` per run
- `mc_cpu_arithmetic_asian_call_rust_mlmc`: `4.627 ms` per run, `stderr_ratio_vs_standard = 2.013`
- `mc_cpu_arithmetic_asian_call_rust_mlqmc`: `5.845 ms` per run, `stderr_ratio_vs_standard = 0.418`
- `mc_cpu_down_and_out_call_rust` (`down_and_out_stepwise`): `17.920 ms` per run
- `mc_cpu_basket_call_rust` (`basket_terminal_pseudorandom`): `4.024 ms` per run
- `mc_cpu_basket_call_rust_latin_hypercube` (`basket_terminal_latin_hypercube`): `4.147 ms` per run
- `mc_metal_european_call_native` (`stepwise_paths_native_metal`): `1.429 ms` per run
- `mc_metal_arithmetic_asian_call_native` (`arithmetic_asian_stepwise_native_metal`): `0.585 ms` per run
- `mc_metal_down_and_out_call_native` (`down_and_out_stepwise_native_metal`): `1.425 ms` per run
- `mc_cpu_european_call_numpy` (`stepwise_paths`): `91.110 ms` per run, price `9.486909`
- `mc_cpu_european_call_numba` (`stepwise_paths`): `244.699 ms` per run, price `9.374554`
- `mc_cpu_european_call_quantlib_unavailable`: QuantLib-Python was not installed in the refreshed local release artifact; install QuantLib-Python before refreshing to populate timing and price metrics
- `mc_cpu_qmc_rust_scrambled_sobol_generation`: `75.521 ms` per run, `normal_mean_abs = 0.000004`
- `mc_cpu_qmc_scipy_qmc_sobol_generation`: `123.166 ms` per run, `normal_mean_abs = 0.000002`
- `mc_cpu_qmc_rust_randomized_halton_generation`: `56.695 ms` per run, `normal_mean_abs = 0.000063`
- `mc_cpu_qmc_scipy_qmc_halton_generation`: `141.868 ms` per run, `normal_mean_abs = 0.000017`
- `mc_cpu_qmc_rust_latin_hypercube_generation`: `41.483 ms` per run, `normal_mean_abs = 0.000000`
- `mc_cpu_qmc_scipy_qmc_lhs_generation`: `192.029 ms` per run, `normal_mean_abs = 0.000000`
- `mc_cpu_qmc_realized_error_european_randomized_halton`: `abs_error_ratio_vs_pseudorandom = 0.035`
- `mc_cpu_qmc_realized_error_european_latin_hypercube`: `abs_error_ratio_vs_pseudorandom = 0.021`
- `mc_cpu_qmc_realized_error_european_scrambled_sobol`: `abs_error_ratio_vs_pseudorandom = 0.129`
- `mc_cpu_qmc_realized_error_european_scrambled_sobol_brownian_bridge`: `abs_error_ratio_vs_pseudorandom = 0.001`
- `mc_cpu_gaussian_uncertainty_rust_pseudorandom`: `3.332 ms` per run, `abs_error_vs_analytic_mean = 0.006344`
- `mc_cpu_gaussian_uncertainty_rust_latin_hypercube`: `2.184 ms` per run, `abs_error_vs_analytic_mean = 0.000039`

## Competitiveness Output

Running benchmarks also generates:

- `benchmarks/improvement-plan.md`

That file documents where we lead, where breadth is improving, and what still needs work when a new path is honest but not yet competitive.

## Regeneration

Compact smoke profile for local checks:

```bash
cargo run -p mc-bench -- --profile compact
```

The compact profile covers schema validation, planner checks, representative
CPU workload timing, and core variance-reduction quality gates without
rewriting `benchmarks/improvement-plan.md`.

Full tracked artifact refresh:

```bash
cargo run -p mc-bench -- --output benchmarks/latest-results.json
```

```bash
cargo run -p mc-bench --release --features metal-native -- --output benchmarks/release-results.json
```

Benchmark thresholds are defined in `docs/benchmark-gates.md` and enforced by `crates/mc-bench/tests/gates.rs`.
