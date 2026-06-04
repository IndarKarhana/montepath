"""Typed inventory simulation APIs backed by the Rust runtime.

The dependency-free reference implementation exists for semantic audits and
deterministic fixtures. Production throughput comes from ``montepath._native``.
"""

from __future__ import annotations

import math
import random
from dataclasses import asdict, dataclass, replace
from typing import Any, Mapping

from .native import (
    DEFAULT_NATIVE_MODULE,
    NativeFunctionUnavailableError,
    native_runtime_status,
    require_native_runtime,
)
from .pricing import McConfigurationError, ReproductionRecipe


@dataclass(frozen=True)
class InventoryDemandConfig:
    distribution: str = "normal"
    units: float = 0.0
    mean: float = 10.0
    std_dev: float = 2.0


@dataclass(frozen=True)
class InventoryPolicy:
    reorder_point: float = 50.0
    order_up_to: float = 100.0


@dataclass(frozen=True)
class InventoryConstraints:
    minimum_order_quantity: float = 0.0
    case_pack: float = 1.0
    supplier_capacity_per_period: float | None = None
    warehouse_capacity: float | None = None


@dataclass(frozen=True)
class InventoryCostConfig:
    holding_cost_per_unit_period: float = 0.0
    backorder_cost_per_unit_period: float = 0.0
    lost_sale_cost_per_unit: float = 0.0
    fixed_order_cost: float = 0.0
    variable_order_cost_per_unit: float = 0.0


@dataclass(frozen=True)
class InventoryTraceConfig:
    path_indices: tuple[int, ...] = ()
    max_periods: int = 0


@dataclass(frozen=True)
class InventorySimulationConfig:
    n_paths: int = 10_000
    n_periods: int = 52
    warmup_periods: int = 0
    seed: int = 42
    n_threads: int = 0
    initial_on_hand: float = 100.0
    initial_backorder: float = 0.0
    lead_time_periods: int = 1
    demand: InventoryDemandConfig = InventoryDemandConfig()
    shortage_policy: str = "lost_sales"
    policy: InventoryPolicy = InventoryPolicy()
    constraints: InventoryConstraints = InventoryConstraints()
    costs: InventoryCostConfig = InventoryCostConfig()
    trace: InventoryTraceConfig = InventoryTraceConfig()


@dataclass(frozen=True)
class InventorySimulationResult:
    summary: Mapping[str, Any]
    paths: tuple[Mapping[str, Any], ...]
    traces: tuple[Mapping[str, Any], ...]
    manifest: Mapping[str, Any]
    warnings: tuple[str, ...] = ()

    @property
    def backend(self) -> str:
        return str(self.manifest.get("backend", "unknown"))

    def explain(self) -> str:
        return (
            f"Inventory policy simulated with {self.backend}: "
            f"mean total cost={self.summary['total_cost']['mean']:.6f}, "
            f"mean fill rate={self.summary['fill_rate']['mean']:.6f}, "
            f"paths={self.manifest.get('n_paths')}, "
            f"periods={self.manifest.get('n_periods')}."
        )

    def reproduce(self) -> ReproductionRecipe:
        config = self.manifest["config"]
        function_name = (
            "simulate_inventory_policy_reference"
            if self.backend == "python_reference"
            else "simulate_inventory_policy"
        )
        return ReproductionRecipe(
            python=(
                f"from montepath import {function_name}\n"
                f"result = {function_name}(**{config!r})\n"
                "print(result.summary)\n"
            ),
            manifest=self.manifest,
        )


