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

This catches most correctness regressions even before native GPU kernels exist.

## Layer 4: Hardware-Only Validation

When native CUDA or Metal kernels land, we add dedicated hardware validation jobs.

These should run on:

- a self-hosted NVIDIA runner for CUDA
- a self-hosted Apple Silicon runner for Metal

Hardware jobs should validate:

- native compile success
- native execution correctness vs CPU reference
- deterministic or statistically bounded reproducibility
- transfer and kernel timing telemetry
- benchmark results against selected competitors

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
- native Metal kernel correctness
- native Metal performance

### After First Native CUDA Kernel

Required before calling CUDA support real:

- native CUDA smoke tests on hardware
- CPU-vs-CUDA numerical agreement tests
- chunked execution agreement tests
- compile-plus-run artifact reuse tests
- benchmark evidence against at least one meaningful CUDA baseline

### After First Native Metal Kernel

Required before calling Metal support real:

- native Metal smoke tests on hardware
- CPU-vs-Metal numerical agreement tests
- unified-memory and chunking validation
- benchmark evidence on Apple Silicon

## Benchmarking Without Local CUDA

If you do not have a CUDA device, we do not claim native CUDA performance results.

What we can still do:

- prepare benchmark manifests
- validate competitor harness structure
- benchmark CPU baselines
- benchmark fallback path overhead through the backend abstraction
- keep planner and backend contracts stable until hardware is available

## Practical Plan For This Repo

1. Keep CPU reference and fair benchmark suite green on every change.
2. Keep GPU backend conformance tests green on every change.
3. Use GitHub Actions CPU CI for default validation.
4. Add manual hardware workflows now so the repo is ready when a CUDA or Apple runner is available.
5. Only publish native GPU benchmark claims after running hardware workflows.

## Honest Communication Rule

We should use the following language consistently:

- "GPU-ready backend surface" is okay for fallback-backed backend integration
- "native CUDA support" is not okay until kernels execute on NVIDIA hardware
- "GPU benchmark result" is not okay unless measured on real hardware

This rule protects credibility and keeps benchmark claims trustworthy.
