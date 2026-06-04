import sys
import unittest
from types import ModuleType

from montepath import (
    InventoryConstraints,
    InventoryCostConfig,
    InventoryDemandConfig,
    InventoryPolicy,
    InventorySimulationConfig,
    InventoryTraceConfig,
    backend_capabilities,
    execute_workload,
    select_backend,
    simulate_inventory_policy,
    simulate_inventory_policy_reference,
    validate_inventory_config,
)


class InventoryPythonSurfaceTests(unittest.TestCase):
    def deterministic_config(self) -> InventorySimulationConfig:
        return InventorySimulationConfig(
            n_paths=1,
            n_periods=3,
            initial_on_hand=10.0,
            lead_time_periods=1,
            demand=InventoryDemandConfig(distribution="deterministic", units=4.0),
            shortage_policy="lost_sales",
            policy=InventoryPolicy(reorder_point=4.0, order_up_to=10.0),
        )

    def test_reference_runtime_matches_auditable_deterministic_trace(self) -> None:
        result = simulate_inventory_policy_reference(
            self.deterministic_config(),
            trace=InventoryTraceConfig(path_indices=(0,), max_periods=2),
        )
        path = result.paths[0]

        self.assertEqual(result.manifest["schema_version"], "inventory-simulation.v1")
        self.assertEqual(path["orders_placed"], 1)
        self.assertAlmostEqual(path["ordered_units"], 8.0)
        self.assertAlmostEqual(path["received_units"], 8.0)
        self.assertAlmostEqual(path["ending_on_hand"], 6.0)
        self.assertAlmostEqual(path["fill_rate"], 1.0)
        self.assertEqual(len(result.traces), 1)
        self.assertEqual(len(result.traces[0]["periods"]), 2)
        self.assertAlmostEqual(result.traces[0]["periods"][1]["order_quantity"], 8.0)
        self.assertIn("simulate_inventory_policy_reference", result.reproduce().python)

    def test_validation_is_structured_and_does_not_require_native_runtime(self) -> None:
        diagnostics = validate_inventory_config(
            InventorySimulationConfig(
                n_paths=0,
                warmup_periods=52,
                demand=InventoryDemandConfig(distribution="normal", mean=10.0, std_dev=-1.0),
                constraints=InventoryConstraints(case_pack=0.0),
            )
        )
        codes = {item["code"] for item in diagnostics}

        self.assertIn("inventory.paths.invalid", codes)
        self.assertIn("inventory.warmup.invalid", codes)
        self.assertIn("inventory.demand.normal_std_dev.invalid", codes)
        self.assertIn("inventory.constraints.case_pack.invalid", codes)

    def test_trace_validation_is_bounded_and_explicit(self) -> None:
        diagnostics = validate_inventory_config(
            self.deterministic_config(),
            trace=InventoryTraceConfig(path_indices=(0, 0, 1), max_periods=10_001),
        )
        codes = {item["code"] for item in diagnostics}

        self.assertIn("inventory.trace.path_duplicate", codes)
        self.assertIn("inventory.trace.path_out_of_range", codes)
        self.assertIn("inventory.trace.period_limit", codes)

    def test_native_simulation_returns_agent_readable_inventory_result(self) -> None:
        module_name = "_montepath_inventory_native_double"
        module = ModuleType(module_name)
        module.__version__ = "0.2-test"

        def simulate_inventory_policy_native(payload):
            self.assertEqual(payload["policy"]["reorder_point"], 4.0)
            return {
                "values": {
                    "manifest": {"schema_version": "inventory-simulation.v1"},
                    "summary": {"total_cost": {"mean": 12.0}},
                    "paths": [{"path_index": 0, "total_cost": 12.0}],
                    "traces": [],
                },
                "manifest": {
                    "backend": "cpu_native",
                    "execution_mode": "native_extension",
                },
                "warnings": [],
            }

        module.simulate_inventory_policy = simulate_inventory_policy_native
        module.validate_inventory_config = lambda payload: {"ok": True, "diagnostics": []}
        sys.modules[module_name] = module
        try:
            result = simulate_inventory_policy(self.deterministic_config(), native_module=module_name)
            self.assertEqual(result.backend, "cpu_native")
            self.assertEqual(result.summary["total_cost"]["mean"], 12.0)
            self.assertEqual(result.paths[0]["total_cost"], 12.0)
            self.assertEqual(result.traces, ())
            self.assertIn("simulate_inventory_policy", result.reproduce().python)
        finally:
            sys.modules.pop(module_name, None)

    def test_production_capabilities_include_inventory_reference_and_native(self) -> None:
        module_name = "_montepath_inventory_capability_double"
        module = ModuleType(module_name)
        module.__version__ = "0.2-test"
        module.simulate_inventory_policy = lambda payload: {}
        module.validate_inventory_config = lambda payload: {}
        sys.modules[module_name] = module
        try:
            capabilities = {item.backend_id: item for item in backend_capabilities(module_name)}
            selection = select_backend("inventory_policy", native_module=module_name)

            self.assertIn("inventory_policy", capabilities["cpu_native"].workloads)
            self.assertIn("inventory_policy", capabilities["python_reference"].workloads)
            self.assertTrue(selection.ok)
            self.assertEqual(selection.backend_id, "cpu_native")
        finally:
            sys.modules.pop(module_name, None)

    def test_production_inventory_reference_execution_is_structured(self) -> None:
        result = execute_workload(
            "inventory_policy",
            {
                "n_paths": 1,
                "n_periods": 3,
                "initial_on_hand": 10.0,
                "demand": {"distribution": "deterministic", "units": 4.0},
                "policy": {"reorder_point": 4.0, "order_up_to": 10.0},
            },
            backend="python_reference",
        )

        self.assertTrue(result["ok"])
        self.assertEqual(result["selection"]["backend_id"], "python_reference")
        self.assertEqual(result["result"]["paths"][0]["ordered_units"], 8.0)


if __name__ == "__main__":
    unittest.main()