def validate_inventory_config(
    config: InventorySimulationConfig | None = None,
    **overrides: Any,
) -> tuple[dict[str, str], ...]:
    """Validate the stable inventory contract without executing simulation paths."""

    cfg = _coerce_config(config, overrides)
    diagnostics: list[dict[str, str]] = []

    if cfg.n_paths <= 0:
        diagnostics.append(_diagnostic("inventory.paths.invalid", "n_paths", "must be positive"))
    if cfg.n_periods <= 0:
        diagnostics.append(
            _diagnostic("inventory.periods.invalid", "n_periods", "must be positive")
        )
    if cfg.n_threads < 0:
        diagnostics.append(
            _diagnostic("inventory.threads.invalid", "n_threads", "must be non-negative")
        )
    if cfg.n_periods <= 0 or cfg.warmup_periods < 0 or cfg.warmup_periods >= cfg.n_periods:
        diagnostics.append(
            _diagnostic(
                "inventory.warmup.invalid",
                "warmup_periods",
                "must be non-negative and less than n_periods",
            )
        )
    _validate_non_negative(diagnostics, cfg.initial_on_hand, "initial_on_hand")
    _validate_non_negative(diagnostics, cfg.initial_backorder, "initial_backorder")
    if cfg.shortage_policy not in {"backorder", "lost_sales"}:
        diagnostics.append(
            _diagnostic(
                "inventory.shortage_policy.invalid",
                "shortage_policy",
                "must be 'backorder' or 'lost_sales'",
            )
        )
    if cfg.shortage_policy == "lost_sales" and cfg.initial_backorder > 0.0:
        diagnostics.append(
            _diagnostic(
                "inventory.initial_backorder.incompatible",
                "initial_backorder",
                "must be zero for lost-sales simulations",
            )
        )
    if cfg.lead_time_periods < 0 or cfg.lead_time_periods > 10_000:
        diagnostics.append(
            _diagnostic(
                "inventory.lead_time.unsupported",
                "lead_time_periods",
                "must be between 0 and 10,000",
            )
        )
    if len(cfg.trace.path_indices) > 16:
        diagnostics.append(
            _diagnostic(
                "inventory.trace.path_limit",
                "trace.path_indices",
                "must contain at most 16 paths",
            )
        )
    if cfg.trace.max_periods < 0 or cfg.trace.max_periods > 10_000:
        diagnostics.append(
            _diagnostic(
                "inventory.trace.period_limit",
                "trace.max_periods",
                "must be between 0 and 10,000",
            )
        )
    if cfg.trace.path_indices and cfg.trace.max_periods == 0:
        diagnostics.append(
            _diagnostic(
                "inventory.trace.periods_required",
                "trace.max_periods",
                "must be positive when trace paths are requested",
            )
        )
    seen_trace_paths: set[int] = set()
    for path_index in cfg.trace.path_indices:
        if path_index in seen_trace_paths:
            diagnostics.append(
                _diagnostic(
                    "inventory.trace.path_duplicate",
                    "trace.path_indices",
                    "must contain unique path indices",
                )
            )
        seen_trace_paths.add(path_index)
        if path_index < 0 or path_index >= cfg.n_paths:
            diagnostics.append(
                _diagnostic(
                    "inventory.trace.path_out_of_range",
                    "trace.path_indices",
                    "must contain only indices below n_paths",
                )
            )

    if cfg.demand.distribution == "deterministic":
        _validate_non_negative(
            diagnostics,
            cfg.demand.units,
            "demand.units",
            "inventory.demand.deterministic.invalid",
        )
    elif cfg.demand.distribution == "normal":
        _validate_non_negative(
            diagnostics,
            cfg.demand.mean,
            "demand.mean",
            "inventory.demand.normal_mean.invalid",
        )
        _validate_non_negative(
            diagnostics,
            cfg.demand.std_dev,
            "demand.std_dev",
            "inventory.demand.normal_std_dev.invalid",
        )
    else:
        diagnostics.append(
            _diagnostic(
                "inventory.demand.distribution.unsupported",
                "demand.distribution",
                "must be 'deterministic' or 'normal'",
            )
        )

    if not math.isfinite(cfg.policy.reorder_point):
        diagnostics.append(
            _diagnostic(
                "inventory.policy.reorder_point.invalid",
                "policy.reorder_point",
                "must be finite",
            )
        )
    if (
        not math.isfinite(cfg.policy.order_up_to)
        or cfg.policy.order_up_to < cfg.policy.reorder_point
    ):
        diagnostics.append(
            _diagnostic(
                "inventory.policy.order_up_to.invalid",
                "policy.order_up_to",
                "must be finite and at least reorder_point",
            )
        )

    _validate_non_negative(
        diagnostics,
        cfg.constraints.minimum_order_quantity,
        "constraints.minimum_order_quantity",
    )
    if not math.isfinite(cfg.constraints.case_pack) or cfg.constraints.case_pack <= 0.0:
        diagnostics.append(
            _diagnostic(
                "inventory.constraints.case_pack.invalid",
                "constraints.case_pack",
                "must be finite and positive",
            )
        )
    _validate_optional_non_negative(
        diagnostics,
        cfg.constraints.supplier_capacity_per_period,
        "constraints.supplier_capacity_per_period",
    )
    _validate_optional_non_negative(
        diagnostics,
        cfg.constraints.warehouse_capacity,
        "constraints.warehouse_capacity",
    )
    if (
        cfg.constraints.warehouse_capacity is not None
        and cfg.constraints.warehouse_capacity < cfg.initial_on_hand
    ):
        diagnostics.append(
            _diagnostic(
                "inventory.constraints.warehouse_capacity.initial_state",
                "constraints.warehouse_capacity",
                "cannot be below initial_on_hand",
            )
        )

    for name, value in asdict(cfg.costs).items():
        _validate_non_negative(diagnostics, float(value), f"costs.{name}")
    return tuple(diagnostics)


