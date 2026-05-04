"""Python-facing helpers for mc-library.

This package is intentionally thin for now: it preserves typed, inspectable
contracts while the compiled Python extension surface is still being designed.
"""

from .benchmarks import BenchmarkResult, run_benchmarks
from .agent import (
    agent_benchmark,
    agent_compare_methods,
    agent_compare,
    agent_cost_frontier,
    agent_execute,
    agent_mlmc_calibration,
    agent_plan,
    agent_planner_evidence,
    agent_recommend,
    agent_reproduce,
    agent_tool_manifest,
    agent_validate,
    agent_why_not_faster,
    export_json_schemas,
)
from .methods import MethodRecommendation, recommend_method
from .planner_intelligence import (
    compare_methods,
    cost_frontier,
    load_planner_evidence,
    measured_winner_database,
    mlmc_error_calibration,
    why_not_faster,
)
from .pricing import (
    ArithmeticAsianCallConfig,
    DownAndOutCallConfig,
    EuropeanCallConfig,
    GreekReport,
    McConfigurationError,
    PricingResult,
    ReproductionRecipe,
    price_arithmetic_asian_call,
    price_down_and_out_call,
    price_european_call,
    price_european_call_greeks,
)

__all__ = [
    "ArithmeticAsianCallConfig",
    "BenchmarkResult",
    "DownAndOutCallConfig",
    "EuropeanCallConfig",
    "GreekReport",
    "McConfigurationError",
    "MethodRecommendation",
    "PricingResult",
    "ReproductionRecipe",
    "agent_benchmark",
    "agent_compare",
    "agent_compare_methods",
    "agent_cost_frontier",
    "agent_execute",
    "agent_mlmc_calibration",
    "agent_plan",
    "agent_planner_evidence",
    "agent_recommend",
    "agent_reproduce",
    "agent_tool_manifest",
    "agent_validate",
    "agent_why_not_faster",
    "compare_methods",
    "cost_frontier",
    "export_json_schemas",
    "load_planner_evidence",
    "measured_winner_database",
    "mlmc_error_calibration",
    "price_arithmetic_asian_call",
    "price_down_and_out_call",
    "price_european_call",
    "price_european_call_greeks",
    "recommend_method",
    "run_benchmarks",
    "why_not_faster",
]
