# Competitiveness Plan

Current Rust baseline (`mc_cpu_european_call_rust`): `300.109 ms`

Status: Rust is slower than at least one available baseline.

Observed gaps:
- `mc_cpu_european_call_numpy` is faster: `70.524 ms` vs Rust `300.109 ms` (Rust is `4.26x` slower)
- `mc_cpu_european_call_numba` is faster: `254.348 ms` vs Rust `300.109 ms` (Rust is `1.18x` slower)

Action plan to close the gap:
- Introduce SIMD-friendly normal generation and batched exponentials in CPU runtime.
- Add multithreaded path partitioning with deterministic stream splitting.
- Benchmark release profile (`--release`) and optimize hottest functions with profiler evidence.
- Add workload-specialized kernels for common cases (`n_steps` fixed, drift/vol precomputed).