def simulate_inventory_policy(
    config: InventorySimulationConfig | None = None,
    *,
    native_module: str = DEFAULT_NATIVE_MODULE,
    **overrides: Any,
) -> InventorySimulationResult:
    """Execute inventory paths through the compiled Rust CPU runtime."""

    cfg = _coerce_config(config, overrides)
    _raise_for_diagnostics(validate_inventory_config(cfg))
    status = native_runtime_status(native_module)
    module = require_native_runtime(native_module)
    function = getattr(module, "simulate_inventory_policy", None)
    if not callable(function):
        raise NativeFunctionUnavailableError(status, "simulate_inventory_policy")
    raw = function(asdict(cfg))
    if not isinstance(raw, Mapping) or not isinstance(raw.get("values"), Mapping):
        raise McConfigurationError(
            "MC_NATIVE_RESULT",
            "native inventory function returned an unsupported result shape",
            "return values, manifest, and warnings mappings from the native bridge",
        )
    values = raw["values"]
    manifest = dict(raw.get("manifest", {}))
    run_manifest = values.get("manifest")
    if isinstance(run_manifest, Mapping):
        manifest |= dict(run_manifest)
    manifest["backend"] = "cpu_native"
    manifest["function"] = "simulate_inventory_policy"
    manifest["native_module"] = native_module
    manifest["config"] = asdict(cfg)
    return InventorySimulationResult(
        summary=dict(values["summary"]),
        paths=tuple(dict(path) for path in values["paths"]),
        traces=tuple(dict(trace) for trace in values.get("traces", ())),
        manifest=manifest,
        warnings=_warnings(raw),
    )


def simulate_inventory_policy_reference(
    config: InventorySimulationConfig | None = None,
    **overrides: Any,
) -> InventorySimulationResult:
    """Run the transparent scalar Python reference implementation."""

    cfg = _coerce_config(config, overrides)
    _raise_for_diagnostics(validate_inventory_config(cfg))
    paths = tuple(_simulate_reference_path(cfg, path_index) for path_index in range(cfg.n_paths))
    traces = tuple(_simulate_reference_trace(cfg, path_index) for path_index in cfg.trace.path_indices)
    manifest = {
        "schema_version": "inventory-simulation.v1",
        "semantics_version": "periodic-review.v1",
        "backend": "python_reference",
        "execution_mode": "scalar_reference",
        "function": "simulate_inventory_policy_reference",
        "period_order": "receive-demand-fulfill-review-order-record",
        "n_paths": cfg.n_paths,
        "n_periods": cfg.n_periods,
        "warmup_periods": cfg.warmup_periods,
        "observed_periods": cfg.n_periods - cfg.warmup_periods,
        "seed": cfg.seed,
        "trace_paths": list(cfg.trace.path_indices),
        "trace_period_limit": min(cfg.trace.max_periods, cfg.n_periods),
        "config": asdict(cfg),
        "reproducibility": "deterministic for identical config, seed, and Python version",
        "performance_claim": "reference-only; use Rust benchmark artifacts for timing claims",
    }
    return InventorySimulationResult(
        summary=_summarize(paths),
        paths=paths,
        traces=traces,
        manifest=manifest,
        warnings=("Python inventory reference prioritizes auditability, not throughput.",),
    )


