# Agent Tools

The Python package exposes stable, JSON-serializable agent wrappers:

```python
from montepath import (
    agent_capabilities,
    agent_execute,
    agent_inventory_simulate,
    agent_plan,
    agent_production_check,
    agent_tool_manifest,
)

print(agent_tool_manifest())
print(agent_capabilities({})["result"]["native_runtime"])

plan = agent_plan({
    "workload": "european_call",
    "config": {"n_paths": 128, "n_steps": 4, "seed": 5}
})

run = agent_execute({
    "workload": "european_call",
    "config": {"n_paths": 128, "n_steps": 4, "seed": 5}
})

preflight = agent_production_check({
    "workload": "european_call",
    "config": {"n_paths": 128, "n_steps": 4, "seed": 5},
    "backend": "auto"
})

inventory = agent_inventory_simulate({
    "config": {
        "n_paths": 10_000,
        "n_periods": 104,
        "trace": {"path_indices": [0], "max_periods": 12},
    },
    "backend": "auto",
    "max_returned_paths": 10,
})
```

Every response contains either:

- `ok=true` plus structured payload and manifest, or
- `ok=false` plus structured diagnostics and manifest.

See:

- `docs/agent-tooling.md`
- `docs/agent-examples.json`

## MCP Server

Installed distributions include the `montepath-mcp` console entry point. It
serves the same agent tools over a dependency-free MCP-compatible stdio
boundary with schemas, version metadata, execution limits, and structured
failure responses.

After publication, launch with:

```bash
uvx --from montepath montepath-mcp
```

Generic MCP client configuration:

```json
{
  "mcpServers": {
    "montepath": {
      "command": "uvx",
      "args": ["--from", "montepath", "montepath-mcp"]
    }
  }
}
```

Use `tools/list` to discover schemas and `tools/call` to execute a tool. Full
benchmark execution is intentionally blocked through MCP; run the benchmark
harness directly for release artifacts.

The dedicated `montepath.inventory.validate` and
`montepath.inventory.simulate` tools expose inventory-specific schemas,
backend policy, request limits, bounded traces, and capped path-level output.
The complete aggregate summary is returned even when path output is truncated.
