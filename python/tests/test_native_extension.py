import unittest

from montepath import (
    AmericanPutConfig,
    ArithmeticAsianCallConfig,
    ArithmeticAsianMlmcConfig,
    BasketCallConfig,
    BermudanPutConfig,
    DownAndOutCallConfig,
    EuropeanCallConfig,
    EuropeanCallParameterSweepConfig,
    EuropeanCallSweepScenario,
    GaussianUncertaintyConfig,
    HestonEuropeanCallConfig,
    LookbackCallConfig,
    MertonJumpDiffusionCallConfig,
    gaussian_uncertainty_moments,
    native_runtime_status,
    price_american_put,
    price_arithmetic_asian_call,
    price_arithmetic_asian_mlmc,
    price_basket_call,
    price_bermudan_put,
    price_down_and_out_call,
    price_european_call,
    price_european_call_parameter_sweep,
    price_heston_european_call,
    price_lookback_call,
    price_merton_jump_diffusion_call,
)


@unittest.skipUnless(native_runtime_status().available, "montepath._native is not installed")
class NativeExtensionTests(unittest.TestCase):
    def test_native_runtime_exposes_all_bridge_functions(self) -> None:
        supported = set(native_runtime_status().supported_functions)

        self.assertEqual(
            supported,
            {
                "price_european_call",
                "price_arithmetic_asian_call",
                "price_down_and_out_call",
                "price_lookback_call",
                "price_basket_call",
                "price_american_put",
                "price_bermudan_put",
                "price_heston_european_call",
                "price_merton_jump_diffusion_call",
                "price_european_call_parameter_sweep",
                "gaussian_uncertainty_moments",
                "arithmetic_asian_mlmc",
            },
        )

    def test_native_pricing_functions_return_structured_results(self) -> None:
        price_cases = (
            (
                price_european_call,
                EuropeanCallConfig(n_paths=512, n_steps=8, seed=1),
                {"native_module": "montepath._native"},
            ),
            (
                price_arithmetic_asian_call,
                ArithmeticAsianCallConfig(n_paths=512, n_steps=8, seed=2),
                {"native_module": "montepath._native"},
            ),
            (
                price_down_and_out_call,
                DownAndOutCallConfig(n_paths=512, n_steps=8, seed=3),
                {"native_module": "montepath._native"},
            ),
            (price_lookback_call, LookbackCallConfig(n_paths=512, n_steps=8, seed=4), {}),
            (price_basket_call, BasketCallConfig(n_paths=512, seed=5), {}),
            (price_american_put, AmericanPutConfig(n_paths=512, n_steps=8, seed=6), {}),
            (
                price_bermudan_put,
                BermudanPutConfig(
                    n_paths=512,
                    n_steps=8,
                    seed=7,
                    exercise_steps=(4, 8),
                ),
                {},
            ),
            (
                price_heston_european_call,
                HestonEuropeanCallConfig(n_paths=512, n_steps=8, seed=8),
                {},
            ),
            (
                price_merton_jump_diffusion_call,
                MertonJumpDiffusionCallConfig(n_paths=512, seed=9),
                {},
            ),
        )

        for helper, config, kwargs in price_cases:
            with self.subTest(helper=helper.__name__):
                result = helper(config, **kwargs)
                self.assertGreaterEqual(result.price, 0.0)
                self.assertGreaterEqual(result.stderr, 0.0)
                self.assertEqual(result.manifest["backend"], "cpu_native")
                self.assertIn("montepath._native", result.manifest["native_module"])

    def test_native_structured_workload_functions_return_values(self) -> None:
        moments = gaussian_uncertainty_moments(
            GaussianUncertaintyConfig(n_paths=512, dimensions=3, seed=10)
        )
        mlmc = price_arithmetic_asian_mlmc(
            ArithmeticAsianMlmcConfig(
                levels=2,
                paths_per_level=(512, 256),
                base_steps=4,
                seed=11,
            )
        )
        sweep = price_european_call_parameter_sweep(
            EuropeanCallParameterSweepConfig(
                n_paths=512,
                n_steps=8,
                seed=12,
                scenarios=(EuropeanCallSweepScenario(scenario_id="base"),),
            )
        )

        self.assertIn("mean", moments.values)
        self.assertIn("price", mlmc.values)
        self.assertIn("rows", sweep.values)
        self.assertEqual(sweep.values["scenario_count"], 1)


if __name__ == "__main__":
    unittest.main()
