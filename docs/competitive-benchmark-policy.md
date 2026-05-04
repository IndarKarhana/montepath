# Competitive Benchmark Policy

## Goal

Benchmark against best-in-class libraries and keep `mc-library` at or above competitive performance on target workloads.

## Baseline Competitor Set

- CPU: NumPy, Numba, QuantLib for overlapping quantitative-finance workloads
- QMC: SciPy `stats.qmc`
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
- `cargo run -p mc-bench --release -- --output benchmarks/release-results.json`
  - generates optimized-profile benchmark report for competitiveness tracking
  - refreshes competitiveness plan based on release-profile data
- `.github/workflows/accelerator-competitors.yml`
  - manual hardware workflow for JAX, CuPy, and PyTorch accelerator competitor rows
  - requires populated telemetry before uploading accelerator competitor artifacts

Competitor environment manifests are stored under
`benchmarks/competitors/environments/`.

## Definition of Competitive Success

For each tracked workload where competitor baseline is available:

- preferred: `mc-library` runtime <= fastest competitor runtime
- acceptable temporary state: `mc-library` slower, but actionable plan generated and tracked in roadmap
