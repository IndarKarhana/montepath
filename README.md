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
- benchmark claims must stay honest about workload scope and methodology

## Workspace Layout

- `crates/mc-schema`: schema types, diagnostics, compatibility, and validation
- `crates/mc-core`: planner interfaces, backend contract, CPU runtime, and execution planning
- `crates/mc-bench`: benchmark harness and benchmark result schema
- `python/mc_library`: Python-first configs, pricing helpers, benchmark helpers, and method recommendations
- `docs/site`: quickstarts, API reference, benchmark interpretation, and migration notes

## Python Quickstart

```bash
python -m pip install -e .
```

```python
from mc_library import EuropeanCallConfig, price_european_call, price_european_call_greeks

cfg = EuropeanCallConfig(n_paths=20_000, n_steps=64, seed=42)
price = price_european_call(cfg)
greeks = price_european_call_greeks(cfg)

print(price.explain())
print(price.manifest)
print(greeks.greeks)
```

The Python helpers are dependency-free reference UX helpers. Timing claims
remain tied to Rust benchmark artifacts.

## Expressive API Example

```rust
use mc_core::{EuropeanCallMethod, EuropeanCallPricer, SamplingMethod};

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
    .sampling(SamplingMethod::LatinHypercube)
    .control_variate()
    .price();
```

```rust
use mc_core::{EuropeanCallPricer, Greek, GreekEstimator};

let greeks = EuropeanCallPricer::new()
    .paths(100_000)
    .seed(42)
    .greeks(GreekEstimator::Pathwise);

let delta = greeks.estimate(Greek::Delta).unwrap().value;
```

## Current Runtime Surface

The CPU runtime now exposes:

- a fair step-wise path benchmark path
- a specialized terminal-distribution fast path
- variance-reduction techniques including antithetic variates and control variates
- separate sampling selection via `SamplingMethod::{Pseudorandom, RandomizedHalton, LatinHypercube, ScrambledSobol, ScrambledSobolBrownianBridge}`
- arithmetic Asian multilevel Monte Carlo via `ArithmeticAsianMlmcConfig` and `arithmetic_asian_call_price_mlmc_cpu()`
- structured Greek reports via `GreekReport`, with bump-and-revalue support across current CPU workloads and European pathwise / likelihood-ratio estimators where valid
- a machine-readable method capability catalog via `monte_carlo_method_capabilities()`
- method recommendation via `recommend_method()` in Rust and `mc_library.recommend_method()` in Python
- multiple workload families:
  - European call
  - arithmetic Asian call
  - down-and-out call
  - two-asset basket call
  - fixed-strike lookback call
  - Heston European call

The GPU backend layer now exposes:

- truthful delegated fallback semantics for unsupported features
- staged native CUDA and Metal artifact metadata and kernel ABI contracts
- real native Apple Metal execution for the current GBM step-wise workload family on supported Macs

The current native Apple Metal path supports:

- European call: `Standard`, `Antithetic`, `ControlVariate`
- arithmetic Asian call: `Standard`, `ControlVariate`
- down-and-out call: `Standard`, `ControlVariate`

Structured-sampling requests currently run truthfully on CPU reference paths rather than being silently approximated on Metal.

## Native Feature Gates

`mc-core` exposes host-side native staging gates:

- `cuda-native`
- `metal-native`

These validate native backend boundaries, kernel metadata, and toolchain probing without requiring local GPU hardware.

The CUDA path includes a staged kernel source at:

- `crates/mc-core/src/backend/kernels/european_call_stepwise_v1.cu`

When `cuda-native` is enabled and `nvcc` is available, the backend attempts PTX compilation during artifact staging and records the result in native artifact metadata. Execution still falls back to the CPU reference path until native CUDA launch support lands.

The Metal path includes a staged shader source at:

- `crates/mc-core/src/backend/kernels/european_call_stepwise_v1.metal`

