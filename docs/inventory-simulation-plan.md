# Inventory And Supply-Chain Simulation Plan

## 1. Objective

Build a production-grade inventory and supply-chain Monte Carlo domain in
MontePath that is:

- faster than strong NumPy and Numba baselines on targeted workloads
- numerically and operationally correct under explicitly documented semantics
- reproducible across repeated runs
- inspectable by planners, users, and AI agents
- suitable for policy comparison, sensitivity analysis, and optimization
- extensible to Apple Metal only when the workload shape and evidence justify it

This track is the first major proof that MontePath is a general simulation
runtime rather than only a quantitative-finance library.

## 2. Product Position

The first implementation is a **periodic-review discrete-time inventory
simulator**. It is not initially a universal discrete-event simulator.

That boundary is intentional:

- paths remain independently parallel
- periods remain ordered within each path
- state and event capacity remain bounded and planner-readable
- CPU execution can be optimized before introducing general event-scheduler
  overhead
- future Metal execution remains plausible

The implementation should pressure-test and eventually strengthen the general
`SimulationSpec` architecture, but the first fast path should be a typed,
domain-specific runtime.

## 3. Authoritative Period Semantics

Each path represents one possible operating history for one SKU under one
inventory policy.

For every period, operations occur in this exact order:

1. **Receive**
   - receive purchase orders scheduled for the current period
   - satisfy existing backorders before making units available as on-hand stock
   - enforce warehouse-capacity semantics
2. **Demand**
   - advance the demand regime when regime switching is enabled
   - sample non-negative demand from the active distribution
3. **Fulfill**
   - fulfill demand from on-hand stock
   - convert unfulfilled demand to backorders or lost sales according to the
     configured shortage policy
4. **Review**
   - calculate inventory position as:
     `on_hand + on_order - backorder`
   - evaluate the configured replenishment policy
5. **Order**
   - calculate desired quantity
   - enforce MOQ, case-pack, supplier-capacity, and warehouse-capacity rules in
     documented order
   - sample or apply lead time
   - schedule the purchase order arrival
6. **Record**
   - record state, service, stockout, ordering, holding, and shortage metrics

An order placed in period `t` cannot satisfy demand from period `t`.
`lead_time_periods = 0` means arrival at the receive stage of period `t + 1`.

## 4. Core Semantics Requiring Explicit Configuration

### 4.1 Shortage handling

Supported policies:

- `backorder`: unmet units remain owed and are fulfilled from future receipts
- `lost_sales`: unmet units are permanently lost

The selected policy must appear in every run manifest.

### 4.2 Demand units

The runtime initially supports continuous non-negative demand quantities.
Normal draws are clipped at zero.

Future integer-demand support must define rounding and count-distribution
semantics explicitly.

### 4.3 Service metrics

Initial definitions:

- `cycle_service_level`: fraction of observed periods with no unmet demand
- `fill_rate`: fulfilled units divided by demanded units
- `stockout_events`: observed periods with unmet demand

These definitions must never change silently.

### 4.4 Warm-up periods

Warm-up periods update operational state but do not contribute to reported
service or cost metrics. The manifest records both total and observed periods.

### 4.5 Constraint ordering

Order quantity constraints are applied in this order:

1. desired order-up-to quantity
2. minimum order quantity
3. case-pack rounding
4. supplier-capacity clipping
5. warehouse-capacity commitment clipping

The result must report when constraints prevented the desired order.

### 4.6 Cost accounting

Initial cost semantics are explicit and period-based:

- holding cost is charged on end-of-period on-hand units
- backorder cost is charged on end-of-period outstanding backorder units
- lost-sale cost is charged once on units lost in the current period
- fixed and variable ordering costs are charged when an order is placed
- warm-up activity changes state but contributes no reported cost

`total_cost` must equal holding cost plus shortage cost plus ordering cost.
Backorder and lost-sale cost parameters remain separate because they represent
different operational consequences.

### 4.7 Lead-time convention

`lead_time_periods` is the number of period boundaries between order placement
and receipt. Both zero and one therefore arrive at the next period's receive
stage; zero exists as an explicit "no additional delay" value but never enables
same-period fulfillment. Values greater than one arrive after that many period
boundaries.

## 5. Initial Public Rust Surface

