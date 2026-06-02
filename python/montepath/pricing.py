"""Python-first pricing helpers.

These helpers are intentionally dependency-free and conservative. They give
users and agents a typed, inspectable API before compiled Python bindings are
ready, while keeping high-performance claims anchored to the Rust benchmark
artifacts.
"""

from __future__ import annotations

import math
import random
from dataclasses import asdict, dataclass, replace
from typing import Any, Mapping

from .native import (
    DEFAULT_NATIVE_MODULE,
    NativeFunctionUnavailableError,
    NativeRuntimeUnavailableError,
    native_runtime_status,
    require_native_runtime,
)


class McConfigurationError(ValueError):
    """User-facing configuration error with an actionable code."""

    def __init__(self, code: str, message: str, suggestion: str) -> None:
        super().__init__(f"{code}: {message} Suggestion: {suggestion}")
        self.code = code
        self.message = message
        self.suggestion = suggestion


@dataclass(frozen=True)
class EuropeanCallConfig:
    spot: float = 100.0
    strike: float = 100.0
    rate: float = 0.03
    volatility: float = 0.2
    maturity: float = 1.0
    n_paths: int = 100_000
    n_steps: int = 64
    seed: int = 42


@dataclass(frozen=True)
class ArithmeticAsianCallConfig(EuropeanCallConfig):
    pass


@dataclass(frozen=True)
class DownAndOutCallConfig(EuropeanCallConfig):
    barrier: float = 80.0


@dataclass(frozen=True)
class LookbackCallConfig(EuropeanCallConfig):
    sampling: str = "pseudorandom"
    technique: str = "standard"
    n_threads: int = 0


@dataclass(frozen=True)
class BasketCallConfig:
    spot_1: float = 100.0
    spot_2: float = 95.0
    strike: float = 100.0
    rate: float = 0.03
    volatility_1: float = 0.2
    volatility_2: float = 0.25
    correlation: float = 0.35
    weight_1: float = 0.5
    weight_2: float = 0.5
    maturity: float = 1.0
    n_paths: int = 100_000
    seed: int = 42
    sampling: str = "pseudorandom"
    technique: str = "standard"
    n_threads: int = 0


@dataclass(frozen=True)
class AmericanPutConfig(EuropeanCallConfig):
    basis_degree: int = 2
    n_threads: int = 0


@dataclass(frozen=True)
class BermudanPutConfig(AmericanPutConfig):
    exercise_steps: tuple[int, ...] = (16, 32, 48, 64)


@dataclass(frozen=True)
class HestonEuropeanCallConfig:
    spot: float = 100.0
    strike: float = 100.0
    rate: float = 0.03
    initial_variance: float = 0.04
    mean_reversion: float = 1.5
    long_run_variance: float = 0.04
    vol_of_vol: float = 0.3
    correlation: float = -0.6
    maturity: float = 1.0
    n_paths: int = 100_000
    n_steps: int = 252
    seed: int = 42
    technique: str = "standard"
    n_threads: int = 0


@dataclass(frozen=True)
class MertonJumpDiffusionCallConfig(EuropeanCallConfig):
    jump_intensity: float = 0.4
    jump_mean: float = -0.08
    jump_volatility: float = 0.25
    n_threads: int = 0


@dataclass(frozen=True)
class GaussianUncertaintyConfig:
    n_paths: int = 100_000
    dimensions: int = 3
    seed: int = 42
    sampling: str = "pseudorandom"


@dataclass(frozen=True)
class ArithmeticAsianMlmcConfig:
    spot: float = 100.0
    strike: float = 100.0
    rate: float = 0.03
    volatility: float = 0.2
    maturity: float = 1.0
    base_steps: int = 16
    levels: int = 4
    refinement_factor: int = 2
    paths_per_level: tuple[int, ...] = (50_000, 25_000, 12_500, 6_250)
    seed: int = 42
    sampling: str = "pseudorandom"
    scramble_replicates: int = 1
    target_stderr: float | None = None


