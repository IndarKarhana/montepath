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
- `blocked` Run Rust test/lint pipeline in this environment until Rust toolchain (`cargo`) is available.

## Phase 1: Schema and Validation Core

- `done` Build `mc-schema` crate with typed schema objects.
- `in-progress` Build schema validator with structured diagnostics.
- `todo` Add schema serialization round-trip tests.
- `todo` Add compatibility/versioning checks for schema evolution.

## Phase 2: Planner Skeleton

- `todo` Build planner interfaces and initial normalization pipeline.
- `todo` Implement feature extraction from `SimulationSpec`.
- `todo` Add backend feasibility and heuristic selection stubs.
- `todo` Emit `ExecutionPlan` and explainability report skeleton.

## Phase 3: CPU Runtime (Reference Backend)

- `todo` Implement `cpu_native` backend contract.
- `todo` Add deterministic RNG stream mapping.
- `todo` Implement baseline execution loop for path/step simulation.
- `todo` Implement baseline reductions.
- `todo` Validate numeric correctness against benchmark fixtures.

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

- `todo` Implement benchmark harness and result schema.
- `todo` Add baseline comparisons against NumPy / Numba / JAX / CuPy where relevant.
- `todo` Track planner decision quality and cost-model error.
- `todo` Define performance gates and regression thresholds.

## Ongoing Engineering Quality Track

- `in-progress` Test-driven development as default coding workflow.
- `in-progress` Production-grade code quality and reliability standards.
- `in-progress` Keep runtime lightweight, minimal overhead, and dependency-conscious.
- `todo` Add CI for lint, test, and formatting checks.
