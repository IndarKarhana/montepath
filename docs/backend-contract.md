# Backend Contract

## 1. Purpose

This document defines the interface and responsibilities of runtime backends.

A backend is the execution engine for a target platform. In v1, the planned backends are:

- `cpu_native`
- `nvidia_cuda`
- `apple_metal`

The backend contract exists to:

- isolate backend-specific implementation details
- allow the planner to compare backends consistently
- keep the frontend and schema independent from device-specific logic
- make future backends possible without redesigning the system

## 2. Design Principles

1. Stable planner-facing API

Backends must present a uniform capability and cost interface.

2. Internal specialization is allowed

Backends may optimize aggressively internally, as long as public semantics are preserved.

3. Explicit capability reporting

Backends should not hide unsupported features behind runtime crashes.

4. Clear reproducibility guarantees

Each backend must declare what level of determinism and reproducibility it can provide.

## 3. Backend Lifecycle

Each backend participates in the following lifecycle:

1. registration
2. device discovery
3. support analysis
4. cost estimation
5. compilation / lowering
6. execution
7. telemetry emission

## 4. Core Backend Interface

Suggested logical interface:

- `backend_id() -> BackendId`
- `describe_backend() -> BackendInfo`
- `discover_devices() -> Vec<DeviceInfo>`
- `supports(spec, run_config, device) -> SupportReport`
- `estimate_cost(spec, run_config, device) -> CostEstimate`
- `compile(plan, device) -> CompiledArtifact`
- `execute(artifact, inputs, run_config) -> RunOutput`
- `reproducibility_capabilities(device) -> ReproSupport`
- `teardown(artifact) -> Result<()>`

The exact Rust trait structure can evolve, but the logical responsibilities should remain.

## 5. `BackendInfo`

Fields:

- `backend_id`
- `display_name`
- `version`
- `platform`
- `supported_precisions`
- `supported_rngs`
- `supported_sampling_modes`
- `supported_reduction_ops`
- `notes`

Example:

```json
{
  "backend_id": "nvidia_cuda",
  "display_name": "NVIDIA CUDA",
  "version": "0.1.0",
  "platform": "cuda",
  "supported_precisions": ["float32", "float64"],
  "supported_rngs": ["philox", "sobol"],
  "supported_sampling_modes": ["iid", "qmc"],
  "supported_reduction_ops": ["sum", "mean", "variance", "min", "max"]
}
```

## 6. `DeviceInfo`

Fields:

- `device_id`
- `backend_id`
- `name`
- `vendor`
- `memory_total_mb`
- `memory_free_mb`
- `compute_capability` or equivalent
- `supports_float64`
- `supports_unified_memory`
- `max_threads_per_group` or equivalent
- `driver_info`
- `availability_status`

Rules:

- devices should be discovered lazily where expensive
- device info should be serializable into the run manifest

## 7. `SupportReport`

Purpose:

Tell the planner whether a backend can run a workload and under what caveats.

Fields:

- `backend_id`
- `device_id`
- `support_level`: `supported` | `supported_with_fallbacks` | `unsupported`
- `unsupported_features`
- `warnings`
- `max_reproducibility_tier`
- `supported_precisions`
- `notes`

Examples of unsupported features:

- unsupported distribution
- unsupported reduction op
- unsupported expression function
- unsupported dynamic axis behavior

## 8. `CostEstimate`

Fields:

- `backend_id`
- `device_id`
- `estimated_compile_ms`
- `estimated_runtime_ms`
- `estimated_total_ms`
- `estimated_peak_memory_mb`
- `confidence`
- `breakdown`
- `notes`

`breakdown` may include:

- setup
- transfer
- compute
- reduction
- synchronization

## 9. `CompiledArtifact`

A compiled artifact is the backend-specific executable form of an `ExecutionPlan`.

Fields:

- `artifact_id`
- `backend_id`
- `device_id`
- `cache_key`
- `compiled_kernels`
- `specialization_signature`
- `metadata`

Rules:

- artifacts are opaque to the frontend
- artifacts may be cached and reused if compatible
- artifacts must record enough metadata to validate reuse safely

## 10. `RunOutput`

Fields:

- `outputs`
- `telemetry`
- `diagnostics`
- `manifest_fragment`

The backend should return execution-specific telemetry such as:

- measured runtime
- measured peak memory if available
- compile time
- kernel timings if profiling enabled
- actual chunk count

## 11. `ReproSupport`

Backends must declare their reproducibility behavior clearly.

Fields:

- `supports_same_backend_exact`
- `supports_same_backend_deterministic`
- `supports_cross_backend_statistical`
- `supports_stable_chunking`
- `notes`

