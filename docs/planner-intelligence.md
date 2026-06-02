# Planner Intelligence Evidence

Phase 6 turns planner behavior into an inspectable, benchmark-backed surface.
The goal is for users and agents to answer:

- which method/backend is currently fastest for this workload
- which alternative buys accuracy at extra cost
- why a seemingly better method was rejected
- which benchmark artifact and reference fixture support the answer

## Python Surfaces

```python
from montepath import (
    compare_methods,
    cost_frontier,
    load_planner_evidence,
    measured_winner_database,
    mlmc_error_calibration,
    why_not_faster,
)

evidence = load_planner_evidence()
frontier = cost_frontier("european_call")
comparison = compare_methods("arithmetic_asian_call")
explanation = why_not_faster("european_call", method_id="scrambled_sobol")
calibration = mlmc_error_calibration("arithmetic_asian_call")
```

All functions return JSON-serializable dictionaries and read committed benchmark
artifacts by default. Pass `repo_root` or `benchmark_artifact` to inspect a
different checkout or artifact.

## Agent Tools

The same evidence layer is exposed through the agent manifest:

| Tool | Purpose |
| --- | --- |
| `montepath.planner_evidence` | Load measured planner accuracy, winner records, and reference fixture names. |
| `montepath.cost_frontier` | Return measured runtime rows ranked by cost for a workload. |
| `montepath.compare_methods` | Recommend a measured method/backend tradeoff and list alternatives. |
| `montepath.why_not_faster` | Explain why a requested method is not the current recommendation. |
| `montepath.mlmc_calibration` | Show estimated-vs-realized MLMC/MLQMC error evidence. |

## Current Planner Accuracy

The tracked release artifact reports:

| Benchmark | Methodology | Accuracy |
| --- | --- | ---: |
| `planner_choice_accuracy_measured` (ours) | measured local backend winners | `100%` |

This metric is hardware-local and must be refreshed on target machines before
making deployment claims. The benchmark now compares CPU wall-clock execution
against Metal wall-clock execution after warmup, so the timing basis is aligned.

## Caveats

- Native CUDA execution is still deferred, so NVIDIA planner decisions cannot
  yet be called production-calibrated.
- Structured sampling remains CPU-reference for native GPU backends.
- MLMC and MLQMC calibration is currently arithmetic-Asian focused and includes both stderr-ratio evidence and high-budget standard-MC reference absolute-error rows.
- Release artifacts are evidence records, not universal performance constants.
