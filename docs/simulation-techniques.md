# Simulation Techniques Roadmap

This document tracks which Monte Carlo techniques the library supports now, which ones are next, and which newer directions are worth planning for.

## Goals

- improve estimator quality without silently changing target quantities
- add methods that are standard in serious Monte Carlo practice
- prepare the runtime for newer high-performance approaches that matter in modern UQ and quantitative finance

## Current Support

The public `monte_carlo_method_capabilities()` catalog exposes the current method support matrix in machine-readable form, including CPU-reference MLMC and MLQMC work.

### 1. Standard pseudorandom Monte Carlo

Status:

- supported now

Current implementation:

- deterministic PRNG-backed CPU execution
- fair step-wise path simulation
- specialized terminal-distribution fast path when explicitly applicable
- native Apple Metal execution for the current GBM option family

### 2. Antithetic variates

Status:

- supported now for the current European-call runtime family

Why it matters:

- it is a simple, well-established variance-reduction technique
- it improves estimator quality without changing the expectation being estimated
- it is often a strong default for symmetric shock-driven path simulation

### 3. Control variates

Status:

- supported now for the current workload families

Why it matters:

- it can deliver large variance reduction when a strong correlated control is available
- it is one of the highest-value techniques in production Monte Carlo pricing stacks
- it improves quality without requiring more simulated paths

Current implementation:

- uses discounted terminal stock as the control variate
- leverages the known expectation `E[e^{-rT} S_T] = S0`
- currently implemented for European, arithmetic Asian, and down-and-out call paths
- currently implemented on both CPU reference paths and the native Apple Metal path where supported

### 4. First randomized-QMC surface via randomized Halton

Status:

- supported now as a first breadth milestone
- partially optimized by using direct inverse-normal mapping rather than Box-Muller pairing
- still needs benchmark-driven tuning

Why it matters:

- it cleanly separates sampling choice from variance-reduction technique
- it gives us a real low-discrepancy architecture foothold instead of a placeholder roadmap bullet
- it creates a direct path toward stronger QMC families such as scrambled Sobol

Practical notes:

- the current implementation is deterministic and benchmarked
- it should still be treated as an architectural and estimator-quality milestone until release measurements show competitive runtime

### 5. Latin hypercube sampling

Status:

- supported now on CPU for the current European, arithmetic Asian, down-and-out, and two-asset basket workload families
- benchmarked for the tracked European step-wise workload

Why it matters:

- gives better marginal space coverage than plain pseudorandom sampling
- is useful for uncertainty-propagation and sensitivity-analysis style workloads
- broadens the sampling surface without coupling it to any one variance-reduction technique

Practical notes:

- implemented with deterministic per-dimension affine stratum permutations and seeded intra-stratum jitter
- native Metal currently falls back to CPU reference execution for Latin-hypercube requests
- it is useful breadth now, but still needs broader estimator-quality and runtime benchmarking before strong claims

## Near-Term Techniques

### 1. Scrambled Sobol / randomized quasi-Monte Carlo

Status:

- supported now on CPU through Owen-scrambled Sobol sampling
- Brownian-bridge path construction is available for step-wise GBM workloads
- native GPU structured sampling is not implemented yet

Why it matters:

- low-discrepancy sequences can materially improve convergence on many integration problems
- randomized or scrambled variants preserve statistical error estimation while improving space-filling behavior
- in practice this is a more important target than stopping at Halton

Practical notes:

- powers-of-two sample sizing matters for Sobol balance properties
- dimension management and path construction order matter for effectiveness
- use `structured_sampling_guidance_cpu` to surface Sobol power-of-two guidance, Halton high-dimensional caveats, and Latin-hypercube marginal-balance notes in agent-facing workflows
- use `diagnose_standard_normals_cpu` to check generated normal moments, finite values, tail frequency, and per-axis moment errors before treating a sampling configuration as healthy
- this belongs in both CPU and future GPU-oriented sampling plans

Primary sources:

- SciPy Sobol documentation: https://docs.scipy.org/doc/scipy/reference/generated/scipy.stats.qmc.Sobol.html
- Pierre L'Ecuyer and Art Owen references cited there, especially scrambling and randomized QMC references

### 2. Workload-general control variates

Status:

- partially implemented now
- still high-value for broader workloads with known analytic moments or approximations

Why it matters:

- often gives large variance reduction when a strong correlated control is available
- especially useful in option pricing and calibrated model families

Planned direction:

- keep the current discounted-terminal-stock control variate as the first specialized implementation
- later generalize through planner-selected auxiliary statistics and workload-specific analytic references

## Advanced High-Value Techniques

### 0. Smooth Gaussian Uncertainty Propagation

Status:

- supported now as a focused CPU benchmark through `gaussian_uncertainty_mean_cpu`
- tracked with analytic-mean absolute error across pseudorandom, randomized Halton, Latin hypercube, and scrambled Sobol sampling

Why it matters:

- gives QMC a non-option workload where realized error can be measured directly
- helps separate smooth uncertainty-propagation behavior from path-dependent option-pricing behavior

Practical notes:

