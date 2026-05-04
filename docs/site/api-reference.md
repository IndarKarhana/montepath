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
- `load_planner_evidence(...)`
- `measured_winner_database(...)`
- `cost_frontier(workload, ...)`
- `compare_methods(workload, ...)`
- `why_not_faster(workload, method_id=..., ...)`
- `mlmc_error_calibration(workload="arithmetic_asian_call", ...)`
- `run_benchmarks(...)`

## Agent Tools

- `agent_tool_manifest()`
- `export_json_schemas()`
- `agent_validate(request)`
- `agent_recommend(request)`
- `agent_plan(request)`
- `agent_execute(request)`
- `agent_compare(request)`
- `agent_benchmark(request=None)`
- `agent_reproduce(request)`
- `agent_planner_evidence(request=None)`
- `agent_cost_frontier(request)`
- `agent_compare_methods(request)`
- `agent_why_not_faster(request)`
- `agent_mlmc_calibration(request=None)`

Benchmark helpers call the Rust benchmark harness and should be used for
local audit or release artifact generation, not as a low-latency API.
Planner evidence helpers read benchmark artifacts and are deterministic for a
fixed artifact, but their timing claims remain hardware-local.

## Errors

Configuration problems raise `McConfigurationError`, which exposes:

- `code`
- `message`
- `suggestion`

See `docs/error-codes.md`.