@dataclass(frozen=True)
class EuropeanCallSweepScenario:
    scenario_id: str = "base"
    spot: float | None = None
    strike: float | None = None
    rate: float | None = None
    volatility: float | None = None
    maturity: float | None = None
    n_paths: int | None = None
    n_steps: int | None = None
    seed: int | None = None
    sampling: str | None = None
    technique: str | None = None
    method: str | None = None


@dataclass(frozen=True)
class EuropeanCallParameterSweepConfig:
    base_config: EuropeanCallConfig = EuropeanCallConfig()
    n_paths: int | None = None
    n_steps: int | None = None
    seed: int | None = None
    method: str = "terminal_distribution"
    seed_stride: int = 10_000
    scenarios: tuple[EuropeanCallSweepScenario, ...] = (
        EuropeanCallSweepScenario(scenario_id="atm_base"),
        EuropeanCallSweepScenario(scenario_id="down_10pct", spot=90.0),
        EuropeanCallSweepScenario(scenario_id="high_vol", volatility=0.35),
    )


@dataclass(frozen=True)
class ReproductionRecipe:
    python: str
    manifest: Mapping[str, Any]


@dataclass(frozen=True)
class PricingResult:
    workload: str
    price: float
    stderr: float
    n_paths: int
    n_steps: int
    seed: int
    manifest: Mapping[str, Any]
    warnings: tuple[str, ...] = ()

    def explain(self) -> str:
        label = {
            "european_call": "European call",
            "arithmetic_asian_call": "Arithmetic Asian call",
            "down_and_out_call": "Down-and-out call",
            "lookback_call": "Fixed-strike lookback call",
            "basket_call": "Two-asset basket call",
            "american_put": "American put",
            "bermudan_put": "Bermudan put",
            "heston_european_call": "Heston European call",
            "merton_jump_diffusion_call": "Merton jump-diffusion call",
        }.get(self.workload, self.workload)
        backend = self.manifest.get("backend", "python_reference")
        return (
            f"{label} priced with {backend}: "
            f"price={self.price:.6f}, stderr={self.stderr:.6f}, "
            f"paths={self.n_paths}, steps={self.n_steps}, seed={self.seed}."
        )

    def reproduce(self) -> ReproductionRecipe:
        cfg = self.manifest["config"]
        helper = {
            "european_call": "price_european_call",
            "arithmetic_asian_call": "price_arithmetic_asian_call",
            "down_and_out_call": "price_down_and_out_call",
            "lookback_call": "price_lookback_call",
            "basket_call": "price_basket_call",
            "american_put": "price_american_put",
            "bermudan_put": "price_bermudan_put",
            "heston_european_call": "price_heston_european_call",
            "merton_jump_diffusion_call": "price_merton_jump_diffusion_call",
        }[self.workload]
        return ReproductionRecipe(
            python=(
                f"from montepath import {helper}\n"
                f"result = {helper}(**{cfg!r})\n"
                "print(result.price, result.stderr)\n"
            ),
            manifest=self.manifest,
        )


@dataclass(frozen=True)
class NativeWorkloadResult:
    workload: str
    values: Mapping[str, Any]
    manifest: Mapping[str, Any]
    stderr: float | None = None
    warnings: tuple[str, ...] = ()

    def explain(self) -> str:
        backend = self.manifest.get("backend", "native_runtime")
        return f"{self.workload} executed through {backend} with structured values."

    def reproduce(self) -> ReproductionRecipe:
        cfg = self.manifest["config"]
        helper = self.manifest["function"]
        return ReproductionRecipe(
            python=(
                f"from montepath import {helper}\n"
                f"result = {helper}(**{cfg!r})\n"
                "print(result.values)\n"
            ),
            manifest=self.manifest,
        )


@dataclass(frozen=True)
class GreekReport:
    workload: str
    estimator: str
    base_price: float
    greeks: Mapping[str, float]
    manifest: Mapping[str, Any]
    warnings: tuple[str, ...] = ()

    def explain(self) -> str:
        parts = ", ".join(f"{name.title()}={value:.6f}" for name, value in self.greeks.items())
        return (
            f"European call Greeks via {self.estimator}: base price="
            f"{self.base_price:.6f}; {parts}."
        )

    def reproduce(self) -> ReproductionRecipe:
        cfg = self.manifest["config"]
        return ReproductionRecipe(
            python=(
                "from montepath import price_european_call_greeks\n"
                f"report = price_european_call_greeks(**{cfg!r})\n"
                "print(report.greeks)\n"
            ),
            manifest=self.manifest,
        )


