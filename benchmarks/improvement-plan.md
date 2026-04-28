# Competitiveness Plan

Current tracked leaders:
- Rust fair CPU baseline (`mc_cpu_european_call_rust`, step-wise): `14.061 ms`
- Native Metal GBM baseline (`mc_metal_european_call_native`): `1.451 ms`
- Down-and-out breadth check: CPU `60.738 ms`, Metal `0.941 ms`
- Measured planner choice accuracy: `87.5%`

Status: Rust currently leads the available CPU baselines for the tracked fair European workload.

Maintain lead plan:
- Keep the step-wise benchmark as the primary competitive claim.
- Keep RNG and loop hot path allocation-free.
- Keep breadth claims tied to the workloads we have actually benchmarked: European, arithmetic Asian, down-and-out, basket, and Gaussian UQ.
- Expand competitor matrix to GPU baselines (JAX/CuPy/PyTorch/CUDA-native) when hardware is available.
- First randomized-QMC pricing surface is live via randomized Halton (`86.695 ms`), but it is currently a quality-first pricing path rather than a speed leader.
- Latin hypercube pricing is live (`71.105 ms`) as the first non-QMC structured-sampling breadth path.
- Scrambled Sobol pricing is live (`86.261 ms`) as the stronger QMC breadth path.
- Scrambled Sobol with Brownian bridge pricing is live (`114.053 ms`) for path construction experiments.
- QMC generation scoreboard is live: Rust scrambled Sobol generation `106.829 ms`, SciPy scrambled Sobol generation `136.034 ms` (`1.27x` Rust/SciPy speedup).
- Arithmetic Asian MLMC is live (`4.733 ms`) with adaptive tolerance planning as the first multilevel CPU reference path.
- Arithmetic Asian MLQMC is live (`7.266 ms`) with replicated scrambling and adaptive tolerance planning.
- Gaussian UQ benchmark is live: Latin hypercube `2.401 ms`, abs error `0.000039` vs pseudorandom abs error `0.006344`.
- Basket-call QMC breadth is live: scrambled Sobol basket pricing `8.178 ms`, Latin-hypercube stderr ratio vs pseudorandom `0.997`.
- European realized-error study is live against Black-Scholes: Latin hypercube abs-error ratio vs pseudorandom `0.021`, scrambled Sobol ratio `0.129`, Sobol Brownian-bridge ratio `0.001`.
- Preserve the randomized-QMC quality gain (`stderr_ratio_vs_standard = 0.411`) while optimizing sequence generation and path construction.
- Preserve the Latin-hypercube quality gain (`stderr_ratio_vs_standard = 0.410`) while benchmarking it across more workload families.
- Preserve the Sobol Brownian-bridge quality gain (`stderr_ratio_vs_standard = 0.411`) while optimizing its current runtime overhead.
- Track arithmetic Asian MLMC quality (`stderr_ratio_vs_standard = 2.013`) and calibrate tolerance defaults before claiming it as a default winner.
- Preserve arithmetic Asian replicated MLQMC quality (`stderr_ratio_vs_standard = 0.418`) while reducing its runtime overhead and increasing replicate coverage.
- Preserve the specialized terminal-distribution fast path (`0.632 ms`) as a separate optimization track.
- Improve planner calibration beyond the current measured accuracy of `87.5%` as workload breadth increases.
