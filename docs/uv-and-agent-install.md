# uv And Agent Install Guide

This guide describes the intended public-alpha install and agent usage paths.

## Install With uv

After the package is published to PyPI:

```bash
uv pip install montepath
```

Inside a project managed by `uv`:

```bash
uv add montepath
```

From a local checkout:

```bash
uv pip install -e .
```

For development checks:

```bash
uv pip install -e ".[dev]"
uv run python -m pytest python/tests
uv run python -m build
```

The installed package includes:

- `montepath`: Python configs, helpers, benchmark/planner/agent surfaces
- `montepath._native`: Rust-backed CPU extension
- `montepath-mcp`: MCP-compatible stdio server for agents

macOS wheels built with the `metal-native` feature also expose strict Apple
Metal bridge functions for the current GBM option family. Linux and Windows
wheels remain CPU-native.

## Native Runtime Check

```bash
uv run python - <<'PY'
from montepath import native_runtime_status

print(native_runtime_status().as_dict())
PY
```

Expected installed-wheel shape:

```json
{
  "available": true,
  "module_name": "montepath._native",
  "version": "0.2.0",
  "supported_functions": [
    "price_european_call",
    "price_arithmetic_asian_call",
    "price_down_and_out_call",
    "price_european_call_metal",
    "price_arithmetic_asian_call_metal",
    "price_down_and_out_call_metal",
    "price_lookback_call",
    "price_basket_call",
    "price_american_put",
    "price_bermudan_put",
    "price_heston_european_call",
    "price_merton_jump_diffusion_call",
    "price_european_call_parameter_sweep",
    "gaussian_uncertainty_moments",
    "arithmetic_asian_mlmc",
    "validate_inventory_config",
    "simulate_inventory_policy"
  ],
  "reason": null
}
```

The `price_*_metal` functions appear only in Metal-enabled macOS native builds.
Code and agents should inspect `native_runtime_status()` or
`backend_capabilities()` instead of assuming they exist on every wheel.

## Use The MCP Server With uvx

Once published, agents can run the MCP server without manually creating a
project environment:

```bash
uvx --from montepath montepath-mcp
```

Equivalent long form:

```bash
uv tool run --from montepath montepath-mcp
```

From a local checkout during development:

```bash
uv run montepath-mcp
```

## Generic MCP Client Configuration

Use this shape for MCP clients that accept a command and argument list:

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

For local development:

```json
{
  "mcpServers": {
    "montepath-local": {
      "command": "uv",
      "args": ["run", "montepath-mcp"],
      "cwd": "/path/to/montepath"
    }
  }
}
```

## Agent Boundary

The MCP server exposes:

- `initialize`
- `tools/list`
- `tools/call`
- `ping`
- `health`

The tool list includes schemas exported from `montepath.export_json_schemas()`.
Tool calls return content blocks with JSON-serialized `ok=true` or `ok=false`
payloads.

Production-oriented agent tools include:

- `montepath.capabilities`: inspect installed CPU-native, Python-reference,
  Apple Metal, CUDA, MCP, and agent readiness
- `montepath.production_check`: validate a workload/config/backend request
  against production policy and benchmark evidence without executing it
- `montepath.validation_report`: inspect committed reference fixtures, caveats,
  and tolerance policy without running simulations
- `montepath.inventory.validate`: validate inventory semantics, backend support,
  and bounded execution policy without running simulation
- `montepath.inventory.simulate`: execute inventory through CPU native or Python
  reference policy with capped returned paths and optional bounded traces

Built-in limits:

- maximum request size: `1_000_000` bytes
- maximum `config.n_paths`: `1_000_000`
- inventory maximum paths: `100_000`
- inventory maximum periods: `1_000`
- inventory maximum path-period operations: `10_000_000`
- inventory maximum returned paths: `100`
- full benchmark execution is blocked through MCP

Run release/full benchmarks directly instead:

```bash
cargo run -p mc-bench --release -- --output benchmarks/release-results.json
```

## Publishing Notes

Before publishing:

```bash
cargo fmt --all -- --check
cargo test
PYTHONPATH=python python -m pytest python/tests
python -m build
python -m venv /tmp/montepath-wheel-smoke
/tmp/montepath-wheel-smoke/bin/python -m pip install dist/*.whl
/tmp/montepath-wheel-smoke/bin/python python/tests/installed_package_smoke.py
```

Publishing should use the GitHub Actions trusted-publishing workflow at
`.github/workflows/publish-pypi.yml` or an explicit PyPI token. The workflow
publishes the source distribution plus manylinux x86_64, macOS universal2, and
Windows x64 binary wheels. Trusted-publishing reruns prune filenames already
present on PyPI and check the package index so missing supplemental wheels can
be uploaded without republishing artifacts that already exist. Do not publish
from an unverified local environment.

For the first trusted-publishing release, create a PyPI pending publisher with:

- PyPI project name: `montepath`
- owner: `IndarKarhana`
- repository: `montepath`
- workflow: `publish-pypi.yml`
- environment: `pypi`

After publishing, verify public availability through `uv`:

```bash
uv pip install montepath
uvx --from montepath montepath-mcp
```
