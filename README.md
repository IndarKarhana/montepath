# mc-library

Agent-native Monte Carlo runtime with CPU, NVIDIA CUDA, and Apple Metal execution paths.

## Current Stage

This repository is in active build-out.

- architecture and design docs are in `docs/`
- project-level agent instructions are in `AGENTS.md`
- Codex project skills are in `./.codex/skills/`
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

## Expressive API Example

```rust
use mc_core::{EuropeanCallMethod, EuropeanCallPricer};

let result = EuropeanCallPricer::new()
    .s0(100.0)
    .strike(100.0)
    .rate(0.03)
    .volatility(0.2)
    .maturity(1.0)
    .paths(100_000)
    .steps(64)
    .seed(42)
    .method(EuropeanCallMethod::StepwisePaths)
    .price();
```

The current CPU runtime exposes both:

- a fair step-wise path benchmark path
- a specialized terminal-distribution fast path
- variance-reduction techniques including antithetic variates and control variates

The current GPU backends execute through explicit delegated CPU fallback semantics while native CUDA and Metal kernels are being built. That keeps the backend surface real and testable without overstating GPU acceleration.

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
Agent integration guidance is in `docs/agent-integration-plan.md`.
Public function inventory is in `docs/function-catalog.md`.
Technique roadmap is in `docs/simulation-techniques.md`.
GPU testing strategy is in `docs/gpu-testing-strategy.md`.

## Current CPU Results

From the latest release benchmark run:

- fair step-wise Rust CPU path: `18.520 ms`
- step-wise Rust antithetic path: `35.972 ms`
- step-wise Rust control-variate path: `18.703 ms`
- step-wise NumPy baseline: see `benchmarks/release-results.json`
- step-wise Numba baseline: see `benchmarks/release-results.json`
- specialized Rust terminal-distribution fast path: `0.756 ms`
- control-variate stderr ratio vs standard:
  - step-wise: `0.411`
  - terminal: `0.412`
- antithetic stderr ratio vs standard:
  - step-wise: `0.747`
  - terminal: `0.741`

## Next Steps

- implement first NVIDIA CUDA kernels for core workload path
- implement first Apple Metal kernels for core workload path
- calibrate planner recommendations from measured backend winners once native GPU paths exist
- add Apple-specific planner heuristics for Metal-first environments
- expand competitor matrix to JAX/CuPy/PyTorch where environment allows
- extend CI from CPU validation to native hardware validation once dedicated runners exist
