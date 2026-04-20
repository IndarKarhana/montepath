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

- `todo` Implement `cpu_native` backend contract.
- `done` Add deterministic RNG stream mapping.
- `done` Implement baseline execution loop for path/step simulation (European call workload).
- `done` Implement baseline reductions (mean and standard error for payoff).
- `in-progress` Validate numeric correctness against benchmark fixtures.

## Phase 4: NVIDIA Runtime

- `todo` Implement CUDA backend contract and device discovery.
- `todo` Implement first CUDA kernels for core workload path.
- `todo` Add GPU memory and chunking strategy.
- `todo` Add planner heuristics for CUDA selection.

## Phase 5: Apple Runtime

- `todo` Implement Apple Metal backend contract.
- `todo` Implement first Metal compute kernels for core workload path.
- `todo` Add planner heuristics for Apple backend selection.

## Phase 6: Benchmarks and Tuning

- `done` Implement benchmark harness and result schema.
- `in-progress` Add baseline comparisons against NumPy / Numba / JAX / CuPy where relevant.
- `done` Add automated CPU competitor baselines for NumPy and Numba.
- `done` Add explicit availability reporting for JAX / CuPy / PyTorch in benchmark output.
- `done` Auto-generate competitiveness improvement plan when running benchmarks.
- `done` Track planner decision quality via planner-choice accuracy benchmark.
- `done` Define performance gates and regression thresholds.

## Ongoing Engineering Quality Track

- `in-progress` Test-driven development as default coding workflow.
- `in-progress` Production-grade code quality and reliability standards.
- `in-progress` Keep runtime lightweight, minimal overhead, and dependency-conscious.
- `done` Research user-friendliness patterns from leading libraries and implement first UX improvements.
- `todo` Add CI for lint, test, and formatting checks.