def price_european_call(
    config: EuropeanCallConfig | None = None,
    native_module: str | None = None,
    **overrides: Any,
) -> PricingResult:
    cfg = _coerce_config(EuropeanCallConfig, config, overrides)
    _validate_gbm_config(cfg)
    if native_module is not None:
        return _execute_native_pricing("european_call", "price_european_call", cfg, native_module)
    return _simulate_gbm_payoff("european_call", cfg, _european_payoff)


def price_arithmetic_asian_call(
    config: ArithmeticAsianCallConfig | None = None,
    native_module: str | None = None,
    **overrides: Any,
) -> PricingResult:
    cfg = _coerce_config(ArithmeticAsianCallConfig, config, overrides)
    _validate_gbm_config(cfg)
    if native_module is not None:
        return _execute_native_pricing(
            "arithmetic_asian_call",
            "price_arithmetic_asian_call",
            cfg,
            native_module,
        )
    return _simulate_gbm_payoff("arithmetic_asian_call", cfg, _asian_payoff)


def price_down_and_out_call(
    config: DownAndOutCallConfig | None = None,
    native_module: str | None = None,
    **overrides: Any,
) -> PricingResult:
    cfg = _coerce_config(DownAndOutCallConfig, config, overrides)
    _validate_gbm_config(cfg)
    if cfg.barrier <= 0.0 or cfg.barrier >= cfg.spot:
        raise McConfigurationError(
            "MC_CONFIG_BARRIER",
            "barrier must be positive and below spot for the current down-and-out helper",
            "choose 0 < barrier < spot, for example barrier=80.0 when spot=100.0",
        )
    if native_module is not None:
        return _execute_native_pricing(
            "down_and_out_call",
            "price_down_and_out_call",
            cfg,
            native_module,
        )
    return _simulate_gbm_payoff("down_and_out_call", cfg, _down_and_out_payoff)


def price_european_call_greeks(
    config: EuropeanCallConfig | None = None,
    **overrides: Any,
) -> GreekReport:
    cfg = _coerce_config(EuropeanCallConfig, config, overrides)
    _validate_gbm_config(cfg)
    reference = _black_scholes_greeks(cfg)
    manifest = _base_manifest("european_call_greeks", cfg) | {
        "estimator": "black_scholes_closed_form",
        "reference": "black_scholes_european_call_greeks_atm_1y",
    }
    return GreekReport(
        workload="european_call",
        estimator="black_scholes_closed_form",
        base_price=reference["price"],
        greeks={
            "delta": reference["delta"],
            "vega": reference["vega"],
            "rho": reference["rho"],
            "theta": reference["theta"],
        },
        manifest=manifest,
    )


def price_lookback_call(
    config: LookbackCallConfig | None = None,
    native_module: str = DEFAULT_NATIVE_MODULE,
    **overrides: Any,
) -> PricingResult:
    cfg = _coerce_config(LookbackCallConfig, config, overrides)
    _validate_gbm_config(cfg)
    _validate_sampling_and_technique(cfg.sampling, cfg.technique)
    _validate_non_negative_int("n_threads", cfg.n_threads)
    return _execute_native_pricing("lookback_call", "price_lookback_call", cfg, native_module)


def price_basket_call(
    config: BasketCallConfig | None = None,
    native_module: str = DEFAULT_NATIVE_MODULE,
    **overrides: Any,
) -> PricingResult:
    cfg = _coerce_config(BasketCallConfig, config, overrides)
    _validate_basket_config(cfg)
    return _execute_native_pricing("basket_call", "price_basket_call", cfg, native_module)


