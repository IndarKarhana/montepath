# mc-library

Agent-native Monte Carlo runtime with CPU, NVIDIA CUDA, and Apple Metal execution paths.

## Current Stage

This repository is in early foundation stage.

- architecture and design docs are in `docs/`
- core engineering rules are in `docs/repository-rules.md`
- roadmap is in `roadmap.md`
- initial Rust workspace scaffolding is in `crates/`

## Core Principles

- architecture and design docs are source of truth
- roadmap is a living artifact and must be maintained
- test-driven development is the default workflow
- production-grade quality bar from day one
- fast and lightweight implementation choices

## Initial Workspace Layout

- `crates/mc-schema`: schema types, diagnostics, and validation
- `crates/mc-core`: core runtime interfaces and execution plan skeleton

## Next Steps

- finish schema validation depth and compatibility checks
- scaffold planner normalization and feature extraction
- establish CI for format, lint, and tests
