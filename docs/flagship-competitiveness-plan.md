# Flagship Competitiveness Plan

This document is the durable execution plan for becoming better than the current flagship Monte Carlo and numerical-simulation ecosystems on the workloads this runtime chooses to own.

The goal is not to clone every feature from every library. The goal is to become the best Monte Carlo execution runtime for users and AI agents that need speed, explainability, reproducibility, and benchmark-backed method choice.

## Current North Star

An ideal user or agent should be able to ask:

> Run this simulation, choose the best method and backend, explain why, prove the error, and give me a reproducible artifact.

Everything below should move the library toward that outcome.

## Scoreboard Discipline

Every phase must maintain:

- benchmark tables with `(ours)` on our rows
- timing, throughput, accuracy, and estimator-quality metrics
- clear unavailable states for missing competitor packages or hardware
- concrete improvement-plan text when a competitor is faster or broader
- roadmap updates when status changes

## Phase 1: Beat SciPy QMC On Structured Sampling

Status: `in-progress`

Primary competitor: SciPy `stats.qmc`.

Why this first:

- structured sampling is already implemented but not yet fast enough
- SciPy QMC is the most direct benchmark target for Sobol, Halton, and Latin hypercube behavior
- a good QMC foundation helps MLQMC, path construction, scientific UQ, and planner recommendations

Deliverables:

- `done` Add Rust structured-normal generation surface for benchmark and agent inspection.
- `done` Add SciPy QMC generation baseline reporting when SciPy is installed.
- `done` Track Rust-vs-SciPy Sobol, Halton, and Latin-hypercube generation timing and quality.
- `done` Optimize Sobol generation by using the four-dimensional Sobol sampler for direct standard-normal generation.
- `done` Move the optimized QMC generation path into pricing/path construction so pricing workloads benefit from the generation win.
- `done` Add sample-size guidance for Sobol power-of-two balance, Halton dimension limits, and LHS use cases.
- `done` Add distribution-quality diagnostics for generated QMC samples.
- `done` Compare QMC pricing quality across European, arithmetic Asian, and down-and-out workloads.
- `done` Add first non-option UQ workload with analytic-mean error tracking.
- `done` Add basket workload QMC quality coverage.
- `done` Add realized-error QMC studies where analytic references exist.
- `done` Feed first realized-error evidence into method recommendations for European-call structured sampling.

Definition of done:

- Rust QMC generation is faster than SciPy for targeted dimensions and sample counts, or the improvement plan explains why not.
- QMC pricing quality is measured across more than one workload family.
- Planner recommendations use measured QMC wins and warn when QMC is inappropriate.

Next action:

- Broaden realized-error studies beyond the first European Black-Scholes reference where semi-analytic references are available, then move benchmark evidence into a measured recommendation database.

## Phase 2: Beat QuantLib On Selected Monte Carlo Workloads

Status: `todo`

Primary competitor: QuantLib.

Target is not full QuantLib breadth. Target is selected Monte Carlo execution leadership.

Deliverables:

- `todo` Add QuantLib competitor benchmark harness for overlapping option workloads.
- `todo` Add basket option CPU runtime and benchmark.
- `todo` Add lookback option CPU runtime and benchmark.
- `todo` Add Heston path simulation with reference validation.
- `todo` Add Greeks: bump-and-revalue first, then pathwise and likelihood-ratio estimators.
- `todo` Add product/model capability catalog with assumptions and unsupported states.
- `todo` Add accuracy fixtures against analytic or semi-analytic references where available.

Definition of done:

- For selected MC workloads, our runtime is faster, easier to inspect, and more reproducible than QuantLib.
- Docs remain honest that QuantLib is broader on calendars, curves, market conventions, and instruments.

## Phase 3: Become The Most User-Friendly MC Library

Status: `todo`

Deliverables:

- `todo` Add Python-first pricing helpers for common workloads.
- `todo` Add typed Python models or dataclasses mirroring Rust configs.
- `todo` Add `result.explain()`, `result.manifest`, and `result.reproduce()` concepts.
- `todo` Add install profiles and troubleshooting docs for `cpu`, `metal`, and future `cuda`.
- `todo` Add error-code documentation and examples.
- `todo` Add notebooks for quants, researchers, and engineers.

Definition of done:

- A new user can run, explain, and reproduce a path-dependent simulation in under five minutes.
- User-facing errors include actionable fixes.

## Phase 4: Become AI-Agent Native

Status: `in-progress`

Deliverables:

- `todo` Add machine-readable tool manifest.
- `todo` Add JSON schema export for stable tool requests and responses.
- `todo` Add run manifest structs for executed simulations.
- `todo` Add agent-safe wrappers for validate, recommend, plan, execute, compare, benchmark, and reproduce.
- `todo` Add deterministic dry-run planning surface for cost and method comparison.

Definition of done:

- An AI agent can validate a simulation, choose a method/backend, run it, compare alternatives, and cite exact reproducibility metadata without reading source code.

## Phase 5: Match JAX/CuPy/PyTorch Accelerator Credibility

Status: `todo`

Deliverables:

- `todo` Add native CUDA launch and reduction.
- `todo` Add GPU RNG with deterministic stream partitioning.
- `todo` Add GPU QMC generation or explicit unsupported diagnostics.
- `todo` Add JAX, CuPy, and PyTorch executable competitor baselines where hardware allows.
- `todo` Add native GPU hardware CI.
- `todo` Add warmup, compile-time, execution-time, memory, and reproducibility reporting.

Definition of done:

- Release benchmark artifacts compare `(ours)` against JAX/CuPy/PyTorch on timing, accuracy, warmup/compile cost, memory, and reproducibility.

## Phase 6: Planner Intelligence

Status: `in-progress`

Deliverables:

- `todo` Add measured winner database from local benchmark artifacts.
- `todo` Add estimated-vs-realized error calibration for MLMC/MLQMC.
- `todo` Add cost frontier reporting for method/backend choices.
- `todo` Add `compare_methods` and `why_not_faster` surfaces.
- `todo` Raise measured planner-choice accuracy above 95% on the tracked scenario suite.

Definition of done:

- Planner choices are evidence-backed, explainable, overrideable, and accurate across the tracked workload families.

## Always-Next Rule

When a session starts and the user says to resume the flagship push:

1. Check this document.
2. Work the first `in-progress` phase with an unfinished `todo`.
3. Update this document, `roadmap.md`, benchmark artifacts, and docs before closing.
4. Do not move to a later phase if the current phase has no measured scoreboard.
