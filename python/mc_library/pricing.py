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
        }.get(self.workload, self.workload)
        return (
            f"{label} priced with Python reference Monte Carlo: "
            f"price={self.price:.6f}, stderr={self.stderr:.6f}, "
            f"paths={self.n_paths}, steps={self.n_steps}, seed={self.seed}."
        )

    def reproduce(self) -> ReproductionRecipe:
        cfg = self.manifest["config"]
        helper = {
            "european_call": "price_european_call",
            "arithmetic_asian_call": "price_arithmetic_asian_call",
            "down_and_out_call": "price_down_and_out_call",
        }[self.workload]
        return ReproductionRecipe(
            python=(
                f"from mc_library import {helper}\n"
                f"result = {helper}(**{cfg!r})\n"
                "print(result.price, result.stderr)\n"
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
                "from mc_library import price_european_call_greeks\n"
                f"report = price_european_call_greeks(**{cfg!r})\n"
                "print(report.greeks)\n"
            ),
            manifest=self.manifest,
        )


def price_european_call(
    config: EuropeanCallConfig | None = None,
    **overrides: Any,
) -> PricingResult:
    cfg = _coerce_config(EuropeanCallConfig, config, overrides)
    _validate_gbm_config(cfg)
    return _simulate_gbm_payoff("european_call", cfg, _european_payoff)


def price_arithmetic_asian_call(
    config: ArithmeticAsianCallConfig | None = None,
    **overrides: Any,
) -> PricingResult:
    cfg = _coerce_config(ArithmeticAsianCallConfig, config, overrides)
    _validate_gbm_config(cfg)
    return _simulate_gbm_payoff("arithmetic_asian_call", cfg, _asian_payoff)


def price_down_and_out_call(
    config: DownAndOutCallConfig | None = None,
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


def _coerce_config(
    config_type: type[EuropeanCallConfig],
    config: EuropeanCallConfig | None,
    overrides: Mapping[str, Any],
) -> EuropeanCallConfig:
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

