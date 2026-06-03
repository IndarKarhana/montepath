"""Agent-safe tool surfaces for montepath.

The functions in this module accept and return JSON-serializable dictionaries.
They avoid hidden global state, keep unsupported behavior explicit, and attach
reproducibility metadata to planning, execution, benchmark, and reproduction
responses.
"""

from __future__ import annotations

import platform
import sys
from dataclasses import asdict
from importlib import metadata
from typing import Any, Mapping

from .benchmarks import run_benchmarks
from .methods import recommend_method
from .planner_intelligence import (
    compare_methods,
    cost_frontier,
    load_planner_evidence,
    mlmc_error_calibration,
    why_not_faster,
)
from .production import (
    benchmark_report,
    numerical_validation_report,
    production_status,
    validate_workload_request,
)
from .pricing import (
    ArithmeticAsianCallConfig,
    DownAndOutCallConfig,
    EuropeanCallConfig,
    McConfigurationError,
    price_arithmetic_asian_call,
    price_down_and_out_call,
    price_european_call,
    price_european_call_greeks,
)

SUPPORTED_WORKLOADS = {
    "european_call",
    "arithmetic_asian_call",
    "down_and_out_call",
    "european_call_greeks",
}


def agent_tool_manifest() -> dict[str, Any]:
    """Return the stable machine-readable agent tool manifest."""

    tools = [
        _tool(
            "montepath.validate",
            "Validate a supported workload request without running simulation.",
            "montepath.validate.request",
            "montepath.validate.response",
            deterministic=True,
        ),
        _tool(
            "montepath.capabilities",
            "Report installed CPU, Metal, CUDA, Python, MCP, and agent capability status.",
            "montepath.capabilities.request",
            "montepath.capabilities.response",
            deterministic=True,
        ),
        _tool(
            "montepath.production_check",
            "Validate a request against production backend policy and benchmark evidence.",
            "montepath.production_check.request",
            "montepath.production_check.response",
            deterministic=True,
        ),
        _tool(
            "montepath.validation_report",
            "Report committed numerical reference fixtures, caveats, and tolerance policy.",
            "montepath.validation_report.request",
            "montepath.validation_report.response",
            deterministic=True,
        ),
        _tool(
            "montepath.recommend",
            "Recommend a method, sampling policy, and variance-reduction technique.",
            "montepath.recommend.request",
            "montepath.recommend.response",
            deterministic=True,
        ),
        _tool(
            "montepath.plan",
            "Build a deterministic dry-run execution plan with cost and caveat metadata.",
            "montepath.plan.request",
            "montepath.plan.response",
            deterministic=True,
        ),
        _tool(
            "montepath.execute",
            "Execute a narrow Python reference workload with reproducibility metadata.",
            "montepath.execute.request",
            "montepath.execute.response",
            deterministic=True,
        ),
        _tool(
            "montepath.compare",
            "Compare fast and accuracy-oriented method choices for a workload.",
            "montepath.compare.request",
            "montepath.compare.response",
            deterministic=True,
        ),
        _tool(
            "montepath.benchmark",
            "Return benchmark command metadata by default, or run benchmarks when explicitly requested.",
            "montepath.benchmark.request",
            "montepath.benchmark.response",
            deterministic=False,
        ),
        _tool(
            "montepath.reproduce",
            "Create a reproduction recipe from an agent run manifest.",
            "montepath.reproduce.request",
            "montepath.reproduce.response",
            deterministic=True,
        ),
        _tool(
            "montepath.planner_evidence",
            "Load benchmark-backed planner evidence, winner records, and fixture references.",
            "montepath.planner_evidence.request",
            "montepath.planner_evidence.response",
            deterministic=True,
        ),
        _tool(
            "montepath.cost_frontier",
            "Return the measured method/backend cost frontier for a workload.",
            "montepath.cost_frontier.request",
            "montepath.cost_frontier.response",
            deterministic=True,
        ),
        _tool(
            "montepath.compare_methods",
            "Compare measured method/runtime tradeoffs for a workload.",
            "montepath.compare_methods.request",
            "montepath.compare_methods.response",
            deterministic=True,
        ),
        _tool(
            "montepath.why_not_faster",
            "Explain why a requested method is not the current measured recommendation.",
            "montepath.why_not_faster.request",
            "montepath.why_not_faster.response",
            deterministic=True,
        ),
        _tool(
            "montepath.mlmc_calibration",
            "Report estimated-vs-realized MLMC and MLQMC error calibration evidence.",
            "montepath.mlmc_calibration.request",
            "montepath.mlmc_calibration.response",
            deterministic=True,
        ),
    ]
    return {
        "schema_version": "agent-tools.v1",
        "package": "montepath",
        "tools": tools,
    }


