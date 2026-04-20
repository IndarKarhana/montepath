# Benchmark Outputs

This directory stores generated benchmark reports.

## Current Baseline

From `latest-results.json`:

- `schema_validation`: `9.103 us` per iteration (`109,848.72 ops/sec`)
- `planner_overhead_auto`: `0.473 us` per iteration (`2,113,588.02 ops/sec`)
- `planner_choice_accuracy`: `100.0%` on the internal scenario set
- `mc_cpu_european_call_rust`: `1.374 ms` per run (`72,794,914.64 paths/sec`)
- `mc_cpu_european_call_numpy`: `89.786 ms` per run (`1,113,761.45 paths/sec`)
- `mc_cpu_european_call_numba`: `481.461 ms` per run (`207,700.97 paths/sec`)

From `release-results.json`:

- `mc_cpu_european_call_rust`: `1.065 ms` per run (`93,853,884.52 paths/sec`)
- `mc_cpu_european_call_numpy`: `85.180 ms` per run (`1,173,989.48 paths/sec`)
- `mc_cpu_european_call_numba`: `328.711 ms` per run (`304,218.95 paths/sec`)

## Competitiveness Output

Running benchmarks also generates:

- `benchmarks/improvement-plan.md`

That file documents whether we lead or lose against available baselines and includes an action plan when we are behind.

## Regeneration

```bash
cargo run -p mc-bench -- --output benchmarks/latest-results.json
```

Benchmark thresholds are defined in `docs/benchmark-gates.md` and enforced by `crates/mc-bench/tests/gates.rs`.