def _simulate_reference_path(
    cfg: InventorySimulationConfig,
    path_index: int,
    trace_periods: list[dict[str, Any]] | None = None,
) -> dict[str, Any]:
    arrival_offset = max(1, cfg.lead_time_periods)
    arrivals = [0.0] * arrival_offset
    rng = random.Random(_path_seed(cfg.seed, path_index))
    on_hand = cfg.initial_on_hand
    on_order = 0.0
    backorder = cfg.initial_backorder
    totals: dict[str, float] = {
        "total_demand": 0.0,
        "fulfilled_demand": 0.0,
        "unmet_demand": 0.0,
        "received_units": 0.0,
        "ordered_units": 0.0,
        "on_hand_sum": 0.0,
        "holding_cost": 0.0,
        "shortage_cost": 0.0,
        "ordering_cost": 0.0,
    }
    service_periods = stockout_events = orders_placed = 0
    constraint_events = {
        "limited_order_attempts": 0,
        "minimum_order_quantity_adjustments": 0,
        "case_pack_roundups": 0,
        "supplier_capacity_clips": 0,
        "warehouse_capacity_clips": 0,
        "blocked_order_attempts": 0,
    }

    for period in range(cfg.n_periods):
        slot = period % arrival_offset
        receipt = arrivals[slot]
        arrivals[slot] = 0.0
        on_order = max(0.0, on_order - receipt)
        backorder_receipt = min(receipt, backorder)
        backorder -= backorder_receipt
        on_hand += receipt - backorder_receipt

        demand = _sample_demand(cfg.demand, rng)
        fulfilled = min(demand, on_hand)
        on_hand -= fulfilled
        unmet = demand - fulfilled
        if cfg.shortage_policy == "backorder":
            backorder += unmet

        inventory_position = on_hand + on_order - backorder
        quantity, events = _order_quantity(cfg, inventory_position, on_hand, on_order)
        if quantity > 0.0:
            arrival_slot = (period + arrival_offset) % arrival_offset
            arrivals[arrival_slot] += quantity
            on_order += quantity

        observed = period >= cfg.warmup_periods
        period_holding_cost = (
            on_hand * cfg.costs.holding_cost_per_unit_period if observed else 0.0
        )
        period_shortage_cost = 0.0
        if observed:
            period_shortage_cost = (
                backorder * cfg.costs.backorder_cost_per_unit_period
                if cfg.shortage_policy == "backorder"
                else unmet * cfg.costs.lost_sale_cost_per_unit
            )
        period_ordering_cost = (
            cfg.costs.fixed_order_cost + quantity * cfg.costs.variable_order_cost_per_unit
            if observed and quantity > 0.0
            else 0.0
        )

        if observed:
            for name, count in events.items():
                constraint_events[name] += count
            totals["total_demand"] += demand
            totals["fulfilled_demand"] += fulfilled
            totals["unmet_demand"] += unmet
            totals["received_units"] += receipt
            totals["on_hand_sum"] += on_hand
            if unmet == 0.0:
                service_periods += 1
            else:
                stockout_events += 1
            if quantity > 0.0:
                orders_placed += 1
                totals["ordered_units"] += quantity
                totals["ordering_cost"] += period_ordering_cost
            totals["holding_cost"] += period_holding_cost
            totals["shortage_cost"] += period_shortage_cost
        if trace_periods is not None and period < cfg.trace.max_periods:
            trace_periods.append(
                {
                    "period": period,
                    "observed": observed,
                    "received_units": receipt,
                    "demand": demand,
                    "fulfilled_demand": fulfilled,
                    "unmet_demand": unmet,
                    "inventory_position_before_order": inventory_position,
                    "order_quantity": quantity,
                    "constraint_events": events,
                    "holding_cost": period_holding_cost,
                    "shortage_cost": period_shortage_cost,
                    "ordering_cost": period_ordering_cost,
                    "ending_on_hand": max(0.0, on_hand),
                    "ending_on_order": max(0.0, on_order),
                    "ending_backorder": max(0.0, backorder),
                }
            )

    observed = cfg.n_periods - cfg.warmup_periods
    total_cost = (
        totals["holding_cost"] + totals["shortage_cost"] + totals["ordering_cost"]
    )
    return {
        "path_index": path_index,
        "cycle_service_level": service_periods / observed,
        "fill_rate": (
            totals["fulfilled_demand"] / totals["total_demand"]
            if totals["total_demand"] > 0.0
            else 1.0
        ),
        "average_on_hand": totals.pop("on_hand_sum") / observed,
        "stockout_events": stockout_events,
        **totals,
        "orders_placed": orders_placed,
        "constraint_events": constraint_events,
        "total_cost": total_cost,
        "ending_on_hand": max(0.0, on_hand),
        "ending_on_order": max(0.0, on_order),
        "ending_backorder": max(0.0, backorder),
    }


def _simulate_reference_trace(cfg: InventorySimulationConfig, path_index: int) -> dict[str, Any]:
    periods: list[dict[str, Any]] = []
    _simulate_reference_path(cfg, path_index, periods)
    return {"path_index": path_index, "periods": periods}


