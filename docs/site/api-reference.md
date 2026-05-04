# Python API Reference

## Configs

- `EuropeanCallConfig`
- `ArithmeticAsianCallConfig`
- `DownAndOutCallConfig`

Each config is an immutable dataclass with explicit seed, path count, step
count, and model parameters.

## Pricing Helpers

- `price_european_call(config=None, **overrides)`
- `price_arithmetic_asian_call(config=None, **overrides)`
- `price_down_and_out_call(config=None, **overrides)`

Each returns `PricingResult`:

- `price`
- `stderr`
- `manifest`
- `warnings`
- `explain()`
- `reproduce()`

## Greek Helpers

- `price_european_call_greeks(config=None, **overrides)`

Returns `GreekReport` with:

- `base_price`
- `greeks`
- `manifest`
- `warnings`
- `explain()`
- `reproduce()`

## Planning And Benchmarks

- `recommend_method(...)`
- `run_benchmarks(...)`

Benchmark helpers call the Rust benchmark harness and should be used for
local audit or release artifact generation, not as a low-latency API.

## Errors

Configuration problems raise `McConfigurationError`, which exposes:

- `code`
- `message`
- `suggestion`

See `docs/error-codes.md`.

