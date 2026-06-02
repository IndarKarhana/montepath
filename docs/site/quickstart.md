# Quickstart

## Install

After the package is published:

```bash
uv pip install montepath
```

Inside a `uv` project:

```bash
uv add montepath
```

From a checkout:

```bash
uv pip install -e .
```

CPU-only user profile:

```bash
uv pip install "montepath[cpu]"
```

Apple Metal and future CUDA profiles are documented in
`docs/install-profiles.md`. Native accelerator execution still depends on local
hardware and Rust feature gates.

## Price A European Call

```python
from montepath import EuropeanCallConfig, price_european_call

cfg = EuropeanCallConfig(n_paths=20_000, n_steps=64, seed=42)
result = price_european_call(cfg)

print(result.price)
print(result.stderr)
print(result.explain())
print(result.manifest)
```

## Price A Path-Dependent Workload

```python
from montepath import price_arithmetic_asian_call

result = price_arithmetic_asian_call(n_paths=20_000, n_steps=64, seed=42)
print(result.explain())
```

## Get Greeks

```python
from montepath import price_european_call_greeks

report = price_european_call_greeks()
print(report.greeks)
print(report.explain())
```

The Python helpers are dependency-free reference UX helpers. Use benchmark
artifacts for performance claims.

## Check Production Capability

```python
from montepath import production_status, select_backend

status = production_status()
print(status["native_runtime"]["available"])

selection = select_backend("european_call", backend="auto")
print(selection.backend_id, selection.execution_mode)
```

For production Python use, `cpu_native` is the fast installed-package path when
`montepath._native` is available. Apple Metal is currently validated through
Rust feature-gated hardware workflows, not the PyPI Python bridge. CUDA native
execution is deferred.

## Run The MCP Server

After publication:

```bash
uvx --from montepath montepath-mcp
```

From a checkout:

```bash
uv run montepath-mcp
```

See `docs/uv-and-agent-install.md` for MCP client configuration.
