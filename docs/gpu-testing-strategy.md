# GPU Testing Strategy

This document defines how we test CUDA and Metal support when local development machines do not have native GPU hardware.

## Purpose

We need a testing strategy that is:

- honest about what is and is not natively accelerated
- useful on CPU-only developer machines
- capable of scaling to real CUDA and Metal hardware validation later
- strict enough to protect correctness, reproducibility, and benchmark integrity

## Short Answer

If you do not have a CUDA device locally, we still test most of the GPU stack in four layers:

1. CPU reference tests
2. backend contract and fallback tests
3. compile and integration checks in CI
4. hardware-only tests on dedicated runners later

That means local development can still validate planner behavior, backend shape, artifact compatibility, deterministic execution, and numerical equivalence to the CPU reference runtime.

It now also validates feature-gated native backend staging through:

- `cargo test -p mc-core --features cuda-native`
- `cargo test -p mc-core --features metal-native`
- `cargo test -p mc-core --features \"cuda-native metal-native\"`

For CUDA specifically, this now also covers:

- a real staged `.cu` kernel source file
- PTX compile-attempt metadata in compiled artifacts
- environment-sensitive `nvcc` probing without requiring a CUDA device

For Metal specifically, this now also covers:

- a real staged `.metal` shader source file
- `.air` / `.metallib` compile-attempt metadata in compiled artifacts
- environment-sensitive Apple toolchain probing without requiring successful GPU execution
- a native macOS runtime execution path validated against a CPU reference using the same generated normals
- a benchmark path that measures the current native Apple GPU execution end-to-end on macOS

## Layer 1: CPU Reference Truth

The CPU runtime is the numerical source of truth for the current European-call workload family.

We use it to validate:

- deterministic RNG behavior
- Black-Scholes consistency
- variance-reduction correctness
- baseline fair step-wise execution

Rule:

- no GPU path is considered valid unless it matches the CPU reference within the declared reproducibility tier

## Layer 2: GPU Backend Conformance Without Native GPU Hardware

Current CUDA and Metal backends support explicit delegated CPU fallback execution.

This gives us real tests for:

- backend discovery shape
- support reporting
- artifact compilation compatibility
- execution routing through backend interfaces
- deterministic backend execution
- numerical equivalence to the CPU step-wise reference path
- chunking and workload-shape validation

This is the main way we test GPU-facing code on machines without CUDA or Metal hardware.

## Layer 3: CI on Standard Machines

Standard CI should run on ordinary CPU-only GitHub runners and verify:

- formatting
- unit tests
- integration tests
- benchmark harness execution
- feature-gated host-side native backend staging builds

This catches most correctness regressions even before native GPU kernels exist.

## Layer 4: Hardware-Only Validation

Dedicated hardware validation jobs live in
`.github/workflows/gpu-hardware.yml`. The workflow is manual and accepts a
backend selector (`all`, `cuda`, or `metal`) so runner availability is explicit.

These should run on:

- a self-hosted NVIDIA runner for CUDA
- a self-hosted Apple Silicon runner for Metal

The current hardware jobs validate:

- CUDA staging: NVIDIA environment probe, `nvcc` availability,
  `cuda-native` feature-gated tests, and a compact benchmark diagnostic artifact
  that must not contain native CUDA performance rows before native launch ships.
- Apple Metal: Metal toolchain probe, `metal-native` feature-gated tests, a
  full native Metal benchmark artifact, and required native Metal benchmark row
  validation for the current GBM workload families.

Later CUDA hardware jobs must additionally validate native launch,
device-side reductions, deterministic GPU RNG stream partitioning, and
CPU-vs-CUDA numerical agreement.

## Testing Policy By Development Stage

### Current Stage

Allowed on CPU-only machines:

- full local test suite
- full benchmark suite
- backend contract validation
- fallback execution validation

Not yet provable locally:

- native CUDA kernel correctness
- native CUDA performance
- native Metal performance

Provable on Apple Silicon with `metal-native` enabled:

- native Metal kernel smoke execution
- CPU-vs-Metal numerical agreement for the first staged workload
- native Metal steady-state benchmark results for the first staged workload family

### After First Native CUDA Kernel

Required before calling CUDA support real:

- native CUDA smoke tests on hardware
- CPU-vs-CUDA numerical agreement tests
- chunked execution agreement tests
- compile-plus-run artifact reuse tests
- benchmark evidence against at least one meaningful CUDA baseline

### After First Native Metal Kernel

Required before calling Metal support production-grade:

- validate unified-memory and chunking behavior under larger workloads
- extend native execution beyond the first staged workload and beyond standard sampling
- benchmark evidence across multiple Apple Silicon classes and workload sizes

## Benchmarking Without Local CUDA

If you do not have a CUDA device, we do not claim native CUDA performance results.

What we can still do:

- prepare benchmark manifests
- validate competitor harness structure
- benchmark CPU baselines
- benchmark fallback path overhead through the backend abstraction
- keep planner and backend contracts stable until hardware is available
- report JAX, CuPy, and PyTorch competitor rows as explicit unavailable rows
  when packages or GPU devices are missing
- run `.github/workflows/accelerator-competitors.yml` on a self-hosted CUDA
  runner to populate hardware-backed competitor artifacts

## Practical Plan For This Repo

1. Keep CPU reference and fair benchmark suite green on every change.
2. Keep GPU backend conformance tests green on every change.
3. Use GitHub Actions CPU CI for default validation.
4. Run `.github/workflows/gpu-hardware.yml` on dedicated CUDA and Apple Silicon
   runners before publishing hardware-backed accelerator artifacts.
5. Only publish native GPU benchmark claims after running hardware workflows.

## Honest Communication Rule

We should use the following language consistently:

- "GPU-ready backend surface" is okay for fallback-backed backend integration
- "native CUDA support" is not okay until kernels execute on NVIDIA hardware
- "GPU benchmark result" is not okay unless measured on real hardware

This rule protects credibility and keeps benchmark claims trustworthy.
