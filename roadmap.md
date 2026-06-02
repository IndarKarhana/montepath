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

- `done` Implement `cpu_native` backend contract.
- `done` Add deterministic RNG stream mapping.
- `done` Implement baseline execution loop for path/step simulation (European call workload).
- `done` Implement baseline reductions (mean and standard error for payoff).
- `done` Validate numeric correctness against benchmark fixtures (analytic Black-Scholes check).
- `done` Add explicit general CPU step-wise execution path separate from specialized terminal-distribution fast path.
- `done` Add antithetic-variates support for the current CPU European-call runtime.
- `done` Add control-variate support for narrow workloads with strong analytic references.
- `done` Add arithmetic Asian and down-and-out call CPU workload families with deterministic reference execution.
- `done` Add fixed-strike lookback call CPU workload with deterministic reference execution.
- `done` Add American put Longstaff-Schwartz CPU reference execution with explicit estimator metadata.
- `done` Add Bermudan put Longstaff-Schwartz CPU reference execution with explicit custom exercise-step schedules.
- `done` Add CRR binomial-tree early-exercise reference checks for American/Bermudan LSM.
- `done` Add Heston stochastic-volatility European-call CPU workload with Black-Scholes-limit validation.
- `done` Add structured CPU Greek reports with bump-and-revalue breadth plus European pathwise and likelihood-ratio estimators.
- `done` Add first explicit sampling-method abstraction with pseudorandom, randomized Halton, and Latin hypercube CPU execution paths.

## Phase 4: NVIDIA Runtime

- `done` Implement CUDA backend contract and device discovery.
- `done` Add truthful delegated fallback execution path for CUDA backend while native kernels are in progress.
- `done` Add host-side CUDA native staging boundary and feature gate.
- `done` Add shared GPU launch and buffer contracts for staged native kernels.
- `done` Add first staged CUDA kernel source and PTX compile-attempt path for the core workload.
- `deferred` Implement native CUDA launch and reduction path for the staged kernel after the current CPU, Metal, and multilevel-method push.
- `done` Add GPU memory and chunking strategy.
- `done` Add planner heuristics for CUDA selection.

## Phase 5: Apple Runtime

- `done` Implement Apple Metal backend contract.
- `done` Add truthful delegated fallback execution path for Apple Metal backend while native kernels are in progress.
- `done` Add host-side Metal native staging boundary and feature gate.
- `done` Add first staged Metal shader source and `.air` / `.metallib` compile-attempt path for the core workload.
- `done` Implement first native Metal launch path for the staged kernel on macOS using runtime compilation.
- `done` Add CPU-vs-native-Metal benchmark coverage on macOS.
- `done` Move first Metal-native RNG generation and full staged reductions on-device.
- `done` Remove helper-based execution overhead via persistent in-process native host integration.
- `done` Add benchmark-calibrated planner heuristics for Apple backend selection.
- `done` Extend native Metal execution across the first European-call step-wise technique family (`Standard`, `Antithetic`, `ControlVariate`).
- `done` Extend native Metal execution to a second workload family with arithmetic Asian calls and control-variate support.
- `done` Extend native Metal execution to a third GBM workload family with down-and-out calls and control-variate support.
- `done` Keep structured-sampling Metal requests explicit by falling back to CPU reference execution instead of silently approximating unsupported native behavior.

## Phase 6: Benchmarks and Tuning

