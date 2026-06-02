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
    native_runtime_status,
    price_arithmetic_asian_mlmc,
    price_basket_call,
    price_european_call,
    gaussian_uncertainty_moments,
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

    if european.price < 0.0 or basket.price < 0.0:
        raise SystemExit("native pricing smoke produced negative option price")
    if "mean" not in moments.values:
        raise SystemExit("native Gaussian UQ smoke missing mean")
    if "price" not in mlmc.values:
        raise SystemExit("native MLMC smoke missing price")

    metadata = server_metadata()
    if metadata["schema_version"] != "mcp-server.v1":
        raise SystemExit("MCP server metadata missing or invalid")
    tools = handle_jsonrpc({"jsonrpc": "2.0", "id": 1, "method": "tools/list"})
    if tools is None or "result" not in tools:
        raise SystemExit("MCP tools/list smoke failed")

    print("installed package smoke passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
