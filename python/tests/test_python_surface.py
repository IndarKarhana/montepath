import unittest

from mc_library import (
    BenchmarkResult,
    EuropeanCallConfig,
    McConfigurationError,
    MethodRecommendation,
    price_arithmetic_asian_call,
    price_down_and_out_call,
    price_european_call,
    price_european_call_greeks,
    recommend_method,
)


class PythonSurfaceTests(unittest.TestCase):
    def test_method_recommendation_defaults_to_fast_control_variate(self) -> None:
        recommendation = recommend_method(
            workload_family="european_call",
            n_paths=100_000,
            n_steps=64,
        )

        self.assertIsInstance(recommendation, MethodRecommendation)
        self.assertEqual(recommendation.method_id, "control_variates")
        self.assertEqual(recommendation.sampling, "pseudorandom")
        self.assertEqual(recommendation.technique, "control_variate")

    def test_method_recommendation_can_choose_sobol_bridge(self) -> None:
        recommendation = recommend_method(
            workload_family="down_and_out_call",
            n_paths=100_000,
            n_steps=64,
            prefer_accuracy=True,
            allow_slower_structured_sampling=True,
        )

        self.assertEqual(recommendation.method_id, "scrambled_sobol_brownian_bridge")
        self.assertEqual(recommendation.sampling, "scrambled_sobol_brownian_bridge")
        self.assertEqual(recommendation.technique, "control_variate")

    def test_method_recommendation_can_choose_mlqmc(self) -> None:
        recommendation = recommend_method(
            workload_family="arithmetic_asian_call",
            n_paths=100_000,
            n_steps=64,
            prefer_accuracy=True,
            allow_slower_structured_sampling=True,
        )

        self.assertEqual(recommendation.method_id, "multilevel_randomized_qmc")
        self.assertEqual(recommendation.sampling, "scrambled_sobol")
        self.assertEqual(recommendation.technique, "standard")

    def test_method_recommendation_can_choose_mlmc(self) -> None:
        recommendation = recommend_method(
            workload_family="arithmetic_asian_call",
            n_paths=100_000,
            n_steps=64,
            prefer_accuracy=True,
            allow_slower_structured_sampling=False,
        )

        self.assertEqual(recommendation.method_id, "multilevel_monte_carlo")
        self.assertEqual(recommendation.sampling, "pseudorandom")
        self.assertEqual(recommendation.technique, "standard")

    def test_benchmark_result_shape_is_public(self) -> None:
        result = BenchmarkResult(
            benchmark_name="example",
            backend="cpu_native",
            methodology="example_method",
            per_iteration_ms=1.25,
            metric_name="price_estimate",
            metric_value=9.4,
        )

        self.assertEqual(result.per_iteration_ms, 1.25)

    def test_european_price_result_is_explainable_and_reproducible(self) -> None:
        cfg = EuropeanCallConfig(n_paths=2_000, n_steps=16, seed=11)
        result = price_european_call(cfg)

        self.assertGreater(result.price, 0.0)
        self.assertEqual(result.manifest["workload"], "european_call")
        self.assertEqual(result.manifest["seed"], 11)
        self.assertIn("European call", result.explain())
        self.assertIn("price_european_call", result.reproduce().python)

    def test_path_dependent_helpers_are_available(self) -> None:
        asian = price_arithmetic_asian_call(n_paths=512, n_steps=8, seed=13)
        barrier = price_down_and_out_call(n_paths=512, n_steps=8, seed=13)

        self.assertEqual(asian.manifest["workload"], "arithmetic_asian_call")
        self.assertEqual(barrier.manifest["workload"], "down_and_out_call")
        self.assertGreaterEqual(asian.stderr, 0.0)
        self.assertGreaterEqual(barrier.stderr, 0.0)

    def test_european_greeks_have_black_scholes_reference_metadata(self) -> None:
        report = price_european_call_greeks(EuropeanCallConfig(n_paths=2_000, seed=17))

        self.assertIn("delta", report.greeks)
        self.assertIn("black_scholes", report.manifest["reference"])
        self.assertIn("Delta", report.explain())
        self.assertIn("price_european_call_greeks", report.reproduce().python)

    def test_user_facing_errors_include_codes_and_suggestions(self) -> None:
        with self.assertRaises(McConfigurationError) as raised:
            price_european_call(EuropeanCallConfig(n_paths=0))

        self.assertEqual(raised.exception.code, "MC_CONFIG_PATHS")
        self.assertIn("n_paths", raised.exception.suggestion)


if __name__ == "__main__":
    unittest.main()