def _order_quantity(
    cfg: InventorySimulationConfig, inventory_position: float, on_hand: float, on_order: float
) -> tuple[float, dict[str, int]]:
    events = {
        "limited_order_attempts": 0,
        "minimum_order_quantity_adjustments": 0,
        "case_pack_roundups": 0,
        "supplier_capacity_clips": 0,
        "warehouse_capacity_clips": 0,
        "blocked_order_attempts": 0,
    }
    if inventory_position > cfg.policy.reorder_point:
        return 0.0, events
    desired = max(0.0, cfg.policy.order_up_to - inventory_position)
    if desired == 0.0:
        return 0.0, events

    quantity = max(desired, cfg.constraints.minimum_order_quantity)
    if quantity != desired:
        events["minimum_order_quantity_adjustments"] = 1
    packed = math.ceil(quantity / cfg.constraints.case_pack) * cfg.constraints.case_pack
    if packed != quantity:
        events["case_pack_roundups"] = 1
    quantity = packed
    if cfg.constraints.supplier_capacity_per_period is not None:
        clipped = min(quantity, cfg.constraints.supplier_capacity_per_period)
        if clipped != quantity:
            events["supplier_capacity_clips"] = 1
        quantity = clipped
    if cfg.constraints.warehouse_capacity is not None:
        clipped = min(quantity, max(0.0, cfg.constraints.warehouse_capacity - on_hand - on_order))
        if clipped != quantity:
            events["warehouse_capacity_clips"] = 1
        quantity = clipped
    if any(events[name] for name in events if name not in {"limited_order_attempts", "blocked_order_attempts"}):
        events["limited_order_attempts"] = 1
    if quantity == 0.0:
        events["blocked_order_attempts"] = 1
    return max(0.0, quantity), events


def _sample_demand(config: InventoryDemandConfig, rng: random.Random) -> float:
    if config.distribution == "deterministic":
        return config.units
    return max(0.0, rng.gauss(config.mean, config.std_dev))


def _summarize(paths: tuple[Mapping[str, Any], ...]) -> dict[str, Any]:
    names = (
        "cycle_service_level",
        "fill_rate",
        "average_on_hand",
        "stockout_events",
        "total_cost",
        "ending_on_hand",
        "ending_on_order",
        "ending_backorder",
    )
    return {name: _metric_summary([float(path[name]) for path in paths]) for name in names}


def _metric_summary(values: list[float]) -> dict[str, float]:
    values.sort()
    return {
        "mean": sum(values) / len(values),
        "min": values[0],
        "p05": _quantile(values, 0.05),
        "p50": _quantile(values, 0.50),
        "p95": _quantile(values, 0.95),
        "max": values[-1],
    }


def _quantile(values: list[float], probability: float) -> float:
    index = probability * (len(values) - 1)
    lower = math.floor(index)
    upper = math.ceil(index)
    if lower == upper:
        return values[lower]
    weight = index - lower
    return values[lower] * (1.0 - weight) + values[upper] * weight


def _coerce_config(
    config: InventorySimulationConfig | None, overrides: Mapping[str, Any]
) -> InventorySimulationConfig:
    if config is None:
        return InventorySimulationConfig(**_normalize_nested_config(overrides))
    if overrides:
        return replace(config, **_normalize_nested_config(overrides))
    return config


def _normalize_nested_config(values: Mapping[str, Any]) -> dict[str, Any]:
    normalized = dict(values)
    nested_types = {
        "demand": InventoryDemandConfig,
        "policy": InventoryPolicy,
        "constraints": InventoryConstraints,
        "costs": InventoryCostConfig,
        "trace": InventoryTraceConfig,
    }
    for name, config_type in nested_types.items():
        value = normalized.get(name)
        if isinstance(value, Mapping):
            normalized[name] = config_type(**value)
    return normalized


def _raise_for_diagnostics(diagnostics: tuple[dict[str, str], ...]) -> None:
    if diagnostics:
        first = diagnostics[0]
        raise McConfigurationError(first["code"], first["message"], first["suggestion"])


def _diagnostic(code: str, field: str, requirement: str) -> dict[str, str]:
    return {
        "code": code,
        "field": field,
        "message": f"{field} {requirement}",
        "suggestion": f"Provide a valid value for {field}.",
    }


def _validate_non_negative(
    diagnostics: list[dict[str, str]],
    value: float,
    field: str,
    code: str | None = None,
) -> None:
    if not math.isfinite(value) or value < 0.0:
        diagnostics.append(
            _diagnostic(code or f"inventory.{field}.invalid", field, "must be finite and non-negative")
        )


def _validate_optional_non_negative(
    diagnostics: list[dict[str, str]], value: float | None, field: str
) -> None:
    if value is not None:
        _validate_non_negative(diagnostics, value, field)


def _path_seed(base_seed: int, path_index: int) -> int:
    mask = (1 << 64) - 1
    value = (base_seed + path_index * 0x9E3779B97F4A7C15) & mask
    value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & mask
    value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & mask
    return (value ^ (value >> 31)) & mask


def _warnings(raw: Mapping[str, Any]) -> tuple[str, ...]:
    values = raw.get("warnings", ())
    if isinstance(values, str):
        return (values,)
    return tuple(str(value) for value in values)
