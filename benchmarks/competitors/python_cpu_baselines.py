#!/usr/bin/env python3
"""CPU baseline benchmark for Monte Carlo European call pricing.

This script compares commonly used Python libraries when available and emits JSON.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import math
import platform
import sys
import time
import warnings
from dataclasses import asdict, dataclass
from typing import Any


@dataclass
class LibraryResult:
    library: str
    methodology: str | None
    available: bool
    runtime_ms: float | None
    price: float | None
    stderr: float | None
    note: str | None
    metric_name: str | None = None
    metric_value: float | None = None


def has_module(name: str) -> bool:
    return importlib.util.find_spec(name) is not None


def benchmark_numpy(n_paths: int, n_steps: int, repeats: int, seed: int) -> LibraryResult:
    import numpy as np

    dt = 1.0 / n_steps
    drift = (0.03 - 0.5 * 0.2 * 0.2) * dt
    vol = 0.2 * math.sqrt(dt)
    discount = math.exp(-0.03 * 1.0)

    times: list[float] = []
    prices: list[float] = []
    stderrs: list[float] = []

    for rep in range(repeats):
        rng = np.random.default_rng(seed + rep)

        start = time.perf_counter()
        s_t = np.full(n_paths, 100.0, dtype=np.float64)
        for _ in range(n_steps):
            z = rng.standard_normal(n_paths)
            s_t *= np.exp(drift + vol * z)

        payoff = np.maximum(s_t - 100.0, 0.0) * discount
        elapsed = (time.perf_counter() - start) * 1000.0

        price = float(np.mean(payoff))
        stderr = float(np.std(payoff, ddof=0) / np.sqrt(n_paths))

        times.append(elapsed)
        prices.append(price)
        stderrs.append(stderr)

    return LibraryResult(
        library="numpy",
        methodology="stepwise_paths",
        available=True,
        runtime_ms=sum(times) / len(times),
        price=sum(prices) / len(prices),
        stderr=sum(stderrs) / len(stderrs),
        note=None,
    )


def benchmark_numpy_terminal(
    n_paths: int, repeats: int, seed: int
) -> LibraryResult:
    import numpy as np

    drift_t = (0.03 - 0.5 * 0.2 * 0.2) * 1.0
    vol_t = 0.2 * math.sqrt(1.0)
    discount = math.exp(-0.03 * 1.0)

    times: list[float] = []
    prices: list[float] = []
    stderrs: list[float] = []

    for rep in range(repeats):
        rng = np.random.default_rng(seed + rep)

        start = time.perf_counter()
        z = rng.standard_normal(n_paths)
        s_t = 100.0 * np.exp(drift_t + vol_t * z)
        payoff = np.maximum(s_t - 100.0, 0.0) * discount
        elapsed = (time.perf_counter() - start) * 1000.0

        price = float(np.mean(payoff))
        stderr = float(np.std(payoff, ddof=0) / np.sqrt(n_paths))

        times.append(elapsed)
        prices.append(price)
        stderrs.append(stderr)

    return LibraryResult(
        library="numpy",
        methodology="terminal_distribution",
        available=True,
        runtime_ms=sum(times) / len(times),
        price=sum(prices) / len(prices),
        stderr=sum(stderrs) / len(stderrs),
        note=None,
    )


def benchmark_numba(n_paths: int, n_steps: int, repeats: int, seed: int) -> LibraryResult:
    import numba as nb
    import numpy as np

    @nb.njit(cache=True)
    def _price(seed_local: int, n_paths_local: int, n_steps_local: int) -> tuple[float, float]:
        np.random.seed(seed_local)
        dt = 1.0 / n_steps_local
        drift = (0.03 - 0.5 * 0.2 * 0.2) * dt
        vol = 0.2 * math.sqrt(dt)
        discount = math.exp(-0.03 * 1.0)

        payoff_sum = 0.0
        payoff_sq_sum = 0.0
        for _ in range(n_paths_local):
            s_t = 100.0
            for _ in range(n_steps_local):
                z = np.random.normal()
                s_t *= math.exp(drift + vol * z)

            payoff = max(s_t - 100.0, 0.0) * discount
            payoff_sum += payoff
            payoff_sq_sum += payoff * payoff

        n = float(n_paths_local)
        price = payoff_sum / n
        var = payoff_sq_sum / n - price * price
        stderr = math.sqrt(max(var, 0.0)) / math.sqrt(n)
        return price, stderr

    # JIT warm-up compile.
    _price(seed, 4_000, 16)

    times: list[float] = []
    prices: list[float] = []
    stderrs: list[float] = []

    for rep in range(repeats):
        start = time.perf_counter()
        price, stderr = _price(seed + rep, n_paths, n_steps)
        elapsed = (time.perf_counter() - start) * 1000.0
        times.append(float(elapsed))
        prices.append(float(price))
        stderrs.append(float(stderr))

    return LibraryResult(
        library="numba",
        methodology="stepwise_paths",
        available=True,
        runtime_ms=sum(times) / len(times),
        price=sum(prices) / len(prices),
        stderr=sum(stderrs) / len(stderrs),
        note="execution-only timing after JIT warm-up",
    )


def benchmark_numba_terminal(n_paths: int, repeats: int, seed: int) -> LibraryResult:
    import numba as nb
    import numpy as np

    @nb.njit(cache=True)
    def _price(seed_local: int, n_paths_local: int) -> tuple[float, float]:
        np.random.seed(seed_local)
        drift_t = (0.03 - 0.5 * 0.2 * 0.2) * 1.0
        vol_t = 0.2 * math.sqrt(1.0)
        discount = math.exp(-0.03 * 1.0)

        payoff_sum = 0.0
        payoff_sq_sum = 0.0
        for _ in range(n_paths_local):
            z = np.random.normal()
            s_t = 100.0 * math.exp(drift_t + vol_t * z)
            payoff = max(s_t - 100.0, 0.0) * discount
            payoff_sum += payoff
            payoff_sq_sum += payoff * payoff

        n = float(n_paths_local)
        price = payoff_sum / n
        var = payoff_sq_sum / n - price * price
        stderr = math.sqrt(max(var, 0.0)) / math.sqrt(n)
        return price, stderr

    _price(seed, 4_000)

    times: list[float] = []
    prices: list[float] = []
    stderrs: list[float] = []

    for rep in range(repeats):
        start = time.perf_counter()
        price, stderr = _price(seed + rep, n_paths)
        elapsed = (time.perf_counter() - start) * 1000.0
        times.append(float(elapsed))
        prices.append(float(price))
        stderrs.append(float(stderr))

    return LibraryResult(
        library="numba",
        methodology="terminal_distribution",
        available=True,
        runtime_ms=sum(times) / len(times),
        price=sum(prices) / len(prices),
        stderr=sum(stderrs) / len(stderrs),
        note="execution-only timing after JIT warm-up",
    )


def benchmark_scipy_qmc_generation(
    method: str, n_points: int, dimensions: int, repeats: int, seed: int
) -> LibraryResult:
    import numpy as np
    from scipy import stats
    from scipy.special import ndtri

    times: list[float] = []
    means: list[float] = []
    methodology = {
        "sobol": "standard_normal_generation_scrambled_sobol",
        "halton": "standard_normal_generation_randomized_halton",
        "lhs": "standard_normal_generation_latin_hypercube",
    }[method]

    for rep in range(repeats):
        if method == "sobol":
            engine = stats.qmc.Sobol(d=dimensions, scramble=True, seed=seed + rep)
        elif method == "halton":
            engine = stats.qmc.Halton(d=dimensions, scramble=True, seed=seed + rep)
        else:
            engine = stats.qmc.LatinHypercube(d=dimensions, scramble=True, seed=seed + rep)

        start = time.perf_counter()
        with warnings.catch_warnings():
            warnings.simplefilter("ignore")
            uniforms = engine.random(n_points)
        normals = ndtri(np.clip(uniforms, np.finfo(np.float64).tiny, 1.0 - np.finfo(np.float64).eps))
        elapsed = (time.perf_counter() - start) * 1000.0

        times.append(elapsed)
        means.append(float(abs(np.mean(normals))))

    return LibraryResult(
        library=f"scipy_qmc_{method}",
        methodology=methodology,
        available=True,
        runtime_ms=sum(times) / len(times),
        price=None,
        stderr=None,
        note="generation-only benchmark; metric is absolute sample mean of generated normals",
        metric_name="normal_mean_abs",
        metric_value=sum(means) / len(means),
    )


def benchmark_quantlib_european(
    n_paths: int, n_steps: int, repeats: int, seed: int
) -> LibraryResult:
    import QuantLib as ql

    calendar = ql.NullCalendar()
    day_count = ql.Actual365Fixed()
    evaluation_date = ql.Date(2, ql.January, 2023)
    maturity_date = ql.Date(2, ql.January, 2024)
    ql.Settings.instance().evaluationDate = evaluation_date

    spot = ql.QuoteHandle(ql.SimpleQuote(100.0))
    dividend_curve = ql.YieldTermStructureHandle(
        ql.FlatForward(evaluation_date, 0.0, day_count)
    )
    risk_free_curve = ql.YieldTermStructureHandle(
        ql.FlatForward(evaluation_date, 0.03, day_count)
    )
    volatility = ql.BlackVolTermStructureHandle(
        ql.BlackConstantVol(evaluation_date, calendar, 0.2, day_count)
    )
    process = ql.BlackScholesMertonProcess(
        spot, dividend_curve, risk_free_curve, volatility
    )
    payoff = ql.PlainVanillaPayoff(ql.Option.Call, 100.0)
    exercise = ql.EuropeanExercise(maturity_date)

    times: list[float] = []
    prices: list[float] = []
    stderrs: list[float] = []

    for rep in range(repeats):
        option = ql.VanillaOption(payoff, exercise)

        start = time.perf_counter()
        engine = ql.MCEuropeanEngine(
            process,
            "pseudorandom",
            timeSteps=n_steps,
            brownianBridge=False,
            antitheticVariate=False,
            requiredSamples=n_paths,
            seed=seed + rep,
        )
        option.setPricingEngine(engine)
        price = float(option.NPV())
        elapsed = (time.perf_counter() - start) * 1000.0

        try:
            stderr = float(option.errorEstimate())
        except RuntimeError:
            stderr = math.nan

        times.append(elapsed)
        prices.append(price)
        if math.isfinite(stderr):
            stderrs.append(stderr)

    return LibraryResult(
        library="quantlib",
        methodology="stepwise_paths_quantlib_mceuropean",
        available=True,
        runtime_ms=sum(times) / len(times),
        price=sum(prices) / len(prices),
        stderr=(sum(stderrs) / len(stderrs)) if stderrs else None,
        note=(
            "QuantLib-Python MCEuropeanEngine with pseudorandom samples; "
            "calendar/curve setup is fixed to match the tracked European call."
        ),
    )


def unavailable(name: str, methodology: str | None, note: str) -> LibraryResult:
    return LibraryResult(
        library=name,
        methodology=methodology,
        available=False,
        runtime_ms=None,
        price=None,
        stderr=None,
        note=note,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Run Python competitor MC baselines")
    parser.add_argument("--paths", type=int, required=True)
    parser.add_argument("--steps", type=int, required=True)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--seed", type=int, default=42)
    args = parser.parse_args()

    results: list[LibraryResult] = []

    if has_module("numpy"):
        results.append(benchmark_numpy(args.paths, args.steps, args.repeats, args.seed))
        results.append(benchmark_numpy_terminal(args.paths, args.repeats, args.seed))
    else:
        results.append(unavailable("numpy", "stepwise_paths", "package not installed"))
        results.append(
            unavailable("numpy", "terminal_distribution", "package not installed")
        )

    if has_module("numba"):
        results.append(benchmark_numba(args.paths, args.steps, args.repeats, args.seed))
        results.append(benchmark_numba_terminal(args.paths, args.repeats, args.seed))
    else:
        results.append(unavailable("numba", "stepwise_paths", "package not installed"))
        results.append(
            unavailable("numba", "terminal_distribution", "package not installed")
        )

    if has_module("QuantLib"):
        try:
            results.append(
                benchmark_quantlib_european(
                    args.paths, args.steps, args.repeats, args.seed
                )
            )
        except Exception as exc:
            results.append(
                unavailable(
                    "quantlib",
                    "stepwise_paths_quantlib_mceuropean",
                    f"benchmark failed: {type(exc).__name__}: {exc}",
                )
            )
    else:
        results.append(
            unavailable(
                "quantlib",
                "stepwise_paths_quantlib_mceuropean",
                "QuantLib-Python package not installed",
            )
        )

    if has_module("scipy") and has_module("numpy"):
        results.append(
            benchmark_scipy_qmc_generation(
                "sobol", args.paths, args.steps, args.repeats, args.seed
            )
        )
        results.append(
            benchmark_scipy_qmc_generation(
                "halton", args.paths, args.steps, args.repeats, args.seed
            )
        )
        results.append(
            benchmark_scipy_qmc_generation(
                "lhs", args.paths, args.steps, args.repeats, args.seed
            )
        )
    else:
        results.append(
            unavailable(
                "scipy_qmc_sobol",
                "standard_normal_generation_scrambled_sobol",
                "scipy or numpy package not installed",
            )
        )
        results.append(
            unavailable(
                "scipy_qmc_halton",
                "standard_normal_generation_randomized_halton",
                "scipy or numpy package not installed",
            )
        )
        results.append(
            unavailable(
                "scipy_qmc_lhs",
                "standard_normal_generation_latin_hypercube",
                "scipy or numpy package not installed",
            )
        )

    for gpu_lib in ("jax", "cupy", "torch"):
        if has_module(gpu_lib):
            results.append(
                unavailable(
                    gpu_lib,
                    None,
                    "detected but GPU-targeted benchmark path not implemented in this CPU script",
                )
            )
        else:
            results.append(unavailable(gpu_lib, None, "package not installed"))

    payload: dict[str, Any] = {
        "environment": {
            "python_version": sys.version.split()[0],
            "platform": platform.platform(),
            "paths": args.paths,
            "steps": args.steps,
            "repeats": args.repeats,
        },
        "results": [asdict(r) for r in results],
    }

    print(json.dumps(payload))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