- `done` Implement benchmark harness and result schema.
- `in-progress` Add baseline comparisons against NumPy / Numba / QuantLib / JAX / CuPy where relevant.
- `done` Add automated CPU competitor baselines for NumPy and Numba.
- `done` Add explicit availability reporting for JAX / CuPy / PyTorch in benchmark output.
- `done` Add executable JAX / CuPy / PyTorch accelerator competitor lanes with explicit telemetry and unavailable diagnostics.
- `done` Add explicit QuantLib competitor reporting for the overlapping European Monte Carlo workload.
- `done` Add explicit QuantLib competitor reporting for the fixed-strike lookback Monte Carlo workload.
- `done` Add explicit QuantLib competitor reporting for the Heston analytic reference lane.
- `done` Add explicit QuantLib competitor reporting for American/Bermudan Longstaff-Schwartz early-exercise lanes, including unavailable diagnostics when local QuantLib-Python support is missing.
- `done` Add QuantLib-enabled competitor CI profile with environment manifest and uploaded release-style artifact.
- `done` Add dedicated competitor CI profiles for NumPy, Numba, SciPy QMC, QuantLib, JAX, CuPy, and PyTorch with environment manifests.
- `done` Add Phase 2 reference fixture registry for supported products, models, and Greeks using analytic references where available and explicit caveats elsewhere.
- `done` Auto-generate competitiveness improvement plan when running benchmarks.
- `done` Track planner decision quality via planner-choice accuracy benchmark.
- `done` Define performance gates and regression thresholds.
- `done` Add release-profile benchmark output for stronger competitiveness tracking.
- `done` Add competitiveness gate checks (Rust faster than available NumPy/Numba CPU baselines).
- `done` Split European-call benchmarks into fair terminal-distribution and true step-wise benchmark families.
- `done` Add benchmark methodology metadata so specialized fast paths are not confused with general-runtime comparisons.
- `done` Add internal antithetic-quality benchmarking via stderr-ratio tracking.
- `done` Add internal control-variate-quality benchmarking via stderr-ratio tracking.
- `done` Calibrate planner choice quality against measured backend winners, not only hand-authored expected scenarios.
- `done` Add arithmetic Asian CPU and Apple Metal benchmark coverage.
- `done` Add down-and-out CPU and Apple Metal benchmark coverage.
- `done` Add fixed-strike lookback CPU benchmark coverage.
- `done` Add American put Longstaff-Schwartz CPU benchmark coverage and reference/comparison policy.
- `done` Add Bermudan put Longstaff-Schwartz CPU benchmark coverage and custom-schedule reference policy.
- `done` Add American/Bermudan LSM benchmark quality rows against CRR binomial-tree references.
- `done` Add Heston CPU benchmark and Black-Scholes-limit quality coverage.
- `done` Add Greek benchmark coverage with Black-Scholes Delta accuracy gates and all-workload bump-and-revalue breadth tracking.
- `done` Add first randomized-Halton benchmark and estimator-quality coverage.
- `done` Add first Latin-hypercube benchmark and estimator-quality coverage.
- `done` Add compact benchmark profile for fast local gates without overwriting full competitiveness artifacts.
- `done` Add arithmetic Asian MLMC benchmark and estimator-quality coverage.
- `done` Add arithmetic Asian MLQMC benchmark and estimator-quality coverage.
- `done` Add pilot-based MLMC/MLQMC path allocation tuning.
- `done` Add arithmetic Asian MLMC/MLQMC adaptive tolerance planning.
- `done` Add arithmetic Asian MLMC/MLQMC realized-error calibration rows against high-budget standard Monte Carlo references.
- `done` Add Merton jump-diffusion benchmark and analytic series-reference quality coverage.
- `done` Add European-call parameter-sweep benchmark coverage with per-scenario Black-Scholes accuracy tracking.
- `done` Add Gaussian UQ analytic-variance benchmark coverage beyond the first analytic-mean check.

## Phase 7: Agent Experience and Integration

- `done` Add project-level `AGENTS.md` instructions for repo-native agent workflows.
- `done` Add Codex project skills for architecture and agent-surface discipline.
- `done` Add a function catalog for public and future tool-facing surfaces.
- `done` Add an agent integration plan for future tool/plugin wrapping.
- `done` Add a machine-readable Monte Carlo method capability catalog for supported and planned techniques.
- `done` Add first Python-facing wrapper scaffold for method recommendations and benchmark audits.
- `done` Extend method recommendations with machine-readable `method_id` and first MLMC recommendation path.
- `done` Add machine-readable tool manifest and schema export for stable agent integration.
- `done` Add reproducibility run manifests for pricing, Greek, benchmark, and planner outputs.
- `done` Add explain-plan and run-manifest helpers as first-class agent-facing surfaces.
- `done` Add Python-facing agent wrappers that preserve typed, explainable contracts.
- `done` Add planner-intelligence agent tools for evidence loading, cost frontiers, measured method comparison, why-not-faster explanations, and MLMC/MLQMC calibration.
- `done` Add typed European-call parameter-sweep result surfaces so agents can inspect batch scenarios without parsing prose.

## Phase 8: Advanced Simulation Techniques

- `done` Add first randomized-QMC surface through randomized Halton sampling.
- `done` Add Latin hypercube sampling for the current CPU workload families.
- `done` Add scrambled Sobol sampling and Brownian-bridge path construction for CPU structured sampling.
- `done` Remove per-path Brownian-bridge allocation from CPU structured sampling by reusing precomputed bridge plans and work buffers.
- `done` Add multilevel Monte Carlo foundations for arithmetic Asian CPU path simulation.
- `done` Add first multilevel randomized-QMC foundation through arithmetic Asian MLMC with scrambled Sobol increments.
- `done` Add replicated Sobol scrambling for arithmetic Asian MLQMC error estimates.
- `done` Add adaptive tolerance planning on top of pilot MLMC/MLQMC allocation.
- `done` Add first direct Rust-vs-SciPy QMC generation benchmark lane.
- `done` Optimize direct scrambled-Sobol normal generation against the SciPy QMC benchmark lane.
- `done` Move batched structured-normal filling into CPU structured pricing paths.
- `done` Add agent-readable structured-sampling guidance and standard-normal diagnostics.
- `done` Add cross-workload QMC pricing-quality comparisons against pseudorandom baselines.
- `done` Add first non-option Gaussian UQ benchmark with analytic-mean error tracking.
- `done` Add Gaussian UQ analytic moment coverage with variance error and confidence-interval metadata.
- `done` Add two-asset basket-call CPU workload and QMC quality benchmark coverage.
- `done` Add European-call realized-error QMC benchmarks against the Black-Scholes analytic reference.
- `done` Add benchmark-backed structured sampling calibration and guidance for Halton, Latin hypercube, and Sobol variants on the current CPU workload set.
- `done` Calibrate MLMC/MLQMC level/path tolerance defaults on the arithmetic Asian reference path with realized-error benchmark rows.
- `done` Add first artifact-backed realized-vs-estimated error validation surface for arithmetic Asian MLMC/MLQMC.

