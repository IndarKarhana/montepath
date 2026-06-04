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
from importlib import metadata
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
    telemetry: dict[str, Any] | None = None
    reproducibility: str | None = None


def has_module(name: str) -> bool:
    return importlib.util.find_spec(name) is not None


def package_version(distribution_name: str) -> str | None:
    try:
        return metadata.version(distribution_name)
    except metadata.PackageNotFoundError:
        return None


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


def benchmark_inventory_numpy(
    n_paths: int, n_periods: int, repeats: int, seed: int
) -> LibraryResult:
    import numpy as np

    times: list[float] = []
    mean_total_costs: list[float] = []
    for rep in range(repeats):
        rng = np.random.default_rng(seed + rep)
        start = time.perf_counter()
        on_hand = np.full(n_paths, 100.0, dtype=np.float64)
        on_order = np.zeros(n_paths, dtype=np.float64)
        arrivals = np.zeros(n_paths, dtype=np.float64)
        total_cost = np.zeros(n_paths, dtype=np.float64)

        for period in range(n_periods):
            receipt = arrivals
            arrivals = np.zeros(n_paths, dtype=np.float64)
            on_order -= receipt
            on_hand += receipt

            demand = np.maximum(0.0, 10.0 + 2.0 * rng.standard_normal(n_paths))
            fulfilled = np.minimum(demand, on_hand)
            on_hand -= fulfilled
            unmet = demand - fulfilled

            inventory_position = on_hand + on_order
            order_mask = inventory_position <= 50.0
            order_quantity = np.where(
                order_mask,
                np.ceil(np.maximum(0.0, 100.0 - inventory_position)),
                0.0,
            )
            arrivals += order_quantity
            on_order += order_quantity

            if period >= 4:
                total_cost += on_hand * 0.1
                total_cost += unmet * 2.0
                total_cost += np.where(
                    order_quantity > 0.0,
                    5.0 + order_quantity * 0.01,
                    0.0,
                )

        elapsed = (time.perf_counter() - start) * 1000.0
        times.append(elapsed)
        mean_total_costs.append(float(np.mean(total_cost)))

    return LibraryResult(
        library="numpy",
        methodology="inventory_periodic_review_fixed_lead_time",
        available=True,
        runtime_ms=sum(times) / len(times),
        price=None,
        stderr=None,
        note=(
            "Vectorized NumPy inventory baseline with identical periodic-review, "
            "warm-up, lead-time, case-pack, and cost semantics."
        ),
        metric_name="mean_total_cost",
        metric_value=sum(mean_total_costs) / len(mean_total_costs),
        reproducibility=(
            "statistically reproducible for fixed NumPy version and seed; "
            "random paths are not identical to montepath Rust streams"
        ),
    )


