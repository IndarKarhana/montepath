# Benchmark Plan

## 1. Purpose

This document defines the benchmark suite that will guide performance work and backend selection tuning.

The benchmark plan exists to:

- validate that the runtime solves meaningful workloads
- provide objective CPU vs GPU comparisons
- prevent performance regressions
- create the data needed to refine planner heuristics
- compare against realistic ecosystem baselines

## 2. Benchmark Principles

1. Benchmark representative workloads, not toy kernels alone
2. Separate microbenchmarks from end-to-end benchmarks
3. Measure throughput and correctness together
4. Record compile overhead separately from steady-state runtime
5. Use the same simulation semantics across implementations
6. Keep benchmark inputs versioned and reproducible
7. Distinguish specialized fast paths from general simulation paths

## 2.1 Apples-To-Apples Rule

Benchmark claims are only competitive claims when the compared implementations use the same simulation semantics.

Examples:

- direct terminal-distribution sampling must be compared against direct terminal-distribution sampling
- step-wise path simulation must be compared against step-wise path simulation
- execution-only timing must not be mixed with compile-plus-execution timing without explicit labeling

Benchmark outputs should eventually tag each result with methodology metadata so specialized wins are visible but not confused with general-runtime wins.

## 3. Benchmark Categories

The suite should include:

- microbenchmarks
- core simulation benchmarks
- reduction-heavy benchmarks
- branch-heavy benchmarks
- backend selection benchmarks
- reproducibility benchmarks

## 4. Success Criteria

The benchmark suite should answer:

- when is CPU faster?
- when is NVIDIA faster?
- when is Apple GPU faster?
- how much overhead does planning and compilation add?
- how accurate are planner choices?
- how do our results compare to NumPy, Numba, JAX, CuPy, and selected references?

## 5. Metrics

Required metrics:

- end-to-end runtime
- execution-only runtime
- compile time
- planner time
- throughput: paths/sec or samples/sec
- peak memory estimate
- observed peak memory where available
- numeric error vs analytic or reference result
- reproducibility status

Optional metrics:

- kernel launch count
- device utilization hints
- transfer time
- reduction time
- chunk count

## 6. Benchmark Environments

At minimum we should support benchmarking on:

- CPU-only machine
- NVIDIA CUDA machine
- Apple Silicon machine

Related operational guidance:

- benchmark development may proceed on CPU-only machines
- native GPU benchmark claims require dedicated hardware runs
- see `docs/gpu-testing-strategy.md` for the validation split between CPU-only and hardware-backed testing

Benchmark manifests should record:

- OS
- CPU model
- GPU model
- memory size
- driver/toolchain versions
- runtime build version

## 7. Competitor / Baseline Set

Baseline comparisons should include relevant tools per workload.

### 7.1 CPU baselines

- NumPy
- NumPy + SciPy where relevant
- NumPy + Numba
- pure Python only for sanity checks, not as a serious performance baseline

### 7.2 NVIDIA-oriented baselines

- CuPy
- JAX
- PyTorch for selected tensor-like workloads where relevant
- custom Numba CUDA examples where comparable

### 7.3 Apple-oriented baselines

- CPU baseline on same machine
- PyTorch MPS where a comparable workload exists
- JAX on Apple where practical if support is usable for the chosen benchmark environment

We should avoid forcing every competitor onto every benchmark if the comparison is not meaningful.

## 8. Benchmark Workloads

## 8.1 European call option

Purpose:

- closed-form reference available
- simple path-parallel workload
- good early correctness and throughput benchmark

This workload must be split into two benchmark modes:

- terminal-distribution mode
- step-wise path-simulation mode

Outputs:

- estimated option price
- standard error
- error vs Black-Scholes analytic solution

Parameters to sweep:

- `n_paths`
- `n_steps`
- dtype

Rules:

- terminal-distribution mode may ignore `n_steps` only if every compared implementation uses the same terminal formulation
- step-wise mode must materially execute the configured `n_steps`

## 8.2 Barrier option path simulation

Purpose:

- more realistic path-dependent workload
- tests ordered step transitions and branch conditions
- useful for GPU divergence analysis

Outputs:

- estimated price
- standard error
- runtime across backends

Parameters to sweep:

- barrier proximity
- `n_paths`
- `n_steps`

## 8.3 Geometric Brownian motion ensemble

Purpose:

- foundational path simulation benchmark
- isolates state update throughput
- useful for large-scale backend comparisons

Outputs:

- terminal distribution moments
- runtime and memory behavior

## 8.4 Reliability / failure propagation model

Purpose:

- uncertainty propagation beyond finance
- moderate branching
- tests generality of the schema and planner

Example:

- system failure probability with component lifetime uncertainty

Outputs:

- estimated failure probability
- confidence interval

## 8.5 Multivariate uncertainty propagation

Purpose:

- tests vector-valued parameters and outputs
- useful for scientific simulation use cases

Example:

- propagate uncertain inputs through a simplified physical model

Outputs:

- mean and covariance summaries
- runtime and scaling

## 8.6 Reduction-heavy benchmark

Purpose:

