# Agent Integration Plan

This document defines how the library should evolve so AI agents can use it safely, predictably, and efficiently.

## Objective

Make the library easy to wrap as tools without forcing agents to reverse-engineer the codebase.

The target state is:

- clear public entry points
- typed request and response objects
- explicit determinism and backend-support semantics
- machine-readable planning and execution outputs
- a stable catalog of callable surfaces
- MCP-compatible tool wrappers that expose these same typed contracts without
  requiring an agent to scrape prose or infer unsupported behavior

## Principles

1. Structured over magical
- Tool-facing APIs should prefer typed configs and result structs.

2. Explainability is part of the API
- If the planner selects a backend, chunking strategy, or fallback, the reason should be inspectable.

3. Unsupported is a first-class outcome
- Agents should get explicit unsupported states, not silent fallback.

4. Narrow fast path first
- Start with a small set of highly reliable tool surfaces before broadening.

## Recommended Tool Surface Layers

### Layer 1: Analysis

Safe, deterministic surfaces that inspect inputs without running the full runtime.

Examples:

- schema validation
- schema compatibility checks
- feature extraction
- execution planning
- benchmark report analysis

### Layer 2: Reference execution

Narrow execution surfaces with deterministic seed handling and explicit workload scope.

Examples:

- CPU European call reference execution
- CPU American and Bermudan put Longstaff-Schwartz reference execution
- future explain-plan helper

### Layer 3: General runtime execution

Higher-level runtime calls once backend contracts, manifests, and cross-backend guarantees are mature.

## Near-Term Deliverables

1. Keep `docs/function-catalog.md` current.
2. Add a stable explainability helper around `ExecutionPlan`.
3. Add a machine-readable run manifest format for runtime outputs.
4. Add Python-facing wrappers that preserve the same structured semantics.
5. Keep the MCP-compatible server boundary aligned with the public tool
   manifest, JSON-schema export, execution limits, and version metadata.

Current status:

- `explain_execution_plan` exists as the first lightweight planner explanation helper
- Python agent tool manifest and JSON-schema export are live through
  `agent_tool_manifest()` and `export_json_schemas()`
- Python agent-safe wrappers are live for validate, recommend, plan, execute,
  compare, benchmark, and reproduce
- `agent-run.v1` manifests attach seed, backend, method, estimator, config,
  build, hardware, warnings, reference, and determinism metadata to wrapper
  outputs
- `montepath-mcp` packages the current agent tools behind a dependency-free
  MCP-compatible stdio server with request/response schemas, execution limits,
  version metadata, health checks, and structured failure payloads.
- Inventory exposes dedicated `montepath.inventory.validate` and
  `montepath.inventory.simulate` tools with path, period, operation, trace, and
  returned-result bounds plus explicit backend selection.
- Remaining agent work is now integration hardening: client-specific examples,
  broader smoke coverage, authentication or sandbox policy where deployments
  need it, and compatibility testing against MCP clients.

## Tool-Readiness Checklist

A surface is ready to be wrapped as an agent tool when:

- inputs are typed and explicit
- outputs are structured
- errors are actionable
- determinism expectations are documented
- unsupported states are explicit
- source location is documented in `docs/function-catalog.md`
- tests cover the contract

## What We Should Avoid

- global implicit state
- callback-heavy execution APIs
- planner decisions that cannot be explained
- backend behavior that changes silently by environment
- free-form text as the only result channel
