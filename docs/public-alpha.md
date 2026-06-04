# Public Alpha Positioning

`montepath` is ready to share as a public alpha for users and agents who want
an inspectable Monte Carlo runtime with benchmark-backed CPU execution and
production preflight surfaces for supported installed-package workflows.

It should not be described as a general production replacement for mature
scientific or quantitative libraries yet.

## Recommended Public Description

`montepath` is an alpha-stage, agent-native Monte Carlo runtime with:

- Rust-backed CPU execution
- Python-first configs and result objects
- reproducibility manifests
- benchmark artifacts
- structured diagnostics
- MCP-compatible agent tools

The project focuses on honest, explainable execution rather than broad
unsupported magic.

## Good Uses Today

- agent-driven Monte Carlo planning and execution
- reproducible CPU Monte Carlo experiments
- option-pricing and uncertainty-propagation examples covered by the current
  workload catalog
- benchmark-audited method comparison
- local MCP tools for LLM agents
- production capability checks before CPU-native or agent execution
- bounded inventory-policy simulation through Python, native CPU, and dedicated
  MCP tools
- research workflows where explicit manifests and caveats matter

## Not Yet A Fit For

- production promises across arbitrary Monte Carlo models
- native CUDA acceleration
- native Metal inventory acceleration
- broad QuantLib-style market convention coverage
- unsupported Python callbacks or arbitrary control-flow compilation
- claims of universal QMC or MLMC superiority

## Public Release Boundaries

When publishing or announcing, use alpha language:

- say `alpha`, `preview`, or `experimental`
- tie performance claims to committed benchmark artifacts
- state that native CUDA is deferred
- state that Apple Metal is available from supported macOS wheels for the
  documented GBM option family, while unsupported workloads such as inventory
  fail explicitly
- state that public APIs may evolve before `1.0`

Avoid:

- "production ready" without limiting the claim to supported CPU-native
  installed-package surfaces
- "fastest Monte Carlo library"
- "drop-in replacement for NumPy/JAX/QuantLib"
- "CUDA support" without saying staged/deferred

## Upcoming Features

Near-term:

- expand installed-package smoke coverage across more Python versions
- add planner bindings so Python recommendations can call the Rust planner
  directly
- improve public examples and notebooks around the native CPU extension and
  MCP tools

Accelerator-focused later version:

- native CUDA launch and reductions
- deterministic CUDA RNG stream partitioning
- CUDA hardware CI and release artifacts
- Python accelerator profiles for supported hardware
- broader JAX/CuPy/PyTorch competitor artifacts on real GPU runners

Research/runtime roadmap:

- broader MLMC/MLQMC calibration beyond arithmetic Asian calls
- more analytic realized-error studies where references exist
- wider non-option uncertainty-propagation examples
- more native Metal workload families beyond the currently packaged GBM option
  bridge
- inventory policy comparison, random lead times, regime-switching demand, and
  evidence-gated Metal evaluation
