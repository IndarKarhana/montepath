import unittest

from mc_library import (
    compare_methods,
    cost_frontier,
    load_planner_evidence,
    mlmc_error_calibration,
    measured_winner_database,
    why_not_faster,
)


class PlannerIntelligenceTests(unittest.TestCase):
    def test_measured_winner_database_is_evidence_backed(self) -> None:
        database = measured_winner_database()

        self.assertEqual(database["schema_version"], "planner-evidence.v1")
        self.assertGreaterEqual(len(database["winners"]), 3)
        for winner in database["winners"]:
            self.assertIn("benchmark_artifact", winner)
            self.assertIn("workload", winner)
            self.assertIn("selected_method", winner)

    def test_load_planner_evidence_has_accuracy_and_references(self) -> None:
        evidence = load_planner_evidence()

        self.assertEqual(evidence["schema_version"], "planner-evidence.v1")
        self.assertIn("measured_planner_accuracy_pct", evidence)
        self.assertIn("reference_fixtures", evidence)
        self.assertIn(
            "black_scholes_european_call_atm_1y", evidence["reference_fixtures"]
        )
        self.assertGreaterEqual(evidence["measured_planner_accuracy_pct"], 95.0)

    def test_cost_frontier_reports_ranked_methods(self) -> None:
        frontier = cost_frontier("european_call")

        self.assertEqual(frontier["workload"], "european_call")
        self.assertGreaterEqual(len(frontier["frontier"]), 2)
        self.assertLessEqual(
            frontier["frontier"][0]["runtime_ms"],
            frontier["frontier"][-1]["runtime_ms"],
        )

    def test_compare_methods_explains_accuracy_runtime_tradeoff(self) -> None:
        comparison = compare_methods("arithmetic_asian_call")

        self.assertEqual(comparison["workload"], "arithmetic_asian_call")
        self.assertIn("recommended", comparison)
        self.assertIn("alternatives", comparison)
        self.assertTrue(comparison["recommended"]["reason"])
        self.assertIn("quality_metric", comparison["recommended"])

    def test_why_not_faster_explains_rejected_methods(self) -> None:
        explanation = why_not_faster("european_call", method_id="scrambled_sobol")

        self.assertEqual(explanation["workload"], "european_call")
        self.assertEqual(explanation["method_id"], "scrambled_sobol")
        self.assertIn("reasons", explanation)
        self.assertGreater(len(explanation["reasons"]), 0)

    def test_mlmc_error_calibration_reports_estimated_vs_realized(self) -> None:
        calibration = mlmc_error_calibration("arithmetic_asian_call")

        self.assertEqual(calibration["workload"], "arithmetic_asian_call")
        self.assertIn("estimated_error_source", calibration)
        self.assertIn("realized_error_metric", calibration)
        self.assertIn("calibration_status", calibration)


if __name__ == "__main__":
    unittest.main()
