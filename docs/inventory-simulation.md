# Inventory Simulation

MontePath now includes a Rust-first, single-SKU periodic-review inventory
simulation runtime with a typed Python API and a transparent Python reference.

## Supported Contract

The current `inventory-simulation.v1` contract supports:

- `(s, S)` reorder-point and order-up-to policy
- deterministic or non-negative clipped Normal demand
- fixed lead time and multiple outstanding purchase orders
- backorder or lost-sales shortage handling
- MOQ, case-pack, supplier-capacity, and warehouse-capacity constraints
- holding, shortage, fixed-order, and variable-order costs
- warm-up periods
- per-path KPIs, aggregate distributions, structured constraint telemetry, and
  complete reproduction manifests
- explicitly bounded per-period traces for selected path indices

Every period executes:

`receive -> demand -> fulfill -> review -> order -> record`

Orders never satisfy demand from the period in which they are placed.

## Python API

Production execution uses the compiled Rust extension:

```python
from montepath import (
    InventoryDemandConfig,
    InventoryPolicy,
    InventorySimulationConfig,
    InventoryTraceConfig,
    simulate_inventory_policy,
)

config = InventorySimulationConfig(
    n_paths=10_000,
    n_periods=104,
    initial_on_hand=500.0,
    demand=InventoryDemandConfig(distribution="normal", mean=100.0, std_dev=20.0),
    policy=InventoryPolicy(reorder_point=300.0, order_up_to=700.0),
    trace=InventoryTraceConfig(path_indices=(0,), max_periods=12),
)

result = simulate_inventory_policy(config)
print(result.summary["fill_rate"])
print(result.traces[0]["periods"])
print(result.manifest)
```

Use the dependency-free scalar reference for semantic audits and small
deterministic fixtures:

```python
from montepath import simulate_inventory_policy_reference

reference = simulate_inventory_policy_reference(config)
```

The reference is not a performance path.

## Production And Agent Use

The inventory workload is visible through production capability and validation
surfaces:

```python
from montepath import execute_workload, validate_workload_request

validation = validate_workload_request(
    "inventory_policy",
    {"n_paths": 10_000, "n_periods": 104},
    backend="cpu_native",
)

execution = execute_workload(
    "inventory_policy",
    {"n_paths": 10_000, "n_periods": 104},
    backend="cpu_native",
)
```

Dedicated agent and MCP tools expose bounded validation and execution:

```python
from montepath import agent_inventory_simulate, agent_inventory_validate

request = {
    "config": {
        "n_paths": 10_000,
        "n_periods": 104,
        "trace": {"path_indices": [0], "max_periods": 12},
    },
    "backend": "auto",
    "max_returned_paths": 10,
}

validation = agent_inventory_validate(request)
execution = agent_inventory_simulate(request)
```

The `montepath.inventory.validate` and `montepath.inventory.simulate` MCP tools
publish stable schemas, explicit backend behavior, request limits, structured
failures, and capped returned path results. Full summaries and requested bounded
traces remain available even when path-level output is truncated.

## Release Performance Evidence

The committed release-mode artifact generated on the development Apple M2 for
`10,000 paths x 104 periods`, averaged across three measured iterations,
reported:

- Rust CPU: `6.24 ms`
- NumPy: `14.24 ms`
- Numba: `29.26 ms`
- Rust speedup: `2.28x` over NumPy and `4.69x` over Numba

These are committed release-artifact measurements, but they remain
hardware-local rather than universal performance guarantees. Regenerate the
artifact on target hardware before production tuning. The competitor lanes use
identical operational semantics but independent RNG streams.

## Explicitly Deferred

- Markov regime-switching demand
- random per-order lead times
- policy comparison and optimization
- multi-SKU and shared-capacity models
- native Metal inventory execution

See `docs/inventory-simulation-plan.md` for the complete execution roadmap.