def price_american_put(
    config: AmericanPutConfig | None = None,
    native_module: str = DEFAULT_NATIVE_MODULE,
    **overrides: Any,
) -> PricingResult:
    cfg = _coerce_config(AmericanPutConfig, config, overrides)
    _validate_gbm_config(cfg)
    _validate_lsm_config(cfg.basis_degree, cfg.n_threads)
    return _execute_native_pricing("american_put", "price_american_put", cfg, native_module)


def price_bermudan_put(
    config: BermudanPutConfig | None = None,
    native_module: str = DEFAULT_NATIVE_MODULE,
    **overrides: Any,
) -> PricingResult:
    cfg = _coerce_config(BermudanPutConfig, config, overrides)
    _validate_gbm_config(cfg)
    _validate_lsm_config(cfg.basis_degree, cfg.n_threads)
    if not cfg.exercise_steps:
        raise McConfigurationError(
            "MC_CONFIG_EXERCISE_STEPS",
            "exercise_steps must contain at least one exercise date",
            "provide one or more integer step indexes, for example (16, 32, 48, 64)",
        )
    for step in cfg.exercise_steps:
        if step <= 0 or step > cfg.n_steps:
            raise McConfigurationError(
                "MC_CONFIG_EXERCISE_STEPS",
                "exercise_steps must be within 1..n_steps",
                "choose exercise step indexes that are positive and no greater than n_steps",
            )
    return _execute_native_pricing("bermudan_put", "price_bermudan_put", cfg, native_module)


def price_heston_european_call(
    config: HestonEuropeanCallConfig | None = None,
    native_module: str = DEFAULT_NATIVE_MODULE,
    **overrides: Any,
) -> PricingResult:
    cfg = _coerce_config(HestonEuropeanCallConfig, config, overrides)
    _validate_heston_config(cfg)
    return _execute_native_pricing(
        "heston_european_call",
        "price_heston_european_call",
        cfg,
        native_module,
    )


def price_merton_jump_diffusion_call(
    config: MertonJumpDiffusionCallConfig | None = None,
    native_module: str = DEFAULT_NATIVE_MODULE,
    **overrides: Any,
) -> PricingResult:
    cfg = _coerce_config(MertonJumpDiffusionCallConfig, config, overrides)
    _validate_gbm_config(cfg)
    _validate_non_negative_float("jump_intensity", cfg.jump_intensity)
    _validate_non_negative_float("jump_volatility", cfg.jump_volatility)
    _validate_non_negative_int("n_threads", cfg.n_threads)
    return _execute_native_pricing(
        "merton_jump_diffusion_call",
        "price_merton_jump_diffusion_call",
        cfg,
        native_module,
    )


def gaussian_uncertainty_moments(
    config: GaussianUncertaintyConfig | None = None,
    native_module: str = DEFAULT_NATIVE_MODULE,
    **overrides: Any,
) -> NativeWorkloadResult:
    cfg = _coerce_config(GaussianUncertaintyConfig, config, overrides)
    _validate_positive_int("n_paths", cfg.n_paths)
    _validate_positive_int("dimensions", cfg.dimensions)
    _validate_sampling(cfg.sampling)
    return _execute_native_workload(
        "gaussian_uncertainty_moments",
        "gaussian_uncertainty_moments",
        cfg,
        native_module,
    )


def price_arithmetic_asian_mlmc(
    config: ArithmeticAsianMlmcConfig | None = None,
    native_module: str = DEFAULT_NATIVE_MODULE,
    **overrides: Any,
) -> NativeWorkloadResult:
    cfg = _coerce_config(ArithmeticAsianMlmcConfig, config, overrides)
    _validate_mlmc_config(cfg)
    return _execute_native_workload(
        "arithmetic_asian_mlmc",
        "arithmetic_asian_mlmc",
        cfg,
        native_module,
    )


