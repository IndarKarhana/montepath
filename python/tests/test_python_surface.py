import sys
import unittest
from types import ModuleType

from montepath import (
    AmericanPutConfig,
    ArithmeticAsianMlmcConfig,
    BasketCallConfig,
    BenchmarkResult,
    BermudanPutConfig,
    EuropeanCallConfig,
    EuropeanCallParameterSweepConfig,
    EuropeanCallSweepScenario,
    GaussianUncertaintyConfig,
    HestonEuropeanCallConfig,
    LookbackCallConfig,
    McConfigurationError,
    MethodRecommendation,
    MertonJumpDiffusionCallConfig,
    NativeFunctionUnavailableError,
    NativeRuntimeStatus,
    NativeRuntimeUnavailableError,
    NativeWorkloadResult,
    has_native_runtime,
    gaussian_uncertainty_moments,
    native_runtime_status,
    price_american_put,
    price_arithmetic_asian_call,
    price_arithmetic_asian_mlmc,
    price_basket_call,
    price_bermudan_put,
    price_down_and_out_call,
    price_european_call,
    price_european_call_greeks,
    price_european_call_parameter_sweep,
    price_heston_european_call,
    price_lookback_call,
    price_merton_jump_diffusion_call,
    recommend_method,
    require_native_runtime,
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

    def test_native_runtime_status_reports_missing_extension(self) -> None:
        status = native_runtime_status("_montepath_native_missing_test_double")

        self.assertIsInstance(status, NativeRuntimeStatus)
        self.assertFalse(status.available)
        self.assertEqual(status.module_name, "_montepath_native_missing_test_double")
        self.assertEqual(status.supported_functions, ())
        self.assertIn("compiled native runtime module", status.reason or "")
        self.assertFalse(has_native_runtime("_montepath_native_missing_test_double"))
        self.assertFalse(status.as_dict()["available"])

        with self.assertRaises(NativeRuntimeUnavailableError) as raised:
            require_native_runtime("_montepath_native_missing_test_double")

        self.assertEqual(raised.exception.status, status)

    def test_native_runtime_status_reports_available_extension_capabilities(self) -> None:
        module_name = "_montepath_native_test_double"
        module = ModuleType(module_name)
        module.__version__ = "0.1-test"
        module.price_european_call = lambda: None
        module.price_american_put = lambda: None
        sys.modules[module_name] = module

        try:
            status = native_runtime_status(module_name)

            self.assertTrue(status.available)
            self.assertEqual(status.module_name, module_name)
            self.assertEqual(status.version, "0.1-test")
            self.assertEqual(
                status.supported_functions,
                ("price_european_call", "price_american_put"),
            )
            self.assertTrue(has_native_runtime(module_name))
            self.assertIs(require_native_runtime(module_name), module)
            self.assertEqual(
                status.as_dict()["supported_functions"],
                ["price_european_call", "price_american_put"],
            )
        finally:
            sys.modules.pop(module_name, None)

    def test_rust_backed_workload_configs_are_public_and_typed(self) -> None:
        basket = BasketCallConfig(correlation=0.35, n_paths=4096)
        bermudan = BermudanPutConfig(exercise_steps=(8, 16, 32), n_steps=32)
        sweep = EuropeanCallParameterSweepConfig(
            scenarios=(
                EuropeanCallSweepScenario(spot=95.0, volatility=0.18),
                EuropeanCallSweepScenario(spot=105.0, volatility=0.22),
            ),
            n_paths=2048,
            n_steps=16,
        )

        self.assertEqual(basket.correlation, 0.35)
        self.assertEqual(bermudan.exercise_steps, (8, 16, 32))
        self.assertEqual(len(sweep.scenarios), 2)
        self.assertIsInstance(LookbackCallConfig(), LookbackCallConfig)
        self.assertIsInstance(AmericanPutConfig(), AmericanPutConfig)
        self.assertIsInstance(HestonEuropeanCallConfig(), HestonEuropeanCallConfig)
        self.assertIsInstance(MertonJumpDiffusionCallConfig(), MertonJumpDiffusionCallConfig)
        self.assertIsInstance(GaussianUncertaintyConfig(), GaussianUncertaintyConfig)
        self.assertIsInstance(ArithmeticAsianMlmcConfig(), ArithmeticAsianMlmcConfig)

    def test_native_only_pricing_surfaces_report_missing_runtime_explicitly(self) -> None:
        with self.assertRaises(NativeRuntimeUnavailableError) as raised:
            price_lookback_call(
                LookbackCallConfig(n_paths=128, n_steps=8),
                native_module="_montepath_native_missing_test_double",
            )

        self.assertFalse(raised.exception.status.available)

    def test_native_only_pricing_surfaces_report_missing_function_explicitly(self) -> None:
        module_name = "_montepath_native_missing_function"
        module = ModuleType(module_name)
        module.__version__ = "0.1-test"
        sys.modules[module_name] = module

        try:
            with self.assertRaises(NativeFunctionUnavailableError) as raised:
                price_basket_call(BasketCallConfig(n_paths=128), native_module=module_name)

            self.assertEqual(raised.exception.function_name, "price_basket_call")
            self.assertEqual(raised.exception.status.module_name, module_name)
        finally:
            sys.modules.pop(module_name, None)

    def test_native_pricing_wrapper_uses_mapping_abi_and_structured_result(self) -> None:
        module_name = "_montepath_native_pricing_double"
        module = ModuleType(module_name)
        module.__version__ = "0.1-test"

        def price_lookback_call_native(config: dict[str, object]) -> dict[str, object]:
            self.assertEqual(config["n_paths"], 321)
            return {
                "price": 12.5,
                "stderr": 0.125,
                "manifest": {"backend": "cpu_native", "note": "test-double"},
                "warnings": ["native test warning"],
            }

        module.price_lookback_call = price_lookback_call_native
        sys.modules[module_name] = module

        try:
            result = price_lookback_call(
                LookbackCallConfig(n_paths=321, n_steps=12, seed=7),
                native_module=module_name,
            )

            self.assertEqual(result.price, 12.5)
            self.assertEqual(result.stderr, 0.125)
            self.assertEqual(result.manifest["workload"], "lookback_call")
            self.assertEqual(result.manifest["backend"], "cpu_native")
            self.assertIn("price_lookback_call", result.reproduce().python)
            self.assertIn("native test warning", result.warnings)
        finally:
            sys.modules.pop(module_name, None)

    def test_native_generic_wrappers_return_structured_workload_results(self) -> None:
        module_name = "_montepath_native_generic_double"
        module = ModuleType(module_name)
        module.__version__ = "0.1-test"
        module.gaussian_uncertainty_moments = lambda config: {
            "values": {"mean": 1.0, "variance": 4.0},
            "manifest": {"backend": "cpu_native"},
        }
        module.price_european_call_parameter_sweep = lambda config: {
            "values": {"rows": [{"price": 9.1}, {"price": 10.2}]},
            "manifest": {"backend": "cpu_native"},
        }
        module.arithmetic_asian_mlmc = lambda config: {
            "values": {"price": 5.4, "levels": 4},
            "stderr": 0.05,
            "manifest": {"backend": "cpu_native"},
        }
        sys.modules[module_name] = module

        try:
            moments = gaussian_uncertainty_moments(
                GaussianUncertaintyConfig(n_paths=512),
                native_module=module_name,
            )
            sweep = price_european_call_parameter_sweep(
                EuropeanCallParameterSweepConfig(
                    scenarios=(EuropeanCallSweepScenario(spot=100.0),),
                    n_paths=512,
                ),
                native_module=module_name,
            )
            mlmc = price_arithmetic_asian_mlmc(
                ArithmeticAsianMlmcConfig(target_stderr=0.05),
                native_module=module_name,
            )

            self.assertIsInstance(moments, NativeWorkloadResult)
            self.assertEqual(moments.values["variance"], 4.0)
            self.assertEqual(sweep.values["rows"][1]["price"], 10.2)
            self.assertEqual(mlmc.stderr, 0.05)
            self.assertIn("gaussian_uncertainty_moments", moments.reproduce().python)
        finally:
            sys.modules.pop(module_name, None)

    def test_all_native_only_price_wrappers_validate_configs_before_handoff(self) -> None:
        native_surfaces = (
            (price_american_put, AmericanPutConfig(n_paths=0)),
            (price_bermudan_put, BermudanPutConfig(n_steps=0)),
            (price_heston_european_call, HestonEuropeanCallConfig(initial_variance=-0.1)),
            (
                price_merton_jump_diffusion_call,
                MertonJumpDiffusionCallConfig(jump_intensity=-1.0),
            ),
        )

        for helper, config in native_surfaces:
            with self.subTest(helper=helper.__name__):
                with self.assertRaises(McConfigurationError):
                    helper(config)


if __name__ == "__main__":
    unittest.main()
