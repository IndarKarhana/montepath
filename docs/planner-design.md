# Execution Planner Design

## 1. Purpose

The execution planner transforms a validated `SimulationSpec` and `RunConfig` into an `ExecutionPlan` that can be compiled and executed by a backend.

This subsystem is a primary differentiator of the runtime. It should not be a hidden implementation detail. The planner should make explicit, explainable choices about:

- backend selection
- precision
- memory layout
- chunking
- reduction strategy
- RNG strategy
- compilation decisions

## 2. Planner Responsibilities

The planner must:

- inspect simulation structure
- determine backend feasibility
- estimate cost and memory use
- choose a backend
- generate execution strategies
- produce an explanation report
- fail clearly when constraints cannot be met

The planner must not:

- change simulation semantics silently
- enable approximations without policy support
- obscure backend limitations behind generic errors

## 3. Planner Inputs

### 3.1 Required inputs

- `SimulationSpec`
- `RunConfig`
- available hardware inventory
- backend capability registry

### 3.2 Optional inputs

- cached benchmark data
- historical cost model data
- compile cache state
- user planner overrides

## 4. Planner Outputs

- `ExecutionPlan`
- `BackendDecisionReport`
- `MemoryPlan`
- `RngPlan`
- `ReductionPlan`
- `CompilationPlan`
- structured diagnostics

## 5. Planner Phases

The planner should be implemented as a pipeline.

1. Normalization
2. Feature extraction
3. Feasibility analysis
4. Cost estimation
5. Strategy selection
6. Plan materialization
7. Explanation generation

## 6. Phase 1: Normalization

Responsibilities:

- apply default values
- normalize dtypes and shapes
- resolve runtime axis sizes
- canonicalize expressions and references
- derive implicit metadata such as state lifetime expectations

Outputs:

- normalized simulation graph
- normalized run config

## 7. Phase 2: Feature Extraction

The planner should extract features relevant to execution.

### 7.1 Structural features

- number of random variables
- number of state variables
- number of steps
- number of observations
- number of reductions
- dependency graph depth

### 7.2 Workload features

- path count
- step count
- total element count
- arithmetic operations per path-step
- math function density
- branching density
- reduction intensity

### 7.3 Memory features

- live state footprint
- observation footprint
- reduction workspace footprint
- full-path vs rolling-state requirements

### 7.4 Backend-sensitive features

- expected branch divergence
- expected memory coalescing quality
- transfer sensitivity
- kernel fusion opportunities
- required unsupported operations

## 8. Phase 3: Feasibility Analysis

The planner should query each backend with a support check.

Support result categories:

- `supported`
- `supported_with_fallbacks`
- `unsupported`

Support report fields:

- backend name
- support level
- unsupported features
- soft limitations
- reproducibility tier availability
- precision support
- maximum tested dimensions if relevant

Example reasons for rejection:

- unsupported distribution type on backend
- required `float64` unsupported or too slow by policy
- dynamic control flow outside backend support in v1
- memory estimate exceeds device capacity

## 9. Phase 4: Cost Estimation

The v1 planner should use explicit heuristics plus lightweight analytic estimates.

### 9.1 Cost model components

- setup cost
- compile cost
- data movement cost
- kernel execution cost
- reduction cost
- synchronization cost

### 9.2 CPU estimate factors

- total scalar ops
- vectorization suitability
- branching behavior
- thread-level parallelism
- cache friendliness

### 9.3 NVIDIA estimate factors

- host-device transfers
- occupancy estimate
- arithmetic intensity
- warp divergence estimate
- shared/global memory usage
- kernel launch overhead

### 9.4 Apple Metal estimate factors

- unified memory behavior
- compute pipeline setup overhead
- threadgroup utilization estimate
- divergence estimate
- reduction strategy fit

### 9.5 Cost model output

Each backend should return an estimate object:

- `estimated_compile_ms`
- `estimated_runtime_ms`
- `estimated_total_ms`
- `estimated_peak_memory_mb`
- `confidence`: `low` | `medium` | `high`
- `notes`

