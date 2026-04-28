use std::collections::BTreeMap;

use mc_core::{
    explain_execution_plan, extract_features, normalize_run_config, plan_execution,
    recommend_method, BackendId, BackendPreference, BackendSupportReport,
    MethodRecommendationRequest, MonteCarloTechnique, PlannerError, PlannerMode, RunConfig,
    SamplingMethod, WorkloadFamily,
};
use mc_schema::{
    AxisKind, AxisSpec, Expr, ObservationSpec, ParameterSpec, RandomVarSpec, ReductionSpec,
    SimulationSpec, StateUpdate, StateVarSpec, StepSpec,
};

fn sample_spec(with_conditional: bool) -> SimulationSpec {
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

    let update_expr = if with_conditional {
        Expr::BinaryOp {
            op: "gt".to_string(),
            lhs: Box::new(Expr::StateRef {
                value: "price".to_string(),
            }),
            rhs: Box::new(Expr::Literal { value: 0.0 }),
        }
    } else {
        Expr::StateRef {
            value: "price".to_string(),
        }
    };

    SimulationSpec {
        schema_version: "0.1".to_string(),
        name: "planner_case".to_string(),
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
                expr: update_expr,
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

fn support_all() -> Vec<BackendSupportReport> {
    vec![
        BackendSupportReport::supported(BackendId::CpuNative),
        BackendSupportReport::supported(BackendId::NvidiaCuda),
        BackendSupportReport::supported(BackendId::AppleMetal),
    ]
}

#[test]
fn normalize_run_config_rejects_zero_paths() {
    let config = RunConfig {
        n_paths: 0,
        n_steps: 10,
        planner_mode: PlannerMode::Balanced,
        backend_preference: BackendPreference::Auto,
    };

    let err = normalize_run_config(config).expect_err("expected invalid run config error");
    assert!(matches!(err, PlannerError::InvalidRunConfig(_)));
}

#[test]
fn method_recommendation_prefers_fast_control_variate_by_default() {
    let recommendation = recommend_method(MethodRecommendationRequest {
        workload_family: WorkloadFamily::EuropeanCall,
        n_paths: 100_000,
        n_steps: 64,
        prefer_accuracy: false,
        allow_slower_structured_sampling: false,
    });

    assert_eq!(recommendation.sampling, SamplingMethod::Pseudorandom);
    assert_eq!(
        recommendation.technique,
        MonteCarloTechnique::ControlVariate
    );
}

#[test]
fn method_recommendation_prefers_sobol_bridge_when_accuracy_is_prioritized() {
    let recommendation = recommend_method(MethodRecommendationRequest {
        workload_family: WorkloadFamily::DownAndOutCall,
        n_paths: 100_000,
        n_steps: 64,
        prefer_accuracy: true,
        allow_slower_structured_sampling: true,
    });

    assert_eq!(
        recommendation.sampling,
        SamplingMethod::ScrambledSobolBrownianBridge
    );
    assert_eq!(
        recommendation.technique,
        MonteCarloTechnique::ControlVariate
    );
    assert!(recommendation
        .caveats
        .iter()
        .any(|caveat| caveat.contains("CPU-reference only")));
}

#[test]
fn method_recommendation_cites_realized_error_for_european_structured_accuracy() {
    let recommendation = recommend_method(MethodRecommendationRequest {
        workload_family: WorkloadFamily::EuropeanCall,
        n_paths: 100_000,
        n_steps: 64,
        prefer_accuracy: true,
        allow_slower_structured_sampling: true,
    });

    assert!(recommendation
        .reasons
        .iter()
        .any(|reason| reason.contains("realized-error")));
    assert!(recommendation
        .caveats
        .iter()
        .any(|caveat| caveat.contains("Black-Scholes")));
}

#[test]
fn method_recommendation_prefers_mlqmc_for_arithmetic_asian_accuracy_with_structured_sampling() {
    let recommendation = recommend_method(MethodRecommendationRequest {
        workload_family: WorkloadFamily::ArithmeticAsianCall,
        n_paths: 100_000,
        n_steps: 64,
        prefer_accuracy: true,
        allow_slower_structured_sampling: true,
    });

    assert_eq!(recommendation.method_id, "multilevel_randomized_qmc");
    assert_eq!(recommendation.sampling, SamplingMethod::ScrambledSobol);
    assert_eq!(recommendation.technique, MonteCarloTechnique::Standard);
}

#[test]
fn method_recommendation_prefers_mlmc_for_path_dependent_accuracy_when_structured_sampling_is_not_allowed(
) {
    let recommendation = recommend_method(MethodRecommendationRequest {
        workload_family: WorkloadFamily::ArithmeticAsianCall,
        n_paths: 100_000,
        n_steps: 64,
        prefer_accuracy: true,
        allow_slower_structured_sampling: false,
    });

    assert_eq!(recommendation.method_id, "multilevel_monte_carlo");
    assert_eq!(recommendation.sampling, SamplingMethod::Pseudorandom);
    assert_eq!(recommendation.technique, MonteCarloTechnique::Standard);
    assert!(recommendation
        .reasons
        .iter()
        .any(|reason| reason.contains("path-dependent")));
}

#[test]
fn extract_features_counts_conditionals() {
    let features = extract_features(&sample_spec(true));
    assert_eq!(features.conditional_expression_count, 1);
    assert_eq!(features.random_variable_count, 1);
    assert_eq!(features.state_variable_count, 1);
}

#[test]
fn auto_planner_prefers_cpu_for_small_workloads() {
    let spec = sample_spec(false);
    let plan = plan_execution(
        &spec,
        RunConfig {
            n_paths: 50_000,
            n_steps: 100,
            planner_mode: PlannerMode::Balanced,
            backend_preference: BackendPreference::Auto,
        },
        &support_all(),
    )
    .expect("expected plan to be created");

    assert_eq!(plan.backend, BackendId::CpuNative);
}

#[test]
fn auto_planner_prefers_nvidia_for_large_parallel_workloads() {
    let spec = sample_spec(false);
    let plan = plan_execution(
        &spec,
        RunConfig {
            n_paths: 1_000_000,
            n_steps: 100,
            planner_mode: PlannerMode::Balanced,
            backend_preference: BackendPreference::Auto,
        },
        &support_all(),
    )
    .expect("expected plan to be created");

    assert_eq!(plan.backend, BackendId::NvidiaCuda);
}

#[test]
fn auto_planner_prefers_cpu_when_steps_are_too_small_for_gpu() {
    let spec = sample_spec(false);
    let plan = plan_execution(
        &spec,
        RunConfig {
            n_paths: 1_000_000,
            n_steps: 8,
            planner_mode: PlannerMode::Balanced,
            backend_preference: BackendPreference::Auto,
        },
        &support_all(),
    )
    .expect("expected plan to be created");

    assert_eq!(plan.backend, BackendId::CpuNative);
}

#[test]
fn auto_planner_prefers_apple_for_benchmarked_medium_large_workloads() {
    let spec = sample_spec(false);
    let plan = plan_execution(
        &spec,
        RunConfig {
            n_paths: 100_000,
            n_steps: 64,
            planner_mode: PlannerMode::Balanced,
            backend_preference: BackendPreference::Auto,
        },
        &support_all(),
    )
    .expect("expected plan to be created");

    assert_eq!(plan.backend, BackendId::AppleMetal);
    assert!(plan
        .decision_report
        .reasons
        .iter()
        .any(|reason| reason.contains("Apple Metal")));
}

#[test]
fn explicit_backend_request_fails_when_unsupported() {
    let spec = sample_spec(false);
    let err = plan_execution(
        &spec,
        RunConfig {
            n_paths: 1_000_000,
            n_steps: 100,
            planner_mode: PlannerMode::Balanced,
            backend_preference: BackendPreference::NvidiaCuda,
        },
        &[
            BackendSupportReport::supported(BackendId::CpuNative),
            BackendSupportReport::unsupported(BackendId::NvidiaCuda, "missing CUDA runtime"),
        ],
    )
    .expect_err("expected explicit unsupported backend to fail");

    assert!(matches!(
        err,
        PlannerError::RequestedBackendUnsupported {
            requested: BackendId::NvidiaCuda
        }
    ));
}

#[test]
fn auto_planner_falls_back_to_apple_when_nvidia_is_unavailable() {
    let spec = sample_spec(false);
    let plan = plan_execution(
        &spec,
        RunConfig {
            n_paths: 1_000_000,
            n_steps: 100,
            planner_mode: PlannerMode::Balanced,
            backend_preference: BackendPreference::Auto,
        },
        &[
            BackendSupportReport::supported(BackendId::CpuNative),
            BackendSupportReport::unsupported(BackendId::NvidiaCuda, "cuda unavailable"),
            BackendSupportReport::supported(BackendId::AppleMetal),
        ],
    )
    .expect("expected planner to fall back to Apple Metal");

    assert_eq!(plan.backend, BackendId::AppleMetal);
}

#[test]
fn explain_execution_plan_includes_backend_and_reasons() {
    let spec = sample_spec(false);
    let plan = plan_execution(
        &spec,
        RunConfig {
            n_paths: 1_000_000,
            n_steps: 100,
            planner_mode: PlannerMode::Balanced,
            backend_preference: BackendPreference::Auto,
        },
        &support_all(),
    )
    .expect("expected plan to be created");

    let explanation = explain_execution_plan(&plan);
    assert!(explanation.contains("selected_backend=NvidiaCuda"));
    assert!(explanation.contains("reasons="));
}
