# Benchmark Gates

This document defines early benchmark quality gates for local development and CI.

## Purpose

The gates prevent obvious regressions while the codebase is still early.

## Initial Gates

1. `schema_validation` per-iteration latency should stay below `50 us` in debug benchmark runs.
2. `planner_overhead_auto` per-iteration latency should stay below `10 us` in debug benchmark runs.
3. `planner_choice_accuracy` should remain at or above `75%` on the internal scenario set.
4. `mc_cpu_european_call_rust` must be present in benchmark results.
5. A competitiveness plan must always be generated (`benchmarks/improvement-plan.md`) and include either:
- `Maintain lead plan` when we win
- `Action plan to close the gap` when we lose

These thresholds are intentionally conservative for early development and should be tightened as functionality grows.

## Notes

- These gates are measured against `crates/mc-bench` outputs.
- Debug builds are currently used for convenience and fast iteration.
- Release-mode benchmarking should be introduced for formal performance reporting.