- current benchmark is intentionally narrow and analytic-reference-backed
- use it as the first UQ quality signal, not as a general UQ modeling interface

### 0.25. European Realized-Error QMC Validation

Status:

- supported now through `compare_european_call_realized_error_cpu`
- benchmarked against the Black-Scholes analytic European-call reference
- tracked across randomized Halton, Latin hypercube, scrambled Sobol, and scrambled Sobol with Brownian bridge

Why it matters:

- standard-error ratios can be neutral even when realized error improves materially for one seed and workload
- analytic references let users and agents distinguish estimator uncertainty from actual miss distance to the target
- this is the first bridge from QMC health checks into benchmark-backed method recommendations

Practical notes:

- realized error is seed- and path-count-sensitive, so treat one benchmark as evidence, not a universal convergence theorem
- the Black-Scholes reference only covers the vanilla European GBM workload
- path-dependent and multi-asset payoffs still need analytic, semi-analytic, or high-precision reference fixtures

### 0.5. Two-Asset Basket Pricing

Status:

- supported now as a CPU reference workload through `basket_call_price_mc_cpu`
- benchmarked across pseudorandom, randomized Halton, Latin hypercube, and scrambled Sobol terminal sampling
- tracked with QMC pricing-quality comparisons against the pseudorandom baseline

Why it matters:

- adds the first explicit multi-asset pricing workload
- tests low-dimensional correlated sampling without Brownian-bridge path construction
- gives agents a clearer bridge from single-asset examples toward portfolio-style payoffs

Practical notes:

- current support is a two-asset terminal GBM basket call, not a general basket-product framework
- realized-error validation is still needed before claiming a QMC convergence win for basket options

### 1. Multilevel Monte Carlo

Status:

- supported now as a CPU reference path for arithmetic Asian calls
- benchmarked as the first multilevel foundation
- includes pilot-based level/path allocation tuning
- still needs adaptive reruns and broader workload support

Why it matters:

- one of the most important modern advances for simulation efficiency
- especially powerful when a hierarchy of discretizations exists
- directly relevant for SDE path simulation and future path-dependent workloads

Current implementation:

- `arithmetic_asian_call_price_mlmc_cpu`
- deterministic coupled fine/coarse Brownian increments
- explicit `ArithmeticAsianMlmcConfig` with base steps, level count, refinement factor, paths per level, seed, sampling, and scramble replicates
- `tune_arithmetic_asian_mlmc_allocation_cpu` for pilot-variance path allocation under a target step-update budget
- structured per-level output with means, variances, standard errors, path counts, and step-update cost

Practical notes:

- current support is CPU reference only
- barrier MLMC is intentionally deferred because discontinuous knock-out payoffs need separate coupling and smoothing analysis
- current path allocation can be explicit, pilot-budget tuned, or tolerance-planned with `solve_arithmetic_asian_mlmc_tolerance_cpu`

Primary sources:

- Mike Giles MLMC overview page: https://people.maths.ox.ac.uk/gilesm/mlmc.html
- original 2008 MLMC path simulation paper linked there
- Giles and Waterhouse multilevel quasi-Monte Carlo path simulation reference linked there

### 2. Multilevel randomized quasi-Monte Carlo

Status:

- supported now as a CPU reference path for arithmetic Asian calls using scrambled Sobol increments
- supports replicated scrambling through `scramble_replicates`
- still early because replicated scrambling and tolerance planning are available only for the arithmetic Asian CPU reference surface

Why it matters:

- combines two of the strongest efficiency ideas available for many workloads
- highly relevant for expensive nested or discretized simulation problems

Implementation note:

- current implementation shares the arithmetic Asian MLMC surface through `ArithmeticAsianMlmcConfig::sampling`
- use `SamplingMethod::ScrambledSobol` for the current MLQMC path
- use `scramble_replicates > 1` for replicate-based randomized-QMC error estimates
- use `solve_arithmetic_asian_mlmc_tolerance_cpu` when the desired input is an accuracy target rather than an explicit path count

## Emerging / Trending Directions To Track

These are promising, but should follow after the classical high-value techniques above are stable:

- multifidelity multilevel Monte Carlo
- learned or model-assisted control variates
- subset simulation / rare-event specialized methods
- Markov-chain RQMC for specific classes of sequential simulation

These matter, but they should not displace the core roadmap of:

1. standard MC done well
2. strong variance reduction
3. high-value RQMC, especially scrambled Sobol
4. MLMC
5. calibrated planner support

## Recommended Implementation Order

1. Keep standard, antithetic, control-variate, randomized Halton, Latin hypercube, and scrambled Sobol paths correct and benchmarked.
2. Continue optimizing structured sampling so Halton, Latin hypercube, and Sobol variants are not purely breadth features.
3. Expand randomized QMC quality and realized-error studies across more workload shapes.
4. Generalize control variates beyond the current discounted-terminal-stock pattern.
5. Calibrate arithmetic Asian MLMC and MLQMC tolerance defaults against measured variance and cost.
6. Broaden tolerance planning beyond the first arithmetic Asian reference path.
7. Extend MLMC beyond arithmetic Asian once coupling behavior is documented for each payoff family.
