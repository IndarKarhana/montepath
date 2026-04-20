# mc-library

Agent-native Monte Carlo runtime with CPU, NVIDIA CUDA, and Apple Metal execution paths.

## Current Stage

This repository is in active build-out.

- architecture and design docs are in `docs/`
- core engineering rules are in `docs/repository-rules.md`
- roadmap is in `roadmap.md`
- Rust workspace scaffolding is in `crates/`

## Core Principles

- architecture and design docs are source of truth
- roadmap is a living artifact and must be maintained
- test-driven development is the default workflow
- production-grade quality bar from day one
- fast and lightweight implementation choices

## Workspace Layout

- `crates/mc-schema`: schema types, diagnostics, compatibility, and validation
- `crates/mc-core`: planner interfaces, backend contract, CPU runtime, and execution planning
- `crates/mc-bench`: benchmark harness and benchmark result schema

## Running Tests

```bash
cargo test
```

## Running Benchmarks

```bash
cargo run -p mc-bench -- --output benchmarks/latest-results.json
```

```bash
cargo run -p mc-bench --release -- --output benchmarks/release-results.json
```

## Benchmark Gates

Benchmark thresholds are documented in `docs/benchmark-gates.md`.
Competitive benchmark policy is documented in `docs/competitive-benchmark-policy.md`.
User-experience research and UX implementation plan is in `docs/user-friendliness-research.md`.

## Next Steps

- implement first NVIDIA CUDA kernels for core workload path
- implement first Apple Metal kernels for core workload path
- add planner heuristics for GPU backend selection
- expand competitor matrix to JAX/CuPy/PyTorch where environment allows
- add CI for lint, test, and benchmark gate checks
