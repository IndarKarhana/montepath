# Competitive Benchmark Policy

## Goal

Benchmark against best-in-class libraries and keep `mc-library` at or above competitive performance on target workloads.

## Baseline Competitor Set

- CPU: NumPy, Numba
- GPU: JAX, CuPy, PyTorch (when hardware and environment permit)

## Policy Rules

1. Every benchmark cycle must run internal runtime benchmarks plus competitor benchmarks.
2. If `mc-library` is slower than an available competitor, a written improvement plan must be generated in the same run.
3. Benchmark artifacts must be committed for traceability (`benchmarks/latest-results.json`, `benchmarks/improvement-plan.md`).
4. "Unavailable" competitor results must be explicit in the report (never silently skipped).

## Current Automation

- `cargo run -p mc-bench -- --output benchmarks/latest-results.json`
  - generates benchmark report
  - generates competitiveness plan

## Definition of Competitive Success

For each tracked workload where competitor baseline is available:

- preferred: `mc-library` runtime <= fastest competitor runtime
- acceptable temporary state: `mc-library` slower, but actionable plan generated and tracked in roadmap
