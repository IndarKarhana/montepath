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

Status: `done`

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

Status: `done`

Primary competitor: QuantLib.

Target is not full QuantLib breadth. Target is selected Monte Carlo execution leadership.

Deliverables:

- `done` Add QuantLib competitor benchmark harness for overlapping option workloads.
- `done` Add basket option CPU runtime and benchmark.
- `done` Add lookback option CPU runtime and benchmark.
- `done` Add Heston path simulation with reference validation.
- `done` Add Greeks: bump-and-revalue first, then pathwise and likelihood-ratio estimators.
- `done` Add product/model capability catalog with assumptions and unsupported states.
- `done` Add Greek estimator capability matrix by product/model, including bump, pathwise, likelihood-ratio, and unsupported states.
- `done` Add accuracy fixtures against analytic or semi-analytic references where available.
- `done` Add QuantLib-enabled benchmark environment so QuantLib lanes are populated in CI/release artifacts instead of only reporting unavailable locally.

Definition of done:

- For selected MC workloads, our runtime is faster, easier to inspect, and more reproducible than QuantLib.
- Docs remain honest that QuantLib is broader on calendars, curves, market conventions, and instruments.
- Every selected product/model/Greek has a documented reference source or an explicit "no trusted fixture yet" caveat.

Evidence:

- Capability catalog: `docs/product-model-capability-catalog.md`
- Machine-readable catalog: `docs/product-model-capability-catalog.json`
- Reference fixture registry: `benchmarks/reference-fixtures.json`
- QuantLib competitor environment: `benchmarks/competitors/requirements-quantlib.txt`
- QuantLib CI artifact path: `benchmarks/quantlib-ci-results.json`

## Phase 3: Become The Most User-Friendly MC Library

Status: `done`

Deliverables:

- `done` Add Python-first pricing helpers for common workloads.
- `done` Add typed Python models or dataclasses mirroring Rust configs.
- `done` Add `result.explain()`, `result.manifest`, and `result.reproduce()` concepts.
- `done` Add install profiles and troubleshooting docs for `cpu`, `metal`, and future `cuda`.
- `done` Add error-code documentation and examples.
- `done` Add notebooks for quants, researchers, and engineers.
- `done` Add PyPI packaging, wheel build automation, versioning policy, changelog, and release checklist.
- `done` Add docs-site structure with quickstarts, API reference, benchmark interpretation, and migration notes.

Definition of done:

- A new user can run, explain, and reproduce a path-dependent simulation in under five minutes.
- User-facing errors include actionable fixes.
- A user can install the package without checking out the repo, run common pricing/Greek workflows, and understand benchmark claims without reading source code.

Evidence:

- Python UX helpers: `python/mc_library/pricing.py`
- Quickstart docs: `docs/site/quickstart.md`
- API reference: `docs/site/api-reference.md`
- Install profiles: `docs/install-profiles.md`
- Error codes: `docs/error-codes.md`
- Release checklist and changelog: `docs/release-checklist.md`, `CHANGELOG.md`
- Package build workflow: `.github/workflows/python-package.yml`

## Phase 4: Become AI-Agent Native

Status: `done`

Deliverables:

- `done` Add machine-readable tool manifest.
- `done` Add JSON schema export for stable tool requests and responses.
- `done` Add run manifest structs for executed simulations.
- `done` Add agent-safe wrappers for validate, recommend, plan, execute, compare, benchmark, and reproduce.
- `done` Add deterministic dry-run planning surface for cost and method comparison.
- `done` Add reproducibility manifests to pricing, Greek, benchmark, and planner outputs, including seed, backend, method, estimator, build, hardware, warnings, and reference metadata.
- `done` Add stable agent-facing examples that show exact request/response payloads.

Definition of done:

- An AI agent can validate a simulation, choose a method/backend, run it, compare alternatives, and cite exact reproducibility metadata without reading source code.
- Agent tools can be called safely without hidden global state or ambiguous free-form outputs.

Evidence:

- Agent tools: `python/mc_library/agent.py`
- Agent tests: `python/tests/test_agent_surface.py`
- Agent docs: `docs/agent-tooling.md`
- Exact payload examples: `docs/agent-examples.json`

## Phase 5: Match JAX/CuPy/PyTorch Accelerator Credibility

Status: `in-progress`