This object should explain limitations such as:

- non-associative reduction ordering
- device-dependent math implementation differences
- unsupported exact replay across driver versions

## 12. Backend Responsibilities

Every backend must handle:

- memory allocation and deallocation
- RNG initialization and stream partitioning
- kernel or loop execution
- reduction execution
- error capture and diagnostics
- telemetry capture

Backends should not:

- reinterpret simulation semantics
- apply hidden approximations unless the plan explicitly requested them
- silently fall back to another backend during execution

## 13. CPU Backend Expectations

`cpu_native` should serve as:

- reference backend for correctness
- baseline for planner fallback
- likely most mature reproducibility backend in v1

Special expectations:

- robust validation support
- clear debugging paths
- multi-threading via native runtime primitives such as rayon
- strong support for `float64`

## 14. NVIDIA Backend Expectations

`nvidia_cuda` should provide:

- high-throughput execution for large path-parallel workloads
- CUDA-native kernels and reductions
- explicit device memory management
- detailed telemetry where available

Special concerns:

- warp divergence
- transfer overhead
- kernel launch overhead
- compatibility with selected CUDA toolchains

## 15. Apple Backend Expectations

`apple_metal` should provide:

- Metal compute execution on Apple Silicon
- unified-memory-aware planning cooperation
- support for the same narrow v1 workload family as CUDA where practical

Special concerns:

- different compute model and tooling vs CUDA
- potentially different optimal tiling and reduction strategies
- avoiding lowest-common-denominator abstraction leakage

## 16. Error Model

Backends should return structured errors with codes.

Suggested categories:

- device discovery errors
- unsupported feature errors
- compilation errors
- execution errors
- memory allocation errors
- reproducibility guarantee errors

Example codes:

- `B_DEVICE_UNAVAILABLE`
- `B_UNSUPPORTED_DTYPE`
- `B_COMPILE_FAILED`
- `B_OUT_OF_MEMORY`
- `B_EXECUTION_FAILED`

## 17. Capability Registry

The runtime should maintain a backend registry.

Responsibilities:

- register available backends
- enumerate devices
- expose backend metadata to the planner
- support testing with mock backends

This registry helps us keep the planner and runtime decoupled from concrete backend implementations.

## 18. Capability Matrix

We should maintain a capability matrix per backend.

Dimensions:

- dtypes
- distributions
- math functions
- reductions
- sampling modes
- reproducibility tiers
- profiling support

This matrix should be queryable programmatically.

## 19. Compilation Contract

A backend compile step receives an `ExecutionPlan` and must either:

- produce a `CompiledArtifact`, or
- return structured diagnostics explaining why compilation cannot proceed

Compilation responsibilities:

- lower IR fragments to backend kernels or loops
- bind specialization parameters
- build reduction pipelines
- allocate static workspaces if needed
- emit cacheable artifacts where appropriate

## 20. Execution Contract

The execute step should:

- validate input compatibility with the compiled artifact
- prepare runtime buffers
- initialize RNG streams
- run simulation kernels / loops
- run reductions
- collect telemetry
- return outputs and diagnostics

Execution must not mutate the original `SimulationSpec` or `ExecutionPlan`.

## 21. Profiling Contract

If profiling is enabled, a backend may emit:

- kernel timings
- occupancy hints
- memory transfer timings
- reduction timings
- backend-specific performance notes

Profiling output should be optional and structured.

## 22. Testing Requirements for Backends

Each backend should pass:

- capability reporting tests
- support analysis tests
- compile success / failure tests
- execution correctness tests
- reproducibility tests
- error handling tests

Cross-backend tests should compare backend outputs against CPU reference behavior within defined tolerances.

## 23. Mock Backend Support

To make planner and orchestration testing easier, we should support mock backends that implement the contract but do not execute real kernels.

Use cases:

- planner tests
- support-report tests
- error-path tests
- deterministic CI without GPUs

## 24. Versioning and Compatibility

Backends may evolve independently, but the runtime should define a compatibility contract.

Recommendations:

- version the backend interface explicitly
- reject incompatible compiled artifact cache entries
- include backend version in run manifests

## 25. Open Questions

1. Should `compile()` be allowed to require device presence, or can we support offline compilation for some backends later?
2. How much of kernel specialization should be owned by the backend vs the planner?
3. Should profiling be a separate trait / capability rather than part of the main backend contract?
4. How much backend-specific telemetry should be normalized vs passed through as raw extension data?

## 26. Recommendation

Keep the backend contract:

- planner-friendly
- explicit about capabilities
- strict about semantics
- flexible internally for platform-specific optimization

That will let us scale backend support without destabilizing the rest of the system.
