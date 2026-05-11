# Benchmark Gates

This document defines early benchmark quality gates for local development and CI.

## Purpose

The gates prevent obvious regressions while the codebase is still early.

## Initial Gates

1. `schema_validation` per-iteration latency should stay below `100 us` in compact debug benchmark runs.
2. `planner_overhead_auto` per-iteration latency should stay below `10 us` in debug benchmark runs.
3. `planner_choice_accuracy` should remain at or above `75%` on the internal scenario set.
4. `mc_cpu_european_call_rust`, `mc_cpu_down_and_out_call_rust`, `mc_cpu_lookback_call_rust`, `mc_cpu_american_put_lsm_rust`, `mc_cpu_bermudan_put_lsm_rust`, and `mc_cpu_heston_european_call_rust` must be present in benchmark results.
5. The competitiveness-plan builder must produce a plan that includes either:
- `Maintain lead plan` when we win
- `Action plan to close the gap` when we lose
6. If NumPy or Numba benchmarks are available, Rust CPU MC runtime should be faster on the tracked European-call workload.
7. `mc_cpu_heston_black_scholes_limit_quality` must report finite non-negative `abs_error_vs_black_scholes` below `0.5` in compact benchmark runs.
8. European Greek benchmarks for bump-and-revalue, pathwise, and likelihood-ratio estimators must report finite `abs_delta_error_vs_black_scholes` below `0.08`.
9. `mc_cpu_all_workload_greeks_bump_rust` must report at least `24` Greek estimates across the current CPU workload families.

These thresholds are intentionally conservative for early development and should be tightened as functionality grows.

## Notes

- These gates are measured against the compact `crates/mc-bench` profile for fast local and CI feedback.
- Full benchmark runs still refresh `benchmarks/improvement-plan.md`; compact runs do not overwrite the tracked plan artifact.
- Release-mode benchmark artifacts are used for formal performance reporting.
