"""Typed method recommendation helpers.

These mirror the Rust planner surface for early Python users and agents. The
implementation is intentionally dependency-free until planner bindings land.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

WorkloadFamily = Literal[
    "european_call",
    "arithmetic_asian_call",
    "down_and_out_call",
    "generic_path_simulation",
]
SamplingMethod = Literal[
    "pseudorandom",
    "randomized_halton",
    "latin_hypercube",
    "scrambled_sobol",
    "scrambled_sobol_brownian_bridge",
]
MonteCarloTechnique = Literal["standard", "antithetic", "control_variate"]


@dataclass(frozen=True)
class MethodRecommendation:
    method_id: str
    sampling: SamplingMethod
    technique: MonteCarloTechnique
    reasons: tuple[str, ...]
    caveats: tuple[str, ...]


def recommend_method(
    *,
    workload_family: WorkloadFamily,
    n_paths: int,
    n_steps: int,
    prefer_accuracy: bool = False,
    allow_slower_structured_sampling: bool = False,
) -> MethodRecommendation:
    if n_paths <= 0:
        raise ValueError("n_paths must be > 0")
    if n_steps <= 0:
        raise ValueError("n_steps must be > 0")

    path_dependent = workload_family != "european_call"
    large_enough_for_qmc = n_paths >= 32_768 and n_steps >= 16
    high_dimension = n_steps >= 32

    if (
        prefer_accuracy
        and path_dependent
        and n_steps >= 16
        and not allow_slower_structured_sampling
    ):
        return MethodRecommendation(
            method_id="multilevel_monte_carlo",
            sampling="pseudorandom",
            technique="standard",
            reasons=(
                "path-dependent accuracy preference benefits from a coupled multilevel estimator",
                "pseudorandom MLMC is the current CPU-reference advanced path when slower structured sampling is not requested",
            ),
            caveats=(
                "MLMC CPU support currently covers arithmetic Asian calls; barrier workloads need separate discontinuity handling",
                "use the allocation tuner or explicit arithmetic Asian MLMC config to set paths per level",
            ),
        )

    if (
        prefer_accuracy
        and allow_slower_structured_sampling
        and large_enough_for_qmc
        and workload_family == "arithmetic_asian_call"
    ):
        return MethodRecommendation(
            method_id="multilevel_randomized_qmc",
            sampling="scrambled_sobol",
            technique="standard",
            reasons=(
                "arithmetic Asian accuracy preference can use coupled MLMC with scrambled Sobol increments",
                "MLQMC combines the first CPU MLMC foundation with the randomized-QMC sampling surface",
            ),
            caveats=(
                "MLQMC support is CPU-reference only and still needs measured allocation tuning per workload",
                "use scramble_replicates > 1 for defensible randomized-QMC error estimates",
            ),
        )

    if prefer_accuracy and allow_slower_structured_sampling and large_enough_for_qmc:
        sampling: SamplingMethod = (
            "scrambled_sobol_brownian_bridge"
            if high_dimension or path_dependent
            else "scrambled_sobol"
        )
        return MethodRecommendation(
            method_id=(
                "scrambled_sobol_brownian_bridge"
                if sampling == "scrambled_sobol_brownian_bridge"
                else "scrambled_sobol"
            ),
            sampling=sampling,
            technique="control_variate",
            reasons=(
                "accuracy preference allows slower structured sampling",
                "control variate is the strongest measured variance-reduction technique on current workloads",
                "Brownian bridge concentrates path variance into early Sobol dimensions"
                if sampling == "scrambled_sobol_brownian_bridge"
                else "scrambled Sobol is preferred over Halton for serious randomized-QMC coverage",
            ),
            caveats=(
                "structured sampling is currently CPU-reference only and falls back on native GPU backends",
                "recommendation is heuristic until more measured winner scenarios are collected",
            ),
        )

    return MethodRecommendation(
        method_id="control_variates",
        sampling="pseudorandom",
        technique="control_variate",
        reasons=(
            "pseudorandom sampling is the fastest measured CPU and native Metal path today",
            "control variate gives strong measured stderr reduction with modest runtime overhead",
        ),
        caveats=(
            "antithetic may be useful when control-variate assumptions are unavailable",
            "structured sampling can improve estimator quality but is slower in current benchmarks",
        ),
    )
