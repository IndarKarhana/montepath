# User-Friendliness Research and Implementation

This document summarizes what established libraries do well for usability and how we are applying those patterns.

## Sources

- Pydantic Error Handling: https://docs.pydantic.dev/latest/errors/errors/
- scikit-learn estimator developer conventions: https://scikit-learn.org/stable/developers/develop.html
- scikit-learn Pipeline API behavior: https://scikit-learn.org/stable/modules/generated/sklearn.pipeline.Pipeline.html
- NumPy random Generator guidance: https://numpy.org/doc/1.26/reference/random/generator.html
- NumPy multithreaded random generation notes: https://numpy.org/doc/2.4/reference/random/multithreading.html
- JAX benchmarking guidance: https://docs.jax.dev/en/latest/benchmarking.html
- CuPy performance best practices: https://docs.cupy.dev/en/stable/user_guide/performance.html
- Python Packaging versioning discussion: https://packaging.python.org/en/latest/discussions/versioning/

## What Makes Libraries User-Friendly

1. Actionable validation output
- Pydantic emphasizes structured errors (`errors()`, locations, human-readable and machine-readable detail).
- Implementation in this repo:
  - diagnostics include code, location, message, optional suggestion
  - validation errors now include fix suggestions where possible

2. Predictable and consistent API conventions
- scikit-learn’s API consistency is a major usability factor (`fit` / `predict` style and consistent object behavior).
- Implementation direction in this repo:
  - consistent typed structs and normalized planner/run config
  - explicit, inspectable `ExecutionPlan` and decision report

3. Simple and explicit object creation patterns
- Widely adopted libraries provide straightforward constructors with sensible defaults.
- Implementation in this repo:
  - `SimulationSpecBuilder` to reduce schema construction friction

4. Reproducibility-first randomness model
- NumPy docs emphasize explicit RNG objects and seeding semantics.
- Implementation in this repo:
  - deterministic CPU RNG with explicit seed in runtime config

5. Honest benchmarking practices
- JAX and CuPy docs call out warm-up, compile overhead, and asynchronous behavior concerns.
- Implementation in this repo:
  - competitor benchmark harness explicitly warms Numba
  - benchmark report separates benchmark identities and metric fields
  - competitiveness plan generated when slower than competitors

6. Versioning clarity
- Packaging guidance highlights compatibility signaling.
- Implementation in this repo:
  - schema compatibility checks against supported schema version

## Implemented UX Improvements (Current)

- `SimulationSpecBuilder` for ergonomic schema creation
- diagnostic suggestions via `Diagnostic::with_suggestion`
- compatibility validation with explicit unsupported-version errors
- benchmark reporting that marks unavailable competitor libraries instead of silently omitting

## Next UX Implementation Steps

- add user-facing Python API layer with builder and dataclass/Pydantic model parity
- add error codes documentation table and troubleshooting pages
- add `explain_plan()` style textual helper for easier planner interpretation
- add installation profiles (`cpu`, `cuda`, `metal`) with clear messaging
