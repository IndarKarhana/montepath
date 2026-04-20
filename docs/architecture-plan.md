# Monte Carlo Runtime Architecture Plan

## 1. Vision

Build an agent-native Monte Carlo runtime with:

- a Python-first developer experience
- a compiled execution core
- automatic CPU / NVIDIA GPU / Apple MPS backend selection
- reproducible stochastic execution
- structured metadata that humans and AI agents can inspect, optimize, and explain

The product is not just a math library. It is a simulation runtime that accepts a structured simulation description, builds an execution plan, chooses an execution backend, runs the workload efficiently, and returns both simulation results and an explanation of how the run was executed.

## 2. Product Thesis

Current Monte Carlo workflows are often split across several layers:

- model definition in Python or notebooks
- random sampling in NumPy, SciPy, PyTorch, JAX, or custom code
- ad hoc performance tuning by hand
- separate CPU and GPU implementations
- limited reproducibility and poor explainability around runtime choices

This project should compete on four axes together:

- performance
- usability
- reproducibility
- agent-readability

The differentiator is not just speed. The differentiator is a runtime that can understand a simulation structurally and make execution decisions explicitly.

## 3. Initial Scope

### In scope for v1

- Monte Carlo simulation for forward simulation and uncertainty propagation
- scalar and vector outputs
- repeated independent trials and batched execution
- CPU backend
- NVIDIA GPU backend
- Apple GPU backend via MPS / Metal path
- pseudorandom and quasi-random sampling
- basic variance reduction
- backend auto-selection with explainable decisions
- deterministic reproducibility controls
- Python bindings and typed API
- structured run reports for human and agent consumption

### Out of scope for v1

- full probabilistic programming language replacement
- advanced MCMC framework
- distributed multi-node execution
- AMD GPU support
- TPU support
- symbolic algebra system
- arbitrary Python control flow compiled automatically to every backend

The v1 should be a well-bounded simulation runtime, not an attempt to replace the whole scientific stack.

## 4. Design Principles

1. Structured over implicit

The runtime should accept an explicit simulation representation rather than opaque callback-heavy user code wherever possible.

2. Explainable automation

Backend selection, precision choice, batching strategy, and variance reduction decisions should be visible and overridable.

3. Portable core, specialized backends

The execution core should expose a stable intermediate representation and backend contract. Each backend can optimize aggressively without leaking backend-specific complexity into the user API.

4. Reproducibility by default

Every run should capture seeds, RNG family, backend, precision, planner decisions, software version, and device metadata.

5. Narrow fast path first

Design the v1 around workloads we can optimize well:

- independent path simulations
- batched parameter sweeps
- common distributions
- reduction-heavy metrics
- moderate branching, but not fully unstructured agent-based simulation

6. Agent-native metadata

Every simulation, compiled plan, run artifact, and optimization hint should be machine-readable.

## 5. Primary Users

- Python developers doing Monte Carlo simulation
- quants and risk engineers
- scientific computing teams doing uncertainty propagation
- reliability and operations researchers
- AI agents generating, tuning, and explaining simulations on behalf of users

## 6. High-Level Architecture

The system should be layered as follows:

1. Frontend API layer
2. Simulation IR layer
3. Planner and optimizer layer
4. Backend runtime layer
5. Execution and reporting layer
6. Validation and benchmarking layer

### 6.1 Frontend API layer

Responsibilities:

- user-facing Python API
- typed schema for simulation definitions
- validation and ergonomic defaults
- conversion from user code to internal IR
- optional agent-facing JSON schema

Suggested API styles:

- imperative builder API
- declarative schema / config API
- lightweight DSL for common simulation patterns

Example conceptual API:

```python
sim = mc.Simulation("barrier_option") \
    .parameters(S0=100.0, sigma=0.2, r=0.03, T=1.0) \
    .random("z", dist="normal", shape=(n_steps,)) \
    .state("price", init="S0") \
    .step("price", expr="price * exp((r - 0.5*sigma**2)*dt + sigma*sqrt(dt)*z_t)") \
    .observe("payoff", expr="max(price - K, 0.0)") \
    .reduce(mean="mean(payoff)", stderr="std(payoff)/sqrt(n_paths)")

result = sim.run(paths=1_000_000, backend="auto")
```

The exact syntax can evolve. The key point is that the simulation structure must be inspectable.

### 6.2 Simulation IR layer

