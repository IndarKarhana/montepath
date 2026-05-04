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
- `mc_cpu_qmc_quality_*`: tracks structured-pricing stderr ratios against pseudorandom baselines across European, arithmetic Asian, down-and-out, lookback, and basket workloads
- `mc_cpu_gaussian_uncertainty_rust_*`: tracks a non-option Gaussian uncertainty-propagation benchmark with analytic-mean error
- structured-sampling benchmarks now cover European, arithmetic Asian, down-and-out, fixed-strike lookback, and two-asset basket CPU workload families
- `mc_cpu_down_and_out_call_rust` (`down_and_out_stepwise`): tracks the third CPU workload family
- `mc_cpu_down_and_out_call_rust_control_variate`: tracks the down-and-out CPU control-variate path
- `mc_cpu_down_and_out_call_rust_control_variate_quality`: reports `stderr_ratio_vs_standard`
- `mc_cpu_lookback_call_rust`: tracks the fixed-strike lookback CPU workload with discrete step-wise monitoring
- `mc_cpu_lookback_call_quantlib` or `mc_cpu_lookback_call_quantlib_unavailable`: tracks the selected QuantLib-Python lookback competitor lane
- `mc_cpu_qmc_quality_lookback_latin_hypercube`: reports lookback `stderr_ratio_vs_pseudorandom`
- `mc_cpu_heston_european_call_rust`: tracks the Heston stochastic-volatility European-call CPU workload with full-truncation Euler stepping
- `mc_cpu_heston_black_scholes_limit_quality`: reports Heston zero-vol-of-vol constant-variance absolute error against the Black-Scholes reference
- `mc_cpu_heston_european_call_quantlib` or `mc_cpu_heston_european_call_quantlib_unavailable`: tracks the selected QuantLib-Python analytic Heston reference lane
- `mc_cpu_european_call_greeks_*_rust`: tracks bump-and-revalue, pathwise, and likelihood-ratio European-call Greek timing with Black-Scholes Delta error
- `mc_cpu_heston_greeks_black_scholes_limit_delta_quality`: tracks Heston Black-Scholes-limit bump Greek quality
- `mc_cpu_all_workload_greeks_bump_rust`: tracks bump-and-revalue Greek breadth across current CPU workload families
- `docs/product-model-capability-catalog.json`: machine-readable Phase 2 product/model capability catalog, including Greek estimator support and unsupported states
- `benchmarks/reference-fixtures.json`: trusted reference fixture registry and explicit caveats for workloads without committed analytic or high-precision fixtures
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
- `mc_gpu_european_call_jax` or `mc_gpu_european_call_jax_unavailable`: tracks the JAX GPU competitor lane when hardware and package support are available
- `mc_gpu_european_call_cupy` or `mc_gpu_european_call_cupy_unavailable`: tracks the CuPy CUDA competitor lane when hardware and package support are available
- `mc_gpu_european_call_torch` or `mc_gpu_european_call_torch_unavailable`: tracks the PyTorch CUDA competitor lane when hardware and package support are available

From `release-results.json`:

- `planner_choice_accuracy_measured`: `87.5%` on the measured local backend-winner suite
- `mc_cpu_european_call_rust` (`stepwise_paths`): `12.845 ms` per run, price `9.430456`
- `mc_cpu_european_call_rust_control_variate` (`stepwise_paths_control_variate`): see `release-results.json`, `stderr_ratio_vs_standard = 0.411`
- `mc_cpu_european_call_rust_terminal` (`terminal_distribution`): `0.524 ms` per run
- `mc_cpu_arithmetic_asian_call_rust` (`arithmetic_asian_stepwise`): `17.693 ms` per run
- `mc_cpu_arithmetic_asian_call_rust_mlmc`: `4.303 ms` per run, `stderr_ratio_vs_standard = 2.013`
- `mc_cpu_arithmetic_asian_call_rust_mlqmc`: `5.749 ms` per run, `stderr_ratio_vs_standard = 0.418`
- `mc_cpu_down_and_out_call_rust` (`down_and_out_stepwise`): `18.449 ms` per run
- `mc_cpu_lookback_call_rust` (`lookback_fixed_strike_stepwise`): `16.349 ms` per run, price `16.671609`
- `mc_cpu_lookback_call_rust_control_variate` (`lookback_fixed_strike_stepwise_control_variate`): `17.048 ms` per run, price `16.649141`
- `mc_cpu_qmc_quality_lookback_latin_hypercube`: `stderr_ratio_vs_pseudorandom = 1.000`
- `mc_cpu_heston_european_call_rust` (`heston_full_truncation_euler_stepwise`): `26.461 ms` per run, price `9.173258`
- `mc_cpu_heston_black_scholes_limit_quality`: `abs_error_vs_black_scholes = 0.039572`
- `mc_cpu_european_call_greeks_bump_rust`: `3.225 ms` per run, `abs_delta_error_vs_black_scholes = 0.000126`
- `mc_cpu_european_call_greeks_pathwise_rust`: `1.523 ms` per run, `abs_delta_error_vs_black_scholes = 0.000281`
- `mc_cpu_european_call_greeks_likelihood_ratio_rust`: `1.460 ms` per run, `abs_delta_error_vs_black_scholes = 0.002631`
- `mc_cpu_heston_greeks_black_scholes_limit_delta_quality`: `136.517 ms` per run, `abs_delta_error_vs_black_scholes = 0.000227`
- `mc_cpu_all_workload_greeks_bump_rust`: `216.317 ms` per run, `greek_estimate_count = 26`
- `mc_cpu_basket_call_rust` (`basket_terminal_pseudorandom`): `4.147 ms` per run
- `mc_cpu_basket_call_rust_latin_hypercube` (`basket_terminal_latin_hypercube`): `4.179 ms` per run
- `mc_metal_european_call_native` (`stepwise_paths_native_metal`): `1.495 ms` per run
- `mc_metal_arithmetic_asian_call_native` (`arithmetic_asian_stepwise_native_metal`): `0.634 ms` per run
- `mc_metal_down_and_out_call_native` (`down_and_out_stepwise_native_metal`): `1.140 ms` per run
- `mc_cpu_european_call_numpy` (`stepwise_paths`): `76.321 ms` per run, price `9.486909`
- `mc_cpu_european_call_numba` (`stepwise_paths`): `222.326 ms` per run, price `9.374554`
- `mc_cpu_european_call_quantlib_unavailable`: QuantLib-Python was not installed in the refreshed local release artifact; install QuantLib-Python before refreshing to populate timing and price metrics
- `mc_cpu_lookback_call_quantlib_unavailable`: QuantLib-Python was not installed in the refreshed local release artifact; install QuantLib-Python before refreshing to populate timing and price metrics
- `mc_cpu_heston_european_call_quantlib_unavailable`: QuantLib-Python was not installed in the refreshed local release artifact; install QuantLib-Python before refreshing to populate the analytic Heston reference lane
- `mc_cpu_qmc_rust_scrambled_sobol_generation`: `74.437 ms` per run, `normal_mean_abs = 0.000004`
- `mc_cpu_qmc_scipy_qmc_sobol_generation`: `115.477 ms` per run, `normal_mean_abs = 0.000002`
- `mc_cpu_qmc_rust_randomized_halton_generation`: `56.383 ms` per run, `normal_mean_abs = 0.000063`
- `mc_cpu_qmc_scipy_qmc_halton_generation`: `145.888 ms` per run, `normal_mean_abs = 0.000017`
- `mc_cpu_qmc_rust_latin_hypercube_generation`: `39.611 ms` per run, `normal_mean_abs = 0.000000`
- `mc_cpu_qmc_scipy_qmc_lhs_generation`: `195.198 ms` per run, `normal_mean_abs = 0.000000`
- `mc_cpu_qmc_realized_error_european_randomized_halton`: `abs_error_ratio_vs_pseudorandom = 0.035`
- `mc_cpu_qmc_realized_error_european_latin_hypercube`: `abs_error_ratio_vs_pseudorandom = 0.021`
- `mc_cpu_qmc_realized_error_european_scrambled_sobol`: `abs_error_ratio_vs_pseudorandom = 0.129`
- `mc_cpu_qmc_realized_error_european_scrambled_sobol_brownian_bridge`: `abs_error_ratio_vs_pseudorandom = 0.001`
- `mc_cpu_gaussian_uncertainty_rust_pseudorandom`: `3.131 ms` per run, `abs_error_vs_analytic_mean = 0.006344`
- `mc_cpu_gaussian_uncertainty_rust_latin_hypercube`: `2.135 ms` per run, `abs_error_vs_analytic_mean = 0.000039`

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

QuantLib-populated competitor environment:

```bash
python -m pip install -r benchmarks/competitors/requirements-quantlib.txt
cargo run -p mc-bench --release -- --output benchmarks/quantlib-ci-results.json
```

CI has a dedicated `quantlib-benchmark` job that installs this environment,
preflights the QuantLib rows, and uploads `benchmarks/quantlib-ci-results.json`
as an artifact.

Accelerator competitor environment:

```bash
python -m pip install -r benchmarks/competitors/requirements-accelerators.txt
python benchmarks/competitors/python_cpu_baselines.py --paths 100000 --steps 64 --repeats 3 --seed 42
```

Dedicated competitor environment manifests live in
`benchmarks/competitors/environments/`. The manual
`accelerator-competitors` workflow should be used for hardware-backed JAX,
CuPy, and PyTorch results.

Benchmark thresholds are defined in `docs/benchmark-gates.md` and enforced by `crates/mc-bench/tests/gates.rs`.