def price_european_call_parameter_sweep(
    config: EuropeanCallParameterSweepConfig | None = None,
    native_module: str = DEFAULT_NATIVE_MODULE,
    **overrides: Any,
) -> NativeWorkloadResult:
    cfg = _coerce_config(EuropeanCallParameterSweepConfig, config, overrides)
    _validate_gbm_config(cfg.base_config)
    if cfg.n_paths is not None:
        _validate_positive_int("n_paths", cfg.n_paths)
    if cfg.n_steps is not None:
        _validate_positive_int("n_steps", cfg.n_steps)
    _validate_positive_int("seed_stride", cfg.seed_stride)
    if not cfg.scenarios:
        raise McConfigurationError(
            "MC_CONFIG_SWEEP_SCENARIOS",
            "scenarios must contain at least one scenario",
            "provide one or more EuropeanCallSweepScenario entries",
        )
    for scenario in cfg.scenarios:
        _validate_sweep_scenario(scenario)
    return _execute_native_workload(
        "european_call_parameter_sweep",
        "price_european_call_parameter_sweep",
        cfg,
        native_module,
    )


def _coerce_config(
    config_type: type[Any],
    config: Any | None,
    overrides: Mapping[str, Any],
) -> Any:
    if config is None:
        return config_type(**overrides)
    if overrides:
        return replace(config, **overrides)
    return config


def _validate_gbm_config(cfg: EuropeanCallConfig) -> None:
    if cfg.n_paths <= 0:
        raise McConfigurationError(
            "MC_CONFIG_PATHS",
            "n_paths must be greater than zero",
            "set n_paths to a positive integer, for example n_paths=100_000",
        )
    if cfg.n_steps <= 0:
        raise McConfigurationError(
            "MC_CONFIG_STEPS",
            "n_steps must be greater than zero",
            "set n_steps to a positive integer, for example n_steps=64",
        )
    for name in ("spot", "strike", "maturity"):
        if getattr(cfg, name) <= 0.0:
            raise McConfigurationError(
                "MC_CONFIG_POSITIVE",
                f"{name} must be greater than zero",
                f"set {name} to a positive number",
            )
    if cfg.volatility < 0.0:
        raise McConfigurationError(
            "MC_CONFIG_VOLATILITY",
            "volatility must be non-negative",
            "set volatility to zero or a positive decimal, for example 0.2",
        )


def _validate_positive_int(name: str, value: int) -> None:
    if value <= 0:
        raise McConfigurationError(
            "MC_CONFIG_POSITIVE_INT",
            f"{name} must be greater than zero",
            f"set {name} to a positive integer",
        )


def _validate_non_negative_int(name: str, value: int) -> None:
    if value < 0:
        raise McConfigurationError(
            "MC_CONFIG_NON_NEGATIVE_INT",
            f"{name} must be zero or greater",
            f"set {name} to zero or a positive integer",
        )


def _validate_non_negative_float(name: str, value: float) -> None:
    if value < 0.0:
        raise McConfigurationError(
            "MC_CONFIG_NON_NEGATIVE",
            f"{name} must be non-negative",
            f"set {name} to zero or a positive decimal",
        )


def _validate_correlation(name: str, value: float) -> None:
    if value < -1.0 or value > 1.0:
        raise McConfigurationError(
            "MC_CONFIG_CORRELATION",
            f"{name} must be between -1 and 1",
            f"set {name} within the closed interval [-1, 1]",
        )


def _validate_sampling(sampling: str | None) -> None:
    if sampling is None:
        return
    supported = {
        "pseudorandom",
        "randomized_halton",
        "latin_hypercube",
        "scrambled_sobol",
        "scrambled_sobol_brownian_bridge",
    }
    if sampling not in supported:
        raise McConfigurationError(
            "MC_CONFIG_SAMPLING",
            f"sampling '{sampling}' is not supported by the stable Python config surface",
            f"choose one of {sorted(supported)}",
        )


def _validate_sampling_and_technique(sampling: str | None, technique: str | None) -> None:
    _validate_sampling(sampling)
    supported = {"standard", "antithetic", "control_variate"}
    if technique is not None and technique not in supported:
        raise McConfigurationError(
            "MC_CONFIG_TECHNIQUE",
            f"technique '{technique}' is not supported by the stable Python config surface",
            f"choose one of {sorted(supported)}",
        )


