//! Core runtime interfaces.

pub mod backend;
pub mod planner;
pub mod runtime;

pub use backend::{
    builtin_backends, cuda_native_feature_enabled, estimate_gpu_bytes_per_path,
    metal_native_feature_enabled, plan_gpu_chunking, AppleMetalBackend, ArtifactExecutionMode,
    BackendError, BackendExecutionInput, BackendInfo, CompiledArtifact, CostEstimate,
    CpuNativeBackend, DeviceInfo, GpuBufferBinding, GpuBufferDirection, GpuChunkingConfig,
    GpuChunkingPlan, GpuKernelContract, GpuLaunchDimensions, GpuScalarBinding, GpuValueType,
    NativeArtifactMetadata, NvidiaCudaBackend, ReproSupport, RunOutput, RuntimeBackend,
    SupportReport,
};
pub use planner::{
    explain_execution_plan, extract_features, normalize_run_config, plan_execution,
    recommend_method, BackendDecisionReport, BackendId, BackendPreference, BackendSupportReport,
    ExecutionPlan, FeatureSummary, MethodRecommendation, MethodRecommendationRequest,
    NormalizedRunConfig, PlannerError, PlannerMode, RejectedBackend, RunConfig, SupportLevel,
    WorkloadFamily,
};
pub use runtime::{
    arithmetic_asian_call_price_mc_cpu, arithmetic_asian_call_price_mlmc_cpu,
    basket_call_price_mc_cpu, black_scholes_european_call_price,
    compare_arithmetic_asian_sampling_quality_cpu, compare_basket_call_sampling_quality_cpu,
    compare_down_and_out_sampling_quality_cpu, compare_european_call_realized_error_cpu,
    compare_european_call_sampling_quality_cpu, diagnose_standard_normal_samples_cpu,
    diagnose_standard_normals_cpu, down_and_out_call_price_mc_cpu, european_call_price_mc_cpu,
    european_call_price_mc_cpu_stepwise, european_call_price_mc_cpu_terminal,
    european_call_price_mc_cpu_with_method, gaussian_uncertainty_mean_cpu,
    generate_standard_normals_cpu, monte_carlo_method_capabilities,
    solve_arithmetic_asian_mlmc_tolerance_cpu, structured_sampling_guidance_cpu,
    tune_arithmetic_asian_mlmc_allocation_cpu, AnalyticPricingComparison,
    ArithmeticAsianCallConfig, ArithmeticAsianCallPricer, ArithmeticAsianCallResult,
    ArithmeticAsianMlmcAllocationLevel, ArithmeticAsianMlmcAllocationPlan,
    ArithmeticAsianMlmcConfig, ArithmeticAsianMlmcLevelResult, ArithmeticAsianMlmcPricer,
    ArithmeticAsianMlmcResult, ArithmeticAsianMlmcToleranceConfig,
    ArithmeticAsianMlmcTolerancePlan, BackendMethodSupport, BasketCallConfig, BasketCallPricer,
    BasketCallResult, DownAndOutCallConfig, DownAndOutCallPricer, DownAndOutCallResult,
    EuropeanCallConfig, EuropeanCallMethod, EuropeanCallPricer, EuropeanCallResult,
    GaussianUncertaintyConfig, GaussianUncertaintyResult, MonteCarloMethodCapability,
    MonteCarloMethodCategory, MonteCarloRng, MonteCarloTechnique, PricingQualityComparison,
    PricingWorkloadFamily, SamplingMethod, StandardNormalDiagnostics, StructuredSamplingGuidance,
};
