"""Production-facing capability, validation, and backend selection helpers."""

from __future__ import annotations

import json
import platform
import sys
from dataclasses import asdict, dataclass
from importlib import metadata
from pathlib import Path
from typing import Any, Callable, Mapping

from .native import DEFAULT_NATIVE_MODULE, NativeRuntimeStatus, native_runtime_status
from .pricing import (
    AmericanPutConfig,
    ArithmeticAsianCallConfig,
    ArithmeticAsianMlmcConfig,
    BasketCallConfig,
    BermudanPutConfig,
    DownAndOutCallConfig,
    EuropeanCallConfig,
    EuropeanCallParameterSweepConfig,
    GaussianUncertaintyConfig,
    HestonEuropeanCallConfig,
    LookbackCallConfig,
    McConfigurationError,
    MertonJumpDiffusionCallConfig,
    NativeWorkloadResult,
    PricingResult,
    gaussian_uncertainty_moments,
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
)

PYTHON_REFERENCE_WORKLOADS = {
    "european_call",
    "arithmetic_asian_call",
    "down_and_out_call",
    "european_call_greeks",
}

NATIVE_FUNCTION_BY_WORKLOAD = {
    "european_call": "price_european_call",
    "arithmetic_asian_call": "price_arithmetic_asian_call",
    "down_and_out_call": "price_down_and_out_call",
    "lookback_call": "price_lookback_call",
    "basket_call": "price_basket_call",
    "american_put": "price_american_put",
    "bermudan_put": "price_bermudan_put",
    "heston_european_call": "price_heston_european_call",
    "merton_jump_diffusion_call": "price_merton_jump_diffusion_call",
    "european_call_parameter_sweep": "price_european_call_parameter_sweep",
    "gaussian_uncertainty_moments": "gaussian_uncertainty_moments",
    "arithmetic_asian_mlmc": "arithmetic_asian_mlmc",
}

METAL_WORKLOADS = {
    "european_call",
    "arithmetic_asian_call",
    "down_and_out_call",
}

BACKEND_ALIASES = {
    "auto": "auto",
    "python": "python_reference",
    "python_reference": "python_reference",
    "cpu": "cpu_native",
    "cpu_native": "cpu_native",
    "native": "cpu_native",
    "metal": "apple_metal",
    "apple_metal": "apple_metal",
    "cuda": "nvidia_cuda",
    "nvidia_cuda": "nvidia_cuda",
}


@dataclass(frozen=True)
class BackendCapability:
    """Machine-readable status for a backend visible to users and agents."""

    backend_id: str
    display_name: str
    status: str
    execution_mode: str
    workloads: tuple[str, ...]
    agent_tool_ready: bool
    deterministic: str
    reason: str | None = None
    warnings: tuple[str, ...] = ()
    native_module: str | None = None
    version: str | None = None

    def as_dict(self) -> dict[str, Any]:
        payload = asdict(self)
        payload["workloads"] = list(self.workloads)
        payload["warnings"] = list(self.warnings)
        return payload


@dataclass(frozen=True)
class BackendSelection:
    """Result of selecting an execution backend for one workload."""

    ok: bool
    backend_id: str
    execution_mode: str
    workload: str
    agent_tool_ready: bool
    diagnostics: tuple[dict[str, str], ...] = ()
    warnings: tuple[str, ...] = ()
    native_module: str | None = None

    def as_dict(self) -> dict[str, Any]:
        return {
            "ok": self.ok,
            "backend_id": self.backend_id,
            "execution_mode": self.execution_mode,
            "workload": self.workload,
            "agent_tool_ready": self.agent_tool_ready,
            "diagnostics": [dict(item) for item in self.diagnostics],
            "warnings": list(self.warnings),
            "native_module": self.native_module,
        }


class ProductionCapabilityError(RuntimeError):
    """Raised when explicit backend selection cannot be honored."""

    def __init__(self, selection: BackendSelection):
        self.selection = selection
        message = "; ".join(item["message"] for item in selection.diagnostics)
        super().__init__(message or f"Backend {selection.backend_id} is unavailable")


