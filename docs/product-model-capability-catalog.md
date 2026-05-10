# Product And Model Capability Catalog

This is the human-readable companion to `docs/product-model-capability-catalog.json`.
The JSON file is the machine-readable source for agents and CI tests.

## Scope

Phase 2 is about selected QuantLib competitiveness, not full QuantLib breadth.
The current library is strongest on focused Monte Carlo path workloads where the
execution semantics are explicit and reproducible.

## Current Workload Matrix

| Workload | Model | CPU | Apple Metal | CUDA | Trusted Reference | QuantLib Lane |
| --- | --- | --- | --- | --- | --- | --- |
| European call | GBM / Black-Scholes | supported | supported for current GBM path family | staged, not native execution | Black-Scholes price and Greeks | `mc_cpu_european_call_quantlib` |
| Arithmetic Asian call | GBM | supported | supported for current GBM path family | staged, not native execution | no trusted fixture yet | none |
| Down-and-out call | GBM | supported | supported for current GBM path family | staged, not native execution | no trusted fixture yet | none |
| Fixed-strike lookback call | GBM | supported | not yet native | staged, not native execution | QuantLib competitor lane only | `mc_cpu_lookback_call_quantlib` |
| American put | GBM / Black-Scholes | supported CPU reference: Longstaff-Schwartz | not yet native | staged, not native execution | European put lower-bound reference; external LSM comparisons pending | none |
| Two-asset basket call | correlated GBM | supported | not yet native | staged, not native execution | no trusted fixture yet | none |
| Heston European call | full-truncation Euler Heston | supported | not yet native | staged, not native execution | Black-Scholes limit fixture | `mc_cpu_heston_european_call_quantlib` |
| Gaussian UQ mean | independent standard normals | supported | not yet native | staged, not native execution | analytic mean | none |

## Greek Estimator Matrix

| Workload | Bump And Revalue | Pathwise | Likelihood Ratio | Reference Status |
| --- | --- | --- | --- | --- |
| European call GBM | supported: Delta, Vega, Rho, Theta | supported: Delta | supported: Delta | Black-Scholes Greek fixture |
| Arithmetic Asian call GBM | supported: Delta, Vega, Rho, Theta | unsupported | unsupported | no trusted fixture yet |
| Down-and-out call GBM | supported: Delta, Vega, Rho, Theta | unsupported | unsupported | no trusted fixture yet |
| Fixed-strike lookback call GBM | supported: Delta, Vega, Rho, Theta | unsupported | unsupported | QuantLib lane only |
| American put GBM | unsupported | unsupported | unsupported | LSM pricing only; Greeks not exposed yet |
| Two-asset basket call GBM | supported: Delta, Vega, Rho | unsupported | unsupported | no trusted fixture yet |
| Heston European call | supported: Delta | unsupported | unsupported | Black-Scholes limit fixture |
| Gaussian UQ mean | not applicable | not applicable | not applicable | analytic mean |

Unsupported estimators are explicit. They must stay visible to users and agents;
do not silently substitute bump-and-revalue when a caller asks for pathwise or
likelihood-ratio on unsupported workload families.

## Reference Fixture Policy

Reference fixtures live in `benchmarks/reference-fixtures.json`.

Current trusted references:

- Black-Scholes European-call price at spot `100`, strike `100`, rate `0.03`,
  volatility `0.2`, maturity `1.0`.
- Black-Scholes European-call Greeks for the same configuration.
- Heston Black-Scholes limit when vol-of-vol is zero and variance is constant.
- Gaussian UQ analytic mean for `z_0^2 + 0.5 z_1 + exp(0.1 z_2)`.

Explicit caveats:

- Arithmetic Asian, down-and-out, and two-asset basket workloads do not yet have
  committed analytic or high-precision fixtures for their current semantics.
- Lookback has a QuantLib Monte Carlo competitor lane, but no committed analytic
  fixture for the current discrete-monitoring setup.
- American put has CPU Longstaff-Schwartz execution, a European put lower-bound
  check, and estimator metadata, but no committed high-precision American
  reference grid yet.
- General Heston analytic comparison is delegated to the QuantLib lane when
  QuantLib-Python is installed; the trusted built-in fixture is the
  Black-Scholes limit.

## QuantLib Position

QuantLib remains broader on calendars, market conventions, curves, exercise
styles, and instrument families. Our Phase 2 claim is narrower:

- on selected Monte Carlo workloads, keep our runtime faster and more
  reproducible when benchmark rows are populated;
- keep missing QuantLib packages or missing instrument APIs explicit as
  unavailable benchmark rows;
- run a QuantLib-enabled CI benchmark profile so release artifacts can include
  populated QuantLib lanes where the installed QuantLib-Python build exposes
  the required APIs.