def _validate_basket_config(cfg: BasketCallConfig) -> None:
    for name in ("spot_1", "spot_2", "strike", "maturity"):
        if getattr(cfg, name) <= 0.0:
            raise McConfigurationError(
                "MC_CONFIG_POSITIVE",
                f"{name} must be greater than zero",
                f"set {name} to a positive number",
            )
    _validate_non_negative_float("volatility_1", cfg.volatility_1)
    _validate_non_negative_float("volatility_2", cfg.volatility_2)
    _validate_correlation("correlation", cfg.correlation)
    _validate_positive_int("n_paths", cfg.n_paths)
    _validate_non_negative_int("n_threads", cfg.n_threads)
    _validate_sampling_and_technique(cfg.sampling, cfg.technique)


def _validate_lsm_config(basis_degree: int, n_threads: int) -> None:
    _validate_positive_int("basis_degree", basis_degree)
    _validate_non_negative_int("n_threads", n_threads)


def _validate_heston_config(cfg: HestonEuropeanCallConfig) -> None:
    for name in ("spot", "strike", "maturity", "mean_reversion"):
        if getattr(cfg, name) <= 0.0:
            raise McConfigurationError(
                "MC_CONFIG_POSITIVE",
                f"{name} must be greater than zero",
                f"set {name} to a positive number",
            )
    _validate_positive_int("n_paths", cfg.n_paths)
    _validate_positive_int("n_steps", cfg.n_steps)
    _validate_non_negative_float("initial_variance", cfg.initial_variance)
    _validate_non_negative_float("long_run_variance", cfg.long_run_variance)
    _validate_non_negative_float("vol_of_vol", cfg.vol_of_vol)
    _validate_correlation("correlation", cfg.correlation)
    _validate_sampling_and_technique(None, cfg.technique)
    _validate_non_negative_int("n_threads", cfg.n_threads)


def _validate_mlmc_config(cfg: ArithmeticAsianMlmcConfig) -> None:
    for name in ("spot", "strike", "maturity"):
        if getattr(cfg, name) <= 0.0:
            raise McConfigurationError(
                "MC_CONFIG_POSITIVE",
                f"{name} must be greater than zero",
                f"set {name} to a positive number",
            )
    _validate_non_negative_float("volatility", cfg.volatility)
    _validate_positive_int("base_steps", cfg.base_steps)
    _validate_positive_int("levels", cfg.levels)
    _validate_positive_int("refinement_factor", cfg.refinement_factor)
    _validate_positive_int("scramble_replicates", cfg.scramble_replicates)
    if len(cfg.paths_per_level) != cfg.levels:
        raise McConfigurationError(
            "MC_CONFIG_MLMC_LEVELS",
            "paths_per_level length must match levels",
            "provide one path count for each MLMC level",
        )
    for paths in cfg.paths_per_level:
        _validate_positive_int("paths_per_level", paths)
    if cfg.target_stderr is not None and cfg.target_stderr <= 0.0:
        raise McConfigurationError(
            "MC_CONFIG_TARGET_STDERR",
            "target_stderr must be greater than zero when provided",
            "set target_stderr to a positive decimal such as 0.05",
        )
    _validate_sampling(cfg.sampling)


def _validate_sweep_scenario(scenario: EuropeanCallSweepScenario) -> None:
    for name in ("spot", "strike", "maturity"):
        value = getattr(scenario, name)
        if value is not None and value <= 0.0:
            raise McConfigurationError(
                "MC_CONFIG_POSITIVE",
                f"scenario {name} must be greater than zero when provided",
                f"set scenario {name} to a positive number",
            )
    if scenario.volatility is not None:
        _validate_non_negative_float("scenario volatility", scenario.volatility)
    if scenario.n_paths is not None:
        _validate_positive_int("scenario n_paths", scenario.n_paths)
    if scenario.n_steps is not None:
        _validate_positive_int("scenario n_steps", scenario.n_steps)
    _validate_sampling(scenario.sampling)
    _validate_sampling_and_technique(None, scenario.technique)


