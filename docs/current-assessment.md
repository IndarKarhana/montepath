# Current Assessment

This document captures the current codebase and benchmark reality so the next improvements target the true bottlenecks.

## Executive Summary

Update: the benchmark-fairness gap has now been addressed for the tracked European-call CPU workload.

The repository is in a healthy early state, but it is still stronger in CPU execution and architecture than in full backend breadth.

Today we have:

- a strong documentation and planning foundation
- a fast specialized CPU implementation for terminal-distribution pricing
- a fair step-wise CPU benchmark path that still beats available NumPy and Numba baselines
- antithetic-variates and control-variates support for the current CPU European-call runtime
- good benchmark automation and artifact discipline
- backend contracts, discovery scaffolding, and explicit fallback execution paths for NVIDIA and Apple
- a workable no-GPU testing strategy for backend conformance and CI
- host-side native CUDA and Metal staging gates with kernel-manifest metadata and compile-time validation
- an actual staged CUDA kernel source plus PTX compile-attempt plumbing behind `cuda-native`
- shared GPU launch and buffer contracts for staged native kernels
- an actual staged Metal shader source plus `.air` / `.metallib` compile-attempt plumbing behind `metal-native`
- a first native Metal execution path on macOS using in-process Rust host integration and cached pipelines
- measured CPU-vs-Metal benchmark data on macOS for the first native Apple GPU path

Today we do not yet have:

- native CUDA kernel execution
- planner decisions calibrated from measured backend behavior
- dedicated native GPU hardware CI

## What Is Working Well

### 1. CPU performance is now strong in both fair and specialized modes

- The fair release step-wise benchmark now leads available NumPy and Numba baselines.
- The specialized terminal-distribution path remains dramatically faster and is now labeled separately.
- We now support both antithetic variates and control variates on the current CPU workload.
- The current control-variate implementation is especially strong for European calls because it uses discounted terminal stock as a control with known expectation `S0`.

### 2. Planner overhead is already cheap

Planner latency is sub-microsecond in release benchmarks for the current scenario set, which is a good base for later explainability and richer planning outputs.

### 3. Repo discipline is stronger than typical early-stage libraries

The architecture docs, roadmap, benchmark artifacts, and quality rules are unusually solid for this stage. That will help us scale complexity without losing direction.

## What Is Misleading Or Risky

### 1. GPU acceleration is real, but still narrow

The planner and backend layers now execute through explicit delegated CPU fallback semantics, and they now include host-side native staging boundaries for CUDA and Metal. CUDA has a real staged `.cu` kernel source and PTX compile-attempt path. Metal has a real staged `.metal` source, `.air` / `.metallib` compile-attempt path, and an in-process native runtime execution path on macOS with cached pipelines and on-device reductions. CUDA still does not run native kernels on-device yet, and the current Metal path is still narrow: it only covers the first standard European-call step-wise workload.

Current measured macOS release results for the tracked workload:

- CPU step-wise Rust: about `15.129 ms`
- native Metal step-wise: about `1.252 ms`

So native Metal is now both functionally working and materially faster than the fair CPU baseline on this tracked workload. The honest limitation is breadth, not this specific speed result: we only have one narrow native workload, no native CUDA execution yet, and no benchmark matrix across larger problem shapes or broader simulation techniques.

That means the product now has a genuine Apple GPU acceleration story, but it still is not broad enough yet to claim general GPU leadership across the library.

### 2. Planner “accuracy” is still synthetic

`planner_choice_accuracy` is measured against a small internal set of expected outcomes, not against measured winner backends on real hardware across workload families.

It is useful as a regression check, but it is not yet evidence that the planner is actually choosing the fastest backend in production conditions.

## Priority Order

## Priority 1: Turn fallback GPU runtimes into native GPU runtimes

The next major product leap is native CUDA and Metal execution.

Immediate goals:

- first CUDA kernel for GBM step update or terminal payoff path
- first Metal equivalent
- structured execution telemetry
- reproducibility notes per backend

Why third:

- without native GPU kernels, the library cannot yet win where GPU-native competitors matter most

## Priority 2: Replace heuristic-only planner choices with measured evidence

Planner heuristics are fine for bootstrap, but they need measured calibration.

Needed next:

- benchmark families across CPU, CUDA, and Metal
- record observed winners by workload shape
- use those observations to tune backend selection and confidence
- eventually separate “supported” from “recommended” more clearly

## Priority 3: Improve the agent-facing public surface

The repo now has `AGENTS.md`, skills, and a function catalog, which is great.

The next useful agent-facing runtime features are:

- `explain_plan()`-style helper
- machine-readable run manifest
- stable tool-ready wrappers for validation, planning, and reference execution

## Concrete Build Sequence

1. Implement the first CUDA kernel path.
2. Implement the first Metal kernel path.
3. Recalibrate planner heuristics from observed data.
4. Add agent-facing explain and manifest helpers.
5. Add scrambled Sobol / randomized quasi-Monte Carlo.
6. Add MLMC foundations.

## What We Should Not Do Yet

- present planner accuracy as production-ready backend intelligence
- over-generalize the public API before the general runtime path exists
