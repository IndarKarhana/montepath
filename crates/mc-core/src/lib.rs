//! Core runtime interfaces.

pub mod backend;
pub mod planner;
pub mod runtime;

pub use backend::{
    builtin_backends, cuda_native_feature_enabled, estimate_gpu_bytes_per_path,
    metal_native_feature_enabled, plan_gpu_chunking, AppleMetalBackend, ArtifactExecutionMode,
    BackendError, BackendExecutionInput, BackendInfo, CompiledArtifact, CostEstimate,
    CpuNativeBackend, DeviceInfo, GpuChunkingConfig, GpuChunkingPlan, NativeArtifactMetadata,
    NvidiaCudaBackend, ReproSupport, RunOutput, RuntimeBackend, SupportReport,
};
pub use planner::{
    explain_execution_plan, extract_features, normalize_run_config, plan_execution,
    BackendDecisionReport, BackendId, BackendPreference, BackendSupportReport, ExecutionPlan,
    FeatureSummary, NormalizedRunConfig, PlannerError, PlannerMode, RejectedBackend, RunConfig,
    SupportLevel,
};
pub use runtime::{
    european_call_price_mc_cpu, european_call_price_mc_cpu_stepwise,
    european_call_price_mc_cpu_terminal, european_call_price_mc_cpu_with_method,
    EuropeanCallConfig, EuropeanCallMethod, EuropeanCallPricer, EuropeanCallResult, MonteCarloRng,
    MonteCarloTechnique,
};