Deliverables:

- `todo` Add native CUDA launch and reduction.
- `todo` Add GPU RNG with deterministic stream partitioning.
- `done` Add GPU QMC generation or explicit unsupported diagnostics.
- `done` Add JAX, CuPy, and PyTorch executable competitor baselines where hardware allows.
- `in-progress` Add native GPU hardware CI.
- `done` Add warmup, compile-time, execution-time, memory, and reproducibility reporting.
- `done` Add dedicated competitor CI profiles for NumPy, Numba, SciPy QMC, QuantLib, JAX, CuPy, and PyTorch with explicit environment manifests.

Definition of done:

- Release benchmark artifacts compare `(ours)` against JAX/CuPy/PyTorch on timing, accuracy, warmup/compile cost, memory, and reproducibility.
- Accelerator claims are hardware-backed, not inferred from CPU-only machines.

Current evidence:

- Competitor environment manifests: `benchmarks/competitors/environments/`
- Accelerator requirements: `benchmarks/competitors/requirements-accelerators.txt`
- Accelerator competitor workflow: `.github/workflows/accelerator-competitors.yml`
- Accelerator credibility docs: `docs/accelerator-competitor-benchmarking.md`

CUDA-deferred caveat:

- Native CUDA launch, reductions, and deterministic GPU RNG stream partitioning
  remain intentionally deferred. Phase 5 cannot be marked done until those are
  implemented and measured on hardware.

## Phase 6: Planner Intelligence

Status: `done`

Deliverables:

- `done` Add measured winner database from local benchmark artifacts.
- `done` Add estimated-vs-realized error calibration for MLMC/MLQMC.
- `done` Add cost frontier reporting for method/backend choices.
- `done` Add `compare_methods` and `why_not_faster` surfaces.
- `done` Raise measured planner-choice accuracy above 95% on the tracked scenario suite.
- `done` Add planner evidence records that connect recommendations to benchmark artifact IDs, workload assumptions, and reference fixtures.
- `done` Add user/agent-facing planner explanations for rejected methods, unsupported estimators, and accuracy/runtime tradeoffs.

Definition of done:

- Planner choices are evidence-backed, explainable, overrideable, and accurate across the tracked workload families.

Evidence:

- Planner intelligence docs: `docs/planner-intelligence.md`
- Python evidence surfaces: `python/mc_library/planner_intelligence.py`
- Agent wrappers: `mc.planner_evidence`, `mc.cost_frontier`, `mc.compare_methods`, `mc.why_not_faster`, `mc.mlmc_calibration`
- Release benchmark artifact: `benchmarks/release-results.json`
- Current measured planner-choice accuracy: `100%` on the tracked local scenario suite.

## Phase 7: Broaden Product And Model Coverage

Status: `in-progress`

Primary competitors: QuantLib for finance breadth, SciPy/JAX-style stacks for general simulation breadth.

Deliverables:

- `done` Add an American put Longstaff-Schwartz CPU reference surface with explicit method assumptions, benchmark row, and lower-bound reference fixture.
- `todo` Add Bermudan custom exercise schedules with reference fixtures and benchmark coverage.
- `todo` Add additional diffusion/model families such as jump diffusion, stochastic rates, or generic SDE templates after references are defined.
- `todo` Add batch/portfolio parameter sweeps with reproducible manifests and benchmark coverage.
- `todo` Add a broader scientific UQ surface beyond the first Gaussian analytic-mean workload.
- `todo` Add product families only when unsupported behavior, references, Greeks, and benchmark methodology can be documented honestly.

Definition of done:

- The library is no longer only a set of selected pricing workloads; it has a documented path toward broader quantitative-finance and scientific Monte Carlo coverage.
- Breadth claims are backed by product/model catalog entries, tests, and benchmark rows.

## Remaining Completion Phases

These are the remaining durable phases before a serious v1:

1. Phase 5: accelerator credibility, competitor CI, and native CUDA.
2. Phase 7: broader product/model/UQ coverage.

## Always-Next Rule

When a session starts and the user says to resume the flagship push:

1. Check this document.
2. Work the first `in-progress` phase with an unfinished `todo`.
3. Update this document, `roadmap.md`, benchmark artifacts, and docs before closing.
4. Do not move to a later phase if the current phase has no measured scoreboard.