def backend_capabilities(
    native_module: str = DEFAULT_NATIVE_MODULE,
) -> tuple[BackendCapability, ...]:
    """Return CPU, Metal, CUDA, and Python-reference backend capability states."""

    native_status = native_runtime_status(native_module)
    machine = platform.machine().lower()
    system = platform.system().lower()
    metal_host = system == "darwin" and machine in {"arm64", "aarch64"}

    supported_native_workloads = tuple(
        workload
        for workload, function_name in NATIVE_FUNCTION_BY_WORKLOAD.items()
        if function_name in native_status.supported_functions
    )
    cpu_status = "available" if native_status.available else "unavailable"
    cpu_reason = None if native_status.available else native_status.reason

    metal_status = "unsupported"
    metal_reason = (
        "native Metal execution exists in Rust feature-gated paths, but the PyPI "
        "Python native bridge does not expose apple_metal execution yet"
    )
    if not metal_host:
        metal_status = "unavailable"
        metal_reason = "Apple Metal requires an Apple Silicon macOS host"

    return (
        BackendCapability(
            backend_id="cpu_native",
            display_name="Rust CPU native extension",
            status=cpu_status,
            execution_mode="native_extension",
            workloads=supported_native_workloads,
            agent_tool_ready=native_status.available,
            deterministic="deterministic for same config, seed, native module version, and CPU runtime",
            reason=cpu_reason,
            native_module=native_status.module_name,
            version=native_status.version,
        ),
        BackendCapability(
            backend_id="python_reference",
            display_name="Python reference helpers",
            status="available",
            execution_mode="python_reference",
            workloads=tuple(sorted(PYTHON_REFERENCE_WORKLOADS)),
            agent_tool_ready=True,
            deterministic="deterministic for same config, seed, package version, and Python version",
            warnings=(
                "Python reference helpers prioritize inspectability, not performance claims.",
            ),
        ),
        BackendCapability(
            backend_id="apple_metal",
            display_name="Apple Metal native runtime",
            status=metal_status,
            execution_mode="rust_feature_gated",
            workloads=tuple(sorted(METAL_WORKLOADS)),
            agent_tool_ready=False,
            deterministic="deterministic for same config, seed, Metal runtime, and device class where native Rust path is enabled",
            reason=metal_reason,
            warnings=(
                "Use Rust hardware workflow artifacts before making Metal performance claims.",
            ),
        ),
        BackendCapability(
            backend_id="nvidia_cuda",
            display_name="NVIDIA CUDA runtime",
            status="deferred",
            execution_mode="staged_not_native_execution",
            workloads=("european_call",),
            agent_tool_ready=False,
            deterministic="planned; deterministic CUDA RNG stream partitioning is not implemented",
            reason="native CUDA launch, reductions, and GPU RNG are deferred",
        ),
    )


