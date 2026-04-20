# Simulation Schema Draft

## 1. Purpose

This document defines the machine-readable schema for the Monte Carlo runtime. The schema must satisfy four goals:

- provide an ergonomic Python-facing model
- support validation and static analysis
- serve as the internal bridge to the planner and backends
- be serializable for agent, audit, and remote execution workflows

The schema is intentionally narrower than general Python execution. It is designed for simulation workloads that can be represented as explicit state, randomness, transitions, observations, and reductions.

## 2. Design Goals

1. Stable and explicit

The schema should not depend on opaque Python callbacks for its core meaning.

2. Typed and shape-aware

Every value should have a declared or inferable dtype and shape.

3. Execution-oriented

The schema should expose the information needed for planning:

- parallel axes
- data dependencies
- state lifetimes
- randomness requirements
- reduction structure

4. Agent-readable

The schema should be easy to inspect, serialize, and transform programmatically.

## 3. Top-Level Objects

The core public objects are:

- `SimulationSpec`
- `RunConfig`
- `ExecutionPlan`
- `RunManifest`
- `ResultBundle`

This document focuses mainly on `SimulationSpec` and `RunConfig`.

## 4. Schema Representation Strategy

We should support three related forms:

1. Python typed objects
   - dataclasses or Pydantic-like models in the Python API
2. Rust-native structs
   - canonical internal representation
3. JSON schema representation
   - stable interchange format for serialization and agent workflows

Recommended rule:

- Python objects and JSON map closely to the Rust structs
- runtime lowering and optimization details belong in `ExecutionPlan`, not `SimulationSpec`

## 5. Core Schema Concepts

A simulation is defined by:

- parameters
- dimensions / axes
- random variables
- state variables
- transition steps
- observations
- reductions
- constraints
- metadata

The runtime should model paths, steps, batches, and parameter sets explicitly.

## 6. Canonical Axes

The schema needs explicit semantics for iteration axes.

Canonical axis names:

- `path`
- `step`
- `batch`
- `scenario`
- `asset`
- `factor`

Rules:

- a simulation may define custom named axes
- some axes can be dynamic at runtime, such as `path`
- the planner needs to know which axes are parallelizable
- reductions must explicitly state which axes they reduce across

Example:

```json
{
  "axes": {
    "path": {"kind": "runtime", "size": null, "parallel": true},
    "step": {"kind": "runtime", "size": null, "parallel": false},
    "asset": {"kind": "static", "size": 4, "parallel": true}
  }
}
```

## 7. `SimulationSpec`

### 7.1 Required fields

- `name`
- `version`
- `parameters`
- `axes`
- `random_variables`
- `state_variables`
- `steps`
- `observations`
- `reductions`

### 7.2 Optional fields

- `constraints`
- `reproducibility`
- `hints`
- `metadata`
- `extensions`

### 7.3 Suggested structure

```json
{
  "name": "barrier_option",
  "version": "0.1.0",
  "parameters": [],
  "axes": {},
  "random_variables": [],
  "state_variables": [],
  "steps": [],
  "observations": [],
  "reductions": [],
  "constraints": {},
  "reproducibility": {},
  "hints": {},
  "metadata": {}
}
```

## 8. `ParameterSpec`

Parameters are immutable values provided by the user at simulation build time or run time.

Fields:

- `name`: string
- `dtype`: enum
- `shape`: shape expression
- `default`: optional literal or array literal
- `required`: boolean
- `bounds`: optional numeric bounds
- `description`: optional string
- `tags`: optional string list

Rules:

- parameters are read-only inside the simulation
- parameters may be scalar or tensor-shaped
- parameter shapes may depend on static axes only in v1

Example:

```json
{
  "name": "sigma",
  "dtype": "float64",
  "shape": [],
  "required": true,
  "bounds": {"min": 0.0}
}
```

## 9. `AxisSpec`

Fields:

