# Current Assessment

This document captures the current codebase and benchmark reality so the next improvements target the true bottlenecks.

## Executive Summary

The repository is in a healthier place than before on both performance and breadth, but we still need to keep two truths in view at once:

- the tracked CPU and Apple Metal results are genuinely strong
- the library is not yet broad enough to claim general market leadership

Today we have:

- a strong documentation and planning foundation
- a durable flagship competitiveness plan in `docs/flagship-competitiveness-plan.md`
- a fast specialized CPU implementation for terminal-distribution pricing
- a fair step-wise CPU benchmark path that beats the available NumPy and Numba baselines on the tracked European workload
- variance-reduction support via antithetic variates and control variates
- a first sampling abstraction via `SamplingMethod`
- a first randomized-QMC surface via `RandomizedHalton`
- deterministic Latin hypercube sampling across the current CPU workload families
- scrambled Sobol and Brownian-bridge CPU structured sampling
- a first direct Rust-vs-SciPy QMC generation benchmark lane
- agent-readable structured-sampling guidance and standard-normal diagnostics
- arithmetic Asian CPU MLMC/MLQMC with explicit per-level estimator metadata, pilot allocation tuning, tolerance planning, and replicated Sobol scrambling
- benchmark automation and artifact discipline
- Phase 2 capability evidence for selected QuantLib competitiveness:
  - product/model capability catalog
  - Greek estimator matrix
  - reference fixture registry with explicit caveats
  - QuantLib-enabled CI benchmark artifact path
- Phase 3 Python-first UX:
  - typed pricing configs
  - dependency-free pricing and European Greek helpers
  - `manifest`, `explain()`, and `reproduce()` result concepts
  - install profiles, error-code docs, docs-site skeleton, notebooks, and package build workflow
- Phase 4 AI-agent-native surface:
  - machine-readable tool manifest
  - JSON schema export
  - agent-safe validate, recommend, plan, execute, compare, benchmark, and reproduce wrappers
  - `agent-run.v1` manifests with seed, backend, method, estimator, build, hardware, warnings, and reference metadata
- Phase 5 accelerator credibility foundation:
  - JAX, CuPy, and PyTorch competitor lanes are executable when hardware is available
  - accelerator rows include warmup, compile, execution, memory, device, and reproducibility metadata
  - dedicated competitor environment manifests cover NumPy, Numba, SciPy QMC, QuantLib, JAX, CuPy, and PyTorch
  - a manual accelerator competitor workflow exists for self-hosted CUDA runners
- backend contracts, discovery scaffolding, and explicit fallback execution paths for NVIDIA and Apple
- host-side native CUDA and Metal staging gates with kernel-manifest metadata and compile-time validation
- a real native Metal execution path on macOS using in-process Rust host integration and cached pipelines
- three benchmarked workload families across CPU and Metal:
  - European call
  - arithmetic Asian call
  - down-and-out call
- a first non-option Gaussian uncertainty-propagation benchmark with an analytic mean
- measured planner calibration against local backend winners

Today we do not yet have:

- native CUDA kernel execution
- native GPU structured sampling
- dedicated native GPU hardware CI
- broad market-leader coverage across more simulation families such as American-style exercise, advanced adjoint Greeks, adaptive MLMC, or scientific UQ workflows
- full QuantLib breadth across calendars, curves, market conventions, and instrument families

## What Is Working Well

### 1. CPU performance is strong on the tracked fair workload

The fair release step-wise benchmark remains comfortably ahead of the available Python CPU baselines.

Current release results:

- Rust CPU European step-wise: `12.845 ms`
- NumPy European step-wise: `76.321 ms`
- Numba European step-wise: `222.326 ms`

That is a real win, not just a specialized fast-path artifact.

### 2. Native Metal is now a meaningful product capability

Native Metal is no longer a narrow demo for one single benchmark case. It now covers three workload families.

Current release results:

- Metal European step-wise: `0.934 ms`
- Metal arithmetic Asian step-wise: `0.860 ms`
- Metal down-and-out step-wise: `0.721 ms`

Relative to our tracked CPU baselines, that makes Apple Metal materially faster on each currently supported native workload family.

### 3. Breadth is improving in the right direction

We now support:

- three option workload families
- variance reduction across those families
- a first separated sampling abstraction instead of baking sampling and variance reduction together
- randomized Halton, Latin hypercube, scrambled Sobol, and Brownian-bridge implementation paths
- the first MLMC and MLQMC CPU reference paths for arithmetic Asian calls

That is a much better foundation for future adaptive tolerance work.

## What Is Still Risky Or Incomplete

### 1. Structured sampling generation is competitive, and pricing overhead is improving

`RandomizedHalton`, `LatinHypercube`, and scrambled Sobol with Brownian bridge are now implemented as breadth and quality milestones. That is useful because it proves the architectural separation between sampling and variance reduction, and it gives us a real platform for future low-discrepancy and uncertainty-propagation work.

The current release artifacts show the structured paths are now much better than the first Halton pass, but still meaningfully slower than the pseudorandom step-wise CPU path:

- Rust CPU European randomized Halton: `79.482 ms`
- Rust CPU European Latin hypercube: `64.128 ms`
- Rust CPU European scrambled Sobol: `79.564 ms`
- Rust CPU European scrambled Sobol Brownian bridge: `100.166 ms`
- Rust CPU European pseudorandom step-wise: `11.169 ms`

The new generation-only QMC scoreboard shows:

- Rust scrambled Sobol normal generation: `74.457 ms`
- SciPy scrambled Sobol normal generation: `116.551 ms`
- Rust randomized Halton normal generation: `55.989 ms`
- SciPy randomized Halton normal generation: `134.500 ms`
- Rust Latin hypercube normal generation: `39.251 ms`
- SciPy Latin hypercube normal generation: `187.319 ms`