def export_json_schemas() -> dict[str, dict[str, Any]]:
    """Export stable JSON-schema-like contracts for agent tools."""

    request_schema = {
        "type": "object",
        "required": ["workload"],
        "properties": {
            "workload": {"type": "string", "enum": sorted(SUPPORTED_WORKLOADS)},
            "config": {"type": "object"},
            "preferences": {"type": "object"},
        },
        "additionalProperties": False,
    }
    response_schema = {
        "type": "object",
        "required": ["ok", "manifest"],
        "properties": {
            "ok": {"type": "boolean"},
            "result": {"type": "object"},
            "manifest": _manifest_schema(),
            "diagnostics": {"type": "array", "items": {"type": "object"}},
            "reproduction": {"type": "object"},
        },
    }
    benchmark_request_schema = {
        "type": "object",
        "properties": {
            "profile": {"type": "string", "enum": ["compact", "full"]},
            "release": {"type": "boolean"},
            "execute": {"type": "boolean"},
            "repo_root": {"type": "string"},
        },
        "additionalProperties": False,
    }
    capabilities_request_schema = {
        "type": "object",
        "properties": {"native_module": {"type": "string"}},
        "additionalProperties": False,
    }
    production_check_request_schema = {
        "type": "object",
        "required": ["workload"],
        "properties": {
            "workload": {"type": "string"},
            "config": {"type": "object"},
            "backend": {"type": "string"},
            "native_module": {"type": "string"},
            "repo_root": {"type": "string"},
            "benchmark_artifact": {"type": "string"},
        },
        "additionalProperties": False,
    }
    planner_artifact_schema = {
        "type": "object",
        "properties": {
            "repo_root": {"type": "string"},
            "benchmark_artifact": {"type": "string"},
        },
        "additionalProperties": False,
    }
    planner_workload_schema = {
        "type": "object",
        "required": ["workload"],
        "properties": {
            "workload": {"type": "string"},
            "repo_root": {"type": "string"},
            "benchmark_artifact": {"type": "string"},
        },
        "additionalProperties": False,
    }

    schemas = {
        "montepath.validate.request": request_schema,
        "montepath.validate.response": response_schema,
        "montepath.capabilities.request": capabilities_request_schema,
        "montepath.capabilities.response": response_schema,
        "montepath.production_check.request": production_check_request_schema,
        "montepath.production_check.response": response_schema,
        "montepath.validation_report.request": {
            "type": "object",
            "properties": {
                "repo_root": {"type": "string"},
                "reference_artifact": {"type": "string"},
                "capability_catalog": {"type": "string"},
            },
            "additionalProperties": False,
        },
        "montepath.validation_report.response": response_schema,
        "montepath.recommend.request": request_schema,
        "montepath.recommend.response": response_schema,
        "montepath.plan.request": request_schema,
        "montepath.plan.response": response_schema,
        "montepath.execute.request": request_schema,
        "montepath.execute.response": response_schema,
        "montepath.compare.request": request_schema,
        "montepath.compare.response": response_schema,
        "montepath.benchmark.request": benchmark_request_schema,
        "montepath.benchmark.response": response_schema,
        "montepath.reproduce.request": {
            "type": "object",
            "required": ["manifest"],
            "properties": {"manifest": _manifest_schema()},
            "additionalProperties": False,
        },
        "montepath.reproduce.response": response_schema,
        "montepath.planner_evidence.request": planner_artifact_schema,
        "montepath.planner_evidence.response": response_schema,
        "montepath.cost_frontier.request": planner_workload_schema,
        "montepath.cost_frontier.response": response_schema,
        "montepath.compare_methods.request": planner_workload_schema,
        "montepath.compare_methods.response": response_schema,
        "montepath.why_not_faster.request": {
            "type": "object",
            "required": ["workload", "method_id"],
            "properties": {
                "workload": {"type": "string"},
                "method_id": {"type": "string"},
                "repo_root": {"type": "string"},
                "benchmark_artifact": {"type": "string"},
            },
            "additionalProperties": False,
        },
        "montepath.why_not_faster.response": response_schema,
        "montepath.mlmc_calibration.request": {
            "type": "object",
            "properties": {
                "workload": {"type": "string"},
                "repo_root": {"type": "string"},
                "benchmark_artifact": {"type": "string"},
            },
            "additionalProperties": False,
        },
        "montepath.mlmc_calibration.response": response_schema,
    }
    return schemas


