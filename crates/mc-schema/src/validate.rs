use std::collections::{BTreeSet, HashSet};

use crate::{check_schema_compatibility, Diagnostic, Expr, SimulationSpec};

pub fn validate_simulation_spec(spec: &SimulationSpec) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let parameter_names: HashSet<&str> = spec.parameters.iter().map(|p| p.name.as_str()).collect();
    let state_names: HashSet<&str> = spec
        .state_variables
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    let random_names: HashSet<&str> = spec
        .random_variables
        .iter()
        .map(|r| r.name.as_str())
        .collect();
    let observation_names: HashSet<&str> =
        spec.observations.iter().map(|o| o.name.as_str()).collect();
    let axis_names: HashSet<&str> = spec.axes.keys().map(String::as_str).collect();

    diagnostics.extend(check_schema_version(spec));
    diagnostics.extend(check_duplicate_parameters(spec));
    diagnostics.extend(check_duplicate_declared_names(spec));
    diagnostics.extend(check_axis_consistency(spec));
    diagnostics.extend(check_random_var_axes(spec, &axis_names));
    diagnostics.extend(check_step_targets(spec, &state_names));
    diagnostics.extend(check_reduction_axes(spec, &axis_names));
    diagnostics.extend(check_reduction_sources(
        spec,
        &state_names,
        &observation_names,
    ));
    diagnostics.extend(check_expression_refs(
        spec,
        &parameter_names,
        &state_names,
        &random_names,
    ));

    diagnostics
}

fn check_schema_version(spec: &SimulationSpec) -> Vec<Diagnostic> {
    let report = check_schema_compatibility(&spec.schema_version);
    if report.supported {
        return Vec::new();
    }

    vec![Diagnostic::error(
        "E_SCHEMA_VERSION_UNSUPPORTED",
        format!(
            "schema version '{}' is not supported: {}",
            report.found_version,
            report
                .reason
                .unwrap_or_else(|| "unknown compatibility issue".to_string())
        ),
        "schema_version",
    )
    .with_suggestion("Use schema_version '0.1' for this runtime.")]
}

fn check_duplicate_parameters(spec: &SimulationSpec) -> Vec<Diagnostic> {
    let mut seen = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for (idx, param) in spec.parameters.iter().enumerate() {
        if !seen.insert(param.name.as_str()) {
            diagnostics.push(
                Diagnostic::error(
                    "E_DUP_PARAMETER",
                    format!("duplicate parameter '{}' found", param.name),
                    format!("parameters[{idx}].name"),
                )
                .with_suggestion("Rename one of the duplicated parameter declarations."),
            );
        }
    }

    diagnostics
}

fn check_duplicate_declared_names(spec: &SimulationSpec) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(check_duplicate_names(
        "E_DUP_RANDOM",
        spec.random_variables.iter().map(|v| v.name.as_str()),
        "random_variables",
    ));
    diagnostics.extend(check_duplicate_names(
        "E_DUP_STATE",
        spec.state_variables.iter().map(|v| v.name.as_str()),
        "state_variables",
    ));
    diagnostics.extend(check_duplicate_names(
        "E_DUP_OBSERVATION",
        spec.observations.iter().map(|v| v.name.as_str()),
        "observations",
    ));

    diagnostics
}

fn check_duplicate_names<'a>(
    code: &str,
    names: impl Iterator<Item = &'a str>,
    location_prefix: &str,
) -> Vec<Diagnostic> {
    let mut seen = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for (idx, name) in names.enumerate() {
        if !seen.insert(name) {
            diagnostics.push(
                Diagnostic::error(
                    code,
                    format!("duplicate declaration '{}' found", name),
                    format!("{location_prefix}[{idx}].name"),
                )
                .with_suggestion("Use unique names for all declared entities in this section."),
            );
        }
    }

    diagnostics
}

fn check_axis_consistency(spec: &SimulationSpec) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (axis_key, axis_spec) in &spec.axes {
        if axis_key != &axis_spec.name {
            diagnostics.push(Diagnostic::error(
                "E_AXIS_NAME_MISMATCH",
                format!(
                    "axis map key '{}' does not match axis name '{}'",
                    axis_key, axis_spec.name
                ),
                format!("axes.{axis_key}.name"),
            ));
        }
    }

    diagnostics
}

fn check_random_var_axes(spec: &SimulationSpec, axis_names: &HashSet<&str>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (rv_idx, rv) in spec.random_variables.iter().enumerate() {
        for (axis_idx, axis) in rv.axes.iter().enumerate() {
            if !axis_names.contains(axis.as_str()) {
                diagnostics.push(
                    Diagnostic::error(
                        "E_AXIS_UNKNOWN",
                        format!(
                            "random variable '{}' references unknown axis '{}'",
                            rv.name, axis
                        ),
                        format!("random_variables[{rv_idx}].axes[{axis_idx}]"),
                    )
                    .with_suggestion("Declare the axis in `axes` or correct the axis name."),
                );
            }
        }
    }

    diagnostics
}

