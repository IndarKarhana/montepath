# Market Landscape

This document tracks where the library stands against the main open-source Monte Carlo and numerical simulation ecosystems.

## Leader Categories

- `QuantLib`: the strongest open-source quantitative-finance framework for breadth of pricing models, market conventions, instruments, and production finance workflows.
- `NumPy` / `Numba`: the most common practical CPU baseline for Python Monte Carlo implementations and internal research code.
- `SciPy qmc`: one of the strongest mainstream open-source references for quasi-Monte Carlo engines such as scrambled Sobol and Latin hypercube sampling.
- `JAX`, `CuPy`, and `PyTorch`: the strongest general accelerator-first array stacks that users often reach for when building Monte Carlo on GPUs.

## Honest Position

- On the tracked fair CPU European-call workload, this library is currently ahead of the available in-repo NumPy and Numba baselines.
- On Apple Silicon for the tracked native Metal workload family, this library now has a real GPU execution advantage over its own fair CPU path.
- We are not yet in a position to claim leadership against `QuantLib`, `SciPy qmc`, `JAX`, `CuPy`, or `PyTorch` broadly.

## Where We Lead Today

- `Focused performance`: the tracked Rust CPU path and the current Apple Metal path are both fast on the narrow workload families we benchmark directly.
- `Deterministic backend architecture`: the library has a cleaner planner/backend/runtime separation than many ad hoc Monte Carlo implementations.
- `Agent friendliness`: the repo has stronger explainability and tool-surface discipline than most numerical libraries at this stage.

## Where We Still Trail

- `Model breadth`: QuantLib is far ahead in instruments, processes, term structures, and market conventions.
- `Sampling breadth`: SciPy is ahead in mainstream quasi-Monte Carlo surface area today.
- `Accelerator breadth`: JAX, CuPy, and PyTorch are ahead in mature GPU ecosystems and backend portability.
- `Cross-hardware evidence`: we still need native CUDA execution and broader benchmark coverage before making broad performance claims.

## What Must Happen To Compete More Seriously

- add broader simulation kernel families beyond the current GBM option family
- add randomized QMC and other stronger sampling techniques
- add native CUDA execution and benchmark it against accelerator-first libraries
- calibrate planner recommendations from measured backend winners across more workload classes
