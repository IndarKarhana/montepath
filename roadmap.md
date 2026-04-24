# Roadmap

This roadmap is a living document and must be updated with every meaningful scope or status change.

## Status Legend

- `todo`
- `in-progress`
- `done`
- `blocked`

## Phase 0: Foundation and Governance

- `done` Create architecture, schema, planner, backend contract, and benchmark docs.
- `done` Establish repository engineering rules and development workflow.
- `done` Initialize production-grade project scaffolding with tests-first conventions.
- `done` Install Rust toolchain (`cargo` and `rustc`) and run baseline test pipeline.

## Phase 1: Schema and Validation Core

- `done` Build `mc-schema` crate with typed schema objects.
- `done` Build schema validator with structured diagnostics.
- `done` Add schema serialization round-trip tests.
- `done` Add compatibility/versioning checks for schema evolution.

## Phase 2: Planner Skeleton

- `done` Build planner interfaces and initial normalization pipeline.
- `done` Implement feature extraction from `SimulationSpec`.
- `done` Add backend feasibility and heuristic selection stubs.
- `done` Emit `ExecutionPlan` and explainability report skeleton.

## Phase 3: CPU Runtime (Reference Backend)

- `done` Implement `cpu_native` backend contract.
- `done` Add deterministic RNG stream mapping.
- `done` Implement baseline execution loop for path/step simulation (European call workload).
- `done` Implement baseline reductions (mean and standard error for payoff).
- `done` Validate numeric correctness against benchmark fixtures (analytic Black-Scholes check).
- `done` Add explicit general CPU step-wise execution path separate from specialized terminal-distribution fast path.
- `done` Add antithetic-variates support for the current CPU European-call runtime.
- `done` Add control-variate support for narrow workloads with strong analytic references.

## Phase 4: NVIDIA Runtime

- `done` Implement CUDA backend contract and device discovery.
- `done` Add truthful delegated fallback execution path for CUDA backend while native kernels are in progress.
- `done` Add host-side CUDA native staging boundary and feature gate.
- `done` Add shared GPU launch and buffer contracts for staged native kernels.
- `done` Add first staged CUDA kernel source and PTX compile-attempt path for the core workload.
- `todo` Implement native CUDA launch and reduction path for the staged kernel.
- `done` Add GPU memory and chunking strategy.
- `done` Add planner heuristics for CUDA selection.

## Phase 5: Apple Runtime

- `done` Implement Apple Metal backend contract.
- `done` Add truthful delegated fallback execution path for Apple Metal backend while native kernels are in progress.
- `done` Add host-side Metal native staging boundary and feature gate.
- `done` Add first staged Metal shader source and `.air` / `.metallib` compile-attempt path for the core workload.
- `done` Implement first native Metal launch path for the staged kernel on macOS using runtime compilation.
- `done` Add CPU-vs-native-Metal benchmark coverage on macOS.
- `done` Move first Metal-native RNG generation and full staged reductions on-device.
- `done` Remove helper-based execution overhead via persistent in-process native host integration.
- `done` Add benchmark-calibrated planner heuristics for Apple backend selection.
- `done` Extend native Metal execution across the first European-call step-wise technique family (`Standard`, `Antithetic`, `ControlVariate`).
- `done` Extend native Metal execution to a second workload family with arithmetic Asian calls and control-variate support.

## Phase 6: Benchmarks and Tuning

- `done` Implement benchmark harness and result schema.
- `in-progress` Add baseline comparisons against NumPy / Numba / JAX / CuPy where relevant.
- `done` Add automated CPU competitor baselines for NumPy and Numba.
- `done` Add explicit availability reporting for JAX / CuPy / PyTorch in benchmark output.
- `done` Auto-generate competitiveness improvement plan when running benchmarks.
- `done` Track planner decision quality via planner-choice accuracy benchmark.
- `done` Define performance gates and regression thresholds.
- `done` Add release-profile benchmark output for stronger competitiveness tracking.
- `done` Add competitiveness gate checks (Rust faster than available NumPy/Numba CPU baselines).
- `done` Split European-call benchmarks into fair terminal-distribution and true step-wise benchmark families.
- `done` Add benchmark methodology metadata so specialized fast paths are not confused with general-runtime comparisons.
- `done` Add internal antithetic-quality benchmarking via stderr-ratio tracking.
- `done` Add internal control-variate-quality benchmarking via stderr-ratio tracking.
- `in-progress` Calibrate planner choice quality against measured backend winners, not only hand-authored expected scenarios.
- `done` Add arithmetic Asian CPU and Apple Metal benchmark coverage.

## Phase 8: Advanced Simulation Techniques

- `todo` Add scrambled Sobol / randomized quasi-Monte Carlo sampling.
- `todo` Add Latin hypercube sampling for uncertainty propagation and parameter sweeps.
- `todo` Add multilevel Monte Carlo foundations for discretized path simulation.
- `todo` Add multilevel randomized quasi-Monte Carlo after MLMC and RQMC foundations are stable.

## Phase 7: Agent Experience and Integration

- `done` Add project-level `AGENTS.md` instructions for repo-native agent workflows.
- `done` Add Codex project skills for architecture and agent-surface discipline.
- `done` Add a function catalog for public and future tool-facing surfaces.
- `done` Add an agent integration plan for future tool/plugin wrapping.
- `todo` Add machine-readable tool manifest and schema export for stable agent integration.
- `in-progress` Add explain-plan and run-manifest helpers as first-class agent-facing surfaces.
- `todo` Add Python-facing agent wrappers that preserve typed, explainable contracts.

## Ongoing Engineering Quality Track

- `in-progress` Test-driven development as default coding workflow.
- `in-progress` Production-grade code quality and reliability standards.
- `in-progress` Keep runtime lightweight, minimal overhead, and dependency-conscious.
- `done` Research user-friendliness patterns from leading libraries and implement first UX improvements.
- `done` Add an honest market-landscape document against leading Monte Carlo library categories.
- `done` Add baseline CPU CI for format, test, and benchmark smoke checks.
- `done` Validate feature-gated native backend staging in CPU-only CI.
- `todo` Add native CUDA and Metal hardware CI on dedicated runners.
