"""Evidence-backed planner intelligence helpers.

These helpers are deliberately small and JSON-friendly. They read committed
benchmark artifacts and expose method comparison, cost-frontier, calibration,
and "why not faster" explanations for users and agents.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

DEFAULT_REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_BENCHMARK_ARTIFACT = "benchmarks/release-results.json"
DEFAULT_REFERENCE_FIXTURES = "benchmarks/reference-fixtures.json"


def load_planner_evidence(
    *,
    repo_root: str | Path | None = None,
    benchmark_artifact: str = DEFAULT_BENCHMARK_ARTIFACT,
) -> dict[str, Any]:
    root = _repo_root(repo_root)
    results = _load_results(root / benchmark_artifact)
    measured_accuracy = _metric(results, "planner_choice_accuracy_measured")
    winners = _winner_rows(results, root, benchmark_artifact)
    diagnostics = []
    if not results:
        diagnostics.append(
            {
                "code": "MC_PLANNER_ARTIFACT_MISSING",
                "message": f"No benchmark results found in {benchmark_artifact!r}.",
                "suggestion": "Run the benchmark harness or pass a valid benchmark_artifact.",
            }
        )
    return {
        "schema_version": "planner-evidence.v1",
        "benchmark_artifact": benchmark_artifact,
        "measured_planner_accuracy_pct": measured_accuracy,
        "measured_planner_accuracy_source": "planner_choice_accuracy_measured",
        "winners": winners,
        "reference_fixtures": _reference_fixture_names(root),
        "diagnostics": diagnostics,
        "caveats": [
            "Measured planner accuracy is read directly from the committed benchmark artifact.",
            "Runtime values are artifact-local and should not be generalized across hardware without rerunning benchmarks.",
        ],
    }


def measured_winner_database(
    *,
    repo_root: str | Path | None = None,
    benchmark_artifact: str = DEFAULT_BENCHMARK_ARTIFACT,
) -> dict[str, Any]:
    evidence = load_planner_evidence(repo_root=repo_root, benchmark_artifact=benchmark_artifact)
    return {
        "schema_version": "planner-evidence.v1",
        "benchmark_artifact": benchmark_artifact,
        "winners": evidence["winners"],
    }


def cost_frontier(
    workload: str,
    *,
    repo_root: str | Path | None = None,
    benchmark_artifact: str = DEFAULT_BENCHMARK_ARTIFACT,
) -> dict[str, Any]:
    root = _repo_root(repo_root)
    results = _load_results(root / benchmark_artifact)
    rows = _workload_rows(results, workload)
    if not rows:
        return {
            "schema_version": "planner-frontier.v1",
            "workload": workload,
            "frontier": [],
            "diagnostics": [
                {
                    "code": "MC_PLANNER_NO_EVIDENCE",
                    "message": f"No benchmark rows found for workload {workload!r}",
                    "suggestion": "Run or add benchmark coverage before making planner claims.",
                }
            ],
        }

    frontier = sorted(
        [_frontier_entry(row, results) for row in rows if _runtime_ms(row) is not None],
        key=lambda item: item["runtime_ms"],
    )
    return {
        "schema_version": "planner-frontier.v1",
        "workload": workload,
        "benchmark_artifact": benchmark_artifact,
        "frontier": frontier,
        "diagnostics": [],
    }


def compare_methods(
    workload: str,
    *,
    repo_root: str | Path | None = None,
    benchmark_artifact: str = DEFAULT_BENCHMARK_ARTIFACT,
) -> dict[str, Any]:
    frontier = cost_frontier(
        workload, repo_root=repo_root, benchmark_artifact=benchmark_artifact
    )
    rows = frontier["frontier"]
    if not rows:
        return frontier
    recommended = _recommend_from_frontier(workload, rows)
    alternatives = [row for row in rows if row["benchmark_name"] != recommended["benchmark_name"]]
    return {
        "schema_version": "planner-comparison.v1",
        "workload": workload,
        "benchmark_artifact": benchmark_artifact,
        "recommended": recommended,
        "alternatives": alternatives,
        "tradeoffs": _tradeoffs(workload, recommended, alternatives),
    }


def why_not_faster(
    workload: str,
    *,
    method_id: str,
    repo_root: str | Path | None = None,
    benchmark_artifact: str = DEFAULT_BENCHMARK_ARTIFACT,
) -> dict[str, Any]:
    comparison = compare_methods(
        workload, repo_root=repo_root, benchmark_artifact=benchmark_artifact
    )
    recommended = comparison.get("recommended")
    alternatives = comparison.get("alternatives", [])
    selected = next(
        (
            row
            for row in [recommended, *alternatives]
            if row and (row["method_id"] == method_id or row["methodology"] == method_id)
        ),
        None,
    )
    if selected is None:
        return {
            "schema_version": "planner-why-not.v1",
            "workload": workload,
            "method_id": method_id,
            "reasons": [
                f"No benchmark artifact row matches method_id or methodology {method_id!r}."
            ],
            "suggestions": [
                "Add a benchmark row before asking the planner to rank this method."
            ],
        }

    reasons = []
    if recommended and selected["runtime_ms"] > recommended["runtime_ms"]:
        ratio = selected["runtime_ms"] / max(recommended["runtime_ms"], 1e-12)
        reasons.append(
            f"{method_id} is {ratio:.2f}x slower than the current recommended method in {comparison['benchmark_artifact']}."
        )
    if selected.get("quality_metric"):
        reasons.append(
            f"Quality metric {selected['quality_metric']}={selected.get('quality_value')} must be weighed against runtime."
        )
    if "sobol" in method_id or "latin_hypercube" in method_id or "halton" in method_id:
        reasons.append(
            "Structured sampling currently spends extra time on sample generation/path construction; use realized-error rows before choosing it for speed."
        )
    if not reasons:
        reasons.append("Selected method is on the current cost frontier for this workload.")

    return {
        "schema_version": "planner-why-not.v1",
        "workload": workload,
        "method_id": method_id,
        "recommended_method_id": recommended["method_id"] if recommended else None,
        "reasons": reasons,
        "suggestions": [
            "Use compare_methods() for the full runtime/accuracy frontier.",
            "Refresh benchmark artifacts on target hardware before production tuning.",
        ],
    }


def mlmc_error_calibration(
    workload: str = "arithmetic_asian_call",
    *,
    repo_root: str | Path | None = None,
    benchmark_artifact: str = DEFAULT_BENCHMARK_ARTIFACT,
) -> dict[str, Any]:
    root = _repo_root(repo_root)
    results = _load_results(root / benchmark_artifact)
    mlmc_ratio = _metric(results, "mc_cpu_arithmetic_asian_call_rust_mlmc_quality")
    mlqmc_ratio = _metric(results, "mc_cpu_arithmetic_asian_call_rust_mlqmc_quality")
    mlmc_reference_abs_error = _metric(
        results, "mc_cpu_arithmetic_asian_call_rust_mlmc_reference_calibration"
    )
    mlqmc_reference_abs_error = _metric(
        results, "mc_cpu_arithmetic_asian_call_rust_mlqmc_reference_calibration"
    )
    return {
        "schema_version": "planner-mlmc-calibration.v1",
        "workload": workload,
        "benchmark_artifact": benchmark_artifact,
        "estimated_error_source": "adaptive MLMC/MLQMC tolerance planner pilot variances",
        "realized_error_metric": "stderr_ratio_vs_standard_and_abs_error_vs_standard_reference",
        "mlmc_realized_ratio": mlmc_ratio,
        "mlqmc_realized_ratio": mlqmc_ratio,
        "mlmc_reference_abs_error": mlmc_reference_abs_error,
        "mlqmc_reference_abs_error": mlqmc_reference_abs_error,
        "calibration_status": (
            "mlqmc_accuracy_favorable"
            if mlqmc_ratio is not None
            and mlqmc_ratio < 1.0
            and (
                mlqmc_reference_abs_error is None or mlqmc_reference_abs_error < 0.75
            )
            else "needs_more_replicates"
        ),
        "caveats": [
            "Current MLMC/MLQMC calibration is arithmetic-Asian specific.",
            "Barrier and discontinuous payoff MLMC remain separate future work.",
        ],
    }


def _repo_root(repo_root: str | Path | None) -> Path:
    return Path(repo_root) if repo_root is not None else DEFAULT_REPO_ROOT


def _load_results(path: Path) -> list[dict[str, Any]]:
    if not path.exists():
        return []
    payload = json.loads(path.read_text())
    return list(payload.get("results", ()))


def _metric(results: list[dict[str, Any]], benchmark_name: str) -> float | None:
    row = next((row for row in results if row["benchmark_name"] == benchmark_name), None)
    if row is None:
        return None
    value = row.get("metric_value")
    return float(value) if value is not None else None


def _runtime_ms(row: dict[str, Any]) -> float | None:
    value = row.get("per_iteration_us")
    if value is None or float(value) <= 0.0:
        return None
    return float(value) / 1_000.0


def _workload_rows(results: list[dict[str, Any]], workload: str) -> list[dict[str, Any]]:
    prefixes = {
        "european_call": (
            "mc_cpu_european_call_rust",
            "mc_metal_european_call_native",
            "mc_cpu_qmc_realized_error_european",
        ),
        "arithmetic_asian_call": (
            "mc_cpu_arithmetic_asian_call_rust",
            "mc_metal_arithmetic_asian_call_native",
        ),
        "down_and_out_call": (
            "mc_cpu_down_and_out_call_rust",
            "mc_metal_down_and_out_call_native",
        ),
        "lookback_call": ("mc_cpu_lookback_call_rust",),
        "basket_call": ("mc_cpu_basket_call_rust",),
        "heston_european_call": ("mc_cpu_heston_european_call_rust",),
    }.get(workload, (workload,))
    return [
        row
        for row in results
        if any(row["benchmark_name"].startswith(prefix) for prefix in prefixes)
        and _runtime_ms(row) is not None
        and row.get("metric_name") != "unavailable"
    ]


def _frontier_entry(
    row: dict[str, Any], results: list[dict[str, Any]]
) -> dict[str, Any]:
    methodology = row.get("methodology") or row["benchmark_name"]
    quality = _paired_quality(row, results)
    return {
        "benchmark_name": row["benchmark_name"],
        "method_id": _method_id(methodology),
        "methodology": methodology,
        "backend": row["backend"],
        "runtime_ms": _runtime_ms(row),
        "throughput_per_sec": row.get("throughput_per_sec"),
        "estimate_metric": row.get("metric_name"),
        "estimate_value": row.get("metric_value"),
        "quality_metric": quality.get("metric_name", row.get("metric_name")),
        "quality_value": quality.get("metric_value", row.get("metric_value")),
        "artifact_metric": row.get("metric_name"),
    }


def _paired_quality(
    row: dict[str, Any], results: list[dict[str, Any]]
) -> dict[str, Any]:
    candidates = (
        f"{row['benchmark_name']}_quality",
        row["benchmark_name"].replace("_rust_", "_rust_") + "_quality",
    )
    for benchmark_name in candidates:
        quality = next(
            (item for item in results if item["benchmark_name"] == benchmark_name), None
        )
        if quality is not None and quality.get("metric_value") is not None:
            return quality
    return {}


def _method_id(methodology: str) -> str:
    mapping = {
        "stepwise_paths": "standard_stepwise",
        "stepwise_paths_control_variate": "control_variates",
        "stepwise_paths_antithetic": "antithetic",
        "terminal_distribution": "terminal_distribution",
        "arithmetic_asian_multilevel_coupled_adaptive_tolerance": "multilevel_monte_carlo",
        "arithmetic_asian_multilevel_scrambled_sobol_replicated_adaptive_tolerance": "multilevel_randomized_qmc",
    }
    for key, value in mapping.items():
        if methodology == key:
            return value
    if "scrambled_sobol_brownian_bridge" in methodology:
        return "scrambled_sobol_brownian_bridge"
    if "scrambled_sobol" in methodology:
        return "scrambled_sobol"
    if "latin_hypercube" in methodology:
        return "latin_hypercube"
    if "randomized_halton" in methodology:
        return "randomized_halton"
    if "control_variate" in methodology:
        return "control_variates"
    return methodology


def _recommend_from_frontier(workload: str, rows: list[dict[str, Any]]) -> dict[str, Any]:
    if workload == "arithmetic_asian_call":
        mlqmc = next((row for row in rows if row["method_id"] == "multilevel_randomized_qmc"), None)
        if mlqmc is not None and (mlqmc.get("quality_value") or 1.0) < 1.0:
            return mlqmc | {
                "reason": "MLQMC has the strongest current arithmetic-Asian accuracy signal."
            }
    if workload == "european_call":
        cv = next((row for row in rows if row["method_id"] == "control_variates"), None)
        if cv is not None:
            return cv | {
                "reason": "Control variate gives strong variance reduction with modest runtime overhead."
            }
    fastest = rows[0]
    return fastest | {"reason": "Fastest measured row in the current artifact."}


def _tradeoffs(
    workload: str,
    recommended: dict[str, Any],
    alternatives: list[dict[str, Any]],
) -> list[str]:
    tradeoffs = [
        f"Recommended {recommended['method_id']} for {workload}: {recommended['reason']}"
    ]
    if alternatives:
        fastest = min(alternatives + [recommended], key=lambda row: row["runtime_ms"])
        if fastest["benchmark_name"] != recommended["benchmark_name"]:
            tradeoffs.append(
                f"Fastest measured row is {fastest['method_id']} at {fastest['runtime_ms']:.3f} ms, but it may not be the best accuracy tradeoff."
            )
    tradeoffs.append("Benchmark artifacts are hardware-local; refresh before production use.")
    return tradeoffs


def _reference_fixture_names(root: Path) -> list[str]:
    path = root / DEFAULT_REFERENCE_FIXTURES
    if not path.exists():
        return []
    payload = json.loads(path.read_text())
    fixtures = payload.get("fixtures", payload)
    if isinstance(fixtures, dict):
        return sorted(str(name) for name in fixtures)
    if isinstance(fixtures, list):
        return sorted(
            str(item.get("fixture_id") or item.get("name"))
            for item in fixtures
            if item.get("fixture_id") or item.get("name")
        )
    return []


def _winner_rows(
    results: list[dict[str, Any]],
    root: Path,
    benchmark_artifact: str,
) -> list[dict[str, Any]]:
    winners = []
    for workload in [
        "european_call",
        "arithmetic_asian_call",
        "down_and_out_call",
        "basket_call",
        "lookback_call",
        "heston_european_call",
    ]:
        frontier = cost_frontier(
            workload, repo_root=root, benchmark_artifact=benchmark_artifact
        )
        rows = frontier.get("frontier", [])
        if rows:
            winner = rows[0]
            winners.append(
                {
                    "workload": workload,
                    "benchmark_artifact": benchmark_artifact,
                    "selected_method": winner["method_id"],
                    "benchmark_name": winner["benchmark_name"],
                    "backend": winner["backend"],
                    "runtime_ms": winner["runtime_ms"],
                }
            )
    return winners
