use std::time::Instant;
use std::{process::Command, thread};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    european_call_price_mc_cpu, BackendId, EuropeanCallConfig, ExecutionPlan, PlannerMode,
    SupportLevel,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendInfo {
    pub backend_id: BackendId,
    pub display_name: String,
    pub version: String,
    pub platform: String,
    pub supported_precisions: Vec<String>,
    pub supported_rngs: Vec<String>,
    pub supported_sampling_modes: Vec<String>,
    pub supported_reduction_ops: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceInfo {
    pub device_id: String,
    pub backend_id: BackendId,
    pub name: String,
    pub vendor: String,
    pub memory_total_mb: Option<usize>,
    pub memory_free_mb: Option<usize>,
    pub supports_float64: bool,
    pub supports_unified_memory: bool,
    pub max_threads_hint: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupportReport {
    pub backend_id: BackendId,
    pub device_id: String,
    pub support_level: SupportLevel,
    pub unsupported_features: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostEstimate {
    pub backend_id: BackendId,
    pub device_id: String,
    pub estimated_compile_ms: f64,
    pub estimated_runtime_ms: f64,
    pub estimated_total_ms: f64,
    pub estimated_peak_memory_mb: f64,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GpuChunkingConfig {
    pub bytes_per_path: usize,
    pub target_utilization: f64,
    pub minimum_paths_per_chunk: usize,
    pub fallback_budget_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GpuChunkingPlan {
    pub total_paths: usize,
    pub paths_per_chunk: usize,
    pub chunk_count: usize,
    pub target_budget_mb: usize,
    pub estimated_peak_memory_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReproSupport {
    pub supports_same_backend_exact: bool,
    pub supports_same_backend_deterministic: bool,
    pub supports_cross_backend_statistical: bool,
    pub supports_stable_chunking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompiledArtifact {
    pub artifact_id: String,
    pub backend_id: BackendId,
    pub device_id: String,
    pub n_paths: usize,
    pub n_steps: usize,
    pub planner_mode: PlannerMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunOutput {
    pub price: f64,
    pub stderr: f64,
    pub runtime_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackendExecutionInput {
    EuropeanCall(EuropeanCallConfig),
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("device '{0}' is not available for this backend")]
    UnknownDevice(String),
    #[error("execution input is not compatible with compiled artifact")]
    IncompatibleExecutionInput,
    #[error("unsupported feature: {0}")]
    UnsupportedFeature(String),
}

pub trait RuntimeBackend {
    fn backend_id(&self) -> BackendId;
    fn describe_backend(&self) -> BackendInfo;
    fn discover_devices(&self) -> Vec<DeviceInfo>;
    fn supports(&self, plan: &ExecutionPlan, device: &DeviceInfo) -> SupportReport;
    fn estimate_cost(&self, plan: &ExecutionPlan, device: &DeviceInfo) -> CostEstimate;
    fn compile(
        &self,
        plan: &ExecutionPlan,
        device: &DeviceInfo,
    ) -> Result<CompiledArtifact, BackendError>;
    fn execute(
        &self,
        artifact: &CompiledArtifact,
        input: &BackendExecutionInput,
    ) -> Result<RunOutput, BackendError>;
    fn reproducibility_capabilities(&self, _device: &DeviceInfo) -> ReproSupport;
}

#[derive(Debug, Clone, Default)]
pub struct CpuNativeBackend;

impl CpuNativeBackend {
    pub fn new() -> Self {
        Self
    }

    fn validate_device(&self, device: &DeviceInfo) -> Result<(), BackendError> {
        if device.backend_id != BackendId::CpuNative || device.device_id != "cpu:host" {
            return Err(BackendError::UnknownDevice(device.device_id.clone()));
        }
        Ok(())
    }
}

impl RuntimeBackend for CpuNativeBackend {
    fn backend_id(&self) -> BackendId {
        BackendId::CpuNative
    }

    fn describe_backend(&self) -> BackendInfo {
        BackendInfo {
            backend_id: BackendId::CpuNative,
            display_name: "CPU Native".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: "cpu".to_string(),
            supported_precisions: vec!["float32".to_string(), "float64".to_string()],
            supported_rngs: vec!["xorshift64*".to_string()],
            supported_sampling_modes: vec!["iid".to_string()],
            supported_reduction_ops: vec![
                "sum".to_string(),
                "mean".to_string(),
                "variance".to_string(),
                "std".to_string(),
                "min".to_string(),
                "max".to_string(),
            ],
        }
    }

    fn discover_devices(&self) -> Vec<DeviceInfo> {
        vec![DeviceInfo {
            device_id: "cpu:host".to_string(),
            backend_id: BackendId::CpuNative,
            name: "Host CPU".to_string(),
            vendor: "generic".to_string(),
            memory_total_mb: None,
            memory_free_mb: None,
            supports_float64: true,
            supports_unified_memory: true,
            max_threads_hint: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
        }]
    }

    fn supports(&self, _plan: &ExecutionPlan, device: &DeviceInfo) -> SupportReport {
        if self.validate_device(device).is_err() {
            return SupportReport {
                backend_id: BackendId::CpuNative,
                device_id: device.device_id.clone(),
                support_level: SupportLevel::Unsupported,
                unsupported_features: vec!["unknown_device".to_string()],
                warnings: vec![],
            };
        }

        SupportReport {
            backend_id: BackendId::CpuNative,
            device_id: device.device_id.clone(),
            support_level: SupportLevel::Supported,
            unsupported_features: vec![],
            warnings: vec![],
        }
    }

    fn estimate_cost(&self, plan: &ExecutionPlan, device: &DeviceInfo) -> CostEstimate {
        let op_scale = (plan.n_paths as f64) * (plan.n_steps as f64);
        let estimated_runtime_ms = (op_scale / 5_000_000.0).max(0.01);

        CostEstimate {
            backend_id: BackendId::CpuNative,
            device_id: device.device_id.clone(),
            estimated_compile_ms: 0.0,
            estimated_runtime_ms,
            estimated_total_ms: estimated_runtime_ms,
            estimated_peak_memory_mb: 8.0,
            confidence: "low".to_string(),
        }
    }

    fn compile(
        &self,
        plan: &ExecutionPlan,
        device: &DeviceInfo,
    ) -> Result<CompiledArtifact, BackendError> {
        self.validate_device(device)?;

        Ok(CompiledArtifact {
            artifact_id: format!(
                "cpu-native:{}:{}:{}",
                plan.n_paths, plan.n_steps, plan.features.step_count
            ),
            backend_id: BackendId::CpuNative,
            device_id: device.device_id.clone(),
            n_paths: plan.n_paths,
            n_steps: plan.n_steps,
            planner_mode: plan.planner_mode,
        })
    }

    fn execute(
        &self,
        artifact: &CompiledArtifact,
        input: &BackendExecutionInput,
    ) -> Result<RunOutput, BackendError> {
        if artifact.backend_id != BackendId::CpuNative {
            return Err(BackendError::IncompatibleExecutionInput);
        }

        let started = Instant::now();

        let result = match input {
            BackendExecutionInput::EuropeanCall(cfg) => european_call_price_mc_cpu(cfg),
        };

        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
        Ok(RunOutput {
            price: result.price,
            stderr: result.stderr,
            runtime_ms,
        })
    }

    fn reproducibility_capabilities(&self, _device: &DeviceInfo) -> ReproSupport {
        ReproSupport {
            supports_same_backend_exact: true,
            supports_same_backend_deterministic: true,
            supports_cross_backend_statistical: true,
            supports_stable_chunking: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NvidiaCudaBackend;

impl NvidiaCudaBackend {
    pub fn new() -> Self {
        Self
    }

    fn validate_device(&self, device: &DeviceInfo) -> Result<(), BackendError> {
        if device.backend_id != BackendId::NvidiaCuda || !device.device_id.starts_with("cuda:") {
            return Err(BackendError::UnknownDevice(device.device_id.clone()));
        }
        Ok(())
    }
}

impl RuntimeBackend for NvidiaCudaBackend {
    fn backend_id(&self) -> BackendId {
        BackendId::NvidiaCuda
    }

    fn describe_backend(&self) -> BackendInfo {
        BackendInfo {
            backend_id: BackendId::NvidiaCuda,
            display_name: "NVIDIA CUDA".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: "cuda".to_string(),
            supported_precisions: vec!["float32".to_string(), "float64".to_string()],
            supported_rngs: vec!["philox".to_string(), "sobol".to_string()],
            supported_sampling_modes: vec!["iid".to_string(), "qmc".to_string()],
            supported_reduction_ops: vec![
                "sum".to_string(),
                "mean".to_string(),
                "variance".to_string(),
                "min".to_string(),
                "max".to_string(),
            ],
        }
    }

    fn discover_devices(&self) -> Vec<DeviceInfo> {
        discover_nvidia_devices()
    }

    fn supports(&self, _plan: &ExecutionPlan, device: &DeviceInfo) -> SupportReport {
        if self.validate_device(device).is_err() {
            return SupportReport {
                backend_id: BackendId::NvidiaCuda,
                device_id: device.device_id.clone(),
                support_level: SupportLevel::Unsupported,
                unsupported_features: vec!["unknown_device".to_string()],
                warnings: vec![],
            };
        }

        SupportReport {
            backend_id: BackendId::NvidiaCuda,
            device_id: device.device_id.clone(),
            support_level: SupportLevel::Unsupported,
            unsupported_features: vec!["execution_not_implemented".to_string()],
            warnings: vec![
                "CUDA backend contract is present, kernel execution is pending".to_string(),
            ],
        }
    }

    fn estimate_cost(&self, plan: &ExecutionPlan, device: &DeviceInfo) -> CostEstimate {
        let op_scale = (plan.n_paths as f64) * (plan.n_steps as f64);
        let estimated_runtime_ms = (op_scale / 50_000_000.0).max(0.01);
        let estimated_compile_ms = 2.0;
        let chunking = plan_gpu_chunking(
            plan.n_paths,
            device.memory_total_mb,
            GpuChunkingConfig {
                bytes_per_path: estimate_gpu_bytes_per_path(plan),
                target_utilization: 0.75,
                minimum_paths_per_chunk: 32_768,
                fallback_budget_mb: 8_192,
            },
        );

        CostEstimate {
            backend_id: BackendId::NvidiaCuda,
            device_id: device.device_id.clone(),
            estimated_compile_ms,
            estimated_runtime_ms,
            estimated_total_ms: estimated_compile_ms + estimated_runtime_ms,
            estimated_peak_memory_mb: chunking.estimated_peak_memory_mb as f64,
            confidence: "low".to_string(),
        }
    }

    fn compile(
        &self,
        _plan: &ExecutionPlan,
        device: &DeviceInfo,
    ) -> Result<CompiledArtifact, BackendError> {
        self.validate_device(device)?;
        Err(BackendError::UnsupportedFeature(
            "CUDA compile/execute path not implemented yet".to_string(),
        ))
    }

    fn execute(
        &self,
        _artifact: &CompiledArtifact,
        _input: &BackendExecutionInput,
    ) -> Result<RunOutput, BackendError> {
        Err(BackendError::UnsupportedFeature(
            "CUDA execute path not implemented yet".to_string(),
        ))
    }

    fn reproducibility_capabilities(&self, _device: &DeviceInfo) -> ReproSupport {
        ReproSupport {
            supports_same_backend_exact: false,
            supports_same_backend_deterministic: true,
            supports_cross_backend_statistical: true,
            supports_stable_chunking: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppleMetalBackend;

impl AppleMetalBackend {
    pub fn new() -> Self {
        Self
    }

    fn validate_device(&self, device: &DeviceInfo) -> Result<(), BackendError> {
        if device.backend_id != BackendId::AppleMetal || !device.device_id.starts_with("metal:") {
            return Err(BackendError::UnknownDevice(device.device_id.clone()));
        }
        Ok(())
    }
}

impl RuntimeBackend for AppleMetalBackend {
    fn backend_id(&self) -> BackendId {
        BackendId::AppleMetal
    }

    fn describe_backend(&self) -> BackendInfo {
        BackendInfo {
            backend_id: BackendId::AppleMetal,
            display_name: "Apple Metal".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: "metal".to_string(),
            supported_precisions: vec!["float32".to_string()],
            supported_rngs: vec!["philox".to_string(), "sobol".to_string()],
            supported_sampling_modes: vec!["iid".to_string(), "qmc".to_string()],
            supported_reduction_ops: vec![
                "sum".to_string(),
                "mean".to_string(),
                "variance".to_string(),
                "min".to_string(),
                "max".to_string(),
            ],
        }
    }

    fn discover_devices(&self) -> Vec<DeviceInfo> {
        discover_apple_metal_devices()
    }

    fn supports(&self, _plan: &ExecutionPlan, device: &DeviceInfo) -> SupportReport {
        if self.validate_device(device).is_err() {
            return SupportReport {
                backend_id: BackendId::AppleMetal,
                device_id: device.device_id.clone(),
                support_level: SupportLevel::Unsupported,
                unsupported_features: vec!["unknown_device".to_string()],
                warnings: vec![],
            };
        }

        SupportReport {
            backend_id: BackendId::AppleMetal,
            device_id: device.device_id.clone(),
            support_level: SupportLevel::Unsupported,
            unsupported_features: vec!["execution_not_implemented".to_string()],
            warnings: vec![
                "Apple Metal backend contract is present, kernel execution is pending".to_string(),
            ],
        }
    }

    fn estimate_cost(&self, plan: &ExecutionPlan, device: &DeviceInfo) -> CostEstimate {
        let op_scale = (plan.n_paths as f64) * (plan.n_steps as f64);
        let estimated_runtime_ms = (op_scale / 35_000_000.0).max(0.01);
        let estimated_compile_ms = 1.5;
        let chunking = plan_gpu_chunking(
            plan.n_paths,
            device.memory_total_mb,
            GpuChunkingConfig {
                bytes_per_path: estimate_gpu_bytes_per_path(plan),
                target_utilization: 0.70,
                minimum_paths_per_chunk: 32_768,
                fallback_budget_mb: 6_144,
            },
        );

        CostEstimate {
            backend_id: BackendId::AppleMetal,
            device_id: device.device_id.clone(),
            estimated_compile_ms,
            estimated_runtime_ms,
            estimated_total_ms: estimated_compile_ms + estimated_runtime_ms,
            estimated_peak_memory_mb: chunking.estimated_peak_memory_mb as f64,
            confidence: "low".to_string(),
        }
    }

    fn compile(
        &self,
        _plan: &ExecutionPlan,
        device: &DeviceInfo,
    ) -> Result<CompiledArtifact, BackendError> {
        self.validate_device(device)?;
        Err(BackendError::UnsupportedFeature(
            "Apple Metal compile/execute path not implemented yet".to_string(),
        ))
    }

    fn execute(
        &self,
        _artifact: &CompiledArtifact,
        _input: &BackendExecutionInput,
    ) -> Result<RunOutput, BackendError> {
        Err(BackendError::UnsupportedFeature(
            "Apple Metal execute path not implemented yet".to_string(),
        ))
    }

    fn reproducibility_capabilities(&self, _device: &DeviceInfo) -> ReproSupport {
        ReproSupport {
            supports_same_backend_exact: false,
            supports_same_backend_deterministic: true,
            supports_cross_backend_statistical: true,
            supports_stable_chunking: true,
        }
    }
}

pub fn builtin_backends() -> Vec<Box<dyn RuntimeBackend>> {
    vec![
        Box::new(CpuNativeBackend::new()),
        Box::new(NvidiaCudaBackend::new()),
        Box::new(AppleMetalBackend::new()),
    ]
}

pub fn plan_gpu_chunking(
    total_paths: usize,
    device_memory_mb: Option<usize>,
    config: GpuChunkingConfig,
) -> GpuChunkingPlan {
    let total_paths = total_paths.max(1);
    let bytes_per_path = config.bytes_per_path.max(1);
    let utilization = config.target_utilization.clamp(0.10, 0.95);
    let budget_mb = device_memory_mb.unwrap_or(config.fallback_budget_mb).max(1);
    let budget_bytes = ((budget_mb as f64) * 1024.0 * 1024.0 * utilization).floor() as usize;
    let max_paths_fit = (budget_bytes / bytes_per_path).max(1);
    let paths_per_chunk = max_paths_fit
        .max(config.minimum_paths_per_chunk)
        .min(total_paths);
    let chunk_count = total_paths.div_ceil(paths_per_chunk);
    let estimated_peak_memory_mb =
        ((paths_per_chunk.saturating_mul(bytes_per_path)).div_ceil(1024 * 1024)).max(1);

    GpuChunkingPlan {
        total_paths,
        paths_per_chunk,
        chunk_count,
        target_budget_mb: budget_mb,
        estimated_peak_memory_mb,
    }
}

pub fn estimate_gpu_bytes_per_path(plan: &ExecutionPlan) -> usize {
    // Conservative per-path accounting for v1 kernels:
    // - state and payoff buffers
    // - RNG counter/key state
    // - scratch for per-step intermediates (bounded for now)
    let state_bytes = 8usize;
    let payoff_bytes = 8usize;
    let rng_state_bytes = 16usize;
    let step_scratch_bytes = plan.n_steps.min(256).saturating_mul(4);
    state_bytes + payoff_bytes + rng_state_bytes + step_scratch_bytes
}

fn discover_nvidia_devices() -> Vec<DeviceInfo> {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output();

    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            let mut parts = line.split(',');
            let name = parts.next()?.trim().to_string();
            let memory_mb = parts.next()?.trim().parse::<usize>().ok()?;
            Some(DeviceInfo {
                device_id: format!("cuda:{idx}"),
                backend_id: BackendId::NvidiaCuda,
                name,
                vendor: "nvidia".to_string(),
                memory_total_mb: Some(memory_mb),
                memory_free_mb: None,
                supports_float64: true,
                supports_unified_memory: false,
                max_threads_hint: 1024,
            })
        })
        .collect()
}

fn discover_apple_metal_devices() -> Vec<DeviceInfo> {
    if !cfg!(target_os = "macos") {
        return Vec::new();
    }

    vec![DeviceInfo {
        device_id: "metal:0".to_string(),
        backend_id: BackendId::AppleMetal,
        name: "Apple GPU".to_string(),
        vendor: "apple".to_string(),
        memory_total_mb: None,
        memory_free_mb: None,
        supports_float64: false,
        supports_unified_memory: true,
        max_threads_hint: thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1),
    }]
}