def select_backend(
    workload: str,
    *,
    backend: str = "auto",
    native_module: str = DEFAULT_NATIVE_MODULE,
) -> BackendSelection:
    """Select a backend without executing the workload."""

    normalized = _normalize_backend(backend)
    if workload not in SUPPORTED_WORKLOADS:
        return _selection_error(
            normalized,
            workload,
            "MC_BACKEND_UNSUPPORTED_WORKLOAD",
            f"Unsupported workload {workload!r}",
            f"Use one of: {', '.join(sorted(SUPPORTED_WORKLOADS))}",
        )

    native_status = native_runtime_status(native_module)
    native_function = NATIVE_FUNCTION_BY_WORKLOAD.get(workload)
    native_ok = bool(
        native_status.available
        and native_function
        and native_function in native_status.supported_functions
    )

    if normalized == "auto":
        if native_ok:
            return BackendSelection(
                ok=True,
                backend_id="cpu_native",
                execution_mode="native_extension",
                workload=workload,
                agent_tool_ready=True,
                native_module=native_module,
            )
        if workload in PYTHON_REFERENCE_WORKLOADS:
            return BackendSelection(
                ok=True,
                backend_id="python_reference",
                execution_mode="python_reference",
                workload=workload,
                agent_tool_ready=True,
                warnings=(
                    "compiled CPU native runtime unavailable for this workload; using Python reference helper",
                ),
            )
        return _selection_error(
            "cpu_native",
            workload,
            "MC_BACKEND_NATIVE_REQUIRED",
            "This workload requires the Rust CPU native extension in the Python package",
            "Install a wheel/source build with montepath._native, or choose a Python-reference workload.",
        )

    if normalized == "cpu_native":
        if native_ok:
            return BackendSelection(
                ok=True,
                backend_id="cpu_native",
                execution_mode="native_extension",
                workload=workload,
                agent_tool_ready=True,
                native_module=native_module,
            )
        return _selection_error(
            "cpu_native",
            workload,
            "MC_BACKEND_CPU_NATIVE_UNAVAILABLE",
            f"Rust CPU native function for workload {workload!r} is unavailable",
            "Install the compiled montepath wheel or select backend='python_reference' for supported reference workloads.",
        )

    if normalized == "python_reference":
        if workload in PYTHON_REFERENCE_WORKLOADS:
            return BackendSelection(
                ok=True,
                backend_id="python_reference",
                execution_mode="python_reference",
                workload=workload,
                agent_tool_ready=True,
            )
        return _selection_error(
            "python_reference",
            workload,
            "MC_BACKEND_PYTHON_REFERENCE_UNSUPPORTED",
            f"Python reference execution is not implemented for workload {workload!r}",
            "Use backend='auto' or backend='cpu_native' with the compiled native extension.",
        )

    if normalized == "apple_metal":
        return _selection_error(
            "apple_metal",
            workload,
            "MC_BACKEND_METAL_NOT_EXPOSED",
            "Apple Metal execution is benchmarked in Rust feature-gated paths but is not exposed through the PyPI Python bridge yet",
            "Use backend='cpu_native' or backend='auto' from Python; run the Metal hardware workflow for Metal benchmark artifacts.",
        )

    if normalized == "nvidia_cuda":
        return _selection_error(
            "nvidia_cuda",
            workload,
            "MC_BACKEND_CUDA_DEFERRED",
            "Native CUDA launch, reductions, and deterministic GPU RNG are deferred",
            "Use CPU native or Python reference execution for production today.",
        )

    return _selection_error(
        normalized,
        workload,
        "MC_BACKEND_UNKNOWN",
        f"Unknown backend {backend!r}",
        f"Use one of: {', '.join(sorted(BACKEND_ALIASES))}",
    )


def validate_workload_request(
    workload: str,
    config: Mapping[str, Any] | None = None,
    *,
    backend: str = "auto",
    native_module: str = DEFAULT_NATIVE_MODULE,
) -> dict[str, Any]:
    """Validate workload config and backend support without executing."""

    payload = dict(config or {})
    diagnostics = _validate_config(workload, payload)
    selection = select_backend(workload, backend=backend, native_module=native_module)
    diagnostics.extend(selection.diagnostics)
    return {
        "ok": not diagnostics,
        "schema_version": "montepath-validation.v1",
        "workload": workload,
        "config": payload,
        "selection": selection.as_dict(),
        "diagnostics": diagnostics,
        "warnings": list(selection.warnings),
    }


