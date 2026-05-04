# Benchmark Interpretation

Benchmark tables should compare `(ours)` against external libraries on the same
workload, methodology, accuracy metric, and hardware context.

Read every result through these fields:

- `benchmark_name`: workload identity.
- `methodology`: fair step-wise path, terminal fast path, QMC generation, Greek
  estimator, or quality comparison.
- `per_iteration_us`: timing per run.
- `throughput_per_sec`: paths, samples, or work units per second.
- `metric_name` and `metric_value`: price, error, stderr ratio, or accuracy
  measure.

Rules:

- Do not compare terminal-distribution fast paths against path-dependent
  step-wise workloads.
- Treat unavailable competitor rows as environment facts, not wins.
- Use `benchmarks/reference-fixtures.json` to understand accuracy references.
- For accelerator claims, include warmup, compile time, hardware, and backend
  details when available.