def _execute_native_pricing(
    workload: str,
    function_name: str,
    cfg: Any,
    native_module: str,
) -> PricingResult:
    raw = _call_native_function(function_name, cfg, native_module)
    price = _required_float(raw, "price", function_name)
    stderr = float(raw.get("stderr", raw.get("standard_error", 0.0)))
    manifest = _native_manifest(workload, function_name, cfg, native_module, raw)
    return PricingResult(
        workload=workload,
        price=price,
        stderr=stderr,
        n_paths=int(getattr(cfg, "n_paths", getattr(cfg, "n_samples", 0))),
        n_steps=int(getattr(cfg, "n_steps", 1)),
        seed=int(getattr(cfg, "seed", 0)),
        manifest=manifest,
        warnings=_raw_warnings(raw),
    )


def _execute_native_workload(
    workload: str,
    function_name: str,
    cfg: Any,
    native_module: str,
) -> NativeWorkloadResult:
    raw = _call_native_function(function_name, cfg, native_module)
    values = raw.get("values")
    if not isinstance(values, Mapping):
        values = {
            key: value
            for key, value in raw.items()
            if key not in {"manifest", "warnings", "stderr", "standard_error"}
        }
    manifest = _native_manifest(workload, function_name, cfg, native_module, raw)
    stderr = raw.get("stderr", raw.get("standard_error"))
    return NativeWorkloadResult(
        workload=workload,
        values=values,
        stderr=None if stderr is None else float(stderr),
        manifest=manifest,
        warnings=_raw_warnings(raw),
    )


def _call_native_function(function_name: str, cfg: Any, native_module: str) -> Mapping[str, Any]:
    status = native_runtime_status(native_module)
    if not status.available:
        raise NativeRuntimeUnavailableError(status)
    module = require_native_runtime(native_module)
    function = getattr(module, function_name, None)
    if not callable(function):
        raise NativeFunctionUnavailableError(status, function_name)
    raw = function(_config_payload(cfg))
    if not isinstance(raw, Mapping):
        raise McConfigurationError(
            "MC_NATIVE_RESULT",
            f"native function '{function_name}' returned an unsupported result shape",
            "return a mapping containing result fields, manifest, and optional warnings",
        )
    return raw


def _config_payload(cfg: Any) -> dict[str, Any]:
    payload = asdict(cfg)
    if "base_config" in payload and hasattr(cfg, "base_config"):
        payload["base_config"] = asdict(cfg.base_config)
    return payload


def _native_manifest(
    workload: str,
    function_name: str,
    cfg: Any,
    native_module: str,
    raw: Mapping[str, Any],
) -> dict[str, Any]:
    raw_manifest = raw.get("manifest")
    if not isinstance(raw_manifest, Mapping):
        raw_manifest = {}
    manifest = {
        "schema_version": "python-native-bridge.v1",
        "workload": workload,
        "backend": "native_runtime",
        "function": function_name,
        "native_module": native_module,
        "seed": getattr(cfg, "seed", None),
        "config": _config_payload(cfg),
        "reproducibility": "delegated to compiled native runtime manifest",
        "performance_claim": "native timing claims require benchmark artifacts",
    } | dict(raw_manifest)
    manifest["workload"] = workload
    manifest["function"] = function_name
    manifest["native_module"] = native_module
    manifest["config"] = _config_payload(cfg)
    return manifest


def _raw_warnings(raw: Mapping[str, Any]) -> tuple[str, ...]:
    warnings = raw.get("warnings", ())
    if isinstance(warnings, str):
        return (warnings,)
    return tuple(str(item) for item in warnings)


def _required_float(raw: Mapping[str, Any], field: str, function_name: str) -> float:
    if field not in raw:
        raise McConfigurationError(
            "MC_NATIVE_RESULT",
            f"native function '{function_name}' did not return required field '{field}'",
            f"include a numeric '{field}' in the native result mapping",
        )
    return float(raw[field])


