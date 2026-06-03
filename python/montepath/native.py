"""Discovery helpers for the optional compiled native runtime.

The dependency-free Python package is useful for examples, reproducibility,
and agent tooling. High-throughput execution should eventually come from a
compiled Rust extension. These helpers make that boundary explicit so callers
can inspect capability instead of inferring it from failed imports.
"""

from __future__ import annotations

from dataclasses import dataclass
from importlib import import_module
from types import ModuleType
from typing import Any


DEFAULT_NATIVE_MODULE = "montepath._native"

KNOWN_NATIVE_FUNCTIONS = (
    "price_european_call",
    "price_arithmetic_asian_call",
    "price_down_and_out_call",
    "price_european_call_metal",
    "price_arithmetic_asian_call_metal",
    "price_down_and_out_call_metal",
    "price_lookback_call",
    "price_basket_call",
    "price_american_put",
    "price_bermudan_put",
    "price_heston_european_call",
    "price_merton_jump_diffusion_call",
    "price_european_call_parameter_sweep",
    "gaussian_uncertainty_moments",
    "arithmetic_asian_mlmc",
)


@dataclass(frozen=True)
class NativeRuntimeStatus:
    """Structured status for the optional compiled runtime module."""

    available: bool
    module_name: str = DEFAULT_NATIVE_MODULE
    version: str | None = None
    supported_functions: tuple[str, ...] = ()
    reason: str | None = None

    def as_dict(self) -> dict[str, Any]:
        """Return a JSON-serializable representation for agent tools."""

        return {
            "available": self.available,
            "module_name": self.module_name,
            "version": self.version,
            "supported_functions": list(self.supported_functions),
            "reason": self.reason,
        }


class NativeRuntimeUnavailableError(RuntimeError):
    """Raised when a caller explicitly requires the compiled runtime."""

    def __init__(self, status: NativeRuntimeStatus):
        self.status = status
        super().__init__(status.reason or f"{status.module_name} is not available")


class NativeFunctionUnavailableError(RuntimeError):
    """Raised when the native module is installed but lacks a requested function."""

    def __init__(self, status: NativeRuntimeStatus, function_name: str):
        self.status = status
        self.function_name = function_name
        super().__init__(
            f"{status.module_name} does not expose required native function "
            f"'{function_name}'"
        )


def _load_native_module(module_name: str) -> ModuleType:
    return import_module(module_name)


def native_runtime_status(module_name: str = DEFAULT_NATIVE_MODULE) -> NativeRuntimeStatus:
    """Inspect whether the optional compiled runtime module is importable."""

    try:
        module = _load_native_module(module_name)
    except ModuleNotFoundError as exc:
        if exc.name == module_name:
            return NativeRuntimeStatus(
                available=False,
                module_name=module_name,
                reason=(
                    "compiled native runtime module is not installed; "
                    "using Python reference helpers"
                ),
            )
        return NativeRuntimeStatus(
            available=False,
            module_name=module_name,
            reason=f"compiled native runtime import failed: {exc.__class__.__name__}: {exc}",
        )
    except Exception as exc:  # pragma: no cover - exercised by embedders/plugins.
        return NativeRuntimeStatus(
            available=False,
            module_name=module_name,
            reason=f"compiled native runtime import failed: {exc.__class__.__name__}: {exc}",
        )

    version = getattr(module, "__version__", None)
    if version is not None:
        version = str(version)
    supported_functions = tuple(
        name for name in KNOWN_NATIVE_FUNCTIONS if callable(getattr(module, name, None))
    )
    return NativeRuntimeStatus(
        available=True,
        module_name=module_name,
        version=version,
        supported_functions=supported_functions,
    )


def has_native_runtime(module_name: str = DEFAULT_NATIVE_MODULE) -> bool:
    """Return whether the optional compiled native runtime is importable."""

    return native_runtime_status(module_name).available


def require_native_runtime(module_name: str = DEFAULT_NATIVE_MODULE) -> ModuleType:
    """Import the optional native runtime or raise a structured error."""

    status = native_runtime_status(module_name)
    if not status.available:
        raise NativeRuntimeUnavailableError(status)
    return _load_native_module(module_name)
