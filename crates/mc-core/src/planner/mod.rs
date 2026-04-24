use mc_schema::{Expr, SimulationSpec};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlannerMode {
    Safe,
    Balanced,
    Aggressive,
    Explain,
}

impl Default for PlannerMode {
    fn default() -> Self {
        Self::Balanced
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackendId {
    CpuNative,
    NvidiaCuda,
    AppleMetal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackendPreference {
    Auto,
    CpuNative,
    NvidiaCuda,
    AppleMetal,
}

impl BackendPreference {
    fn to_backend(self) -> Option<BackendId> {
        match self {
            Self::Auto => None,
            Self::CpuNative => Some(BackendId::CpuNative),
            Self::NvidiaCuda => Some(BackendId::NvidiaCuda),
            Self::AppleMetal => Some(BackendId::AppleMetal),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunConfig {
    pub n_paths: usize,
    pub n_steps: usize,
    pub planner_mode: PlannerMode,
    pub backend_preference: BackendPreference,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            n_paths: 100_000,
            n_steps: 252,
            planner_mode: PlannerMode::Balanced,
            backend_preference: BackendPreference::Auto,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedRunConfig {
    pub n_paths: usize,
    pub n_steps: usize,
    pub planner_mode: PlannerMode,
    pub backend_preference: BackendPreference,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SupportLevel {
    Supported,
    SupportedWithFallbacks,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendSupportReport {
    pub backend: BackendId,
    pub support_level: SupportLevel,
    pub reason: Option<String>,
}

impl BackendSupportReport {
    pub fn supported(backend: BackendId) -> Self {
        Self {
            backend,
            support_level: SupportLevel::Supported,
            reason: None,
        }
    }

    pub fn unsupported(backend: BackendId, reason: impl Into<String>) -> Self {
        Self {
            backend,
            support_level: SupportLevel::Unsupported,
            reason: Some(reason.into()),
        }
    }

    fn is_supported(&self) -> bool {
        matches!(
            self.support_level,
            SupportLevel::Supported | SupportLevel::SupportedWithFallbacks
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FeatureSummary {
    pub random_variable_count: usize,
    pub state_variable_count: usize,
    pub step_count: usize,
    pub observation_count: usize,
    pub reduction_count: usize,
    pub conditional_expression_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RejectedBackend {
    pub backend: BackendId,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendDecisionReport {
    pub selected_backend: BackendId,
    pub reasons: Vec<String>,
    pub rejected_backends: Vec<RejectedBackend>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub backend: BackendId,
    pub planner_mode: PlannerMode,
    pub n_paths: usize,
    pub n_steps: usize,
    pub features: FeatureSummary,
    pub decision_report: BackendDecisionReport,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlannerError {
    #[error("run config invalid: {0}")]
    InvalidRunConfig(String),
    #[error("requested backend {requested:?} is unsupported")]
    RequestedBackendUnsupported { requested: BackendId },
    #[error("no supported backend available")]
    NoSupportedBackend,
}

pub fn normalize_run_config(run_config: RunConfig) -> Result<NormalizedRunConfig, PlannerError> {
    if run_config.n_paths == 0 {
        return Err(PlannerError::InvalidRunConfig(
            "n_paths must be > 0".to_string(),
        ));
    }

    if run_config.n_steps == 0 {
        return Err(PlannerError::InvalidRunConfig(
            "n_steps must be > 0".to_string(),
        ));
    }

    Ok(NormalizedRunConfig {
        n_paths: run_config.n_paths,
        n_steps: run_config.n_steps,
        planner_mode: run_config.planner_mode,
        backend_preference: run_config.backend_preference,
    })
}

pub fn extract_features(spec: &SimulationSpec) -> FeatureSummary {
    let mut conditional_expression_count = 0;

    for state in &spec.state_variables {
        conditional_expression_count += count_conditionals(&state.init);
    }

    for step in &spec.steps {
        for update in &step.updates {
            conditional_expression_count += count_conditionals(&update.expr);
        }
    }

    for observation in &spec.observations {
        conditional_expression_count += count_conditionals(&observation.expr);
    }

    FeatureSummary {
        random_variable_count: spec.random_variables.len(),
        state_variable_count: spec.state_variables.len(),
        step_count: spec.steps.len(),
        observation_count: spec.observations.len(),
        reduction_count: spec.reductions.len(),
        conditional_expression_count,
    }
}

fn count_conditionals(expr: &Expr) -> usize {
    match expr {
        Expr::Literal { .. } => 0,
        Expr::ParameterRef { .. } => 0,
        Expr::StateRef { .. } => 0,
        Expr::RandomRef { .. } => 0,
        Expr::UnaryOp { arg, .. } => count_conditionals(arg),
        Expr::BinaryOp { op, lhs, rhs } => {
            let is_conditional = matches!(op.as_str(), "gt" | "ge" | "lt" | "le" | "eq" | "ne");
            let base = if is_conditional { 1 } else { 0 };
            base + count_conditionals(lhs) + count_conditionals(rhs)
        }
        Expr::Call { fn_name, args } => {
            let is_conditional = matches!(fn_name.as_str(), "if_else" | "where");
            let base = if is_conditional { 1 } else { 0 };
            base + args.iter().map(count_conditionals).sum::<usize>()
        }
    }
}

pub fn plan_execution(
    spec: &SimulationSpec,
    run_config: RunConfig,
    backend_support: &[BackendSupportReport],
) -> Result<ExecutionPlan, PlannerError> {
    let normalized = normalize_run_config(run_config)?;
    let features = extract_features(spec);

    if let Some(requested) = normalized.backend_preference.to_backend() {
        return plan_with_requested_backend(normalized, features, requested, backend_support);
    }

    plan_with_auto_backend(normalized, features, backend_support)
}

pub fn explain_execution_plan(plan: &ExecutionPlan) -> String {
    let mut lines = Vec::new();
    lines.push(format!("selected_backend={:?}", plan.backend));
    lines.push(format!(
        "planner_mode={:?} workload=n_paths:{} n_steps:{}",
        plan.planner_mode, plan.n_paths, plan.n_steps
    ));

    if !plan.decision_report.reasons.is_empty() {
        lines.push(format!(
            "reasons={}",
            plan.decision_report.reasons.join("; ")
        ));
    }

    if !plan.decision_report.rejected_backends.is_empty() {
        let rejected = plan
            .decision_report
            .rejected_backends
            .iter()
            .map(|entry| format!("{:?}: {}", entry.backend, entry.reason))
            .collect::<Vec<_>>()
            .join(" | ");
        lines.push(format!("rejected={rejected}"));
    }

    lines.join("\n")
}

fn plan_with_requested_backend(
    normalized: NormalizedRunConfig,
    features: FeatureSummary,
    requested: BackendId,
    backend_support: &[BackendSupportReport],
) -> Result<ExecutionPlan, PlannerError> {
    let requested_report = backend_support.iter().find(|r| r.backend == requested);

    let Some(report) = requested_report else {
        return Err(PlannerError::RequestedBackendUnsupported { requested });
    };

    if !report.is_supported() {
        return Err(PlannerError::RequestedBackendUnsupported { requested });
    }

    let rejected_backends = backend_support
        .iter()
        .filter(|report| report.backend != requested)
        .map(|report| RejectedBackend {
            backend: report.backend,
            reason: report
                .reason
                .clone()
                .unwrap_or_else(|| "lower priority than explicit preference".to_string()),
        })
        .collect::<Vec<_>>();

    Ok(ExecutionPlan {
        backend: requested,
        planner_mode: normalized.planner_mode,
        n_paths: normalized.n_paths,
        n_steps: normalized.n_steps,
        features,
        decision_report: BackendDecisionReport {
            selected_backend: requested,
            reasons: vec!["selected by explicit backend preference".to_string()],
            rejected_backends,
        },
    })
}

fn plan_with_auto_backend(
    normalized: NormalizedRunConfig,
    features: FeatureSummary,
    backend_support: &[BackendSupportReport],
) -> Result<ExecutionPlan, PlannerError> {
    let priority = backend_priority(&normalized, &features);
    let mut rejected = Vec::new();

    for backend in priority {
        let report = backend_support.iter().find(|r| r.backend == backend);
        match report {
            Some(r) if r.is_supported() => {
                let mut reasons = Vec::new();
                reasons.push("selected by auto backend policy".to_string());
                reasons.push(match backend {
                    BackendId::CpuNative => {
                        if normalized.n_paths < 100_000 {
                            "small workload favored CPU to avoid launch overhead".to_string()
                        } else {
                            "conditional-heavy workload favored CPU".to_string()
                        }
                    }
                    BackendId::NvidiaCuda => {
                        "large parallel workload favored NVIDIA CUDA".to_string()
                    }
                    BackendId::AppleMetal => {
                        if apple_metal_sweet_spot(&normalized, &features) {
                            "benchmark-calibrated Apple Metal policy favored native Apple GPU execution".to_string()
                        } else {
                            "large parallel workload favored Apple Metal".to_string()
                        }
                    }
                });

                return Ok(ExecutionPlan {
                    backend,
                    planner_mode: normalized.planner_mode,
                    n_paths: normalized.n_paths,
                    n_steps: normalized.n_steps,
                    features,
                    decision_report: BackendDecisionReport {
                        selected_backend: backend,
                        reasons,
                        rejected_backends: rejected,
                    },
                });
            }
            Some(r) => rejected.push(RejectedBackend {
                backend,
                reason: r
                    .reason
                    .clone()
                    .unwrap_or_else(|| "backend reported unsupported".to_string()),
            }),
            None => rejected.push(RejectedBackend {
                backend,
                reason: "backend support report not provided".to_string(),
            }),
        }
    }

    Err(PlannerError::NoSupportedBackend)
}

fn backend_priority(normalized: &NormalizedRunConfig, features: &FeatureSummary) -> Vec<BackendId> {
    const SMALL_WORK_THRESHOLD: usize = 4_000_000;
    const GPU_WORK_THRESHOLD: usize = 40_000_000;
    const GPU_MIN_PATHS: usize = 200_000;
    const GPU_MIN_STEPS: usize = 32;

    let total_work = normalized.n_paths.saturating_mul(normalized.n_steps);
    let prefers_cpu = features.conditional_expression_count > 0
        || normalized.n_paths < 100_000
        || total_work < SMALL_WORK_THRESHOLD
        || normalized.n_steps < GPU_MIN_STEPS;

    if prefers_cpu {
        vec![
            BackendId::CpuNative,
            BackendId::NvidiaCuda,
            BackendId::AppleMetal,
        ]
    } else if normalized.n_paths >= GPU_MIN_PATHS && total_work >= GPU_WORK_THRESHOLD {
        vec![
            BackendId::NvidiaCuda,
            BackendId::AppleMetal,
            BackendId::CpuNative,
        ]
    } else if apple_metal_sweet_spot(normalized, features) {
        vec![
            BackendId::AppleMetal,
            BackendId::CpuNative,
            BackendId::NvidiaCuda,
        ]
    } else {
        vec![
            BackendId::CpuNative,
            BackendId::NvidiaCuda,
            BackendId::AppleMetal,
        ]
    }
}

fn apple_metal_sweet_spot(normalized: &NormalizedRunConfig, features: &FeatureSummary) -> bool {
    const APPLE_METAL_MIN_PATHS: usize = 75_000;
    const APPLE_METAL_MIN_STEPS: usize = 32;
    const APPLE_METAL_WORK_THRESHOLD: usize = 6_000_000;

    let total_work = normalized.n_paths.saturating_mul(normalized.n_steps);
    features.conditional_expression_count == 0
        && normalized.n_paths >= APPLE_METAL_MIN_PATHS
        && normalized.n_steps >= APPLE_METAL_MIN_STEPS
        && total_work >= APPLE_METAL_WORK_THRESHOLD
}
