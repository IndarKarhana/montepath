# Python API Reference

## Configs

- `EuropeanCallConfig`
- `ArithmeticAsianCallConfig`
- `DownAndOutCallConfig`
- `LookbackCallConfig`
- `BasketCallConfig`
- `AmericanPutConfig`
- `BermudanPutConfig`
- `HestonEuropeanCallConfig`
- `MertonJumpDiffusionCallConfig`
- `GaussianUncertaintyConfig`
- `ArithmeticAsianMlmcConfig`
- `EuropeanCallSweepScenario`
- `EuropeanCallParameterSweepConfig`
- `InventoryDemandConfig`
- `InventoryPolicy`
- `InventoryConstraints`
- `InventoryCostConfig`
- `InventoryTraceConfig`
- `InventorySimulationConfig`

Each config is an immutable dataclass with explicit seed, path count, step
count, and model parameters.

## Pricing Helpers

- `price_european_call(config=None, native_module=None, **overrides)`
- `price_arithmetic_asian_call(config=None, native_module=None, **overrides)`
- `price_down_and_out_call(config=None, native_module=None, **overrides)`
- `price_lookback_call(config=None, native_module="montepath._native", **overrides)`
- `price_basket_call(config=None, native_module="montepath._native", **overrides)`
- `price_american_put(config=None, native_module="montepath._native", **overrides)`
- `price_bermudan_put(config=None, native_module="montepath._native", **overrides)`
- `price_heston_european_call(config=None, native_module="montepath._native", **overrides)`
- `price_merton_jump_diffusion_call(config=None, native_module="montepath._native", **overrides)`

Each returns `PricingResult`:

- `price`
- `stderr`
- `manifest`
- `warnings`
- `explain()`
- `reproduce()`

The first three helpers are dependency-free Python reference surfaces by
default; pass `native_module="montepath._native"` to dispatch them through the
installed Rust extension. The remaining pricing helpers are stable
native-bridge surfaces for Rust-backed execution. They validate Python configs
first, then require an installed `montepath._native` module exposing the
matching function. If the native module or function is unavailable, they raise
an explicit native-runtime error instead of falling back silently.

## Native Workload Helpers

- `gaussian_uncertainty_moments(config=None, native_module="montepath._native", **overrides)`
- `price_arithmetic_asian_mlmc(config=None, native_module="montepath._native", **overrides)`
- `price_european_call_parameter_sweep(config=None, native_module="montepath._native", **overrides)`

Each returns `NativeWorkloadResult`:

- `workload`
- `values`
- `stderr`
- `manifest`
- `warnings`
- `explain()`
- `reproduce()`

## Inventory Helpers

- `validate_inventory_config(config=None, native_module="montepath._native", **overrides)`
- `simulate_inventory_policy(config=None, native_module="montepath._native", **overrides)`
- `simulate_inventory_policy_reference(config=None, **overrides)`

`simulate_inventory_policy()` is the Rust-backed production CPU path.
`simulate_inventory_policy_reference()` is the dependency-free scalar audit
path. Both return `InventorySimulationResult` with per-path results, aggregate
summary distributions, constraint events, a reproduction manifest, and
explicitly bounded selected-path traces when `InventoryTraceConfig` is set.
Inventory Metal and CUDA execution are unsupported and never selected through
silent fallback.

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
- `backend_capabilities(native_module="montepath._native")`
- `select_backend(workload, backend="auto", native_module="montepath._native")`
- `validate_workload_request(workload, config=None, backend="auto", native_module="montepath._native")`
- `execute_workload(workload, config=None, backend="auto", native_module="montepath._native")`
- `production_status(native_module="montepath._native")`
- `benchmark_report(benchmark_artifact="benchmarks/release-results.json", workload=None, repo_root=None)`
- `load_planner_evidence(...)`
- `measured_winner_database(...)`
- `cost_frontier(workload, ...)`
- `compare_methods(workload, ...)`
- `why_not_faster(workload, method_id=..., ...)`
- `mlmc_error_calibration(workload="arithmetic_asian_call", ...)`
- `run_benchmarks(...)`

Production helpers are the recommended preflight layer for installed packages.
They report whether CPU native execution is available, keep Apple Metal and
CUDA unsupported/deferred states explicit, and summarize benchmark evidence
without running benchmarks. `execute_workload` honors explicit backend
requests; it raises `ProductionCapabilityError` rather than silently falling
back from unavailable Metal or CUDA requests.

## Agent Tools

- `agent_tool_manifest()`
- `export_json_schemas()`
- `agent_validate(request)`
- `agent_capabilities(request=None)`
- `agent_production_check(request)`
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
- `agent_inventory_validate(request)`
- `agent_inventory_simulate(request)`

Benchmark helpers call the Rust benchmark harness and should be used for
local audit or release artifact generation, not as a low-latency API.
Planner evidence helpers read benchmark artifacts and are deterministic for a
fixed artifact, but their timing claims remain hardware-local.

## Native Runtime Discovery

- `native_runtime_status(module_name="montepath._native")`
- `has_native_runtime(module_name="montepath._native")`
- `require_native_runtime(module_name="montepath._native")`

`native_runtime_status()` returns `NativeRuntimeStatus` with:

- `available`
- `module_name`
- `version`
- `supported_functions`
- `reason`
- `as_dict()`

Use this before requiring high-throughput compiled execution. The current
dependency-free Python helpers remain reference UX surfaces until compiled Rust
bindings are installed.

## Errors

Configuration problems raise `McConfigurationError`, which exposes:

- `code`
- `message`
- `suggestion`

Native runtime problems are explicit:

- `NativeRuntimeUnavailableError` when `montepath._native` is not importable
- `NativeFunctionUnavailableError` when the module is present but does not
  expose the requested function

See `docs/error-codes.md`.