The IR is the heart of the architecture. It should capture simulation meaning without binding to one backend.

Core IR objects:

- `SimulationSpec`
- `ParameterSpec`
- `RandomVarSpec`
- `StateSpec`
- `StepSpec`
- `ObservationSpec`
- `ReductionSpec`
- `ExecutionConstraints`
- `ReproducibilitySpec`

The IR must support:

- static typing and shape inference
- dependency graph construction
- time-step and path dimension semantics
- reduction semantics
- memory lifetime analysis
- side-effect-free execution semantics on the hot path

Suggested internal structure:

- DAG of expressions and state transitions
- explicit iteration axes:
  - path axis
  - step axis
  - batch axis
  - parameter-set axis
- annotations for parallelizability
- annotations for RNG usage
- optional hints for precision and memory locality

### 6.3 Planner and optimizer layer

Responsibilities:

- inspect simulation IR
- estimate execution cost
- choose backend
- choose batching strategy
- choose memory layout
- choose kernel fusion opportunities
- choose variance reduction defaults when safe
- emit a compiled execution plan

Planner outputs:

- `ExecutionPlan`
- `BackendDecisionReport`
- `MemoryPlan`
- `RngPlan`
- `ReductionPlan`
- `CompilationArtifacts`

The planner should be explicit, not magical. It should produce a report like:

- selected backend: `nvidia_cuda`
- reason: large path count, low branch divergence, sufficient device memory
- precision: `float32`
- RNG: `philox_4x32_10`
- batching: 8 chunks of 125,000 paths
- expected transfer overhead: low
- override available: yes

### 6.4 Backend runtime layer

Each backend implements a common interface.

Backend contract:

- compile or lower IR to executable kernels / loops
- allocate and manage memory
- initialize RNG streams
- execute simulation kernels
- perform reductions
- return results and telemetry

Backends in initial plan:

- `cpu_native`
- `nvidia_cuda`
- `apple_metal`

### 6.5 Execution and reporting layer

Responsibilities:

- run orchestration
- metrics collection
- deterministic run manifests
- result packaging
- planner explanation output
- warnings and optimization suggestions

Standard result object should include:

- outputs
- confidence intervals or standard errors where applicable
- backend used
- run timings
- compilation timings
- memory usage estimate and observed peak if available
- seed and RNG metadata
- planner explanation
- warnings

### 6.6 Validation and benchmarking layer

Responsibilities:

- numerical consistency checks across backends
- reproducibility checks
- microbenchmarks
- end-to-end benchmarks
- regression performance tracking

This should be a first-class subsystem, not an afterthought.

## 7. Language and Runtime Choices

## 7.1 Recommended implementation split

- Core runtime: Rust
- Python bindings: PyO3 / maturin
- NVIDIA GPU kernels: CUDA C++ or Rust CUDA path where practical, with a stable FFI boundary
- Apple GPU kernels: Metal Shading Language via Metal API bindings
- Optional portable C ABI for future non-Python bindings

## 7.2 Why Rust for the core

Rust is a strong fit for:

- safety in concurrency and memory management
- deterministic systems programming
- clean API boundaries
- building a stable IR and planner
- maintainability as backend complexity grows

## 7.3 Why not pure C for the whole system

Pure C would maximize portability but slow down development, reduce safety, and make complex planner and IR code harder to evolve cleanly.

## 7.4 GPU language reality

A single language for all GPU targets is not realistic if we want strong performance and platform-native behavior.

Most practical approach:

- Rust owns orchestration, planning, memory metadata, and CPU execution
- CUDA handles NVIDIA kernels
- Metal handles Apple GPU kernels
- shared IR and backend contracts keep the system coherent

This is less elegant than one-language purity, but much more realistic.

## 8. Backend Strategy

## 8.1 CPU backend

Purpose:

- correctness reference backend
- fallback for unsupported workloads
- best choice for small problems and branch-heavy simulations
- likely easiest path for early optimization

Implementation plan:

- native Rust loops and kernels
- rayon for multi-threading
- SIMD via Rust and compiler auto-vectorization where possible
- memory layouts optimized for batch reductions

CPU strengths:

- low launch overhead
- easier debugging
- better for irregular branching
- simpler reproducibility controls

CPU limitations:

- lower throughput on large dense path simulations
- weaker scaling for very wide vectorized workloads compared to GPUs