def _simulate_gbm_payoff(
    workload: str,
    cfg: EuropeanCallConfig,
    payoff_fn: Any,
) -> PricingResult:
    rng = random.Random(cfg.seed)
    dt = cfg.maturity / cfg.n_steps
    drift = (cfg.rate - 0.5 * cfg.volatility * cfg.volatility) * dt
    vol_step = cfg.volatility * math.sqrt(dt)
    discount = math.exp(-cfg.rate * cfg.maturity)
    payoffs: list[float] = []

    for _ in range(cfg.n_paths):
        path = _simulate_gbm_path(cfg.spot, cfg.n_steps, drift, vol_step, rng)
        payoffs.append(discount * payoff_fn(cfg, path))

    price = sum(payoffs) / cfg.n_paths
    variance = _sample_variance(payoffs, price)
    stderr = math.sqrt(variance / cfg.n_paths)
    return PricingResult(
        workload=workload,
        price=price,
        stderr=stderr,
        n_paths=cfg.n_paths,
        n_steps=cfg.n_steps,
        seed=cfg.seed,
        manifest=_base_manifest(workload, cfg),
        warnings=(
            "Python helper is for ergonomics and reproducibility demos; use Rust artifacts for performance claims.",
        ),
    )


def _simulate_gbm_path(
    spot: float,
    n_steps: int,
    drift: float,
    vol_step: float,
    rng: random.Random,
) -> list[float]:
    path = []
    current = spot
    for _ in range(n_steps):
        current *= math.exp(drift + vol_step * rng.gauss(0.0, 1.0))
        path.append(current)
    return path


def _european_payoff(cfg: EuropeanCallConfig, path: list[float]) -> float:
    return max(path[-1] - cfg.strike, 0.0)


def _asian_payoff(cfg: EuropeanCallConfig, path: list[float]) -> float:
    return max((sum(path) / len(path)) - cfg.strike, 0.0)


def _down_and_out_payoff(cfg: DownAndOutCallConfig, path: list[float]) -> float:
    if min(path) <= cfg.barrier:
        return 0.0
    return max(path[-1] - cfg.strike, 0.0)


def _sample_variance(values: list[float], mean: float) -> float:
    if len(values) < 2:
        return 0.0
    return sum((value - mean) ** 2 for value in values) / (len(values) - 1)


def _base_manifest(workload: str, cfg: EuropeanCallConfig) -> dict[str, Any]:
    return {
        "schema_version": "python-ux.v1",
        "workload": workload,
        "backend": "python_reference",
        "seed": cfg.seed,
        "config": asdict(cfg),
        "reproducibility": "deterministic for same Python version, config, and seed",
        "performance_claim": "none; use Rust benchmark artifacts for timing claims",
    }


def _black_scholes_greeks(cfg: EuropeanCallConfig) -> dict[str, float]:
    sqrt_t = math.sqrt(cfg.maturity)
    if cfg.volatility == 0.0:
        forward = cfg.spot - cfg.strike * math.exp(-cfg.rate * cfg.maturity)
        price = max(forward, 0.0)
        delta = 1.0 if forward > 0.0 else 0.0
        return {"price": price, "delta": delta, "vega": 0.0, "rho": 0.0, "theta": 0.0}

    d1 = (
        math.log(cfg.spot / cfg.strike)
        + (cfg.rate + 0.5 * cfg.volatility * cfg.volatility) * cfg.maturity
    ) / (cfg.volatility * sqrt_t)
    d2 = d1 - cfg.volatility * sqrt_t
    norm_d1 = _normal_cdf(d1)
    norm_d2 = _normal_cdf(d2)
    density_d1 = math.exp(-0.5 * d1 * d1) / math.sqrt(2.0 * math.pi)
    discount = math.exp(-cfg.rate * cfg.maturity)
    return {
        "price": cfg.spot * norm_d1 - cfg.strike * discount * norm_d2,
        "delta": norm_d1,
        "vega": cfg.spot * density_d1 * sqrt_t,
        "rho": cfg.strike * cfg.maturity * discount * norm_d2,
        "theta": -(
            cfg.spot * density_d1 * cfg.volatility / (2.0 * sqrt_t)
        )
        - cfg.rate * cfg.strike * discount * norm_d2,
    }


def _normal_cdf(x: float) -> float:
    return 0.5 * (1.0 + math.erf(x / math.sqrt(2.0)))
