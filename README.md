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

`mc-core` now also exposes host-side native staging gates:

- `cuda-native`
- `metal-native`

These feature flags validate native backend boundaries, kernel metadata, and toolchain probing without requiring local GPU hardware.

The CUDA path now includes an actual staged kernel source at:

- `crates/mc-core/src/backend/kernels/european_call_stepwise_v1.cu`

When `cuda-native` is enabled and `nvcc` is available, the backend attempts PTX compilation during artifact staging and records the result in native artifact metadata. Execution still falls back to the CPU reference path until native launch support lands.

The Metal path now includes a matching staged shader source at:

- `crates/mc-core/src/backend/kernels/european_call_stepwise_v1.metal`

When `metal-native` is enabled and Apple developer tools are available, the backend attempts `.air` and `.metallib` compilation during artifact staging and records the result in native artifact metadata.

On macOS, the first Metal-native execution path now runs in-process from Rust using cached Metal library and pipeline state. The current Apple GPU path executes the first European-call step-wise family natively on supported Macs, generates shocks on-device, and reduces aggregates on-device down to a single final value before host readback. The native Apple path now supports `Standard`, `Antithetic`, and `ControlVariate` techniques for this workload family.

## Running Tests

```bash
cargo test
```

```bash
cargo test -p mc-core --features cuda-native
```

```bash
cargo test -p mc-core --features metal-native
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
Market landscape notes are in `docs/market-landscape.md`.

## Current Results

From the latest release benchmark run:

- fair step-wise Rust CPU path: `21.818 ms`
- step-wise Rust antithetic path: `58.726 ms`
- step-wise Rust control-variate path: `39.073 ms`
- arithmetic Asian Rust CPU path: `31.783 ms`
- arithmetic Asian Rust CPU control-variate path: `41.735 ms`
- native Metal step-wise path on macOS: `1.457 ms`
- native Metal antithetic step-wise path on macOS: `1.013 ms`
- native Metal control-variate step-wise path on macOS: `1.439 ms`
- native Metal arithmetic Asian path on macOS: `1.751 ms`
- native Metal arithmetic Asian control-variate path on macOS: `1.520 ms`
- step-wise NumPy baseline: see `benchmarks/release-results.json`
- step-wise Numba baseline: see `benchmarks/release-results.json`
- specialized Rust terminal-distribution fast path: `7.209 ms`
- control-variate stderr ratio vs standard:
  - step-wise: `0.411`
  - terminal: `0.412`
  - arithmetic Asian: `0.607`
- antithetic stderr ratio vs standard:
  - step-wise: `0.747`
  - terminal: `0.741`
- measured planner choice accuracy vs local backend winners: `83.3%`

## Next Steps

- implement first NVIDIA CUDA kernels for the core workload path
- broaden Apple Metal native coverage beyond the current GBM option family
- keep calibrating planner recommendations from measured backend winners across more workload classes
- expand Apple-specific planner heuristics beyond the current benchmark-calibrated medium-large workload band
- expand competitor matrix to JAX/CuPy/PyTorch where environment allows
- extend CI from CPU validation to native hardware validation once dedicated runners exist
