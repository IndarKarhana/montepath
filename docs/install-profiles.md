# Install Profiles

## CPU

Use this for local notebooks, CI, and first-time users:

```bash
python -m pip install "mc-library[cpu]"
```

Current Python UX helpers are dependency-free. The `cpu` extra is intentionally
empty for now so downstream users can depend on a stable profile before
compiled bindings land.

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
python -m pip install "mc-library[cuda]"
```

Until native CUDA execution ships, CUDA requests must remain explicit about
unsupported native execution instead of silently falling back.

## Competitor Benchmark Extras

QuantLib competitor rows can be populated with:

```bash
python -m pip install -r benchmarks/competitors/requirements-quantlib.txt
```

