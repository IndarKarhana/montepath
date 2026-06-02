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

Apple Metal execution is currently exposed through Rust feature gates and
hardware-aware benchmark runs:

```bash
cargo test -p mc-core --features metal-native
cargo run -p mc-bench --release --features metal-native -- --output benchmarks/release-results.json
```

Python packaging for native Metal wheels is planned. Until then, Python users
should treat Metal as a benchmarked Rust runtime capability, not a pip-installed
accelerator profile.

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
