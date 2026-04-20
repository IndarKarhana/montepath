# Repository Rules

These rules are mandatory for all work in this repository.

## 1. Architecture and Design Are Source of Truth

- `docs/architecture-plan.md` and companion design docs are authoritative.
- New code must align with documented architecture.
- If code and docs diverge, either:
  - update code to match docs, or
  - update docs and explain why the architecture changed.

## 2. Roadmap Must Be Maintained

- `roadmap.md` must be updated whenever scope, sequencing, or progress changes.
- Every significant PR should include roadmap status updates.
- Do not leave stale milestone states.

## 3. Test-Driven Development Is Default

Required workflow for feature work:

1. write or update failing tests
2. implement minimal code to pass tests
3. refactor while preserving test coverage
4. run full relevant test suite

Exceptions are allowed only for:

- scaffolding and build-system bootstrap
- emergency fixes where immediate stabilization is required

In those cases, missing tests must be added immediately after stabilization.

## 4. Production-Grade Quality Bar

All code should be treated as production code.

Minimum standards:

- clear APIs and type safety
- deterministic behavior where required
- explicit error handling with actionable diagnostics
- no silent fallback that hides correctness risks
- maintainable module boundaries

## 5. Performance and Lightweight Constraints

This project targets a fast and lightweight runtime.

Rules:

- avoid unnecessary dependencies
- avoid avoidable allocations on hot paths
- keep abstractions zero-cost where practical
- measure before and after non-trivial performance changes

## 6. Explainability and Observability

Planner and runtime choices should be inspectable.

- prefer structured diagnostics over plain strings
- include enough metadata for reproducibility and debugging
- expose decision reasoning for backend selection

## 7. Backward and Forward Discipline

- preserve schema compatibility where possible
- use explicit versioning for breaking changes
- include migration notes when behavior changes

## 8. Definition of Done

A task is done when:

- code is implemented
- tests pass
- docs are updated
- roadmap status is updated
- known risks are documented
