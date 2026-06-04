# Agent Tooling Surface

Phase 4 adds a stable Python-facing tool layer for AI agents. The public
functions accept JSON-serializable dictionaries and return JSON-serializable
dictionaries with `ok`, structured diagnostics, and reproducibility manifests.

## Tool Manifest

Use:

```python
from montepath import agent_tool_manifest, export_json_schemas

manifest = agent_tool_manifest()
schemas = export_json_schemas()
```

The manifest schema version is `agent-tools.v1`. The run manifest schema
version is `agent-run.v1`.

## MCP Server

Installed packages expose a dependency-free MCP-compatible stdio server:

```bash
montepath-mcp
```

The server handles JSON-RPC-style `initialize`, `tools/list`, `tools/call`,
`ping`, and `health` messages. `tools/list` embeds the same request schemas
returned by `export_json_schemas()`. `tools/call` delegates to the stable
Python agent wrappers and returns MCP content blocks whose text is a
JSON-serialized tool response.

Server metadata uses schema version `mcp-server.v1` and includes:

- package and protocol version
- request-size and execution limits
- benchmark execution policy
- structured failure policy

Current limits:

- maximum request size: `1_000_000` bytes
- maximum `config.n_paths` through MCP: `1_000_000`
- inventory maximum paths: `100_000`
- inventory maximum periods: `1_000`
- inventory maximum path-period operations: `10_000_000`
- inventory maximum returned paths: `100`
- full benchmark execution is blocked through MCP; use dry-run benchmark
  metadata or run the benchmark harness directly

Protocol failures return JSON-RPC errors. Tool failures return content with
`ok=false` and structured diagnostics whenever possible.

Available tools:

| Tool | Purpose | Determinism |
| --- | --- | --- |
| `montepath.validate` | Validate a supported workload request without executing simulation. | deterministic |
| `montepath.capabilities` | Report installed CPU, Metal, CUDA, Python, MCP, and agent capability status. | deterministic |
| `montepath.production_check` | Validate backend policy and summarize benchmark evidence without executing simulation. | deterministic |
| `montepath.validation_report` | Report committed reference fixtures, caveat workloads, and tolerance policy. | deterministic |
| `montepath.recommend` | Recommend method, sampling, and technique. | deterministic |
| `montepath.plan` | Build a dry-run execution plan with cost and caveat metadata. | deterministic |
| `montepath.execute` | Execute a narrow Python reference workload. | deterministic for same config and seed |
| `montepath.compare` | Compare fast and accuracy-oriented method choices. | deterministic |
| `montepath.benchmark` | Return benchmark command metadata by default, or run when explicitly requested. | environment-sensitive when executed |
| `montepath.reproduce` | Build a reproduction recipe from a run manifest. | deterministic |
| `montepath.planner_evidence` | Load measured planner accuracy, winner records, and fixture references. | deterministic |
| `montepath.cost_frontier` | Return measured method/backend cost frontier rows for a workload. | deterministic |
| `montepath.compare_methods` | Compare measured method/runtime tradeoffs for a workload. | deterministic |
| `montepath.why_not_faster` | Explain why a requested method is not the measured recommendation. | deterministic |
| `montepath.mlmc_calibration` | Report estimated-vs-realized MLMC/MLQMC calibration evidence. | deterministic |
| `montepath.inventory.validate` | Validate inventory semantics, limits, and backend support. | deterministic |
| `montepath.inventory.simulate` | Execute bounded CPU-native or Python-reference inventory simulation. | deterministic for same config and seed |

## Reproducibility Manifest

Every wrapper returns a manifest with:

- `schema_version`
- `tool`
- `workload`
- `seed`
- `backend`
- `method`
- `estimator`
- `config`
- `build`
- `hardware`
- `warnings`
- `reference`
- `determinism`

The manifest is intentionally explicit about Python reference execution. It
does not claim Rust hot-path performance. Benchmark artifacts remain the source
of truth for timing claims.

## Production Capability Tools

Use `montepath.capabilities` before asking an agent to execute work in a fresh
environment. It reports:

- whether the Rust CPU native extension is installed
- which workloads the native bridge exposes
- that Python reference helpers are available for narrow inspectable examples
- whether Apple Metal Python bridge functions are present in the installed
  native module, or why Metal is unavailable on this host/build
- that CUDA native execution is deferred

Use `montepath.production_check` before production execution. It validates the
request against backend policy, returns explicit diagnostics for unavailable
Metal/CUDA requests, and includes a benchmark-artifact summary. It does not run
the simulation.

Use `montepath.validation_report` before accuracy-sensitive agent workflows. It
summarizes committed reference fixtures, caveat workloads, and tolerance policy
without executing simulations or making timing claims.

## Supported Workloads

- `european_call`
- `arithmetic_asian_call`
- `down_and_out_call`
- `european_call_greeks`
- `inventory_policy` through the dedicated inventory tools and production API

Unsupported workloads return `ok=false` with a diagnostic code such as
`MC_AGENT_UNSUPPORTED_WORKLOAD`.

## Exact Payload Example

Request:

```json
{
  "workload": "european_call",
  "config": {
    "n_paths": 128,
    "n_steps": 4,
    "seed": 5
  },
  "preferences": {
    "prefer_accuracy": false
  }
}
```

Dry-run plan response shape:

```json
{
  "ok": true,
  "plan": {
    "dry_run": true,
    "workload": "european_call",
    "backend": "python_reference",
    "method_id": "control_variates",
    "sampling": "pseudorandom",
    "technique": "control_variate",
    "estimated_cost": {
      "confidence": "low",
      "estimated_runtime_ms": null,
      "estimated_peak_memory_mb": 0.004
    },
    "rejected_methods": [
      {
        "method_id": "multilevel_monte_carlo",
        "reason": "current MLMC recommendation surface is arithmetic Asian focused"
      }
    ],
    "notes": [
      "Dry-run planning does not execute simulation.",
      "Python reference backend is selected for stable agent examples; Rust benchmark artifacts carry performance claims."
    ]
  },
  "manifest": {
    "schema_version": "agent-run.v1",
    "tool": "montepath.plan",
    "workload": "european_call",
    "seed": 5,
    "backend": "python_reference",
    "method": "control_variates",
    "estimator": null,
    "config": {
      "n_paths": 128,
      "n_steps": 4,
      "seed": 5
    },
    "warnings": [
      "antithetic may be useful when control-variate assumptions are unavailable",
      "structured sampling can improve estimator quality but is slower in current benchmarks"
    ],
    "reference": null
  }
}
```

The real response also includes `build`, `hardware`, and `determinism` fields.
Those values vary by environment.

## Planner Evidence Tools

Planner intelligence tools read benchmark artifacts instead of making hidden
performance assumptions. Example:

```python
from montepath import agent_compare_methods, agent_why_not_faster

comparison = agent_compare_methods({"workload": "arithmetic_asian_call"})
explanation = agent_why_not_faster(
    {"workload": "european_call", "method_id": "scrambled_sobol"}
)
```

The response includes the benchmark artifact id, measured runtime frontier, and
the same `agent-run.v1` manifest shape as the execution tools. Timing claims
remain hardware-local; rerun benchmarks on the target machine before production
tuning.
