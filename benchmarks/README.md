# Benchmark Outputs

This directory stores generated benchmark reports.

## Current Baseline

From `latest-results.json`:

- `schema_validation`: `8.364 us` per iteration (`119,556.74 ops/sec`)
- `planner_overhead_auto`: `0.410 us` per iteration (`2,437,241.04 ops/sec`)
- `planner_choice_accuracy`: `100.0%` on the internal scenario set
- `mc_cpu_european_call_rust`: `300.109 ms` per run (`333,211.85 paths/sec`)
- `mc_cpu_european_call_numpy`: `70.524 ms` per run (`1,417,957.56 paths/sec`)
- `mc_cpu_european_call_numba`: `254.348 ms` per run (`393,161.87 paths/sec`)

## Competitiveness Output

Running benchmarks also generates:

- `benchmarks/improvement-plan.md`

That file documents whether we lead or lose against available baselines and includes an action plan when we are behind.

## Regeneration

```bash
cargo run -p mc-bench -- --output benchmarks/latest-results.json
```

Benchmark thresholds are defined in `docs/benchmark-gates.md` and enforced by `crates/mc-bench/tests/gates.rs`.