def agent_validate(request: Mapping[str, Any]) -> dict[str, Any]:
    workload = str(request.get("workload", ""))
    config = _config_payload(request)
    diagnostics = _validate_payload(workload, config)
    ok = not diagnostics
    return {
        "ok": ok,
        "diagnostics": diagnostics,
        "manifest": _agent_manifest(
            tool="montepath.validate",
            workload=workload or "unknown",
            config=config,
            method="validation_only",
            warnings=tuple(item["message"] for item in diagnostics),
        ),
    }


def agent_capabilities(request: Mapping[str, Any] | None = None) -> dict[str, Any]:
    payload = dict(request or {})
    native_module = str(payload.get("native_module", "montepath._native"))
    result = production_status(native_module)
    return {
        "ok": True,
        "result": result,
        "manifest": _agent_manifest(
            tool="montepath.capabilities",
            workload="capability_inspection",
            config={"native_module": native_module},
            method="capability_report",
            warnings=tuple(result.get("production_notes", ())),
        ),
    }


def agent_production_check(request: Mapping[str, Any]) -> dict[str, Any]:
    workload = str(request.get("workload", ""))
    config = _config_payload(request)
    backend = str(request.get("backend", "auto"))
    native_module = str(request.get("native_module", "montepath._native"))
    validation = validate_workload_request(
        workload,
        config,
        backend=backend,
        native_module=native_module,
    )
    benchmark_kwargs = _artifact_kwargs(request)
    report = benchmark_report(**benchmark_kwargs)
    diagnostics = list(validation.get("diagnostics", ()))
    return {
        "ok": not diagnostics,
        "result": {
            "validation": validation,
            "benchmark_report": report,
            "production_policy": {
                "cpu_native": "preferred Python production fast path when installed",
                "python_reference": "allowed for reproducibility demos and small agent-safe executions",
                "apple_metal": "available from Python when the installed native module exposes price_*_metal functions; otherwise explicit unavailable",
                "nvidia_cuda": "deferred until native launch, reductions, and deterministic GPU RNG land",
            },
        },
        "diagnostics": diagnostics,
        "manifest": _agent_manifest(
            tool="montepath.production_check",
            workload=workload or "unknown",
            config=config | {"backend": backend, "native_module": native_module},
            method="production_readiness_check",
            warnings=tuple(validation.get("warnings", ())) + tuple(report.get("notes", ())),
        ),
    }


def agent_validation_report(request: Mapping[str, Any] | None = None) -> dict[str, Any]:
    payload = dict(request or {})
    report = numerical_validation_report(
        reference_artifact=str(
            payload.get("reference_artifact", "benchmarks/reference-fixtures.json")
        ),
        capability_catalog=str(
            payload.get(
                "capability_catalog", "docs/product-model-capability-catalog.json"
            )
        ),
        repo_root=payload.get("repo_root"),
    )
    return {
        "ok": not report.get("diagnostics"),
        "result": report,
        "diagnostics": list(report.get("diagnostics", ())),
        "manifest": _agent_manifest(
            tool="montepath.validation_report",
            workload="numerical_validation",
            config={
                "reference_artifact": report["reference_artifact"],
                "capability_catalog": report["capability_catalog"],
            },
            method="validation_metadata_report",
            warnings=tuple(report.get("notes", ())),
        ),
    }


