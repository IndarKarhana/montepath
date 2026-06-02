# Changelog

## Unreleased

- Added production preflight helpers for installed CPU-native, Python
  reference, Apple Metal, and CUDA capability reporting.
- Added explicit backend selection and workload validation surfaces that avoid
  silent fallback for unavailable Metal or deferred CUDA execution.
- Added agent and MCP tools for installed capability inspection and production
  readiness checks.
- Expanded installed-package smoke coverage to verify production status,
  backend selection, benchmark summaries, and MCP production tools.

## 0.1.1

- Added stable-ABI Python extension builds for Python 3.10+.
- Added trusted-publishing workflow support for source distributions plus
  manylinux x86_64, macOS universal2, and Windows x64 binary wheels.
- Added duplicate-file pruning and checks for trusted-publishing reruns so
  supplemental wheels can be uploaded without republishing existing artifacts.

## 0.1.0

- Renamed the public package, import surface, native module, and MCP command to
  MontePath: `montepath`, `montepath._native`, and `montepath-mcp`.
- Packaged the public alpha positioning for users and LLM agents.
- Added Rust-backed Python CPU extension packaging through `montepath._native`.
- Added `montepath-mcp`, a dependency-free MCP-compatible stdio server for
  the agent tool surface.
- Added uv/uvx install and agent launch guidance.
- Added installed wheel/source distribution smoke tests for native extension
  and MCP availability.
- Scoped Clippy enforcement in CI to warning-clean crates while documenting
  remaining `mc-core` hot-path lint debt.
- Refreshed release benchmark artifacts and competitiveness improvement plan.
- Added Python-first method recommendation and benchmark helper surfaces.
- Added Python-first pricing helpers for European, arithmetic Asian, and
  down-and-out calls.
- Added European-call Greek helper with Black-Scholes reference metadata.
- Added structured result objects with `manifest`, `explain()`, and
  `reproduce()`.
- Added Phase 2 QuantLib competitiveness catalog, reference fixtures, and
  QuantLib benchmark CI profile.
- Added Phase 4 agent-native tool manifest, JSON-schema export, safe wrappers,
  run manifests, and exact request/response examples.
- Added Phase 5 accelerator competitor foundation with JAX, CuPy, and PyTorch
  benchmark lanes, telemetry, environment manifests, and hardware workflow.
- Added Phase 6 planner intelligence surfaces for measured winner databases,
  cost frontiers, method comparison, why-not-faster explanations, and
  MLMC/MLQMC calibration.
- Recalibrated Apple Metal planner heuristics against measured local backend
  winners and refreshed planner-choice accuracy evidence.
- Fixed the QuantLib CI preflight so missing per-instrument QuantLib-Python APIs
  remain explicit unavailable benchmark rows instead of failing the whole CI job.
- Started Phase 7.1 with a CPU Longstaff-Schwartz American-put pricing surface,
  benchmark row, lower-bound reference fixture, and capability documentation.
- Added Phase 7.2 CPU Longstaff-Schwartz Bermudan-put pricing with custom
  exercise-step schedules, benchmark coverage, lower-bound reference fixture,
  and MCP-oriented agent-surface documentation.
- Added CRR binomial-tree reference helpers and quality benchmark rows for
  American and Bermudan put Longstaff-Schwartz accuracy audits.