## 8.2 NVIDIA backend

Purpose:

- high-throughput execution for large, dense, embarrassingly parallel workloads
- flagship acceleration backend in v1

Implementation plan:

- CUDA kernels for sampling, state updates, and reductions
- explicit host-device memory planning
- kernel fusion for common simulation shapes
- device-side RNG streams

Initial assumptions:

- support modern NVIDIA GPUs with CUDA toolkit support
- start with single-GPU execution
- avoid distributed GPU complexity in v1

NVIDIA strengths:

- best ecosystem for general-purpose GPU compute
- strong libraries and debugging tooling
- ideal for path-parallel workloads

NVIDIA limitations:

- transfer overhead for smaller tasks
- branch divergence penalties
- maintenance burden of CUDA-specific kernels

## 8.3 Apple MPS / Metal backend

Important note: Apple MPS is primarily exposed through higher-level frameworks, but for a systems runtime we should treat Metal as the actual backend substrate and optionally expose MPS-backed conveniences where useful.

Recommended design choice:

- name the backend `apple_metal`
- support Apple Silicon GPUs via Metal compute
- use MPS only where it provides a practical advantage, not as the sole abstraction

Purpose:

- first-class Apple Silicon local acceleration
- developer-friendly laptop and workstation support

Implementation plan:

- Metal compute shaders for sampling and core simulation kernels
- runtime compilation or packaged shader library
- unified memory-aware planning on Apple Silicon

Apple backend strengths:

- great local acceleration for Mac users
- unified memory reduces some transfer pain
- strong developer story for local experimentation

Apple backend limitations:

- more limited mature scientific tooling than CUDA
- lower priority for exotic kernel features in v1
- cross-platform abstraction must avoid assuming CUDA semantics

## 9. Backend Abstraction Contract

Define a backend trait / interface like:

- `supports(spec) -> SupportReport`
- `estimate_cost(spec, device, run_config) -> CostEstimate`
- `compile(plan) -> CompiledArtifact`
- `execute(artifact, inputs, run_config) -> RunOutput`
- `describe_device() -> DeviceInfo`
- `reproducibility_capabilities() -> ReproSupport`

The planner uses these methods to compare viable backends.

This interface is critical because it lets us:

- add future backends without rewriting the frontend
- benchmark backend behavior systematically
- keep backend-specific complexity isolated

## 10. Execution Planner Design

The execution planner is the project’s most important differentiator.

### 10.1 Planner inputs

- simulation IR
- run size
- output requirements
- available devices
- reproducibility requirements
- precision constraints
- user overrides

### 10.2 Planner decision factors

- total path count
- number of time steps
- arithmetic intensity
- branch divergence estimate
- memory footprint
- reduction shape
- host-device transfer cost
- device availability and occupancy estimate
- startup / compile overhead
- backend-specific unsupported features

### 10.3 Planner outputs

- backend choice
- precision choice
- chunk size
- memory layout
- reduction strategy
- RNG family and stream layout
- estimated runtime and memory
- explanation report

### 10.4 Planner modes

- `safe`: favor correctness and reproducibility
- `balanced`: default mode
- `aggressive`: favor throughput and fusion
- `explain`: emit detailed reasoning and suggestions

### 10.5 Auto-selection policy for v1

Start with explicit heuristics, not ML.

Examples:

- choose CPU for small path count or branch-heavy workloads
- choose NVIDIA GPU for large dense workloads when CUDA device is available
- choose Apple Metal for large dense workloads on Apple Silicon when no NVIDIA is available
- force CPU if the workload uses unsupported dynamic features

Later, evolve to empirical cost models using benchmark data.

## 11. Simulation Model Representation

To be agent-friendly, the library needs a stable machine-readable schema.

Recommended top-level objects:

- `SimulationSpec`
- `RunConfig`
- `ExecutionPlan`
- `RunManifest`
- `ResultBundle`

### 11.1 Example `SimulationSpec` fields

- `name`
- `version`
- `parameters`
- `random_variables`
- `state_variables`
- `steps`
- `observations`
- `reductions`
- `constraints`
- `metadata`

### 11.2 Example `RunConfig` fields

- `n_paths`
- `n_steps`
- `backend`
- `planner_mode`
- `precision`
- `seed`
- `rng`
- `max_memory_mb`
- `device_preference`

### 11.3 Example agent-facing capabilities

An agent should be able to ask:

- what state depends on randomness?
- what dimensions are parallelizable?
- can this run on GPU?
- what backend was chosen and why?
- what is the estimated memory footprint?
- what variance reduction methods are applicable?
- what performance bottlenecks were detected?

## 12. RNG Architecture

RNG is foundational and must be backend-aware.

Requirements:

- reproducible seeded runs
- independent streams across paths and batches
- support for CPU and GPU execution
- stable semantics exposed to users

Recommended families:

- CPU pseudorandom: `PCG64` or `Xoshiro` family for host-side convenience
- cross-backend parallel-friendly engine: `Philox` as the primary parallel RNG
- quasi-random support: Sobol first, Halton second

Design recommendation:

- define an abstract RNG contract at IR level
- lower to backend-specific implementations with equivalent semantics where possible
- mark when exact bitwise equivalence across backends is guaranteed vs statistical equivalence only

This matters because exact cross-backend reproducibility is much harder than per-backend deterministic reproducibility.

## 13. Precision Policy

Default precision policy should be explicit.

Recommended v1 policy:

- CPU default: `float64`
- GPU default: `float32` for throughput-sensitive workloads unless the user requests `float64`
- planner warns when precision choice can materially affect accuracy

Support modes:

- strict numeric mode
- throughput mode
- mixed mode for specific kernels where safe

Do not hide precision decisions from users.

## 14. Memory Model

Memory behavior should be visible and planned.

### 14.1 Core memory design

- favor structure-of-arrays layouts for vectorized and GPU execution
- support streaming / chunked execution for runs that exceed device memory
- keep intermediate allocations explicit in the memory plan

### 14.2 Apple Silicon note

Apple unified memory changes the transfer cost profile, but it does not remove the need for a planner. Kernel launch and memory pressure still matter.

### 14.3 Reduction strategy

- local partial reductions per thread block or tile
- global reduction pass
- streaming reduction for oversized workloads

## 15. Variance Reduction Strategy

Variance reduction is one of the easiest places to beat naive competitors.

Recommended v1 methods:

- antithetic variates
- control variates for known common patterns
- stratified sampling
- Sobol quasi-Monte Carlo

Planner role:

- suggest applicable methods
- optionally enable safe defaults where semantics are standard
- include effect in explanation report

Do not auto-enable advanced variance reduction when it changes semantics in a non-obvious way.

## 16. Agent-Friendly Features

This area should be designed intentionally, not bolted on.

### 16.1 Structured explainability

Every run should produce:

- why this backend was chosen
- why this precision was chosen
- why chunking was applied
- what bottlenecks were estimated
- what future optimizations are suggested

### 16.2 Introspection APIs

Add methods like:

- `sim.describe()`
- `sim.explain_plan()`
- `sim.supported_backends()`
- `sim.performance_hints()`
- `result.manifest()`

### 16.3 JSON serialization

Support round-trippable JSON or MessagePack for:

- simulation spec
- run config
- execution plan
- result manifest

This is important for agent workflows, remote orchestration, and auditability.

## 17. Packaging and API Surface

### 17.1 Initial package layout

- `mc_runtime` Python package
- `mc-core` Rust crate
- `mc-cuda` backend module
- `mc-metal` backend module
- `mc-schema` shared schema definitions
- `mc-bench` benchmark suite

### 17.2 Python API priorities

- easy install on CPU-only machines
- optional extras for GPU support
- typed models and good docs
- simple first-run experience

Suggested install shape:

- `pip install mc-runtime`
- `pip install mc-runtime[cuda]`
- `pip install mc-runtime[metal]`

The exact packaging details may shift depending on native build complexity.

## 18. Testing Strategy

### 18.1 Correctness tests

- distribution validation
- known closed-form Monte Carlo examples
- backends compared against CPU reference
- tolerance-based checks for GPU results

### 18.2 Reproducibility tests

- same seed, same backend, same result
- chunked vs unchunked consistency
- planner stable output for same inputs

### 18.3 Performance tests

- microbenchmarks for RNG, kernel execution, reduction
- macrobenchmarks for representative workloads
- benchmark history and regression alerts

### 18.4 Cross-backend tests

Representative workloads should be tested across:

- CPU
- NVIDIA CUDA
- Apple Metal

Use statistical consistency where bitwise equivalence is unrealistic.

## 19. Benchmark Suite

We should choose a benchmark set early and keep it stable.