## Phase 9: Flagship Competitiveness Program

See `docs/flagship-competitiveness-plan.md` for the durable execution sequence.

- `done` Phase 1: Beat SciPy QMC on structured sampling for targeted workloads.
- `done` Phase 2: Beat QuantLib on selected Monte Carlo workloads with a capability catalog, Greek matrix, reference fixture registry, and QuantLib CI environment.
- `done` Phase 3: Become the most user-friendly Monte Carlo library with Python-first APIs, packaging, docs, and release discipline.
- `done` Phase 4: Become AI-agent native with manifests, schemas, and safe tool wrappers.
- `in-progress` Phase 5: Match JAX/CuPy/PyTorch accelerator credibility with native CUDA, hardware CI, and competitor environments.
- `done` Phase 6: Turn planner intelligence into a measured advantage with benchmark-backed winner databases and method comparison surfaces.
- `done` Phase 7: Broaden product/model/UQ coverage with documented assumptions, references, Greeks, and unsupported states.

## Phase 10: Non-CUDA V1 Polish

See `docs/non-cuda-v1-completion-plan.md` for the focused release plan.

- `done` Add Rust-backed Python execution surface while preserving the existing Python-first ergonomics.
- `done` Add Python native-runtime discovery so users and agents can detect whether compiled Rust execution is installed.
- `done` Expose main Rust workload families through stable Python configs and native-bridge result surfaces.
- `done` Package agent tools behind an MCP-compatible server boundary with schemas, execution limits, and version metadata.
- `done` Reduce or scope Clippy lint debt so warning enforcement becomes useful in CI.
- `done` Validate clean wheel/source installation and installed-package smoke tests.
- `done` Document that native CUDA execution is deferred to a later library version.

## Phase 11: MontePath PyPI Publication

- `done` Rename the public Python distribution to `montepath`.
- `done` Rename the public Python import package to `montepath`.
- `done` Rename the Rust-backed native extension module to `montepath._native`.
- `done` Rename the MCP console entry point to `montepath-mcp`.
- `done` Update package metadata, docs, examples, function catalog, and release
  checklist for the MontePath public alpha.
- `done` Rename the hosted GitHub repository to `montepath` and verify the
  repository URLs resolve.
- `done` Add the GitHub Actions trusted-publishing workflow and repository
  `pypi` environment for PyPI OIDC publication.
- `todo` Configure the PyPI account-side pending trusted publisher or provide a
  scoped PyPI token.
- `todo` Publish `montepath` `0.1.0` to PyPI after final release checks.
- `todo` Verify post-publication installation through `uv pip install
  montepath`, `uv add montepath`, and `uvx --from montepath montepath-mcp`.

## Ongoing Engineering Quality Track

- `in-progress` Test-driven development as default coding workflow.
- `in-progress` Production-grade code quality and reliability standards.
- `in-progress` Keep runtime lightweight, minimal overhead, and dependency-conscious.
- `done` Research user-friendliness patterns from leading libraries and implement first UX improvements.
- `done` Add an honest market-landscape document against leading Monte Carlo library categories.
- `done` Add baseline CPU CI for format, test, and benchmark smoke checks.
- `done` Validate feature-gated native backend staging in CPU-only CI.
- `done` Add Python package metadata, wheel/source build workflow, changelog, versioning policy, and release checklist.
- `in-progress` Prepare MontePath for first PyPI publication.
- `done` Add docs-site structure with quickstarts, examples, benchmark interpretation, API reference, and migration notes.
- `in-progress` Add native CUDA and Metal hardware CI on dedicated runners.

## Remaining V1 Completion Count

There are `2` completion tracks remaining:

1. Finish Phase 5 native CUDA execution in a later accelerator-focused version.
2. Publish MontePath `0.1.0` to PyPI and verify `uv`/`uvx` installation after
   publication.
