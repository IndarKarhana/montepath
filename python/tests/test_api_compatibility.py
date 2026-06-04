import unittest

import montepath
from montepath import (
    agent_tool_manifest,
    export_json_schemas,
    native_runtime_status,
    numerical_validation_report,
    production_status,
)
from montepath.native import KNOWN_NATIVE_FUNCTIONS


class ApiCompatibilityTests(unittest.TestCase):
    def test_public_exports_include_production_agent_surfaces(self) -> None:
        expected = {
            "backend_capabilities",
            "benchmark_report",
            "execute_workload",
            "numerical_validation_report",
            "production_status",
            "select_backend",
            "validate_workload_request",
            "agent_capabilities",
            "agent_production_check",
            "agent_validation_report",
            "agent_tool_manifest",
            "export_json_schemas",
            "simulate_inventory_policy",
            "simulate_inventory_policy_reference",
            "validate_inventory_config",
            "InventoryTraceConfig",
            "agent_inventory_validate",
            "agent_inventory_simulate",
        }

        self.assertTrue(expected.issubset(set(montepath.__all__)))

    def test_agent_tool_and_schema_ids_are_stable_for_current_contracts(self) -> None:
        tool_names = {tool["name"] for tool in agent_tool_manifest()["tools"]}
        schema_ids = set(export_json_schemas())

        expected_tools = {
            "montepath.validate",
            "montepath.capabilities",
            "montepath.production_check",
            "montepath.validation_report",
            "montepath.inventory.validate",
            "montepath.inventory.simulate",
            "montepath.recommend",
            "montepath.plan",
            "montepath.execute",
            "montepath.compare",
            "montepath.benchmark",
            "montepath.reproduce",
            "montepath.planner_evidence",
            "montepath.cost_frontier",
            "montepath.compare_methods",
            "montepath.why_not_faster",
            "montepath.mlmc_calibration",
        }

        self.assertTrue(expected_tools.issubset(tool_names))
        for tool in expected_tools:
            self.assertIn(f"{tool}.request", schema_ids)
            self.assertIn(f"{tool}.response", schema_ids)

    def test_schema_versions_remain_agent_readable(self) -> None:
        status = production_status("_montepath_native_missing_test_double")
        validation = numerical_validation_report()
        native = native_runtime_status("_montepath_native_missing_test_double")

        self.assertEqual(status["schema_version"], "montepath-production-status.v1")
        self.assertEqual(
            validation["schema_version"], "montepath-numerical-validation.v1"
        )
        self.assertIn("supported_functions", native.as_dict())

    def test_known_native_function_identifiers_include_cpu_and_metal_bridges(self) -> None:
        expected = {
            "price_european_call",
            "price_arithmetic_asian_call",
            "price_down_and_out_call",
            "price_european_call_metal",
            "price_arithmetic_asian_call_metal",
            "price_down_and_out_call_metal",
            "price_basket_call",
            "gaussian_uncertainty_moments",
            "arithmetic_asian_mlmc",
            "validate_inventory_config",
            "simulate_inventory_policy",
        }

        self.assertTrue(expected.issubset(set(KNOWN_NATIVE_FUNCTIONS)))


if __name__ == "__main__":
    unittest.main()
