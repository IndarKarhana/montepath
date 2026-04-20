# Competitiveness Plan

Current Rust baseline (`mc_cpu_european_call_rust`): `1.065 ms`

Status: Rust currently leads available CPU baselines for this workload.

Maintain lead plan:
- Keep RNG and loop hot path allocation-free.
- Add release-mode benchmark gates for MC runtime.
- Expand competitor matrix to GPU baselines (JAX/CuPy/PyTorch) when hardware is available.
