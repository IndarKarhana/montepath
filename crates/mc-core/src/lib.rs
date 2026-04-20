//! Core runtime interfaces.

pub mod planner;
pub mod runtime;

pub use planner::{
    extract_features, normalize_run_config, plan_execution, BackendDecisionReport, BackendId,
    BackendPreference, BackendSupportReport, ExecutionPlan, FeatureSummary, NormalizedRunConfig,
    PlannerError, PlannerMode, RejectedBackend, RunConfig, SupportLevel,
};
pub use runtime::{
    european_call_price_mc_cpu, EuropeanCallConfig, EuropeanCallResult, MonteCarloRng,
};