def execute_workload(
    workload: str,
    config: Mapping[str, Any] | None = None,
    *,
    backend: str = "auto",
    native_module: str = DEFAULT_NATIVE_MODULE,
) -> dict[str, Any]:
    """Execute a supported workload through an explicit production backend policy."""

    payload = dict(config or {})
    validation = validate_workload_request(
        workload, payload, backend=backend, native_module=native_module
    )
    if not validation["ok"]:
        selection = BackendSelection(
            ok=False,
            backend_id=validation["selection"]["backend_id"],
            execution_mode=validation["selection"]["execution_mode"],
            workload=workload,
            agent_tool_ready=False,
            diagnostics=tuple(validation["diagnostics"]),
            warnings=tuple(validation["warnings"]),
            native_module=validation["selection"].get("native_module"),
        )
        raise ProductionCapabilityError(selection)

    selection = validation["selection"]
    result = _execute_selected_workload(
        workload,
        payload,
        backend_id=selection["backend_id"],
        native_module=native_module,
    )
    return {
        "ok": True,
        "schema_version": "montepath-execution.v1",
        "workload": workload,
        "selection": selection,
        "result": result,
        "manifest": result["manifest"],
        "warnings": list(result["warnings"]),
    }


def production_status(native_module: str = DEFAULT_NATIVE_MODULE) -> dict[str, Any]:
    """Return a high-level production readiness snapshot."""

    capabilities = backend_capabilities(native_module)
    native_status = native_runtime_status(native_module)
    return {
        "schema_version": "montepath-production-status.v1",
        "package": "montepath",
        "version": _package_version(),
        "python": sys.version.split()[0],
        "platform": {
            "system": platform.system(),
            "machine": platform.machine(),
            "platform": platform.platform(),
        },
        "native_runtime": native_status.as_dict(),
        "backend_capabilities": [item.as_dict() for item in capabilities],
        "agent_ready": True,
        "mcp_ready": True,
        "production_notes": [
            "CPU native execution is the production Python fast path when montepath._native is installed.",
            "Apple Metal is validated through Rust feature-gated hardware workflows, not the PyPI Python bridge yet.",
            "CUDA native execution remains deferred and must not be claimed as production support.",
            "Benchmark claims should be tied to release artifacts produced on target hardware.",
        ],
    }


def benchmark_report(
    benchmark_artifact: str = "benchmarks/release-results.json",
    *,
    workload: str | None = None,
    repo_root: str | Path | None = None,
) -> dict[str, Any]:
    """Summarize committed benchmark artifacts for humans and agents."""

    root = Path(repo_root) if repo_root is not None else _repo_root()
    path = root / benchmark_artifact
    if not path.exists():
        return {
            "schema_version": "montepath-benchmark-report.v1",
            "benchmark_artifact": benchmark_artifact,
            "workload_filter": workload,
            "artifact_available": False,
            "generated_at_unix_ms": None,
            "row_count": 0,
            "available_row_count": 0,
            "competitor_unavailable": [],
            "fastest_row": None,
            "diagnostics": [
                {
                    "code": "MC_BENCHMARK_ARTIFACT_MISSING",
                    "message": f"Benchmark artifact {benchmark_artifact!r} was not found",
                    "suggestion": "Run benchmarks from a repository checkout or pass repo_root to an existing artifact.",
                }
            ],
            "notes": [
                "Installed wheels may not include repository benchmark artifacts.",
                "Benchmark claims require artifacts generated on target hardware.",
            ],
        }
    payload = json.loads(path.read_text())
    rows = list(payload.get("results", ()))
    if workload:
        rows = [row for row in rows if workload in str(row.get("benchmark_name", ""))]

    available_rows = [
        row for row in rows if not str(row.get("benchmark_name", "")).endswith("_unavailable")
    ]
    unavailable_rows = [
        row for row in rows if str(row.get("benchmark_name", "")).endswith("_unavailable")
    ]
    fastest = min(
        (
            row
            for row in available_rows
            if isinstance(row.get("per_iteration_us"), (int, float))
        ),
        key=lambda row: float(row["per_iteration_us"]),
        default=None,
    )
    return {
        "schema_version": "montepath-benchmark-report.v1",
        "benchmark_artifact": benchmark_artifact,
        "workload_filter": workload,
        "artifact_available": True,
        "generated_at_unix_ms": payload.get("generated_at_unix_ms"),
        "row_count": len(rows),
        "available_row_count": len(available_rows),
        "competitor_unavailable": [
            {
                "benchmark_name": row.get("benchmark_name"),
                "implementation": row.get("implementation"),
                "backend": row.get("backend"),
                "methodology": row.get("methodology"),
            }
            for row in unavailable_rows
        ],
        "fastest_row": None if fastest is None else _benchmark_row_summary(fastest),
        "notes": [
            "Benchmark artifacts are hardware-local; rerun on target production hardware before making performance claims.",
            "Unavailable competitor rows are kept explicit rather than hidden.",
        ],
        "diagnostics": [],
    }


