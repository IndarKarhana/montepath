# Competitiveness Plan

Current tracked leaders:
- Rust fair CPU baseline (`mc_cpu_european_call_rust`, step-wise): `12.612 ms`
- Native Metal GBM baseline (`mc_metal_european_call_native`): `1.531 ms`
- Down-and-out breadth check: CPU `17.267 ms`, Metal `0.663 ms`
- Fixed-strike lookback CPU breadth is live (`16.832 ms`) with explicit QuantLib comparison reporting.
- American put LSM CPU breadth is live (`270.324 ms`) with European-put lower-bound validation.
- Bermudan put custom-schedule LSM CPU breadth is live (`148.461 ms`) with explicit exercise-date metadata.
- Heston European CPU path simulation is live (`26.419 ms`) with Black-Scholes-limit validation.
- European pathwise Greeks are live (`1.513 ms`) with Delta abs error `0.000281` vs Black-Scholes.
- Measured planner choice accuracy: `100.0%`

Status: Rust currently leads the available CPU baselines for the tracked fair European workload.

Maintain lead plan:
- Keep the step-wise benchmark as the primary competitive claim.
- Keep RNG and loop hot path allocation-free.
- Keep breadth claims tied to the workloads we have actually benchmarked: European, Heston European, arithmetic Asian, down-and-out, lookback, American/Bermudan put LSM, basket, Greeks, and Gaussian UQ.
- Greek breadth is live with `26` bump-and-revalue estimates across current CPU workload families.
- QuantLib comparison lane is wired but currently unavailable in this environment; install QuantLib-Python to populate the selected-workload scoreboard.
- Expand competitor matrix to QuantLib exotic workloads and GPU baselines (JAX/CuPy/PyTorch/CUDA-native) when packages and hardware are available.
- First randomized-QMC pricing surface is live via randomized Halton (`79.766 ms`), but it is currently a quality-first pricing path rather than a speed leader.
- Latin hypercube pricing is live (`64.832 ms`) as the first non-QMC structured-sampling breadth path.
- Scrambled Sobol pricing is live (`81.164 ms`) as the stronger QMC breadth path.
- Scrambled Sobol with Brownian bridge pricing is live (`105.168 ms`) for path construction experiments.
- QMC generation scoreboard is live: Rust scrambled Sobol generation `78.900 ms`, SciPy scrambled Sobol generation `130.970 ms` (`1.66x` Rust/SciPy speedup).
- Arithmetic Asian MLMC is live (`4.499 ms`) with adaptive tolerance planning as the first multilevel CPU reference path.
- Arithmetic Asian MLQMC is live (`6.072 ms`) with replicated scrambling and adaptive tolerance planning.
- Gaussian UQ benchmark is live: Latin hypercube `2.115 ms`, abs error `0.000039` vs pseudorandom abs error `0.006344`.
- Basket-call QMC breadth is live: scrambled Sobol basket pricing `7.456 ms`, Latin-hypercube stderr ratio vs pseudorandom `0.997`.
- European realized-error study is live against Black-Scholes: Latin hypercube abs-error ratio vs pseudorandom `0.021`, scrambled Sobol ratio `0.129`, Sobol Brownian-bridge ratio `0.001`.
- Preserve the randomized-QMC quality gain (`stderr_ratio_vs_standard = 0.411`) while optimizing sequence generation and path construction.
- Preserve the Latin-hypercube quality gain (`stderr_ratio_vs_standard = 0.410`) while benchmarking it across more workload families.
- Preserve the Sobol Brownian-bridge quality gain (`stderr_ratio_vs_standard = 0.411`) while optimizing its current runtime overhead.
- Track arithmetic Asian MLMC quality (`stderr_ratio_vs_standard = 2.013`) and calibrate tolerance defaults before claiming it as a default winner.
- Preserve arithmetic Asian replicated MLQMC quality (`stderr_ratio_vs_standard = 0.418`) while reducing its runtime overhead and increasing replicate coverage.
- Preserve the specialized terminal-distribution fast path (`0.618 ms`) as a separate optimization track.
- Improve planner calibration beyond the current measured accuracy of `100.0%` as workload breadth increases.