fn check_step_targets(spec: &SimulationSpec, state_names: &HashSet<&str>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (step_idx, step) in spec.steps.iter().enumerate() {
        if !spec.axes.contains_key(&step.axis) {
            diagnostics.push(
                Diagnostic::error(
                    "E_AXIS_UNKNOWN",
                    format!(
                        "step '{}' references unknown axis '{}'",
                        step.name, step.axis
                    ),
                    format!("steps[{step_idx}].axis"),
                )
                .with_suggestion("Set `step.axis` to a declared axis such as `step`."),
            );
        }

        for (update_idx, update) in step.updates.iter().enumerate() {
            if !state_names.contains(update.target.as_str()) {
                diagnostics.push(
                    Diagnostic::error(
                        "E_STATE_UNKNOWN",
                        format!(
                            "step '{}' update target '{}' is not a declared state variable",
                            step.name, update.target
                        ),
                        format!("steps[{step_idx}].updates[{update_idx}].target"),
                    )
                    .with_suggestion("Add the missing state variable or fix the target name."),
                );
            }
        }
    }

    diagnostics
}

fn check_reduction_axes(spec: &SimulationSpec, axis_names: &HashSet<&str>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (reduction_idx, reduction) in spec.reductions.iter().enumerate() {
        for (axis_idx, axis) in reduction.axes.iter().enumerate() {
            if !axis_names.contains(axis.as_str()) {
                diagnostics.push(
                    Diagnostic::error(
                        "E_AXIS_UNKNOWN",
                        format!(
                            "reduction '{}' references unknown axis '{}'",
                            reduction.name, axis
                        ),
                        format!("reductions[{reduction_idx}].axes[{axis_idx}]"),
                    )
                    .with_suggestion("Use only axis names declared in `axes`."),
                );
            }
        }
    }

    diagnostics
}

fn check_reduction_sources(
    spec: &SimulationSpec,
    state_names: &HashSet<&str>,
    observation_names: &HashSet<&str>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (idx, reduction) in spec.reductions.iter().enumerate() {
        let source = reduction.source.as_str();
        if !(state_names.contains(source) || observation_names.contains(source)) {
            diagnostics.push(
                Diagnostic::error(
                    "E_REDUCTION_SOURCE_UNKNOWN",
                    format!(
                        "reduction '{}' source '{}' is not a known state or observation",
                        reduction.name, reduction.source
                    ),
                    format!("reductions[{idx}].source"),
                )
                .with_suggestion("Set reduction source to a declared observation or state name."),
            );
        }
    }

    diagnostics
}

fn check_expression_refs(
    spec: &SimulationSpec,
    parameter_names: &HashSet<&str>,
    state_names: &HashSet<&str>,
    random_names: &HashSet<&str>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (idx, state) in spec.state_variables.iter().enumerate() {
        validate_expr(
            &state.init,
            parameter_names,
            state_names,
            random_names,
            &format!("state_variables[{idx}].init"),
            &mut diagnostics,
        );
    }

    for (step_idx, step) in spec.steps.iter().enumerate() {
        for (update_idx, update) in step.updates.iter().enumerate() {
            validate_expr(
                &update.expr,
                parameter_names,
                state_names,
                random_names,
                &format!("steps[{step_idx}].updates[{update_idx}].expr"),
                &mut diagnostics,
            );
        }
    }

    for (idx, observation) in spec.observations.iter().enumerate() {
        validate_expr(
            &observation.expr,
            parameter_names,
            state_names,
            random_names,
            &format!("observations[{idx}].expr"),
            &mut diagnostics,
        );
    }

    diagnostics
}

fn validate_expr(
    expr: &Expr,
    parameter_names: &HashSet<&str>,
    state_names: &HashSet<&str>,
    random_names: &HashSet<&str>,
    location: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match expr {
        Expr::Literal { .. } => {}
        Expr::ParameterRef { value } => {
            if !parameter_names.contains(value.as_str()) {
                diagnostics.push(
                    Diagnostic::error(
                        "E_PARAMETER_UNKNOWN",
                        format!("expression references unknown parameter '{}'", value),
                        location.to_string(),
                    )
                    .with_suggestion("Declare the parameter or fix the parameter reference."),
                );
            }
        }
        Expr::StateRef { value } => {
            if !state_names.contains(value.as_str()) {
                diagnostics.push(
                    Diagnostic::error(
                        "E_STATE_UNKNOWN",
                        format!("expression references unknown state '{}'", value),
                        location.to_string(),
                    )
                    .with_suggestion("Declare the state variable or fix the state reference."),
                );
            }
        }
        Expr::RandomRef { value } => {
            if !random_names.contains(value.as_str()) {
                diagnostics.push(
                    Diagnostic::error(
                        "E_RANDOM_UNKNOWN",
                        format!("expression references unknown random variable '{}'", value),
                        location.to_string(),
                    )
                    .with_suggestion("Declare the random variable or fix the random reference."),
                );
            }
        }
        Expr::UnaryOp { arg, .. } => {
            validate_expr(
                arg,
                parameter_names,
                state_names,
                random_names,
                location,
                diagnostics,
            );
        }
        Expr::BinaryOp { lhs, rhs, .. } => {
            validate_expr(
                lhs,
                parameter_names,
                state_names,
                random_names,
                location,
                diagnostics,
            );
            validate_expr(
                rhs,
                parameter_names,
                state_names,
                random_names,
                location,
                diagnostics,
            );
        }
        Expr::Call { args, .. } => {
            for arg in args {
                validate_expr(
                    arg,
                    parameter_names,
                    state_names,
                    random_names,
                    location,
                    diagnostics,
                );
            }
        }
    }
}
