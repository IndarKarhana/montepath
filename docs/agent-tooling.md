# Agent Tooling Surface

Phase 4 adds a stable Python-facing tool layer for AI agents. The public
functions accept JSON-serializable dictionaries and return JSON-serializable
dictionaries with `ok`, structured diagnostics, and reproducibility manifests.

## Tool Manifest

Use:

```python
from mc_library import agent_tool_manifest, export_json_schemas

manifest = agent_tool_manifest()
schemas = export_json_schemas()
```

The manifest schema version is `agent-tools.v1`. The run manifest schema
version is `agent-run.v1`.

Available tools:

| Tool | Purpose | Determinism |
| --- | --- | --- |
| `mc.validate` | Validate a supported workload request without executing simulation. | deterministic |
| `mc.recommend` | Recommend method, sampling, and technique. | deterministic |
| `mc.plan` | Build a dry-run execution plan with cost and caveat metadata. | deterministic |
| `mc.execute` | Execute a narrow Python reference workload. | deterministic for same config and seed |
| `mc.compare` | Compare fast and accuracy-oriented method choices. | deterministic |
| `mc.benchmark` | Return benchmark command metadata by default, or run when explicitly requested. | environment-sensitive when executed |
| `mc.reproduce` | Build a reproduction recipe from a run manifest. | deterministic |
| `mc.planner_evidence` | Load measured planner accuracy, winner records, and fixture references. | deterministic |
| `mc.cost_frontier` | Return measured method/backend cost frontier rows for a workload. | deterministic |
| `mc.compare_methods` | Compare measured method/runtime tradeoffs for a workload. | deterministic |
| `mc.why_not_faster` | Explain why a requested method is not the measured recommendation. | deterministic |
| `mc.mlmc_calibration` | Report estimated-vs-realized MLMC/MLQMC calibration evidence. | deterministic |

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

## Supported Workloads

- `european_call`
- `arithmetic_asian_call`
- `down_and_out_call`
- `european_call_greeks`

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
    "tool": "mc.plan",
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
from mc_library import agent_compare_methods, agent_why_not_faster

comparison = agent_compare_methods({"workload": "arithmetic_asian_call"})
explanation = agent_why_not_faster(
    {"workload": "european_call", "method_id": "scrambled_sobol"}
)
```

The response includes the benchmark artifact id, measured runtime frontier, and
the same `agent-run.v1` manifest shape as the execution tools. Timing claims
remain hardware-local; rerun benchmarks on the target machine before production
tuning.