def agent_recommend(request: Mapping[str, Any]) -> dict[str, Any]:
    validation = agent_validate(request)
    if not validation["ok"]:
        return validation

    workload = str(request["workload"])
    config = _config_payload(request)
    preferences = dict(request.get("preferences") or {})
    recommendation = recommend_method(
        workload_family=_recommendation_workload(workload),
        n_paths=int(config.get("n_paths", 100_000)),
        n_steps=int(config.get("n_steps", 64)),
        prefer_accuracy=bool(preferences.get("prefer_accuracy", False)),
        allow_slower_structured_sampling=bool(
            preferences.get("allow_slower_structured_sampling", False)
        ),
    )
    return {
        "ok": True,
        "recommendation": asdict(recommendation),
        "manifest": _agent_manifest(
            tool="montepath.recommend",
            workload=workload,
            config=config,
            method=recommendation.method_id,
            warnings=recommendation.caveats,
        ),
    }


def agent_plan(request: Mapping[str, Any]) -> dict[str, Any]:
    recommended = agent_recommend(request)
    if not recommended["ok"]:
        return recommended

    workload = str(request["workload"])
    config = _config_payload(request)
    recommendation = recommended["recommendation"]
    plan = {
        "dry_run": True,
        "workload": workload,
        "backend": "python_reference",
        "method_id": recommendation["method_id"],
        "sampling": recommendation["sampling"],
        "technique": recommendation["technique"],
        "estimated_cost": {
            "confidence": "low",
            "estimated_runtime_ms": None,
            "estimated_peak_memory_mb": _estimate_memory_mb(config),
        },
        "rejected_methods": _rejected_methods(workload, recommendation["method_id"]),
        "notes": [
            "Dry-run planning does not execute simulation.",
            "Python reference backend is selected for stable agent examples; Rust benchmark artifacts carry performance claims.",
        ],
    }
    return {
        "ok": True,
        "plan": plan,
        "manifest": _agent_manifest(
            tool="montepath.plan",
            workload=workload,
            config=config,
            method=recommendation["method_id"],
            warnings=tuple(recommendation["caveats"]),
        ),
    }


def agent_execute(request: Mapping[str, Any]) -> dict[str, Any]:
    validation = agent_validate(request)
    if not validation["ok"]:
        return validation

    workload = str(request["workload"])
    config = _config_payload(request)
    try:
        if workload == "european_call":
            result = price_european_call(**config)
            payload = _pricing_payload(result)
            recipe = result.reproduce()
            reference = "black_scholes_european_call_atm_1y"
        elif workload == "arithmetic_asian_call":
            result = price_arithmetic_asian_call(**config)
            payload = _pricing_payload(result)
            recipe = result.reproduce()
            reference = "no_trusted_fixture_yet"
        elif workload == "down_and_out_call":
            result = price_down_and_out_call(**config)
            payload = _pricing_payload(result)
            recipe = result.reproduce()
            reference = "no_trusted_fixture_yet"
        elif workload == "european_call_greeks":
            report = price_european_call_greeks(**config)
            payload = {
                "base_price": report.base_price,
                "greeks": dict(report.greeks),
                "explanation": report.explain(),
            }
            recipe = report.reproduce()
            reference = "black_scholes_european_call_greeks_atm_1y"
        else:
            raise AssertionError("validated workload should be supported")
    except McConfigurationError as exc:
        return _error_response("montepath.execute", workload, config, exc)

    manifest = _agent_manifest(
        tool="montepath.execute",
        workload=workload,
        config=config,
        method="python_reference",
        estimator="black_scholes_closed_form" if workload == "european_call_greeks" else None,
        warnings=tuple(payload.get("warnings", ())),
        reference=reference,
    )
    return {
        "ok": True,
        "result": payload,
        "manifest": manifest,
        "reproduction": {
            "python": recipe.python,
            "manifest": dict(recipe.manifest),
        },
    }


