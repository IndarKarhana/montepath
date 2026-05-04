# Accelerator Competitor Benchmarking

This document covers the Phase 5 accelerator credibility work that can be done
before native CUDA execution ships.

## Scope

Current status:

- native Apple Metal execution exists for selected GBM workload families
- native CUDA launch and reduction are still deferred
- JAX, CuPy, and PyTorch competitor lanes are executable when packages and GPU
  hardware are available
- CPU-only machines report accelerator rows as explicit unavailable results

## Competitor Environment Manifests

Environment manifests live in `benchmarks/competitors/environments/`:

- `numpy.json`
- `numba.json`
- `scipy-qmc.json`
- `quantlib.json`
- `jax.json`
- `cupy.json`
- `pytorch.json`

Accelerator competitor dependencies are listed in:

- `benchmarks/competitors/requirements-accelerators.txt`

## Accelerator Rows

The Python competitor script reports these GPU methodologies:

- `terminal_distribution_gpu_jax`
- `terminal_distribution_gpu_cupy`
- `terminal_distribution_gpu_torch`

Each row includes:

- availability
- runtime
- price
- stderr
- warmup time
- compile time when applicable
- execution time
- memory estimate or observed peak memory
- device label
- reproducibility note

If a package or GPU device is unavailable, the row remains present with
`available=false` and telemetry keys set to `null`.

## Hardware Workflow

The manual workflow `.github/workflows/accelerator-competitors.yml` is intended
for self-hosted CUDA runners. It installs the accelerator competitor profile,
runs the competitor script, requires populated JAX/CuPy/PyTorch rows, and
uploads `benchmarks/accelerator-competitors-results.json`.

Do not claim accelerator wins unless this workflow or an equivalent hardware
run produced the artifact.

## Current Limitation

These competitor rows are accelerator baselines, not native CUDA support for
`mc-library`. Native CUDA remains deferred until CUDA kernels, reductions, RNG
stream partitioning, and hardware CI are implemented and measured.