## 10. Phase 5: Strategy Selection

The planner chooses:

- backend
- device
- precision
- chunking policy
- storage policy
- reduction implementation
- RNG implementation
- compilation strategy

### 10.1 Backend selection priority order

1. Respect hard constraints
2. Filter unsupported backends
3. Rank remaining backends by planner mode and estimated cost
4. Apply user preferences as tie-breakers or weighted preferences
5. Emit an explanation for the winner and non-selected candidates

### 10.2 Planner modes

#### `safe`

Optimize for:

- deterministic execution
- mature backend behavior
- conservative memory planning
- fewer aggressive transformations

#### `balanced`

Optimize for:

- strong throughput with reliable behavior
- moderate fusion and chunking
- user-friendly defaults

#### `aggressive`

Optimize for:

- throughput first
- stronger fusion
- lower precision when acceptable
- less conservative chunk sizes

#### `explain`

Same as `balanced`, but always produce detailed diagnostics and reasoning traces.

## 11. Backend Selection Heuristics for v1

These are initial rules, not permanent policy.

### 11.1 Choose CPU when

- step count is small enough that accelerator setup overhead dominates
- branching estimate is high
- unsupported GPU features are required
- user requests strongest deterministic behavior
- memory footprint is modest and CPU parallelism is sufficient

### 11.2 Choose NVIDIA when

- CUDA device is available
- workload is large and path-parallel
- arithmetic intensity is medium or high
- branch divergence is low or moderate
- data transfer cost is amortized by total runtime

### 11.3 Choose Apple Metal when

- Apple Silicon GPU is available
- CUDA is not available or not preferred
- workload is path-parallel and within the measured native Metal sweet spot
- operations are within the supported kernel subset
- unified memory reduces transfer concerns

The current measured native Metal sweet spot includes smaller step-wise
workloads than the original conservative heuristic. Release artifacts should
remain the source of truth for recalibrating these thresholds.

## 12. Chunking Strategy

Chunking is required when:

- memory exceeds backend capacity
- planner estimates better utilization with tiled execution
- user memory constraints demand bounded execution

Chunking plan fields:

- chunk axis
- chunk size
- number of chunks
- state carry policy
- reduction merge policy

Preferred chunk axis in v1:

- `path`

Rules:

- chunking must preserve reduction semantics
- chunking must preserve RNG stream independence
- planner must note when chunking can affect exact floating-point summation order

## 13. Memory Planning

The planner should output a `MemoryPlan` with:

- state buffers
- observation buffers
- reduction workspaces
- transfer buffers
- expected peak memory
- reuse opportunities

Memory planning decisions:

- full-path vs rolling storage
- in-place state updates where safe
- reduction staging location
- host vs device residency

## 14. Precision Planning

The planner selects precision based on:

- user constraints
- backend capabilities
- operation mix
- expected numeric sensitivity
- planner mode

Default policy:

- CPU: prefer `float64`
- GPU: prefer `float32` for throughput unless constrained otherwise

The planner should emit warnings when:

- user asked for `float64` on a backend with poor support
- reduced precision could impact accuracy materially
- mixed precision is used in any experimental mode

## 15. RNG Planning

The planner must create a backend-aware RNG strategy.

Fields:

- RNG family
- stream splitting policy
- per-path or per-thread mapping
- QMC vs IID plan
- chunk-stability guarantees

Requirements:

- independent streams across chunks and paths
- deterministic per-backend replay where promised
- clear indication of exact vs statistical reproducibility

## 16. Reduction Planning

The planner must decide how reductions are implemented.

Strategies:

- on-the-fly reduction during simulation
- staged reduction after observation materialization
- hierarchical parallel reduction
- streaming reduction across chunks

Planner inputs:

- reduction op
- output shape
- backend parallel reduction support
- memory footprint of materialized observations

## 17. Compilation Planning