The pricing-quality comparison rows now cover European, arithmetic Asian, and down-and-out workloads. The current stderr ratios versus pseudorandom are near neutral rather than clear wins:

- European scrambled Sobol stderr ratio: `1.000`
- arithmetic Asian Latin hypercube stderr ratio: `1.003`
- down-and-out randomized Halton stderr ratio: `0.994`

So direct structured-normal generation is now a speed-competitive surface against SciPy QMC on the tracked rows. Moving batched path-level normal filling into pricing cut a large share of the Sobol pricing overhead, but structured pricing still trails the pseudorandom CPU path and needs realized-error studies before it becomes a default speed or convergence recommendation.

### 2. MLMC and MLQMC now have adaptive tolerance planning, but still need broader validation

Current release results:

- Rust CPU arithmetic Asian step-wise: `15.642 ms`
- Rust CPU arithmetic Asian control-variate: `15.899 ms`
- Rust CPU arithmetic Asian MLMC: `4.330 ms`
- Rust CPU arithmetic Asian MLQMC: `5.760 ms`
- arithmetic Asian MLMC stderr ratio vs step-wise: `2.013`
- arithmetic Asian MLQMC stderr ratio vs step-wise: `0.418`

That is a useful multilevel foundation with a real speed signal for MLMC and a strong accuracy signal for replicated MLQMC. The new tolerance solver makes timing and estimator error explicit, but it still needs broader calibration before becoming a default recommendation claim.

### 3. Non-option UQ coverage has started, with a strong structured-sampling signal

The Gaussian uncertainty-propagation benchmark is intentionally small and analytic-reference-backed. Current release results:

- Rust Gaussian UQ pseudorandom: `3.226 ms`, abs error `0.006344`
- Rust Gaussian UQ randomized Halton: `5.589 ms`, abs error `0.000056`
- Rust Gaussian UQ Latin hypercube: `2.086 ms`, abs error `0.000039`
- Rust Gaussian UQ scrambled Sobol: `6.948 ms`, abs error `0.000043`

This is the clearest current QMC quality win. It is not an option-pricing workload, and it shows why the runtime should keep separate workload classes rather than judging QMC only by path-dependent option standard errors.

### 4. Metal breadth is still GBM-family breadth, not broad Monte Carlo breadth

The current native Apple path is strong, but it is still within one general family of GBM-style path simulation kernels. We do not yet have native Metal support for a larger cross-section of the real market landscape.

### 5. Planner calibration is improving, but not finished

`planner_choice_accuracy_measured` is now `87.5%` on the current local scenario set. That is a good directional signal and a useful regression metric, but it is still not enough to claim production-grade backend intelligence across hardware and workload classes.

### 6. CUDA remains a major unfilled competitive gap

The CUDA artifact and staging layers are solid, but native execution is still not there. Until CUDA is live, we remain unable to compete honestly against accelerator-first ecosystems on NVIDIA-heavy environments.

### 7. MLMC is real and tolerance-planned, but not yet broad

The arithmetic Asian MLMC path proves the coupled fine/coarse estimator surface and returns the metadata we need for serious tuning. It is still CPU-reference only, supports explicit, pilot-budget, and pilot-tolerance path allocation, and does not yet cover discontinuous barrier payoffs.

## Priority Order

### Priority 1: Make structured sampling worth using at scale

Immediate goals:

- add realized-error QMC quality studies where analytic references exist
- continue improving path construction and dimensional mapping
- benchmark Latin hypercube and Sobol quality across more than the tracked European path
- optimize Brownian-bridge construction and Sobol dimension mapping
- keep estimator-quality validation explicit in benchmarks

Why first:

- breadth matters, but breadth only really sticks if the added techniques are performant enough to be practical

### Priority 2: Calibrate MLMC tolerance planning and compare it against structured sampling

Immediate goals:

- calibrate tolerance defaults against measured realized stderr
- add benchmark coverage that records estimated vs realized MLMC/MLQMC error
- compare MLMC against step-wise, control-variate, Sobol, and Sobol Brownian-bridge Asian paths
- keep barrier MLMC separate until discontinuity behavior is documented

Why second:

- MLMC is one of the highest-value additions for path-dependent simulation, but it should earn a default recommendation through measured efficiency rather than novelty

### Priority 3: Broaden native Metal beyond the current GBM option family

Immediate goals:

- add another workload family that is meaningfully different from the current GBM call set
- keep fallback behavior truthful for unsupported features
- preserve the clean shared GPU ABI while broadening kernel support

Why second:

- the current Metal story is good and benchmark-backed, so extending it compounds a real strength

### Priority 4: Keep moving planner choices from heuristics toward evidence

Needed next:

- add more measured winner scenarios across the new workload families
- separate support from recommendation more clearly
- eventually ingest hardware-specific observations once CUDA comes online

## Concrete Build Sequence

1. Add realized-error QMC quality studies and keep measuring randomized Halton, Latin hypercube, and Sobol variants until the structured sampling story is practical.
2. Use measured structured-sampling winners to improve method recommendations.
3. Calibrate adaptive MLMC/MLQMC tolerance planning and document estimator efficiency.
4. Broaden Metal beyond the current GBM option family.
5. Keep recalibrating planner heuristics from observed backend winners.
6. Keep native CUDA launch deferred while CPU, Metal, and multilevel-method quality are the active focus.

## What We Should Not Do Yet

- present full structured-sampling pricing paths as speed-competitive before pricing/path-construction benchmarks support it
- over-generalize current Metal wins into broad GPU leadership claims
- present `87.5%` measured planner accuracy as finished planner intelligence
- present first MLMC support as workload-general
- claim parity with QuantLib, broad SciPy QMC surface area, or CUDA-first libraries on breadth