def agent_compare(request: Mapping[str, Any]) -> dict[str, Any]:
    validation = agent_validate(request)
    if not validation["ok"]:
        return validation

    workload = str(request["workload"])
    config = _config_payload(request)
    fast = agent_recommend(
        {
            "workload": workload,
            "config": config,
            "preferences": {"prefer_accuracy": False},
        }
    )
    accurate = agent_recommend(
        {
            "workload": workload,
            "config": config,
            "preferences": {
                "prefer_accuracy": True,
                "allow_slower_structured_sampling": True,
            },
        }
    )
    alternatives = [
        {"label": "fast_default", **fast.get("recommendation", {})},
        {"label": "accuracy_oriented", **accurate.get("recommendation", {})},
    ]
    return {
        "ok": True,
        "alternatives": alternatives,
        "manifest": _agent_manifest(
            tool="montepath.compare",
            workload=workload,
            config=config,
            method="method_comparison",
            warnings=(
                "Comparison is planner-policy based; use benchmark artifacts before making performance claims.",
            ),
        ),
    }


def agent_benchmark(request: Mapping[str, Any] | None = None) -> dict[str, Any]:
    payload = dict(request or {})
    profile = str(payload.get("profile", "compact"))
    release = bool(payload.get("release", False))
    execute = bool(payload.get("execute", False))
    repo_root = str(payload.get("repo_root", "."))
    command = _benchmark_command(profile, release)
    manifest = _agent_manifest(
        tool="montepath.benchmark",
        workload="benchmark_suite",
        config={"profile": profile, "release": release, "execute": execute},
        method="benchmark_harness",
        warnings=(
            "Benchmark timing is environment-sensitive.",
            "Compact profile is for smoke checks, not competitiveness claims.",
        ),
    )
    if not execute:
        return {
            "ok": True,
            "status": "dry_run",
            "command": " ".join(command),
            "manifest": manifest,
        }

    results = run_benchmarks(repo_root=repo_root, release=release, profile=profile)  # type: ignore[arg-type]
    return {
        "ok": True,
        "status": "executed",
        "results": [asdict(row) for row in results],
        "manifest": manifest,
    }


def agent_reproduce(request: Mapping[str, Any]) -> dict[str, Any]:
    manifest = dict(request.get("manifest") or {})
    workload = str(manifest.get("workload", ""))
    config = dict(manifest.get("config") or {})
    helper = {
        "european_call": "price_european_call",
        "arithmetic_asian_call": "price_arithmetic_asian_call",
        "down_and_out_call": "price_down_and_out_call",
        "european_call_greeks": "price_european_call_greeks",
    }.get(workload)
    if helper is None:
        return {
            "ok": False,
            "diagnostics": [
                {
                    "code": "MC_AGENT_REPRODUCE_UNSUPPORTED",
                    "message": f"Cannot reproduce workload {workload!r}",
                    "suggestion": "Pass a manifest produced by montepath.execute for a supported workload.",
                }
            ],
            "manifest": _agent_manifest(
                tool="montepath.reproduce",
                workload=workload or "unknown",
                config=config,
                method="reproduce",
            ),
        }

    return {
        "ok": True,
        "reproduction": {
            "python": (
                f"from montepath import {helper}\n"
                f"result = {helper}(**{config!r})\n"
                "print(result)\n"
            ),
            "manifest": manifest,
        },
        "manifest": _agent_manifest(
            tool="montepath.reproduce",
            workload=workload,
            config=config,
            method="reproduce",
            reference=manifest.get("reference"),
        ),
    }


def agent_planner_evidence(request: Mapping[str, Any] | None = None) -> dict[str, Any]:
    payload = dict(request or {})
    evidence = load_planner_evidence(**_artifact_kwargs(payload))
    return _planner_response(
        tool="montepath.planner_evidence",
        workload="planner_evidence",
        result=evidence,
        method="planner_evidence",
        reference=evidence.get("reference_fixtures"),
    )


def agent_cost_frontier(request: Mapping[str, Any]) -> dict[str, Any]:
    workload = str(request.get("workload", ""))
    result = cost_frontier(workload, **_artifact_kwargs(request))
    return _planner_response(
        tool="montepath.cost_frontier",
        workload=workload,
        result=result,
        method="cost_frontier",
    )


def agent_compare_methods(request: Mapping[str, Any]) -> dict[str, Any]:
    workload = str(request.get("workload", ""))
    result = compare_methods(workload, **_artifact_kwargs(request))
    return _planner_response(
        tool="montepath.compare_methods",
        workload=workload,
        result=result,
        method=result.get("recommended", {}).get("method_id", "method_comparison"),
    )


