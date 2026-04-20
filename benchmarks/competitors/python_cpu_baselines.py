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
from dataclasses import asdict, dataclass
from typing import Any


@dataclass
class LibraryResult:
    library: str
    available: bool
    runtime_ms: float | None
    price: float | None
    stderr: float | None
    note: str | None


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
        available=True,
        runtime_ms=sum(times) / len(times),
        price=sum(prices) / len(prices),
        stderr=sum(stderrs) / len(stderrs),
        note="execution-only timing after JIT warm-up",
    )


def unavailable(name: str, note: str) -> LibraryResult:
    return LibraryResult(
        library=name,
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
    else:
        results.append(unavailable("numpy", "package not installed"))

    if has_module("numba"):
        results.append(benchmark_numba(args.paths, args.steps, args.repeats, args.seed))
    else:
        results.append(unavailable("numba", "package not installed"))

    for gpu_lib in ("jax", "cupy", "torch"):
        if has_module(gpu_lib):
            results.append(
                unavailable(
                    gpu_lib,
                    "detected but GPU-targeted benchmark path not implemented in this CPU script",
                )
            )
        else:
            results.append(unavailable(gpu_lib, "package not installed"))

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
