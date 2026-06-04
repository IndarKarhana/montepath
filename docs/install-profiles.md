# Install Profiles

## CPU

Use this for local notebooks, CI, and first-time users:

```bash
uv pip install "montepath[cpu]"
```

Equivalent pip command:

```bash
python -m pip install "montepath[cpu]"
```

Current wheels include the dependency-free Python UX helpers plus the
Rust-backed `montepath._native` CPU extension. The `cpu` extra is intentionally
empty for now so downstream users can depend on a stable profile while CPU
native execution remains part of the base wheel.

Installed packages expose production preflight helpers:

```python
from montepath import backend_capabilities, production_status, select_backend

print(production_status()["native_runtime"])
print(select_backend("european_call").backend_id)
```

These helpers are also available to agents through `montepath.capabilities` and
`montepath.production_check`.

## Apple Metal

Supported macOS wheels are built with the `metal-native` feature and expose
strict Python bridge functions for the documented GBM option family. Inspect
`backend_capabilities()` because Metal availability still depends on the host,
wheel, and requested workload.

Metal development and hardware-aware benchmark runs use:

```bash
cargo test -p mc-core --features metal-native
cargo run -p mc-bench --release --features metal-native -- --output benchmarks/release-results.json
```

Inventory and the wider CPU workload catalog do not silently fall back when
Metal is requested but unsupported.

## Future CUDA

CUDA native execution is deferred. The future install shape is reserved as:

```bash
uv pip install "montepath[cuda]"
```

Until native CUDA execution ships, CUDA requests must remain explicit about
unsupported native execution instead of silently falling back.

## Competitor Benchmark Extras

QuantLib competitor rows can be populated with:

```bash
python -m pip install -r benchmarks/competitors/requirements-quantlib.txt
```
