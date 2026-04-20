//! Core runtime interfaces.

pub mod backend;
pub mod planner;
pub mod runtime;

pub use backend::{
    builtin_backends, estimate_gpu_bytes_per_path, plan_gpu_chunking, AppleMetalBackend,
    BackendError, BackendExecutionInput, BackendInfo, CompiledArtifact, CostEstimate,
    CpuNativeBackend, DeviceInfo, GpuChunkingConfig, GpuChunkingPlan, NvidiaCudaBackend,
    ReproSupport, RunOutput, RuntimeBackend, SupportReport,
};
pub use planner::{
    extract_features, normalize_run_config, plan_execution, BackendDecisionReport, BackendId,
    BackendPreference, BackendSupportReport, ExecutionPlan, FeatureSummary, NormalizedRunConfig,
    PlannerError, PlannerMode, RejectedBackend, RunConfig, SupportLevel,
};
pub use runtime::{
    european_call_price_mc_cpu, EuropeanCallConfig, EuropeanCallResult, MonteCarloRng,
};