The first CPU reference slice should expose:

- `InventoryDemandDistribution`
- `InventoryShortagePolicy`
- `InventoryPolicy`
- `InventoryConstraints`
- `InventoryConstraintEvents`
- `InventoryCostConfig`
- `InventorySimulationConfig`
- `InventoryPathResult`
- `InventoryMetricSummary`
- `InventorySimulationResult`
- `InventorySimulationSummary`
- `InventoryRunManifest`
- `InventorySimulationError`
- `InventoryValidationDiagnostic`
- `validate_inventory_config`
- `simulate_inventory_policy_cpu`

All public types must be serializable and documented in
`docs/function-catalog.md`.

The run manifest must preserve the complete validated input config and name
the period order, lead-time convention, warehouse-capacity convention, Normal
demand clipping, RNG stream mapping, and aggregate quantile method.

## 6. Initial Scope

### 6.1 Included in the first reference kernel

- one SKU per simulation
- periodic review
- `(s, S)` reorder-point/order-up-to policy
- deterministic or Normal non-negative demand
- fixed lead time
- backorder and lost-sales shortage modes
- multiple outstanding purchase orders through a bounded arrival schedule
- MOQ
- case-pack rounding
- supplier capacity per period
- warehouse capacity
- holding, shortage, fixed-order, and variable-order costs
- warm-up periods
- per-path results
- aggregate P5/P50/P95 and mean metric summaries
- deterministic seeded execution
- structured validation diagnostics

### 6.2 Deferred from the first reference kernel

- multi-SKU coupling
- shared supplier and warehouse capacity
- substitutions and bill-of-material dependencies
- random lead time
- Markov regime switching
- seasonal demand
- continuous-review policies
- external optimizer integration
- native Metal execution

Deferred capabilities must remain explicit in docs and agent responses.

## 7. Correctness And Validation Program

### 7.1 Deterministic fixtures

Required deterministic tests:

- zero demand preserves inventory and produces perfect service
- deterministic demand with no replenishment depletes inventory exactly
- fixed demand plus fixed lead time produces an auditable replenishment trace
- backorder receipts satisfy old backorders before new demand
- lost-sales mode never carries backorders
- MOQ is enforced
- case-pack rounding is enforced
- supplier capacity clips order quantity
- warehouse capacity prevents over-commitment
- warm-up periods are excluded from reported metrics

### 7.2 Invariant/property tests

Required invariants:

- all inventory quantities remain finite and non-negative where semantically
  required
- lost-sales mode always has zero ending backorder
- on-order equals the sum of outstanding scheduled arrivals
- fill rate remains in `[0, 1]`
- cycle service level remains in `[0, 1]`
- total cost equals the sum of component costs within floating-point tolerance
- identical configuration and seed produce identical results

### 7.3 External reference comparison

Create a transparent NumPy reference implementation with identical period
semantics. Compare path-level and aggregate outputs under fixed random inputs.

Do not claim correctness from stochastic agreement alone. Deterministic traces
and invariants are mandatory.

### 7.4 Analytic and formula references

Where assumptions match, add comparisons against:

- deterministic inventory-balance calculations
- newsvendor-style special cases
- classic safety-stock and reorder-point formulas

Formula references are validation aids, not replacements for simulation.

## 8. Performance Program

### 8.1 Performance principles

- keep per-path state compact
- reuse arrival-schedule buffers
- avoid allocation inside path/period loops
- use deterministic stream partitioning
- preserve exact path and summary results across CPU thread counts
- separate narrow optimized kernels from future general IR execution
- measure before claiming wins

### 8.2 Benchmark lanes

Required benchmark families:

- Rust CPU inventory reference kernel
- NumPy vectorized baseline
- Numba compiled-loop baseline
- Python scalar reference for correctness only
- policy sweep with common random numbers
- regime-switching workload after it lands
- random-lead-time workload after it lands
- future Metal lane only after native support exists

### 8.3 Initial benchmark shapes

Track at least:

- small: `1_000 paths x 52 periods`
- medium: `10_000 paths x 104 periods`
- large: `100_000 paths x 365 periods`
- policy sweep: `128 policies x 10_000 paths x 104 periods`

### 8.4 Performance gates
Before describing the inventory runtime as competitive:

- Rust must beat the available NumPy and Numba baselines on at least the
  medium targeted workload
- benchmark methodology must use identical semantics and inputs
- unavailable competitors remain explicit
- release-mode artifacts must include hardware and environment metadata
- policy comparison benchmarks must distinguish independent randomness from
  common-random-number execution

### 8.5 Committed release baseline

The committed release benchmark profile on the development Apple M2 machine
running macOS 26.2 reported:

- shape: `10,000 paths x 104 periods`
- methodology: `single_sku_periodic_review_fixed_lead_time`
- measured iterations: `3`
- Rust CPU runtime: `6.24 ms`
- NumPy runtime: `14.24 ms`
- Numba runtime: `29.26 ms`
- Rust speedup: `2.28x` over NumPy and `4.69x` over Numba
- Rust throughput: approximately `166.7 million path-period updates/second`
- benchmark metric: mean total cost `778.8608`
- Rust path and summary results are identical across tested one-thread and
  four-thread execution

These are committed release-mode measurements, not universal performance
guarantees. The competitor lanes use identical operational semantics but
independent library RNG streams.

## 9. Variance Reduction And Policy Comparison

### 9.1 Common random numbers

Common random numbers are the first required variance-reduction feature.

Policy comparisons must be able to reuse identical demand and lead-time
streams. Results should report paired differences and uncertainty for cost and
service metrics.

### 9.2 Antithetic variates

Antithetic demand paths are experimental until measured on:

- symmetric demand without hard constraints
- constrained ordering policies
- regime-switching demand

Do not enable by default without evidence.

### 9.3 Control variates

Control variates require a documented analytical or high-precision reference
with measured correlation. They remain research work until validated.

## 10. Policy Search And Optimization

Optimization is layered above simulation.

Responsibilities remain separated:

- `validate`: check model and policy feasibility
- `simulate`: execute one policy
- `compare`: compare policies using common random numbers
- `sweep`: execute a caller-provided policy set
- `frontier`: identify non-dominated cost/service policies
- `optimize`: generate and refine candidate policies
- `recommend`: apply an explicit decision policy to validated evidence

Initial search sequence:

1. seed candidates from formulas
2. generate a Latin-hypercube policy grid
3. run common-random-number comparisons
4. construct a cost/service efficient frontier
5. refine around feasible frontier candidates

Recommendations must include objective, constraints, confidence, caveats, and
reproduction metadata.

## 11. Markov Regime Demand

Regime switching is a major second milestone.

Required typed configuration:

- ordered regime ids
- initial regime or initial probability vector
- transition matrix
- one demand distribution per regime

Validation requirements:

- transition matrix is square
- every probability is finite and in `[0, 1]`
- every row sums to one within explicit tolerance
- every regime has a demand distribution
- initial state/probabilities are valid

Execution requirements:

- regime transition and demand RNG streams are separately identifiable
- path results can optionally report regime occupancy
- manifests record transition semantics and sampling order

## 12. Random Lead Time

Random lead time is modeled per purchase order, not per path.

Required semantics:

- lead time is converted to a non-negative integer period count
- conversion/rounding policy is explicit
- maximum supported lead time is bounded for the optimized runtime
- overflow beyond configured event capacity is an explicit error

A bounded ring buffer or arrival histogram is preferred over a dynamic heap for
the optimized CPU and future Metal paths.

## 13. Metal Readiness Criteria

Do not implement Metal merely because the CPU kernel exists.

Metal work begins only when:

- CPU semantics and reference fixtures are stable
- benchmark evidence shows meaningful scale
- dynamic state can be represented with bounded arrays
- branch divergence and memory-layout risks are measured
- cross-backend statistical tolerance policy is documented

Likely first Metal candidate:

- one SKU
- fixed lead time
- deterministic or Normal demand
- `(s, S)` policy
- bounded arrival schedule
- no shared capacities

Unsupported Metal inventory behavior must fail explicitly without CPU fallback
when Metal is requested directly.

## 14. Python API Plan

Initial Python surface:

```python
from montepath import InventorySimulationConfig, simulate_inventory_policy

result = simulate_inventory_policy(
    InventorySimulationConfig(
        n_paths=10_000,
        n_periods=104,
        initial_on_hand=500.0,
        reorder_point=300.0,
        order_up_to=700.0,
    ),
    backend="auto",
)
```

