# Competitiveness Plan

Current Rust fair baseline (`mc_cpu_european_call_rust`, step-wise): `15.129 ms`

Status: Rust currently leads available CPU baselines for this workload.

Maintain lead plan:
- Keep the step-wise benchmark as the primary competitive claim.
- Keep RNG and loop hot path allocation-free.
- Add release-mode benchmark gates for MC runtime.
- Expand competitor matrix to GPU baselines (JAX/CuPy/PyTorch) when hardware is available.
- Preserve the specialized terminal-distribution fast path (`0.621 ms`) as a separate optimization track.