Recommended benchmark classes:

- Black-Scholes European option
- barrier option path simulation
- geometric Brownian motion ensemble
- simple reliability / failure propagation model
- multivariate uncertainty propagation
- reduction-heavy scalar-output simulation

Compare against:

- NumPy
- NumPy + Numba
- JAX
- CuPy where relevant
- PyTorch for Apple GPU reference if useful

Benchmark outputs should include:

- throughput
- end-to-end runtime
- compile overhead
- memory use
- error vs analytic or reference result

## 20. Roadmap

## Phase 0: Architecture and specification

Deliverables:

- architecture plan
- simulation schema draft
- backend interface draft
- benchmark suite definition
- product positioning and non-goals

## Phase 1: CPU-first prototype

Deliverables:

- Rust core crate
- Python bindings
- simulation IR
- planner with CPU-only backend
- pseudorandom engine
- reductions
- basic run manifest

Goal:

- prove the frontend, IR, planner, and reproducibility model before GPU complexity

## Phase 2: NVIDIA backend

Deliverables:

- CUDA backend
- device capability detection
- GPU-aware planner heuristics
- chunking and transfer planner
- GPU reduction kernels

Goal:

- prove major throughput gains on the benchmark suite

## Phase 3: Apple Metal backend

Deliverables:

- Metal backend
- Apple Silicon device detection
- unified-memory-aware planning
- parity for the core supported workload set

Goal:

- first-class Mac support without distorting the overall architecture

## Phase 4: Planner refinement and agent tooling

Deliverables:

- richer explanation engine
- performance hint system
- benchmark-driven cost model refinement
- JSON schema stabilization

## Phase 5: Scale-out features

Potential later additions:

- distributed execution
- AMD ROCm backend
- domain-specific libraries on top of the core runtime
- advanced MCMC modules
- multi-GPU support

## 21. Risks and Mitigations

### Risk: backend abstraction becomes too weak or too generic

Mitigation:

- keep the IR narrow for v1
- optimize a small set of simulation shapes deeply

### Risk: GPU support dominates engineering effort too early

Mitigation:

- do not start with all backends at once
- prove the architecture on CPU first

### Risk: Apple backend complexity creates a lowest-common-denominator design

Mitigation:

- keep backend contracts stable but allow backend-specific optimizations internally
- do not force CUDA semantics onto Metal or vice versa

### Risk: reproducibility expectations become unrealistic

Mitigation:

- define reproducibility tiers clearly:
  - exact same-backend reproducibility
  - cross-backend statistical reproducibility
  - exact cross-backend only where explicitly guaranteed

### Risk: agent-native features become marketing-only

Mitigation:

- require every major subsystem to expose machine-readable artifacts and explanations

## 22. Recommended v1 Decisions

These are the concrete recommendations I would lock in now:

- Python-first public API
- Rust core runtime
- explicit simulation IR and schema
- CPU backend first
- NVIDIA CUDA backend second
- Apple Metal backend third
- planner based on explicit heuristics first
- Philox as the primary parallel RNG abstraction
- Sobol as first QMC engine
- explainable execution plan as a first-class output
- benchmark suite defined before backend optimization work begins

## 23. Proposed Repository Structure

```text
mc-library/
  docs/
    architecture-plan.md
    schema-draft.md
    benchmark-plan.md
    roadmap.md
  crates/
    mc-core/
    mc-schema/
    mc-cpu/
    mc-python/
  backends/
    mc-cuda/
    mc-metal/
  python/
    mc_runtime/
  benchmarks/
  examples/
  tests/
```

## 24. Next Design Docs To Write

The next four documents should be:

1. `docs/schema-draft.md`
   Define the exact simulation objects, types, and serialization format.

2. `docs/planner-design.md`
   Define the backend selection heuristics, cost estimates, and planner reports.

3. `docs/backend-contract.md`
   Define the trait / interface each backend implements.

4. `docs/benchmark-plan.md`
   Define representative workloads, metrics, and competitor baselines.

## 25. Final Recommendation

The right initial architecture is:

- structured Python API
- Rust core for IR, planner, manifests, and CPU runtime
- backend-specific native GPU implementations for NVIDIA and Apple
- explainable planner as a first-class subsystem
- v1 optimized for forward simulation and uncertainty propagation, not universal stochastic computing

That gives us a practical path to something both ambitious and buildable.