def agent_why_not_faster(request: Mapping[str, Any]) -> dict[str, Any]:
    workload = str(request.get("workload", ""))
    method_id = str(request.get("method_id", ""))
    result = why_not_faster(workload, method_id=method_id, **_artifact_kwargs(request))
    return _planner_response(
        tool="montepath.why_not_faster",
        workload=workload,
        result=result,
        method=method_id or "unknown",
    )


def agent_mlmc_calibration(request: Mapping[str, Any] | None = None) -> dict[str, Any]:
    payload = dict(request or {})
    workload = str(payload.get("workload", "arithmetic_asian_call"))
    result = mlmc_error_calibration(workload, **_artifact_kwargs(payload))
    return _planner_response(
        tool="montepath.mlmc_calibration",
        workload=workload,
        result=result,
        method="mlmc_error_calibration",
    )


def _tool(
    name: str,
    description: str,
    input_schema: str,
    output_schema: str,
    *,
    deterministic: bool,
) -> dict[str, Any]:
    return {
        "name": name,
        "description": description,
        "input_schema": input_schema,
        "output_schema": output_schema,
        "determinism": "deterministic" if deterministic else "environment_sensitive",
        "failure_mode": "structured diagnostics with ok=false",
    }


def _manifest_schema() -> dict[str, Any]:
    return {
        "type": "object",
        "required": ["schema_version", "tool", "workload", "backend", "seed"],
        "properties": {
            "schema_version": {"type": "string"},
            "tool": {"type": "string"},
            "workload": {"type": "string"},
            "seed": {"type": ["integer", "null"]},
            "backend": {"type": "string"},
            "method": {"type": ["string", "null"]},
            "estimator": {"type": ["string", "null"]},
            "config": {"type": "object"},
            "build": {"type": "object"},
            "hardware": {"type": "object"},
            "warnings": {"type": "array", "items": {"type": "string"}},
            "reference": {"type": ["string", "null"]},
        },
    }


def _config_payload(request: Mapping[str, Any]) -> dict[str, Any]:
    return dict(request.get("config") or {})


def _validate_payload(workload: str, config: Mapping[str, Any]) -> list[dict[str, str]]:
    if workload not in SUPPORTED_WORKLOADS:
        return [
            {
                "code": "MC_AGENT_UNSUPPORTED_WORKLOAD",
                "message": f"Unsupported workload {workload!r}",
                "suggestion": f"Use one of: {', '.join(sorted(SUPPORTED_WORKLOADS))}",
            }
        ]
    try:
        if workload in {"european_call", "european_call_greeks"}:
            EuropeanCallConfig(**config)
        elif workload == "arithmetic_asian_call":
            ArithmeticAsianCallConfig(**config)
        elif workload == "down_and_out_call":
            DownAndOutCallConfig(**config)
    except TypeError as exc:
        return [
            {
                "code": "MC_AGENT_CONFIG_SHAPE",
                "message": str(exc),
                "suggestion": "Use documented config keys for the selected workload.",
            }
        ]
    diagnostics: list[dict[str, str]] = []
    if int(config.get("n_paths", 100_000)) <= 0:
        diagnostics.append(
            {
                "code": "MC_CONFIG_PATHS",
                "message": "n_paths must be greater than zero",
                "suggestion": "Set n_paths to a positive integer.",
            }
        )
    if int(config.get("n_steps", 64)) <= 0:
        diagnostics.append(
            {
                "code": "MC_CONFIG_STEPS",
                "message": "n_steps must be greater than zero",
                "suggestion": "Set n_steps to a positive integer.",
            }
        )
    for name in ("spot", "strike", "maturity"):
        if float(config.get(name, 1.0 if name == "maturity" else 100.0)) <= 0.0:
            diagnostics.append(
                {
                    "code": "MC_CONFIG_POSITIVE",
                    "message": f"{name} must be greater than zero",
                    "suggestion": f"Set {name} to a positive number.",
                }
            )
    if float(config.get("volatility", 0.2)) < 0.0:
        diagnostics.append(
            {
                "code": "MC_CONFIG_VOLATILITY",
                "message": "volatility must be non-negative",
                "suggestion": "Set volatility to zero or a positive decimal.",
            }
        )
    if workload == "down_and_out_call":
        barrier = float(config.get("barrier", 80.0))
        spot = float(config.get("spot", 100.0))
        if barrier <= 0.0 or barrier >= spot:
            diagnostics.append(
                {
                    "code": "MC_CONFIG_BARRIER",
                    "message": "barrier must be positive and below spot",
                    "suggestion": "Use 0 < barrier < spot.",
                }
            )
    return diagnostics