def benchmark_inventory_numba(
    n_paths: int, n_periods: int, repeats: int, seed: int
) -> LibraryResult:
    import numba as nb
    import numpy as np

    @nb.njit(cache=True)
    def _simulate(seed_local: int, path_count: int, period_count: int) -> float:
        np.random.seed(seed_local)
        total_cost_sum = 0.0
        for _ in range(path_count):
            on_hand = 100.0
            on_order = 0.0
            arrival = 0.0
            total_cost = 0.0
            for period in range(period_count):
                receipt = arrival
                arrival = 0.0
                on_order -= receipt
                on_hand += receipt

                demand = max(0.0, 10.0 + 2.0 * np.random.normal())
                fulfilled = min(demand, on_hand)
                on_hand -= fulfilled
                unmet = demand - fulfilled

                inventory_position = on_hand + on_order
                order_quantity = 0.0
                if inventory_position <= 50.0:
                    order_quantity = math.ceil(max(0.0, 100.0 - inventory_position))
                    arrival += order_quantity
                    on_order += order_quantity

                if period >= 4:
                    total_cost += on_hand * 0.1 + unmet * 2.0
                    if order_quantity > 0.0:
                        total_cost += 5.0 + order_quantity * 0.01
            total_cost_sum += total_cost
        return total_cost_sum / path_count

    _simulate(seed, min(n_paths, 256), min(n_periods, 16))
    times: list[float] = []
    mean_total_costs: list[float] = []
    for rep in range(repeats):
        start = time.perf_counter()
        mean_total_cost = _simulate(seed + rep, n_paths, n_periods)
        times.append((time.perf_counter() - start) * 1000.0)
        mean_total_costs.append(float(mean_total_cost))

    return LibraryResult(
        library="numba",
        methodology="inventory_periodic_review_fixed_lead_time",
        available=True,
        runtime_ms=sum(times) / len(times),
        price=None,
        stderr=None,
        note=(
            "Compiled Numba inventory loop with identical periodic-review, warm-up, "
            "lead-time, case-pack, and cost semantics; timing excludes JIT warm-up."
        ),
        metric_name="mean_total_cost",
        metric_value=sum(mean_total_costs) / len(mean_total_costs),
        reproducibility=(
            "statistically reproducible for fixed Numba/NumPy versions and seed; "
            "random paths are not identical to montepath Rust streams"
        ),
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


def _accelerator_methodology(library: str) -> str:
    return f"terminal_distribution_gpu_{library}"


def _accelerator_unavailable(library: str, note: str) -> LibraryResult:
    return unavailable(
        library,
        _accelerator_methodology(library),
        note,
        telemetry={
            "warmup_ms": None,
            "compile_ms": None,
            "execution_ms": None,
            "memory_peak_mb": None,
            "device": None,
        },
        reproducibility=(
            "unavailable in this environment; no accelerator reproducibility "
            "claim is made"
        ),
    )


def benchmark_jax_gpu(n_paths: int, repeats: int, seed: int) -> LibraryResult:
    import jax
    import jax.numpy as jnp
    from jax import random as jrandom

    gpu_devices = [device for device in jax.devices() if device.platform == "gpu"]
    if not gpu_devices:
        return _accelerator_unavailable("jax", "JAX installed but no GPU device found")

    device = gpu_devices[0]
    drift_t = (0.03 - 0.5 * 0.2 * 0.2) * 1.0
    vol_t = 0.2
    discount = math.exp(-0.03)

    @jax.jit
    def _price(key: Any) -> Any:
        z = jrandom.normal(key, (n_paths,), dtype=jnp.float64)
        s_t = 100.0 * jnp.exp(drift_t + vol_t * z)
        payoff = jnp.maximum(s_t - 100.0, 0.0) * discount
        return jnp.mean(payoff), jnp.std(payoff) / jnp.sqrt(float(n_paths))

    warmup_start = time.perf_counter()
    warm_price, warm_stderr = _price(jax.device_put(jrandom.PRNGKey(seed), device))
    warm_price.block_until_ready()
    compile_ms = (time.perf_counter() - warmup_start) * 1000.0

    times: list[float] = []
    prices: list[float] = []
    stderrs: list[float] = []
    for rep in range(repeats):
        start = time.perf_counter()
        price, stderr = _price(jax.device_put(jrandom.PRNGKey(seed + rep + 1), device))
        price.block_until_ready()
        elapsed = (time.perf_counter() - start) * 1000.0
        times.append(elapsed)
        prices.append(float(price))
        stderrs.append(float(stderr))

    runtime_ms = sum(times) / len(times)
    return LibraryResult(
        library="jax",
        methodology=_accelerator_methodology("jax"),
        available=True,
        runtime_ms=runtime_ms,
        price=sum(prices) / len(prices),
        stderr=sum(stderrs) / len(stderrs),
        note="JAX GPU terminal-distribution benchmark; compile/warmup reported separately.",
        telemetry={
            "warmup_ms": compile_ms,
            "compile_ms": compile_ms,
            "execution_ms": runtime_ms,
            "memory_peak_mb": n_paths * 8 / (1024 * 1024),
            "device": str(device),
        },
        reproducibility=(
            "statistical reproducibility for same seed and JAX/device stack; "
            "exact replay is not guaranteed across devices or versions"
        ),
    )


def benchmark_cupy_gpu(n_paths: int, repeats: int, seed: int) -> LibraryResult:
    import cupy as cp

    try:
        device_count = cp.cuda.runtime.getDeviceCount()
    except Exception as exc:
        return _accelerator_unavailable(
            "cupy", f"CuPy installed but CUDA device probe failed: {type(exc).__name__}: {exc}"
        )
    if device_count <= 0:
        return _accelerator_unavailable("cupy", "CuPy installed but no CUDA device found")

    drift_t = (0.03 - 0.5 * 0.2 * 0.2) * 1.0
    vol_t = 0.2
    discount = math.exp(-0.03)

    cp.random.seed(seed)
    warmup_start = time.perf_counter()
    z = cp.random.standard_normal(n_paths, dtype=cp.float64)
    s_t = 100.0 * cp.exp(drift_t + vol_t * z)
    payoff = cp.maximum(s_t - 100.0, 0.0) * discount
    _ = float(cp.mean(payoff).get())
    cp.cuda.Stream.null.synchronize()
    warmup_ms = (time.perf_counter() - warmup_start) * 1000.0

    try:
        mempool = cp.get_default_memory_pool()
        mempool.free_all_blocks()
    except Exception:
        mempool = None

    times: list[float] = []
    prices: list[float] = []
    stderrs: list[float] = []
    for rep in range(repeats):
        cp.random.seed(seed + rep + 1)
        start = time.perf_counter()
        z = cp.random.standard_normal(n_paths, dtype=cp.float64)
        s_t = 100.0 * cp.exp(drift_t + vol_t * z)
        payoff = cp.maximum(s_t - 100.0, 0.0) * discount
        price = cp.mean(payoff)
        stderr = cp.std(payoff) / math.sqrt(n_paths)
        price_host = float(price.get())
        stderr_host = float(stderr.get())
        cp.cuda.Stream.null.synchronize()
        elapsed = (time.perf_counter() - start) * 1000.0
        times.append(elapsed)
        prices.append(price_host)
        stderrs.append(stderr_host)

    runtime_ms = sum(times) / len(times)
    memory_peak_mb = None
    if mempool is not None:
        try:
            memory_peak_mb = mempool.total_bytes() / (1024 * 1024)
        except Exception:
            memory_peak_mb = None
    return LibraryResult(
        library="cupy",
        methodology=_accelerator_methodology("cupy"),
        available=True,
        runtime_ms=runtime_ms,
        price=sum(prices) / len(prices),
        stderr=sum(stderrs) / len(stderrs),
        note="CuPy CUDA terminal-distribution benchmark with explicit synchronization.",
        telemetry={
            "warmup_ms": warmup_ms,
            "compile_ms": 0.0,
            "execution_ms": runtime_ms,
            "memory_peak_mb": memory_peak_mb,
            "device": str(cp.cuda.Device()),
        },
        reproducibility=(
            "statistical reproducibility for same seed and CUDA/CuPy stack; "
            "exact replay is not guaranteed across devices or versions"
        ),
    )


def benchmark_torch_gpu(n_paths: int, repeats: int, seed: int) -> LibraryResult:
    import torch

    if not torch.cuda.is_available():
        return _accelerator_unavailable(
            "torch", "PyTorch installed but torch.cuda.is_available() is false"
        )

    device = torch.device("cuda")
    drift_t = (0.03 - 0.5 * 0.2 * 0.2) * 1.0
    vol_t = 0.2
    discount = math.exp(-0.03)

    torch.manual_seed(seed)
    warmup_start = time.perf_counter()
    z = torch.randn(n_paths, device=device, dtype=torch.float64)
    s_t = 100.0 * torch.exp(drift_t + vol_t * z)
    payoff = torch.clamp(s_t - 100.0, min=0.0) * discount
    _ = float(torch.mean(payoff).cpu())
    torch.cuda.synchronize()
    warmup_ms = (time.perf_counter() - warmup_start) * 1000.0
    torch.cuda.reset_peak_memory_stats(device)

    times: list[float] = []
    prices: list[float] = []
    stderrs: list[float] = []
    for rep in range(repeats):
        torch.manual_seed(seed + rep + 1)
        start = time.perf_counter()
        z = torch.randn(n_paths, device=device, dtype=torch.float64)
        s_t = 100.0 * torch.exp(drift_t + vol_t * z)
        payoff = torch.clamp(s_t - 100.0, min=0.0) * discount
        price = torch.mean(payoff)
        stderr = torch.std(payoff, unbiased=False) / math.sqrt(n_paths)
        price_host = float(price.cpu())
        stderr_host = float(stderr.cpu())
        torch.cuda.synchronize()
        elapsed = (time.perf_counter() - start) * 1000.0
        times.append(elapsed)
        prices.append(price_host)
        stderrs.append(stderr_host)

    runtime_ms = sum(times) / len(times)
    return LibraryResult(
        library="torch",
        methodology=_accelerator_methodology("torch"),
        available=True,
        runtime_ms=runtime_ms,
        price=sum(prices) / len(prices),
        stderr=sum(stderrs) / len(stderrs),
        note="PyTorch CUDA terminal-distribution benchmark with explicit synchronization.",
        telemetry={
            "warmup_ms": warmup_ms,
            "compile_ms": 0.0,
            "execution_ms": runtime_ms,
            "memory_peak_mb": torch.cuda.max_memory_allocated(device) / (1024 * 1024),
            "device": torch.cuda.get_device_name(device),
        },
        reproducibility=(
            "statistical reproducibility for same seed and PyTorch/CUDA stack; "
            "exact replay is not guaranteed across devices or versions"
        ),
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


def benchmark_quantlib_fixed_strike_lookback(
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
    max_so_far = 100.0

    times: list[float] = []
    prices: list[float] = []
    stderrs: list[float] = []

    for rep in range(repeats):
        option = ql.FixedStrikeLookbackOption(max_so_far, payoff, exercise)

        start = time.perf_counter()
        engine = ql.MCLookbackEngine(
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
        methodology="lookback_fixed_strike_stepwise_quantlib_mc",
        available=True,
        runtime_ms=sum(times) / len(times),
        price=sum(prices) / len(prices),
        stderr=(sum(stderrs) / len(stderrs)) if stderrs else None,
        note=(
            "QuantLib-Python fixed-strike lookback option with MCLookbackEngine; "
            "max-so-far starts at spot to match the tracked discrete-monitoring workload."
        ),
    )


def benchmark_quantlib_heston_european(
    n_paths: int, n_steps: int, repeats: int, seed: int
) -> LibraryResult:
    import QuantLib as ql

    _ = (n_paths, n_steps, seed)
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
    process = ql.HestonProcess(
        risk_free_curve,
        dividend_curve,
        spot,
        0.04,
        1.5,
        0.04,
        0.3,
        -0.6,
    )
    model = ql.HestonModel(process)
    payoff = ql.PlainVanillaPayoff(ql.Option.Call, 100.0)
    exercise = ql.EuropeanExercise(maturity_date)

    times: list[float] = []
    prices: list[float] = []

    for _ in range(repeats):
        option = ql.VanillaOption(payoff, exercise)

        start = time.perf_counter()
        engine = ql.AnalyticHestonEngine(model)
        option.setPricingEngine(engine)
        price = float(option.NPV())
        elapsed = (time.perf_counter() - start) * 1000.0

        times.append(elapsed)
        prices.append(price)

    return LibraryResult(
        library="quantlib",
        methodology="heston_analytic_reference_quantlib",
        available=True,
        runtime_ms=sum(times) / len(times),
        price=sum(prices) / len(prices),
        stderr=None,
        note=(
            "QuantLib-Python AnalyticHestonEngine reference; this is an analytic "
            "reference lane, not a Monte Carlo path-runtime timing."
        ),
    )


def _quantlib_black_scholes_process(ql: Any) -> tuple[Any, Any, Any]:
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
    return process, evaluation_date, maturity_date


def benchmark_quantlib_american_put_lsm(
    n_paths: int, n_steps: int, repeats: int, seed: int
) -> LibraryResult:
    import QuantLib as ql

    process, evaluation_date, maturity_date = _quantlib_black_scholes_process(ql)
    payoff = ql.PlainVanillaPayoff(ql.Option.Put, 100.0)
    exercise = ql.AmericanExercise(evaluation_date, maturity_date)

    times: list[float] = []
    prices: list[float] = []
    stderrs: list[float] = []

    for rep in range(repeats):
        option = ql.VanillaOption(payoff, exercise)

        start = time.perf_counter()
        engine = ql.MCAmericanEngine(
            process,
            "pseudorandom",
            timeSteps=n_steps,
            requiredSamples=n_paths,
            seed=seed + rep,
            polynomOrder=2,
            polynomType=0,
            nCalibrationSamples=max(16, min(2048, n_paths)),
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
        methodology="american_put_lsm_quantlib_mc",
        available=True,
        runtime_ms=sum(times) / len(times),
        price=sum(prices) / len(prices),
        stderr=(sum(stderrs) / len(stderrs)) if stderrs else None,
        note=(
            "QuantLib-Python MCAmericanEngine with pseudorandom Longstaff-Schwartz "
            "Monte Carlo under the same flat Black-Scholes inputs."
        ),
        reproducibility=(
            "statistical reproducibility for fixed seed and QuantLib-Python build; "
            "exact path identity with montepath is not expected"
        ),
    )


def benchmark_quantlib_bermudan_put_lsm(
    n_paths: int, n_steps: int, repeats: int, seed: int
) -> LibraryResult:
    import QuantLib as ql

    process, _evaluation_date, maturity_date = _quantlib_black_scholes_process(ql)
    payoff = ql.PlainVanillaPayoff(ql.Option.Put, 100.0)
    exercise = ql.BermudanExercise(
        [
            ql.Date(2, ql.April, 2023),
            ql.Date(2, ql.July, 2023),
            ql.Date(2, ql.October, 2023),
            maturity_date,
        ]
    )

    times: list[float] = []
    prices: list[float] = []
    stderrs: list[float] = []

    for rep in range(repeats):
        option = ql.VanillaOption(payoff, exercise)

        start = time.perf_counter()
        engine = ql.MCAmericanEngine(
            process,
            "pseudorandom",
            timeSteps=n_steps,
            requiredSamples=n_paths,
            seed=seed + rep,
            polynomOrder=2,
            polynomType=0,
            nCalibrationSamples=max(16, min(2048, n_paths)),
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
        methodology="bermudan_put_lsm_quantlib_mc",
        available=True,
        runtime_ms=sum(times) / len(times),
        price=sum(prices) / len(prices),
        stderr=(sum(stderrs) / len(stderrs)) if stderrs else None,
        note=(
            "QuantLib-Python MCAmericanEngine applied to a quarterly Bermudan "
            "exercise schedule matching the tracked montepath fixture."
        ),
        reproducibility=(
            "statistical reproducibility for fixed seed and QuantLib-Python build; "
            "exercise-date calendar discretization differs from simulation-step indices"
        ),
    )


def unavailable(
    name: str,
    methodology: str | None,
    note: str,
    *,
    telemetry: dict[str, Any] | None = None,
    reproducibility: str | None = None,
) -> LibraryResult:
    return LibraryResult(
        library=name,
        methodology=methodology,
        available=False,
        runtime_ms=None,
        price=None,
        stderr=None,
        note=note,
        telemetry=telemetry,
        reproducibility=reproducibility,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Run Python competitor MC baselines")
    parser.add_argument("--paths", type=int, required=True)
    parser.add_argument("--steps", type=int, required=True)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--inventory-paths", type=int, default=10_000)
    parser.add_argument("--inventory-periods", type=int, default=104)
    args = parser.parse_args()

    results: list[LibraryResult] = []

    if has_module("numpy"):
        results.append(benchmark_numpy(args.paths, args.steps, args.repeats, args.seed))
        results.append(benchmark_numpy_terminal(args.paths, args.repeats, args.seed))
        results.append(
            benchmark_inventory_numpy(
                args.inventory_paths,
                args.inventory_periods,
                args.repeats,
                args.seed,
            )
        )
    else:
        results.append(unavailable("numpy", "stepwise_paths", "package not installed"))
        results.append(
            unavailable("numpy", "terminal_distribution", "package not installed")
        )
        results.append(
            unavailable(
                "numpy",
                "inventory_periodic_review_fixed_lead_time",
                "package not installed",
            )
        )

    if has_module("numba"):
        results.append(benchmark_numba(args.paths, args.steps, args.repeats, args.seed))
        results.append(benchmark_numba_terminal(args.paths, args.repeats, args.seed))
        results.append(
            benchmark_inventory_numba(
                args.inventory_paths,
                args.inventory_periods,
                args.repeats,
                args.seed,
            )
        )
    else:
        results.append(unavailable("numba", "stepwise_paths", "package not installed"))
        results.append(
            unavailable("numba", "terminal_distribution", "package not installed")
        )
        results.append(
            unavailable(
                "numba",
                "inventory_periodic_review_fixed_lead_time",
                "package not installed",
            )
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
        try:
            results.append(
                benchmark_quantlib_fixed_strike_lookback(
                    args.paths, args.steps, args.repeats, args.seed
                )
            )
        except Exception as exc:
            results.append(
                unavailable(
                    "quantlib",
                    "lookback_fixed_strike_stepwise_quantlib_mc",
                    f"benchmark failed: {type(exc).__name__}: {exc}",
                )
            )
        try:
            results.append(
                benchmark_quantlib_heston_european(
                    args.paths, args.steps, args.repeats, args.seed
                )
            )
        except Exception as exc:
            results.append(
                unavailable(
                    "quantlib",
                    "heston_analytic_reference_quantlib",
                    f"benchmark failed: {type(exc).__name__}: {exc}",
                )
            )
        try:
            results.append(
                benchmark_quantlib_american_put_lsm(
                    args.paths, args.steps, args.repeats, args.seed
                )
            )
        except Exception as exc:
            results.append(
                unavailable(
                    "quantlib",
                    "american_put_lsm_quantlib_mc",
                    f"benchmark failed: {type(exc).__name__}: {exc}",
                )
            )
        try:
            results.append(
                benchmark_quantlib_bermudan_put_lsm(
                    args.paths, args.steps, args.repeats, args.seed
                )
            )
        except Exception as exc:
            results.append(
                unavailable(
                    "quantlib",
                    "bermudan_put_lsm_quantlib_mc",
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
        results.append(
            unavailable(
                "quantlib",
                "american_put_lsm_quantlib_mc",
                "QuantLib-Python package not installed",
            )
        )
        results.append(
            unavailable(
                "quantlib",
                "bermudan_put_lsm_quantlib_mc",
                "QuantLib-Python package not installed",
            )
        )
        results.append(
            unavailable(
                "quantlib",
                "lookback_fixed_strike_stepwise_quantlib_mc",
                "QuantLib-Python package not installed",
            )
        )
        results.append(
            unavailable(
                "quantlib",
                "heston_analytic_reference_quantlib",
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

    accelerator_benchmarks = {
        "jax": benchmark_jax_gpu,
        "cupy": benchmark_cupy_gpu,
        "torch": benchmark_torch_gpu,
    }
    for gpu_lib, benchmark in accelerator_benchmarks.items():
        if has_module(gpu_lib):
            try:
                results.append(benchmark(args.paths, args.repeats, args.seed))
            except Exception as exc:
                results.append(
                    _accelerator_unavailable(
                        gpu_lib,
                        f"accelerator benchmark failed: {type(exc).__name__}: {exc}",
                    )
                )
        else:
            results.append(_accelerator_unavailable(gpu_lib, "package not installed"))

    payload: dict[str, Any] = {
        "environment": {
            "python_version": sys.version.split()[0],
            "platform": platform.platform(),
            "paths": args.paths,
            "steps": args.steps,
            "repeats": args.repeats,
            "inventory_paths": args.inventory_paths,
            "inventory_periods": args.inventory_periods,
            "package_versions": {
                "numpy": package_version("numpy"),
                "numba": package_version("numba"),
                "scipy": package_version("scipy"),
                "QuantLib": package_version("QuantLib"),
                "jax": package_version("jax"),
                "cupy": package_version("cupy-cuda12x")
                or package_version("cupy-cuda11x")
                or package_version("cupy"),
                "torch": package_version("torch"),
            },
        },
        "results": [asdict(r) for r in results],
    }

    print(json.dumps(payload))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