- stress reduction implementation and chunk merge behavior
- compare full materialization vs streaming reduction strategies

Outputs:

- throughput
- memory footprint
- reduction stability

## 8.7 Branch-heavy benchmark

Purpose:

- give the planner a workload where CPU may beat GPU
- validate branch divergence heuristics

Example:

- state machine style simulation with multiple conditional branches per step

Outputs:

- runtime by backend
- planner choice accuracy

## 9. Microbenchmarks

Microbenchmarks should isolate subsystems.

### 9.1 RNG throughput

Measure:

- IID normal generation
- uniform generation
- Sobol generation
- per-backend throughput

### 9.2 State update kernel throughput

Measure:

- simple arithmetic update
- transcendental-heavy update
- update with one branch

### 9.3 Reduction throughput

Measure:

- sum
- mean
- variance
- min / max

### 9.4 Planner overhead

Measure:

- schema normalization time
- feature extraction time
- backend support query time
- cost-estimation time

## 10. Reproducibility Benchmarks

We should explicitly benchmark reproducibility behavior.

Cases:

- same seed, same backend, same result
- same seed, same backend, chunked vs unchunked
- same seed, different backend, statistical consistency
- compile cache reuse vs fresh compile

Outputs:

- exact match status where promised
- statistical agreement where expected
- run manifest comparison

## 11. Planner Evaluation Benchmarks

We should evaluate the planner as a system, not just runtime speed.

Metrics:

- planner-selected backend vs fastest observed backend
- planner runtime overhead
- planner memory estimate error
- planner runtime estimate error
- percent of cases where planner choice is within an acceptable margin of optimal

Suggested success threshold for v1:

- planner should choose a backend within 20% of the observed best runtime for most supported benchmark cases

This threshold can evolve as we gather more evidence.

## 12. Benchmark Matrix

Each benchmark should run across a matrix where applicable:

- backend: CPU / NVIDIA / Apple
- precision: float32 / float64
- path count: small / medium / large
- step count: small / medium / large
- planner mode: safe / balanced / aggressive

Example size buckets:

- small: `1e4` paths
- medium: `1e6` paths
- large: `1e7` paths

Actual values should be tuned per device memory and runtime practicality.

## 13. Warmup and Repetition Policy

To reduce noise:

- separate cold and warm runs
- report compile time separately
- use multiple repetitions
- report median and spread
- note when benchmark time is dominated by startup overhead

Suggested policy:

- 1 cold run
- 5 warm runs
- report median execution time and interquartile range

## 14. Output Format

Benchmark results should be stored in a structured format.

Recommended fields:

- benchmark name
- benchmark version
- simulation parameters
- backend
- device
- planner mode
- precision
- compile time
- execution time
- total time
- throughput
- error metrics
- reproducibility flags
- environment metadata

JSON or Parquet would both be good candidates.

## 15. Visualization Goals

We should generate a small stable set of charts:

- runtime vs path count
- speedup over CPU baseline
- planner choice accuracy
- compile overhead fraction
- error vs runtime tradeoff by precision
- backend crossover points

These plots will help both users and us understand where the runtime actually wins.

## 16. Acceptance Gates

Before calling the v1 CPU backend solid:

- European option benchmark passes correctness checks
- planner overhead remains small relative to execution for medium and large runs
- CPU throughput beats naive NumPy in targeted workloads

Before calling the CUDA backend solid:

- CUDA wins clearly on large dense workloads
- planner chooses CUDA appropriately on supported NVIDIA systems
- reductions and chunking remain numerically stable

Before calling the Apple backend solid:

- Apple GPU wins over CPU for selected large workloads on Apple Silicon
- planner correctly distinguishes small vs large workloads
- runtime parity exists for the supported core workload subset

## 17. CI vs Full Benchmarking

Not all benchmarks belong in CI.

### 17.1 CI benchmarks

- small microbenchmarks
- planner overhead checks
- CPU smoke benchmarks
- regression checks on output shape and correctness

### 17.2 Scheduled benchmarks

- full end-to-end suite
- GPU benchmarks
- comparative baseline runs
- planner evaluation suite

This separation keeps CI fast while preserving serious performance tracking.

## 18. Risks

### Risk: benchmarking toy cases that flatter the architecture

Mitigation:

- include path-dependent and branch-heavy cases
- include workloads where CPU is expected to win

### Risk: comparing against weak baselines

Mitigation:

- include strong ecosystem references like Numba, JAX, and CuPy where appropriate

### Risk: unstable results due to environment noise

Mitigation:

- capture environment metadata
- use repeated runs and medians
- isolate cold vs warm behavior

## 19. Open Questions

1. Which exact Apple framework baseline is most fair for GPU comparisons on Mac?
2. Should we prioritize finance-heavy benchmarks first, or balance with scientific workloads immediately?
3. What planner-choice accuracy threshold is ambitious but realistic for the first usable release?
4. Should benchmark fixtures ship with the repo or be generated dynamically?

## 20. Recommendation

Build the benchmark suite early and treat it as part of the product.

If we do that, we gain:

- honest performance visibility
- better planner tuning
- credibility with users
- a much lower chance of optimizing the wrong things
