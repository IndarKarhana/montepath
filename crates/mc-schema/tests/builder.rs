use mc_schema::{
    validate_simulation_spec, AxisKind, Expr, ObservationSpec, RandomVarSpec, ReductionSpec,
    SimulationSpecBuilder, StateUpdate, StateVarSpec, StepSpec,
};

#[test]
fn simulation_spec_builder_creates_valid_minimal_spec() {
    let spec = SimulationSpecBuilder::new("builder_case", "0.1.0")
        .axis("path", AxisKind::Runtime, None, true, false)
        .axis("step", AxisKind::Runtime, None, false, true)
        .parameter("s0", "float64")
        .random_variable(RandomVarSpec {
            name: "z".to_string(),
            distribution: "normal".to_string(),
            dtype: "float32".to_string(),
            axes: vec!["step".to_string()],
        })
        .state_variable(StateVarSpec {
            name: "price".to_string(),
            dtype: "float32".to_string(),
            init: Expr::ParameterRef {
                value: "s0".to_string(),
            },
        })
        .step(StepSpec {
            name: "advance".to_string(),
            axis: "step".to_string(),
            updates: vec![StateUpdate {
                target: "price".to_string(),
                expr: Expr::StateRef {
                    value: "price".to_string(),
                },
            }],
        })
        .observation(ObservationSpec {
            name: "payoff".to_string(),
            expr: Expr::StateRef {
                value: "price".to_string(),
            },
        })
        .reduction(ReductionSpec {
            name: "expected_payoff".to_string(),
            op: "mean".to_string(),
            source: "payoff".to_string(),
            axes: vec!["path".to_string()],
        })
        .build();

    let diagnostics = validate_simulation_spec(&spec);
    assert!(
        diagnostics.is_empty(),
        "expected no diagnostics, got: {diagnostics:?}"
    );
}

#[test]
fn validation_errors_include_suggestions() {
    let spec = SimulationSpecBuilder::new("bad", "0.1.0")
        .axis("step", AxisKind::Runtime, None, false, true)
        .parameter("s0", "float64")
        .state_variable(StateVarSpec {
            name: "price".to_string(),
            dtype: "float32".to_string(),
            init: Expr::ParameterRef {
                value: "missing".to_string(),
            },
        })
        .step(StepSpec {
            name: "advance".to_string(),
            axis: "unknown_axis".to_string(),
            updates: vec![StateUpdate {
                target: "unknown_state".to_string(),
                expr: Expr::StateRef {
                    value: "unknown_state".to_string(),
                },
            }],
        })
        .build();

    let diagnostics = validate_simulation_spec(&spec);
    assert!(!diagnostics.is_empty());
    assert!(diagnostics.iter().all(|d| d.suggestion.is_some()));
}
