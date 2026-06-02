import unittest

from montepath import (
    agent_benchmark,
    agent_compare,
    agent_compare_methods,
    agent_cost_frontier,
    agent_execute,
    agent_mlmc_calibration,
    agent_plan,
    agent_planner_evidence,
    agent_recommend,
    agent_reproduce,
    agent_tool_manifest,
    agent_validate,
    agent_why_not_faster,
    export_json_schemas,
)


class AgentSurfaceTests(unittest.TestCase):
    def test_tool_manifest_lists_stable_agent_tools(self) -> None:
        manifest = agent_tool_manifest()
        tool_names = {tool["name"] for tool in manifest["tools"]}

        self.assertEqual(manifest["schema_version"], "agent-tools.v1")
        self.assertIn("montepath.validate", tool_names)
        self.assertIn("montepath.recommend", tool_names)
        self.assertIn("montepath.plan", tool_names)
        self.assertIn("montepath.execute", tool_names)
        self.assertIn("montepath.compare", tool_names)
        self.assertIn("montepath.benchmark", tool_names)
        self.assertIn("montepath.reproduce", tool_names)
        self.assertIn("montepath.planner_evidence", tool_names)
        self.assertIn("montepath.cost_frontier", tool_names)
        self.assertIn("montepath.compare_methods", tool_names)
        self.assertIn("montepath.why_not_faster", tool_names)
        self.assertIn("montepath.mlmc_calibration", tool_names)

    def test_json_schema_export_contains_execute_contract(self) -> None:
        schemas = export_json_schemas()

        self.assertIn("montepath.execute.request", schemas)
        self.assertIn("montepath.execute.response", schemas)
        self.assertIn("montepath.compare_methods.request", schemas)
        self.assertIn("montepath.why_not_faster.request", schemas)
        self.assertEqual(schemas["montepath.execute.request"]["type"], "object")
        self.assertIn("workload", schemas["montepath.execute.request"]["required"])

    def test_validate_reports_supported_and_unsupported_states(self) -> None:
        ok = agent_validate({"workload": "european_call", "config": {"n_paths": 128}})
        unsupported = agent_validate({"workload": "american_call", "config": {}})
        bad_config = agent_validate({"workload": "european_call", "config": {"n_paths": 0}})

        self.assertTrue(ok["ok"])
        self.assertFalse(unsupported["ok"])
        self.assertEqual(unsupported["diagnostics"][0]["code"], "MC_AGENT_UNSUPPORTED_WORKLOAD")
        self.assertFalse(bad_config["ok"])
        self.assertEqual(bad_config["diagnostics"][0]["code"], "MC_CONFIG_PATHS")

    def test_recommend_and_plan_are_deterministic_dry_run_surfaces(self) -> None:
        request = {
            "workload": "arithmetic_asian_call",
            "config": {"n_paths": 100_000, "n_steps": 64, "seed": 7},
            "preferences": {"prefer_accuracy": True},
        }
        recommendation = agent_recommend(request)
        plan = agent_plan(request)

        self.assertTrue(recommendation["ok"])
        self.assertEqual(recommendation["recommendation"]["method_id"], "multilevel_monte_carlo")
        self.assertTrue(plan["ok"])
        self.assertTrue(plan["plan"]["dry_run"])
        self.assertEqual(plan["manifest"]["seed"], 7)
        self.assertEqual(plan["manifest"]["backend"], "python_reference")

    def test_execute_returns_structured_manifest_and_reproduction_recipe(self) -> None:
        response = agent_execute(
            {"workload": "european_call", "config": {"n_paths": 256, "n_steps": 8, "seed": 19}}
        )

        self.assertTrue(response["ok"])
        self.assertEqual(response["manifest"]["schema_version"], "agent-run.v1")
        self.assertEqual(response["manifest"]["seed"], 19)
        self.assertEqual(response["manifest"]["backend"], "python_reference")
        self.assertIn("hardware", response["manifest"])
        self.assertIn("build", response["manifest"])
        self.assertIn("price", response["result"])
        self.assertIn("price_european_call", response["reproduction"]["python"])

    def test_compare_and_benchmark_are_agent_safe(self) -> None:
        comparison = agent_compare(
            {"workload": "european_call", "config": {"n_paths": 512, "n_steps": 8, "seed": 23}}
        )
        benchmark = agent_benchmark({"profile": "compact"})

        self.assertTrue(comparison["ok"])
        self.assertIn("alternatives", comparison)
        self.assertEqual(benchmark["status"], "dry_run")
        self.assertIn("cargo run -p mc-bench", benchmark["command"])

    def test_reproduce_accepts_agent_manifest(self) -> None:
        executed = agent_execute(
            {"workload": "down_and_out_call", "config": {"n_paths": 128, "n_steps": 4, "seed": 29}}
        )
        reproduced = agent_reproduce({"manifest": executed["manifest"]})

        self.assertTrue(reproduced["ok"])
        self.assertIn("price_down_and_out_call", reproduced["reproduction"]["python"])

    def test_planner_evidence_tools_are_agent_safe(self) -> None:
        evidence = agent_planner_evidence()
        frontier = agent_cost_frontier({"workload": "european_call"})
        comparison = agent_compare_methods({"workload": "arithmetic_asian_call"})
        why_not = agent_why_not_faster(
            {"workload": "european_call", "method_id": "scrambled_sobol"}
        )
        calibration = agent_mlmc_calibration({"workload": "arithmetic_asian_call"})

        self.assertTrue(evidence["ok"])
        self.assertGreaterEqual(evidence["result"]["measured_planner_accuracy_pct"], 95.0)
        self.assertTrue(frontier["ok"])
        self.assertGreaterEqual(len(frontier["result"]["frontier"]), 2)
        self.assertTrue(comparison["ok"])
        self.assertIn("recommended", comparison["result"])
        self.assertTrue(why_not["ok"])
        self.assertGreater(len(why_not["result"]["reasons"]), 0)
        self.assertTrue(calibration["ok"])
        self.assertIn("calibration_status", calibration["result"])


if __name__ == "__main__":
    unittest.main()