- `name`
- `kind`: `static` | `runtime`
- `size`: integer or `null`
- `parallel`: boolean
- `ordered`: boolean
- `description`: optional string

Rules:

- ordered axes imply sequential semantics for operations depending on prior values
- `step` is typically ordered
- `path` is typically parallel and unordered

## 10. `RandomVarSpec`

Random variables define sources of stochasticity.

Fields:

- `name`
- `distribution`
- `dtype`
- `shape`
- `axes`
- `parameters`
- `sampling_mode`: `iid` | `qmc` | `stratified`
- `rng_stream`: optional string
- `description`

Rules:

- random variables are pure inputs to the simulation graph
- shape and axes must be explicit
- backend lowering decides how streams are materialized
- distribution support is versioned by backend capability

Example:

```json
{
  "name": "z",
  "distribution": "normal",
  "dtype": "float32",
  "shape": ["step"],
  "axes": ["step"],
  "parameters": {"mean": 0.0, "std": 1.0},
  "sampling_mode": "iid"
}
```

## 11. `StateVarSpec`

State variables represent mutable simulation state across ordered transitions.

Fields:

- `name`
- `dtype`
- `shape`
- `axes`
- `init`
- `persistent`: boolean
- `storage`: `full_path` | `rolling` | `ephemeral`
- `description`

Definitions:

- `full_path`: retain every step value
- `rolling`: retain only current and possibly previous step state
- `ephemeral`: used within a step and not persisted

Rules:

- the planner may downgrade storage where semantics allow, but only with explicit equivalence
- ordered transitions can only mutate declared state variables

Example:

```json
{
  "name": "price",
  "dtype": "float32",
  "shape": [],
  "axes": [],
  "init": {"kind": "parameter_ref", "value": "S0"},
  "persistent": true,
  "storage": "rolling"
}
```

## 12. Expressions

Expressions appear in:

- state initialization
- transition updates
- observations
- reductions
- conditions

### 12.1 Expression requirements

Expressions should be representable as a typed AST, not only as strings.

Supported kinds in v1:

- literal
- parameter reference
- state reference
- random reference
- unary op
- binary op
- n-ary math function
- comparison
- conditional expression
- reduction helper within allowed contexts

### 12.2 Expression example

```json
{
  "kind": "binary_op",
  "op": "mul",
  "lhs": {"kind": "state_ref", "value": "price"},
  "rhs": {
    "kind": "call",
    "fn": "exp",
    "args": [
      {
        "kind": "binary_op",
        "op": "add",
        "lhs": {"kind": "parameter_ref", "value": "drift_term"},
        "rhs": {"kind": "random_ref", "value": "z_t"}
      }
    ]
  }
}
```

### 12.3 Expression restrictions for v1

- no arbitrary user-defined Python functions inside hot-path expressions
- no side effects
- no recursion
- no dynamic shape mutation
- limited conditional support, designed to keep backend lowering tractable

## 13. `StepSpec`

A step is an ordered state transition.

Fields:

- `name`
- `axis`: usually `step`
- `index_symbol`: optional string
- `updates`: list of state assignments
- `condition`: optional expression
- `annotations`: optional planner hints

Each update contains:

- `target`: state variable name
- `expr`: expression AST

Rules:

- steps are evaluated in declaration order unless we later support dependency-based reordering
- each update target must reference an existing state variable
- intra-step dependency rules should be explicit in implementation

Example:

```json
{
  "name": "advance_price",
  "axis": "step",
  "updates": [
    {
      "target": "price",
      "expr": {"kind": "call", "fn": "gbm_step", "args": []}
    }
  ]
}
```

## 14. `ObservationSpec`

Observations derive values from the final or intermediate simulation state.

Fields:

- `name`
- `dtype`
- `shape`
- `axes`
- `expr`
- `when`: `per_step` | `final` | `custom_condition`
- `condition`: optional expression

Rules:

- observations are read-only derived values
- observations may feed reductions
- observations should not mutate state

Example:

```json
{
  "name": "payoff",
  "dtype": "float32",
  "shape": [],
  "axes": [],
  "expr": {
    "kind": "call",
    "fn": "max",
    "args": [
      {"kind": "binary_op", "op": "sub", "lhs": {"kind": "state_ref", "value": "price"}, "rhs": {"kind": "parameter_ref", "value": "K"}},
      {"kind": "literal", "value": 0.0}
    ]
  },
  "when": "final"
}
```

## 15. `ReductionSpec`

Reductions define the final statistics returned by a run.

Fields:

- `name`
- `op`: `mean` | `sum` | `variance` | `std` | `quantile` | `min` | `max` | `custom_builtin`
- `source`: observation or state reference
- `axes`: list of axes reduced over
- `options`: optional parameters such as quantile level
- `dtype`
- `return_shape`

Rules:

- reduction axes must exist on the source
- the planner can choose hierarchical reduction implementation but not alter semantic meaning

Example:

```json
{
  "name": "expected_payoff",
  "op": "mean",
  "source": "payoff",
  "axes": ["path"],
  "dtype": "float64",
  "return_shape": []
}
```

## 16. `ConstraintSpec`

Constraints shape execution legality and user expectations.

Fields:

- `allowed_backends`
- `forbidden_backends`
- `max_memory_mb`
- `required_precision`
- `determinism_level`
- `allow_chunking`
- `allow_approximate_reduction`

Rules:

- constraints are user-level declarations
- planner must respect hard constraints or fail clearly

## 17. `ReproducibilitySpec`

Fields:

- `seed`
- `rng_family`
- `reproducibility_tier`
- `stable_chunking`: boolean
- `capture_manifest`: boolean

Recommended reproducibility tiers:

- `same_backend_exact`
- `same_backend_deterministic`
- `cross_backend_statistical`
- `best_effort`

## 18. `HintSpec`

Hints are advisory, not mandatory.

Examples:

- preferred backend
- preferred precision
- expected arithmetic intensity
- branchiness estimate
- suggest rolling state storage
- suggest Sobol sampling

Hints should influence planning but never silently override constraints.

## 19. `RunConfig`

`RunConfig` supplies runtime execution settings.

Fields:

- `n_paths`
- `n_steps`
- `backend`
- `device`
- `planner_mode`
- `precision`
- `seed`
- `rng`
- `chunk_size`
- `max_memory_mb`
- `compile_cache`
- `profile`
- `explain`

Rules:

- `RunConfig` may fill in runtime sizes for `runtime` axes
- runtime configuration does not change simulation semantics
- runtime configuration may constrain planner choices

## 20. Validation Rules

Validation happens in four stages:

1. Schema validation
   - missing fields
   - invalid enum values
   - malformed shapes
2. Semantic validation
   - unknown references
   - invalid dtype combinations
   - illegal axis usage
3. Planner validation
   - unsupported features for selected or available backends
   - impossible memory limits
4. Run validation
   - device availability
   - compilation feasibility

Validation should return structured diagnostics, not only string errors.

## 21. Diagnostics Model

Recommended diagnostic format:

- `code`
- `severity`: `error` | `warning` | `info`
- `message`
- `location`: path into schema
- `suggestion`: optional string

Example:

```json
{
  "code": "E_AXIS_UNKNOWN",
  "severity": "error",
  "message": "Reduction references unknown axis 'paths'",
  "location": "reductions[0].axes[0]",
  "suggestion": "Did you mean 'path'?"
}
```

## 22. Versioning Strategy

The schema must be versioned independently from the package version where needed.

Recommendations:

- include `schema_version` in serialized top-level objects
- prefer additive evolution
- use explicit migration logic for breaking changes

## 23. Extension Strategy

We should leave room for future features without destabilizing the core schema.

Recommended extension points:

- `metadata`: user-owned annotations
- `extensions`: namespaced experimental fields

Example:

```json
{
  "extensions": {
    "mc.experimental.branch_profile_hint": {
      "estimated_divergence": "medium"
    }
  }
}
```

## 24. Example End-to-End Schema

```json
{
  "schema_version": "0.1",
  "name": "european_call",
  "version": "0.1.0",
  "parameters": [
    {"name": "S0", "dtype": "float64", "shape": [], "required": true},
    {"name": "K", "dtype": "float64", "shape": [], "required": true},
    {"name": "r", "dtype": "float64", "shape": [], "required": true},
    {"name": "sigma", "dtype": "float64", "shape": [], "required": true},
    {"name": "T", "dtype": "float64", "shape": [], "required": true}
  ],
  "axes": {
    "path": {"name": "path", "kind": "runtime", "size": null, "parallel": true, "ordered": false},
    "step": {"name": "step", "kind": "runtime", "size": null, "parallel": false, "ordered": true}
  },
  "random_variables": [
    {
      "name": "z",
      "distribution": "normal",
      "dtype": "float32",
      "shape": ["step"],
      "axes": ["step"],
      "parameters": {"mean": 0.0, "std": 1.0},
      "sampling_mode": "iid"
    }
  ],
  "state_variables": [
    {
      "name": "price",
      "dtype": "float32",
      "shape": [],
      "axes": [],
      "init": {"kind": "parameter_ref", "value": "S0"},
      "persistent": true,
      "storage": "rolling"
    }
  ],
  "steps": [
    {
      "name": "advance_price",
      "axis": "step",
      "updates": [
        {
          "target": "price",
          "expr": {
            "kind": "call",
            "fn": "gbm_euler_step",
            "args": [
              {"kind": "state_ref", "value": "price"},
              {"kind": "random_ref", "value": "z"},
              {"kind": "parameter_ref", "value": "r"},
              {"kind": "parameter_ref", "value": "sigma"},
              {"kind": "parameter_ref", "value": "T"}
            ]
          }
        }
      ]
    }
  ],
  "observations": [
    {
      "name": "payoff",
      "dtype": "float32",
      "shape": [],
      "axes": [],
      "when": "final",
      "expr": {
        "kind": "call",
        "fn": "max",
        "args": [
          {"kind": "binary_op", "op": "sub", "lhs": {"kind": "state_ref", "value": "price"}, "rhs": {"kind": "parameter_ref", "value": "K"}},
          {"kind": "literal", "value": 0.0}
        ]
      }
    }
  ],
  "reductions": [
    {
      "name": "expected_payoff",
      "op": "mean",
      "source": "payoff",
      "axes": ["path"],
      "dtype": "float64",
      "return_shape": []
    }
  ],
  "reproducibility": {
    "seed": 42,
    "rng_family": "philox",
    "reproducibility_tier": "same_backend_deterministic",
    "capture_manifest": true
  }
}
```

## 25. Recommended v1 Boundaries

We should explicitly keep these out of the first schema version:

- user-defined recursive functions
- arbitrary host callbacks during execution
- mutable parameter values
- dynamic axis creation during runtime
- irregular graph mutation mid-simulation
- distributed scheduling hints

## 26. Open Questions

These should be resolved before implementation starts:

1. Should the Python layer use dataclasses, Pydantic, or a custom typed model layer?
2. How much of expression parsing should be string-based vs builder-based in v1?
3. Do we want builtin domain operators like `gbm_step`, or only primitive math ops in the canonical IR?
4. Should quantile reduction be exact, approximate, or backend-dependent in v1?
5. How strict should schema compatibility guarantees be before `1.0`?

## 27. Recommendation

Use a schema that is:

- explicit
- typed
- serializable
- planner-oriented
- narrow enough to compile efficiently across CPU, CUDA, and Metal

That gives us a solid base for both the runtime and agent workflows.