SUPPORTED_WORKLOADS = PYTHON_REFERENCE_WORKLOADS | set(NATIVE_FUNCTION_BY_WORKLOAD)


def _normalize_backend(backend: str) -> str:
    return BACKEND_ALIASES.get(str(backend), str(backend))


def _selection_error(
    backend_id: str,
    workload: str,
    code: str,
    message: str,
    suggestion: str,
) -> BackendSelection:
    return BackendSelection(
        ok=False,
        backend_id=backend_id,
        execution_mode="unavailable",
        workload=workload,
        agent_tool_ready=False,
        diagnostics=({"code": code, "message": message, "suggestion": suggestion},),
    )


def _validate_config(workload: str, config: Mapping[str, Any]) -> list[dict[str, str]]:
    cls: type[Any] | None
    cls = {
        "european_call": EuropeanCallConfig,
        "arithmetic_asian_call": ArithmeticAsianCallConfig,
        "down_and_out_call": DownAndOutCallConfig,
        "european_call_greeks": EuropeanCallConfig,
        "lookback_call": LookbackCallConfig,
        "basket_call": BasketCallConfig,
        "american_put": AmericanPutConfig,
        "bermudan_put": BermudanPutConfig,
        "heston_european_call": HestonEuropeanCallConfig,
        "merton_jump_diffusion_call": MertonJumpDiffusionCallConfig,
        "european_call_parameter_sweep": EuropeanCallParameterSweepConfig,
        "gaussian_uncertainty_moments": GaussianUncertaintyConfig,
        "arithmetic_asian_mlmc": ArithmeticAsianMlmcConfig,
    }.get(workload)
    if cls is None:
        if workload in NATIVE_FUNCTION_BY_WORKLOAD:
            return []
        return [
            {
                "code": "MC_WORKLOAD_UNKNOWN",
                "message": f"Unsupported workload {workload!r}",
                "suggestion": f"Use one of: {', '.join(sorted(SUPPORTED_WORKLOADS))}",
            }
        ]
    try:
        cfg = cls(**config)
    except TypeError as exc:
        return [
            {
                "code": "MC_CONFIG_SHAPE",
                "message": str(exc),
                "suggestion": "Use documented config keys for the selected workload.",
            }
        ]
    diagnostics: list[dict[str, str]] = []
    if int(getattr(cfg, "n_paths", 100_000)) <= 0:
        diagnostics.append(
            {
                "code": "MC_CONFIG_PATHS",
                "message": "n_paths must be greater than zero",
                "suggestion": "Set n_paths to a positive integer.",
            }
        )
    if int(getattr(cfg, "n_steps", 64)) <= 0:
        diagnostics.append(
            {
                "code": "MC_CONFIG_STEPS",
                "message": "n_steps must be greater than zero",
                "suggestion": "Set n_steps to a positive integer.",
            }
        )
    for name in ("spot", "strike", "maturity"):
        if float(getattr(cfg, name, 1.0)) <= 0.0:
            diagnostics.append(
                {
                    "code": "MC_CONFIG_POSITIVE",
                    "message": f"{name} must be greater than zero",
                    "suggestion": f"Set {name} to a positive number.",
                }
            )
    if float(getattr(cfg, "volatility", 0.2)) < 0.0:
        diagnostics.append(
            {
                "code": "MC_CONFIG_VOLATILITY",
                "message": "volatility must be non-negative",
                "suggestion": "Set volatility to zero or a positive decimal.",
            }
        )
    if isinstance(cfg, DownAndOutCallConfig) and (cfg.barrier <= 0.0 or cfg.barrier >= cfg.spot):
        diagnostics.append(
            {
                "code": "MC_CONFIG_BARRIER",
                "message": "barrier must be positive and below spot",
                "suggestion": "Use 0 < barrier < spot.",
            }
        )
    return diagnostics


