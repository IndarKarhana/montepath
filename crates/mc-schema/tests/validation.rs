use std::collections::BTreeMap;

use mc_schema::{
    validate_simulation_spec, AxisKind, AxisSpec, Expr, ObservationSpec, ParameterSpec, RandomVarSpec,
    ReductionSpec, SimulationSpec, StateUpdate, StateVarSpec, StepSpec,
};

fn valid_spec() -> SimulationSpec {
    let mut axes = BTreeMap::new();
    axes.insert(
        "path".to_string(),
        AxisSpec {
            name: "path".to_string(),
            kind: AxisKind::Runtime,
            size: None,
            parallel: true,
            ordered: false,
        },
    );
    axes.insert(
        "step".to_string(),
        AxisSpec {
            name: "step".to_string(),
            kind: AxisKind::Runtime,
            size: None,
            parallel: false,
            ordered: true,
        },
    );

    SimulationSpec {
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        parameters: vec![ParameterSpec {
            name: "s0".to_string(),
            dtype: "float64".to_string(),
        }],
        axes,
        random_variables: vec![RandomVarSpec {
            name: "z".to_string(),
            distribution: "normal".to_string(),
            dtype: "float32".to_string(),
            axes: vec!["step".to_string()],
        }],
        state_variables: vec![StateVarSpec {
            name: "price".to_string(),
            dtype: "float32".to_string(),
            init: Expr::ParameterRef {
                value: "s0".to_string(),
            },
        }],
        steps: vec![StepSpec {
            name: "advance".to_string(),
            axis: "step".to_string(),
            updates: vec![StateUpdate {
                target: "price".to_string(),
                expr: Expr::StateRef {
                    value: "price".to_string(),
                },
            }],
        }],
        observations: vec![ObservationSpec {
            name: "payoff".to_string(),
            expr: Expr::StateRef {
                value: "price".to_string(),
            },
        }],
        reductions: vec![ReductionSpec {
            name: "expected_payoff".to_string(),
            op: "mean".to_string(),
            source: "payoff".to_string(),
            axes: vec!["path".to_string()],
        }],
    }
}

#[test]
fn valid_spec_has_no_errors() {
    let diagnostics = validate_simulation_spec(&valid_spec());
    assert!(diagnostics.is_empty(), "expected no diagnostics, got: {diagnostics:?}");
}

#[test]
fn unknown_reduction_axis_returns_error() {
    let mut spec = valid_spec();
    spec.reductions[0].axes = vec!["paths".to_string()];

    let diagnostics = validate_simulation_spec(&spec);
    assert!(diagnostics.iter().any(|d| d.code == "E_AXIS_UNKNOWN"));
}

#[test]
fn unknown_step_target_returns_error() {
    let mut spec = valid_spec();
    spec.steps[0].updates[0].target = "unknown_state".to_string();

    let diagnostics = validate_simulation_spec(&spec);
    assert!(diagnostics.iter().any(|d| d.code == "E_STATE_UNKNOWN"));
}

#[test]
fn duplicate_parameter_names_return_error() {
    let mut spec = valid_spec();
    spec.parameters.push(ParameterSpec {
        name: "s0".to_string(),
        dtype: "float64".to_string(),
    });

    let diagnostics = validate_simulation_spec(&spec);
    assert!(diagnostics.iter().any(|d| d.code == "E_DUP_PARAMETER"));
}

#[test]
fn unknown_param_reference_in_state_init_returns_error() {
    let mut spec = valid_spec();
    spec.state_variables[0].init = Expr::ParameterRef {
        value: "missing_param".to_string(),
    };

    let diagnostics = validate_simulation_spec(&spec);
    assert!(diagnostics.iter().any(|d| d.code == "E_PARAMETER_UNKNOWN"));
}
