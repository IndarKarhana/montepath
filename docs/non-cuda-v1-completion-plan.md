# Non-CUDA V1 Completion Plan

This plan tracks the work needed to make the library excellent before native
CUDA execution ships. CUDA remains important, but it is intentionally deferred
to a later version so the CPU, Apple Metal, Python, documentation, and
agent-facing surfaces can become release-grade first.

## Position

The non-CUDA v1 target is:

- Rust CPU and Apple Metal execution paths are benchmark-backed and honest.
- Python users can call the fast runtime through a stable public API.
- Agents can discover capabilities, validate requests, run supported work, and
  reproduce outputs without source-code archaeology.
- Benchmark claims remain tied to release artifacts and competitor rows.
- Unsupported CUDA behavior is explicit, not silent fallback disguised as
  acceleration.

## Required Before Non-CUDA V1

1. Rust-backed Python execution surface
- Status: `done`
- `done` Add a stable native-runtime discovery layer.
- `done` Add stable Python configs and native-bridge result surfaces for the
  current Rust workload families.
- `done` Add compiled CPU bindings for the current native bridge workload set.
- `done` Preserve the existing Python dataclass/result ergonomics.
- `done` Keep pure-Python reference helpers as fallback and teaching surfaces.

2. API parity and user-facing coverage
- Status: `done`
- `done` Expose the main Rust workload families through Python-facing configs:
  European, arithmetic Asian, down-and-out, lookback, basket, American put,
  Bermudan put, Heston, Merton, Gaussian UQ, MLMC/MLQMC, and parameter sweeps.
- `done` Keep method, sampling, technique, seed, and native-module choices explicit.
- `done` Return structured errors for missing native runtime or missing native
  function support instead of falling back silently.

3. Agent/MCP packaging
- Status: `done`
- `done` Package the current agent tool functions behind a stable
  MCP-compatible stdio server boundary.
- `done` Publish request/response schemas for the selected tools through
  `tools/list`.
- `done` Add execution limits, version metadata, and compatibility/failure
  policy metadata.

4. Production polish
- Status: `done`
- `done` Scope current Clippy lint debt so CI enforces warning-clean checks for
  `mc-schema` and `montepath-python`; `mc-core` hot-path lint debt remains documented
  for future refactoring.
- `done` Validate clean installation from wheel/source distribution; local
  wheel and source-distribution install smokes pass for the Python API/native
  extension.
- `done` Add automated smoke tests from installed wheel and source
  distributions, not only editable checkout.
- `done` Keep docs and examples synchronized with the native Python surface.

5. Benchmark and comparison hygiene
- Status: `done`
- `done` Keep release-mode benchmark tables current.
- `done` Keep unavailable competitor rows explicit.
- `done` Separate fair path-simulation comparisons from specialized terminal
  fast paths.

## Completion Status

Non-CUDA v1 polish is complete. Remaining accelerator work is tracked in the
deferred CUDA version below.

## Deferred To Later CUDA Version

Native CUDA execution is not part of non-CUDA v1. Later CUDA work includes:

- native CUDA launch and reductions
- deterministic GPU RNG stream partitioning
- CUDA hardware CI and release artifacts
- JAX/CuPy/PyTorch comparisons on actual NVIDIA hardware

Until that version lands, CUDA support must be described as staged,
diagnostic, or deferred rather than production native execution.
