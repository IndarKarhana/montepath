"""Smoke checks for an installed montepath package.

This module is intended to run from a clean virtual environment after installing
the built wheel or source distribution. It should not rely on `PYTHONPATH`.
"""

from __future__ import annotations

from montepath import (
    ArithmeticAsianMlmcConfig,
    BasketCallConfig,
    EuropeanCallConfig,
    GaussianUncertaintyConfig,
    InventoryDemandConfig,
    InventoryPolicy,
    InventorySimulationConfig,
    backend_capabilities,
    benchmark_report,
    native_runtime_status,
    numerical_validation_report,
    price_arithmetic_asian_mlmc,
    price_basket_call,
    price_european_call,
    production_status,
    gaussian_uncertainty_moments,
    select_backend,
    simulate_inventory_policy,
    agent_inventory_simulate,
)
from montepath.mcp_server import handle_jsonrpc, server_metadata


def main() -> int:
    status = native_runtime_status()
    if not status.available:
        raise SystemExit(f"native runtime unavailable after install: {status.as_dict()}")

    expected = {
        "price_european_call",
        "price_basket_call",
        "gaussian_uncertainty_moments",
        "arithmetic_asian_mlmc",
        "validate_inventory_config",
        "simulate_inventory_policy",
    }
    missing = expected.difference(status.supported_functions)
    if missing:
        raise SystemExit(f"native runtime missing functions: {sorted(missing)}")

    european = price_european_call(
        EuropeanCallConfig(n_paths=512, n_steps=8, seed=101),
        native_module=status.module_name,
    )
    basket = price_basket_call(BasketCallConfig(n_paths=512, seed=102))
    moments = gaussian_uncertainty_moments(
        GaussianUncertaintyConfig(n_paths=512, dimensions=3, seed=103)
    )
    mlmc = price_arithmetic_asian_mlmc(
        ArithmeticAsianMlmcConfig(
            levels=2,
            paths_per_level=(512, 256),
            base_steps=4,
            seed=104,
        )
    )
    inventory = simulate_inventory_policy(
        InventorySimulationConfig(
            n_paths=16,
            n_periods=3,
            initial_on_hand=10.0,
            demand=InventoryDemandConfig(distribution="deterministic", units=4.0),
            policy=InventoryPolicy(reorder_point=4.0, order_up_to=10.0),
        )
    )

    if european.price < 0.0 or basket.price < 0.0:
        raise SystemExit("native pricing smoke produced negative option price")
    if "mean" not in moments.values:
        raise SystemExit("native Gaussian UQ smoke missing mean")
    if "price" not in mlmc.values:
        raise SystemExit("native MLMC smoke missing price")
    if inventory.paths[0]["orders_placed"] != 1:
        raise SystemExit("native inventory smoke produced unexpected deterministic trace")
    agent_inventory = agent_inventory_simulate(
        {
            "config": {
                "n_paths": 4,
                "n_periods": 3,
                "initial_on_hand": 10.0,
                "demand": {"distribution": "deterministic", "units": 4.0},
                "policy": {"reorder_point": 4.0, "order_up_to": 10.0},
            },
            "max_returned_paths": 2,
        }
    )
    if not agent_inventory["ok"] or agent_inventory["result"]["returned_path_count"] != 2:
        raise SystemExit("installed inventory agent tool smoke failed")

    capabilities = {item.backend_id: item for item in backend_capabilities()}
    if capabilities["cpu_native"].status != "available":
        raise SystemExit("production capabilities did not detect installed CPU native backend")
    selected = select_backend("european_call")
    if not selected.ok or selected.backend_id != "cpu_native":
        raise SystemExit(f"production backend selection failed: {selected.as_dict()}")
    inventory_selected = select_backend("inventory_policy")
    if not inventory_selected.ok or inventory_selected.backend_id != "cpu_native":
        raise SystemExit(
            f"production inventory backend selection failed: {inventory_selected.as_dict()}"
        )
    status_payload = production_status()
    if status_payload["schema_version"] != "montepath-production-status.v1":
        raise SystemExit("production status schema missing")
    report = benchmark_report()
    if report["schema_version"] != "montepath-benchmark-report.v1":
        raise SystemExit("benchmark report schema missing")
    validation_report = numerical_validation_report()
    if validation_report["schema_version"] != "montepath-numerical-validation.v1":
        raise SystemExit("numerical validation report schema missing")

    metadata = server_metadata()
    if metadata["schema_version"] != "mcp-server.v1":
        raise SystemExit("MCP server metadata missing or invalid")
    tools = handle_jsonrpc({"jsonrpc": "2.0", "id": 1, "method": "tools/list"})
    if tools is None or "result" not in tools:
        raise SystemExit("MCP tools/list smoke failed")
    tool_names = {tool["name"] for tool in tools["result"]["tools"]}
    required_tools = {
        "montepath.capabilities",
        "montepath.production_check",
        "montepath.validation_report",
        "montepath.inventory.validate",
        "montepath.inventory.simulate",
    }
    if not required_tools.issubset(tool_names):
        raise SystemExit("MCP production tools missing")

    print("installed package smoke passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
