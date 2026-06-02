"""Benchmark helpers for Python callers.

The current implementation shells out to the Rust benchmark binary and parses
its structured JSON output. That keeps the Python surface typed and stable while
native Python bindings are still pending.
"""

from __future__ import annotations

import json
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Literal


@dataclass(frozen=True)
class BenchmarkResult:
    benchmark_name: str
    backend: str
    methodology: str | None
    per_iteration_ms: float
    metric_name: str | None
    metric_value: float | None


def run_benchmarks(
    *,
    repo_root: str | Path = ".",
    release: bool = True,
    profile: Literal["full", "compact"] = "full",
) -> tuple[BenchmarkResult, ...]:
    root = Path(repo_root)
    with tempfile.NamedTemporaryFile(suffix=".json") as output:
        cmd = ["cargo", "run", "-p", "mc-bench"]
        if release:
            cmd.append("--release")
        cmd.extend(["--", "--profile", profile, "--output", output.name])

        subprocess.run(cmd, cwd=root, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        payload: dict[str, Any] = json.loads(Path(output.name).read_text())

    return tuple(_parse_result(item) for item in payload.get("results", ()))


def _parse_result(item: dict[str, Any]) -> BenchmarkResult:
    per_iteration_us = float(item.get("per_iteration_us") or 0.0)
    metric_value = item.get("metric_value")
    return BenchmarkResult(
        benchmark_name=str(item["benchmark_name"]),
        backend=str(item["backend"]),
        methodology=item.get("methodology"),
        per_iteration_ms=per_iteration_us / 1_000.0,
        metric_name=item.get("metric_name"),
        metric_value=float(metric_value) if metric_value is not None else None,
    )