def _execute_selected_workload(
    workload: str,
    config: Mapping[str, Any],
    *,
    backend_id: str,
    native_module: str,
) -> dict[str, Any]:
    native_arg = native_module if backend_id == "cpu_native" else None
    if workload == "european_call":
        result = price_european_call(**config, native_module=native_arg)
    elif workload == "arithmetic_asian_call":
        result = price_arithmetic_asian_call(**config, native_module=native_arg)
    elif workload == "down_and_out_call":
        result = price_down_and_out_call(**config, native_module=native_arg)
    elif workload == "european_call_greeks":
        report = price_european_call_greeks(**config)
        return {
            "base_price": report.base_price,
            "greeks": dict(report.greeks),
            "manifest": dict(report.manifest) | {"backend": backend_id},
            "warnings": list(report.warnings),
        }
    elif workload == "lookback_call":
        result = price_lookback_call(**config, native_module=native_module)
    elif workload == "basket_call":
        result = price_basket_call(**config, native_module=native_module)
    elif workload == "american_put":
        result = price_american_put(**config, native_module=native_module)
    elif workload == "bermudan_put":
        result = price_bermudan_put(**config, native_module=native_module)
    elif workload == "heston_european_call":
        result = price_heston_european_call(**config, native_module=native_module)
    elif workload == "merton_jump_diffusion_call":
        result = price_merton_jump_diffusion_call(**config, native_module=native_module)
    elif workload == "european_call_parameter_sweep":
        native_result = price_european_call_parameter_sweep(**config, native_module=native_module)
        return _native_workload_payload(native_result, backend_id)
    elif workload == "gaussian_uncertainty_moments":
        native_result = gaussian_uncertainty_moments(**config, native_module=native_module)
        return _native_workload_payload(native_result, backend_id)
    elif workload == "arithmetic_asian_mlmc":
        native_result = price_arithmetic_asian_mlmc(**config, native_module=native_module)
        return _native_workload_payload(native_result, backend_id)
    else:
        raise ProductionCapabilityError(
            _selection_error(
                backend_id,
                workload,
                "MC_EXECUTION_UNSUPPORTED",
                f"Production execute_workload does not yet execute {workload!r} directly",
                "Use the workload-specific native Python helper or MCP planning surfaces.",
            )
        )

    return _pricing_result_payload(result, backend_id)


def _pricing_result_payload(result: PricingResult, backend_id: str) -> dict[str, Any]:
    return {
        "price": result.price,
        "stderr": result.stderr,
        "n_paths": result.n_paths,
        "n_steps": result.n_steps,
        "seed": result.seed,
        "manifest": dict(result.manifest) | {"backend": backend_id},
        "warnings": list(result.warnings),
        "explanation": result.explain(),
    }


def _native_workload_payload(result: NativeWorkloadResult, backend_id: str) -> dict[str, Any]:
    return {
        "values": dict(result.values),
        "stderr": result.stderr,
        "manifest": dict(result.manifest) | {"backend": backend_id},
        "warnings": list(result.warnings),
        "explanation": result.explain(),
    }


def _benchmark_row_summary(row: Mapping[str, Any]) -> dict[str, Any]:
    return {
        "benchmark_name": row.get("benchmark_name"),
        "implementation": row.get("implementation"),
        "backend": row.get("backend"),
        "methodology": row.get("methodology"),
        "per_iteration_us": row.get("per_iteration_us"),
        "metric_name": row.get("metric_name"),
        "metric_value": row.get("metric_value"),
    }


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _package_version() -> str:
    try:
        return metadata.version("montepath")
    except metadata.PackageNotFoundError:
        return "editable-or-source-tree"