The Python API must expose:

- typed immutable configs
- structured diagnostics
- explicit backend selection
- result manifests
- explanation and reproduction helpers
- no silent unsupported fallback

## 15. Agent And MCP Plan

Available bounded tools:

- `montepath.inventory.validate`
- `montepath.inventory.simulate`

Planned expansion tools:

- `montepath.inventory.compare`
- `montepath.inventory.sensitivity`
- `montepath.inventory.frontier`
- `montepath.inventory.optimize`
- `montepath.inventory.recommend`

Tool progression:

1. validation
2. bounded reference execution
3. comparison and sensitivity
4. frontier reporting
5. optimization
6. recommendation

MCP execution limits must cap:

- paths
- periods
- policy count
- returned path-result count
- total estimated path-period operations

## 16. Implementation Milestones

### Milestone A: Semantic contract and CPU reference foundation

- [x] Review external feature request against architecture.
- [x] Define authoritative period semantics and initial scope.
- [x] Add typed Rust inventory configs, diagnostics, path results, and aggregate
  result schemas.
- [x] Add first deterministic, constraint, reproducibility, and invariant tests.
- [x] Implement first single-SKU fixed-lead-time CPU reference kernel.
- [x] Add function-catalog entries and capability documentation.
- [x] Add transparent external Python reference and deterministic native
  cross-language comparisons.
- [x] Add per-period trace output behind an explicit bounded diagnostic option.

### Milestone B: Python and native bridge

- [x] Expose typed Python configs and results.
- [x] Add Rust-backed native extension validation and execution functions.
- [x] Integrate production backend capability and selection surfaces.
- [x] Add installed wheel/source smoke coverage.
- [x] Add examples and reproduction manifests.

### Milestone C: Benchmark-backed competitiveness

- [x] Add first Rust CPU periodic-review benchmark row and positive-runtime
  smoke gate.
- [x] Add NumPy, Numba, and scalar Python baselines with identical semantics.
- [x] Add benchmark result rows using existing competitor environment manifests.
- [x] Add conditional performance gates and improvement-plan generation.
- [x] Add reusable lead-time buffers and deterministic CPU path partitioning.
- [x] Publish release-mode inventory benchmark artifacts.

### Milestone D: Policy comparison and search

- [ ] Add common-random-number policy comparison.
- [ ] Add paired-difference uncertainty reporting.
- [ ] Add policy sweeps and efficient-frontier output.
- [ ] Add Latin-hypercube candidate generation.
- [ ] Add local refinement with explicit stopping rules.

### Milestone E: Operational realism

- [ ] Add Markov regime-switching demand.
- [ ] Add random lead times.
- [ ] Add seasonal and disruption scenarios.
- [ ] Add bounded outstanding-order/event capacity diagnostics.
- [ ] Expand validation fixtures and external reference comparisons.

### Milestone F: Agent-native inventory workflows

- [x] Add inventory validation and simulation MCP tools.
- [ ] Add comparison and sensitivity tools.
- [ ] Add frontier, optimization, and recommendation tools.
- [x] Add schemas, limits, failure policy, and compatibility tests.

### Milestone G: Metal evaluation and implementation

- [ ] Benchmark and profile CPU inventory workload shapes on Apple hardware.
- [ ] Define bounded Metal-compatible state layout.
- [ ] Implement first strict native Metal inventory kernel if evidence supports
  it.
- [ ] Add CPU-vs-Metal correctness, reproducibility, and release benchmark
  artifacts.

## 17. Definition Of Done

The inventory domain is production-ready for a supported workload only when:

1. period semantics are documented and versioned
2. implementation and invariant tests pass
3. deterministic fixtures and external reference comparisons pass
4. performance claims are backed by release artifacts
5. public Rust/Python/API contracts are documented
6. unsupported behavior is explicit
7. agent tools have schemas, limits, and failure policy
8. roadmap and capability catalogs reflect real status

## 18. Immediate Execution Order

1. Build common-random-number policy comparison.
2. Add comparison, sensitivity, and efficient-frontier agent tools.
3. Add Markov regime-switching demand and random per-order lead times.
4. Evaluate a bounded native Metal layout against the mature CPU path.