def _recommendation_workload(workload: str) -> str:
    if workload == "european_call_greeks":
        return "european_call"
    return workload


def _estimate_memory_mb(config: Mapping[str, Any]) -> float:
    paths = int(config.get("n_paths", 100_000))
    steps = int(config.get("n_steps", 64))
    return round(paths * max(steps, 1) * 8 / (1024 * 1024), 3)


def _rejected_methods(workload: str, selected_method: str) -> list[dict[str, str]]:
    rejected = []
    if selected_method != "control_variates":
        rejected.append(
            {
                "method_id": "control_variates",
                "reason": "not selected under current accuracy preferences",
            }
        )
    if workload != "arithmetic_asian_call":
        rejected.append(
            {
                "method_id": "multilevel_monte_carlo",
                "reason": "current MLMC recommendation surface is arithmetic Asian focused",
            }
        )
    return rejected


def _pricing_payload(result: Any) -> dict[str, Any]:
    return {
        "price": result.price,
        "stderr": result.stderr,
        "explanation": result.explain(),
        "warnings": list(result.warnings),
    }


def _error_response(
    tool: str,
    workload: str,
    config: Mapping[str, Any],
    error: McConfigurationError,
) -> dict[str, Any]:
    return {
        "ok": False,
        "diagnostics": [
            {
                "code": error.code,
                "message": error.message,
                "suggestion": error.suggestion,
            }
        ],
        "manifest": _agent_manifest(
            tool=tool,
            workload=workload,
            config=config,
            method="failed_validation",
            warnings=(error.message,),
        ),
    }


def _artifact_kwargs(request: Mapping[str, Any]) -> dict[str, Any]:
    kwargs: dict[str, Any] = {}
    if request.get("repo_root") is not None:
        kwargs["repo_root"] = request["repo_root"]
    if request.get("benchmark_artifact") is not None:
        kwargs["benchmark_artifact"] = request["benchmark_artifact"]
    return kwargs


def _planner_response(
    *,
    tool: str,
    workload: str,
    result: dict[str, Any],
    method: str | None,
    reference: Any = None,
) -> dict[str, Any]:
    diagnostics = list(result.get("diagnostics", ()))
    return {
        "ok": not diagnostics,
        "result": result,
        "diagnostics": diagnostics,
        "manifest": _agent_manifest(
            tool=tool,
            workload=workload or "unknown",
            config={},
            method=method,
            warnings=tuple(result.get("caveats", ())),
            reference=reference,
        ),
    }


def _agent_manifest(
    *,
    tool: str,
    workload: str,
    config: Mapping[str, Any],
    method: str | None,
    estimator: str | None = None,
    warnings: tuple[str, ...] = (),
    reference: Any = None,
) -> dict[str, Any]:
    return {
        "schema_version": "agent-run.v1",
        "tool": tool,
        "workload": workload,
        "seed": config.get("seed"),
        "backend": "python_reference",
        "method": method,
        "estimator": estimator,
        "config": dict(config),
        "build": {
            "package": "montepath",
            "version": _package_version(),
            "python": sys.version.split()[0],
        },
        "hardware": {
            "platform": platform.platform(),
            "machine": platform.machine(),
            "processor": platform.processor(),
        },
        "warnings": list(warnings),
        "reference": reference,
        "determinism": "deterministic for same config, seed, package version, and Python version unless tool notes say environment-sensitive",
    }


def _package_version() -> str:
    try:
        return metadata.version("montepath")
    except metadata.PackageNotFoundError:
        return "editable-or-source-tree"


def _benchmark_command(profile: str, release: bool) -> list[str]:
    command = ["cargo", "run", "-p", "mc-bench"]
    if release:
        command.append("--release")
    command.extend(["--", "--profile", profile])
    return command