The planner should decide whether to:

- reuse a cached compiled artifact
- specialize kernels to exact shapes and dtypes
- use generic kernels
- fuse common operations

Compilation plan fields:

- specialization key
- cache key
- expected compile time
- fusion opportunities selected
- generated kernel set

## 18. Explanation Engine

Every plan should include a human-readable and machine-readable explanation.

### 18.1 Minimum explanation fields

- selected backend
- selected device
- why this backend was chosen
- why other candidates were rejected or ranked lower
- precision choice and rationale
- chunking choice and rationale
- notable bottlenecks
- warnings

### 18.2 Example machine-readable explanation

```json
{
  "selected_backend": "nvidia_cuda",
  "selected_device": "RTX_4090",
  "reasons": [
    "Large path count favors GPU throughput",
    "Branch divergence estimate is low",
    "Estimated peak device memory fits within available memory"
  ],
  "rejected_backends": [
    {
      "backend": "cpu_native",
      "reason": "Estimated runtime 5.3x slower"
    },
    {
      "backend": "apple_metal",
      "reason": "No compatible Apple GPU detected"
    }
  ],
  "precision": "float32",
  "chunking": {"enabled": true, "axis": "path", "chunk_size": 125000},
  "warnings": [
    "Reduction order differs from single-pass CPU baseline; bitwise identity is not guaranteed"
  ]
}
```

## 19. Planner Diagnostics

Planner diagnostics should include codes and structured locations.

Examples:

- `P_BACKEND_UNSUPPORTED`
- `P_MEMORY_EXCEEDED`
- `P_PRECISION_DEGRADED`
- `P_CHUNKING_REQUIRED`
- `P_REPRO_TIER_UNAVAILABLE`

## 20. Caching Strategy

The planner should produce stable cache keys based on:

- backend
- device family if relevant
- simulation structure hash
- dtype choices
- runtime shape specializations
- planner mode where it affects lowering

Cache goals:

- avoid recompiling identical workloads
- preserve correctness under schema changes
- invalidate cleanly on backend version changes

## 21. Empirical Feedback Loop

The planner now has a first empirical feedback loop through committed benchmark
artifacts and Python planner-intelligence surfaces.

Current and future sources:

- actual runtime vs estimated runtime
- actual peak memory vs estimated peak memory
- compile time history
- backend-specific performance fingerprints

We should design the planner so heuristics can later be augmented by measured data without changing the public contract.

Current surfaces:

- `load_planner_evidence()`
- `measured_winner_database()`
- `cost_frontier(workload)`
- `compare_methods(workload)`
- `why_not_faster(workload, method_id=...)`
- `mlmc_error_calibration(workload)`

## 22. Planner Pseudocode

```text
normalize(spec, run_config)
extract_features(normalized)
collect_devices()
for backend in registered_backends:
    support = backend.supports(normalized)
    if support is unsupported:
        record rejection
        continue
    cost = backend.estimate_cost(normalized, devices, run_config)
    candidate_plans.append((backend, support, cost))
rank candidates according to planner mode and constraints
select winner
build memory plan
build rng plan
build reduction plan
build compilation plan
emit execution plan + explanation
```

## 23. Recommended v1 Constraints

To keep implementation realistic:

- planner heuristics should be simple, explicit, and testable
- no opaque AI planner for runtime decisions
- no distributed planning in v1
- only one selected backend per run in v1
- CPU fallback should always be considered where possible

## 24. Open Questions

1. How detailed should branch divergence estimation be in v1?
2. Should planner explanations be stable for snapshot testing?
3. How aggressively should kernel fusion be attempted before benchmarks exist?
4. Should the planner be allowed to suggest alternative simulation encodings?
5. How much device-specific tuning should happen before we have benchmark evidence?

## 25. Recommendation

The v1 planner should be:

- heuristic-driven
- explainable
- conservative where correctness matters
- structured so empirical tuning can be layered in later

That gives us automation without hiding important tradeoffs.