When `metal-native` is enabled, the backend can compile and execute the current staged Metal workload family in-process on supported macOS hosts.

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

Fast smoke profile for local checks and benchmark-gate tests:

```bash
cargo run -p mc-bench -- --profile compact
```

Full profile for competitiveness artifacts:

```bash
cargo run -p mc-bench -- --output benchmarks/latest-results.json
```

```bash
cargo run -p mc-bench --release --features metal-native -- --output benchmarks/release-results.json
```

The compact profile reports a representative subset and does not overwrite
`benchmarks/improvement-plan.md`. The full profile remains the source for
tracked performance claims.

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

- fair step-wise Rust CPU European path: `12.845 ms`
- step-wise Rust antithetic path: `29.021 ms`
- step-wise Rust control-variate path: `13.099 ms`
- arithmetic Asian Rust CPU path: `17.693 ms`
- arithmetic Asian Rust CPU control-variate path: `17.670 ms`
- arithmetic Asian Rust CPU MLMC path: `4.303 ms`
- arithmetic Asian Rust CPU MLQMC path: `5.749 ms`
- randomized Halton Rust CPU European path: `78.125 ms`
- Latin hypercube Rust CPU European path: `63.870 ms`
- scrambled Sobol Rust CPU European path: `78.774 ms`
- scrambled Sobol Brownian bridge Rust CPU European path: `102.601 ms`
- down-and-out Rust CPU path: `18.449 ms`
- down-and-out Rust CPU control-variate path: `22.168 ms`
- fixed-strike lookback Rust CPU path: `16.349 ms`
- Heston European Rust CPU path: `26.461 ms`
- basket Rust CPU pseudorandom path: `4.147 ms`
- basket Rust CPU Latin hypercube path: `4.003 ms`
- basket Rust CPU scrambled Sobol path: `7.002 ms`
- European bump-and-revalue Delta error vs Black-Scholes: `0.000126` in `3.225 ms`
- European pathwise Delta error vs Black-Scholes: `0.000281` in `1.523 ms`
- European likelihood-ratio Delta error vs Black-Scholes: `0.002631` in `1.460 ms`
- all-current-workload bump Greek breadth: `26` estimates in `216.317 ms`
- specialized Rust terminal-distribution fast path: `0.524 ms`
- native Metal European path on macOS: `1.495 ms`
- native Metal European antithetic path on macOS: `0.985 ms`
- native Metal European control-variate path on macOS: `0.987 ms`
- native Metal arithmetic Asian path on macOS: `0.696 ms`
- native Metal arithmetic Asian control-variate path on macOS: `0.703 ms`
- native Metal down-and-out path on macOS: `0.789 ms`
- native Metal down-and-out control-variate path on macOS: `0.752 ms`
- NumPy fair CPU baseline: `76.321 ms`
- Numba fair CPU baseline: `222.326 ms`
- measured planner choice accuracy vs local backend winners: `87.5%`

Current QMC generation scoreboard from the same release run:

- Rust scrambled Sobol normal generation: `74.437 ms`
- SciPy scrambled Sobol normal generation: `115.477 ms`
- Rust randomized Halton normal generation: `56.383 ms`
- SciPy randomized Halton normal generation: `145.888 ms`
- Rust Latin hypercube normal generation: `39.611 ms`
- SciPy Latin hypercube normal generation: `195.198 ms`

Current QMC pricing-quality and UQ scoreboard from the same release run:

- European scrambled Sobol stderr ratio vs pseudorandom: `1.000`
- arithmetic Asian Latin hypercube stderr ratio vs pseudorandom: `1.003`
- down-and-out randomized Halton stderr ratio vs pseudorandom: `0.994`
- basket Latin hypercube stderr ratio vs pseudorandom: `0.997`
- basket scrambled Sobol stderr ratio vs pseudorandom: `0.996`
- European randomized Halton abs-error ratio vs pseudorandom/Black-Scholes: `0.035`
- European Latin hypercube abs-error ratio vs pseudorandom/Black-Scholes: `0.021`
- European scrambled Sobol abs-error ratio vs pseudorandom/Black-Scholes: `0.129`
- European scrambled Sobol Brownian bridge abs-error ratio vs pseudorandom/Black-Scholes: `0.001`
- Gaussian UQ pseudorandom abs error vs analytic mean: `0.006344`
- Gaussian UQ randomized Halton abs error vs analytic mean: `0.000056`
- Gaussian UQ Latin hypercube abs error vs analytic mean: `0.000039`
- Gaussian UQ scrambled Sobol abs error vs analytic mean: `0.000043`

Current quality ratios from the same release run:

- European control-variate stderr ratio: `0.411`
- European antithetic stderr ratio: `0.747`
- arithmetic Asian control-variate stderr ratio: `0.607`
- arithmetic Asian MLMC stderr ratio: `2.013`
- arithmetic Asian MLQMC stderr ratio: `0.418`
- randomized Halton European control-variate stderr ratio: `0.411`
- Latin hypercube European control-variate stderr ratio: `0.410`
- down-and-out control-variate stderr ratio: `0.418`
- basket randomized Halton stderr ratio vs pseudorandom: `0.996`
- basket Latin hypercube stderr ratio vs pseudorandom: `0.997`
- basket scrambled Sobol stderr ratio vs pseudorandom: `0.996`
- native Metal European control-variate stderr ratio: `0.409`
- native Metal arithmetic Asian control-variate stderr ratio: `0.609`
- native Metal down-and-out control-variate stderr ratio: `0.417`

## Honest Status

What we can honestly claim now:

- CPU performance is strong against the available NumPy and Numba baselines on the tracked fair European workload.
- Native Apple Metal is materially faster than our CPU baseline on the tracked European, arithmetic Asian, and down-and-out workloads.
- The library has better breadth than before, with four option workload families including a two-asset basket call, one non-option Gaussian UQ workload, randomized Halton, Latin hypercube, scrambled Sobol, and Brownian-bridge path construction. Direct QMC normal generation now beats the available SciPy QMC baselines on the tracked Sobol, Halton, and Latin-hypercube rows, and batched path-level filling has materially reduced structured-pricing overhead.
- European QMC now has an analytic realized-error scoreboard against Black-Scholes, so accuracy claims for that workload are no longer limited to standard-error ratios.
- MLMC and MLQMC foundations are live for arithmetic Asian calls with per-level estimator metadata, pilot-based allocation tuning, adaptive tolerance planning, and replicated Sobol scrambling. Adaptive MLMC is now a very fast CPU reference path on the tracked benchmark but needs better default calibration, while replicated MLQMC is faster than Asian step-wise and materially lower-error in the current run.

What we should not overclaim yet:

- structured sampling generation is now competitive and pricing overhead is much lower, but full structured-pricing paths still trail the pseudorandom CPU baseline; realized-error wins are currently benchmark evidence for the European analytic-reference case, not a universal guarantee
- MLMC and MLQMC are CPU-reference only, and their tolerance planning is pilot-estimated rather than broadly calibrated across workload families
- native CUDA execution is not implemented yet
- planner calibration is improving, but `87.5%` measured local accuracy is not broad production-grade backend intelligence yet

## Next Steps

- broaden Metal beyond the current GBM option family
- broaden QMC realized-error studies beyond the first European Black-Scholes reference where analytic or semi-analytic references exist
- calibrate MLMC and MLQMC tolerance planning against realized estimator error across more workloads
- keep calibrating planner recommendations from measured backend winners across more workload classes
- expand competitor matrix to JAX/CuPy/PyTorch where environment allows
- extend CI from CPU validation to native hardware validation once dedicated runners exist
- keep native CUDA launch deferred while CPU, Metal, and multilevel-method quality are the active focus
