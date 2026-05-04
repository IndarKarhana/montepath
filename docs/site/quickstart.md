# Quickstart

## Install

From a checkout:

```bash
python -m pip install -e .
```

CPU-only user profile:

```bash
python -m pip install "mc-library[cpu]"
```

Apple Metal and future CUDA profiles are documented in
`docs/install-profiles.md`. Native accelerator execution still depends on local
hardware and Rust feature gates.

## Price A European Call

```python
from mc_library import EuropeanCallConfig, price_european_call

cfg = EuropeanCallConfig(n_paths=20_000, n_steps=64, seed=42)
result = price_european_call(cfg)

print(result.price)
print(result.stderr)
print(result.explain())
print(result.manifest)
```

## Price A Path-Dependent Workload

```python
from mc_library import price_arithmetic_asian_call

result = price_arithmetic_asian_call(n_paths=20_000, n_steps=64, seed=42)
print(result.explain())
```

## Get Greeks

```python
from mc_library import price_european_call_greeks

report = price_european_call_greeks()
print(report.greeks)
print(report.explain())
```

The Python helpers are dependency-free reference UX helpers. Use benchmark
artifacts for performance claims.

