use std::process::Command;

use super::{
    compile_gpu_fallback_artifact, execute_gpu_fallback, make_native_artifact_metadata,
    plan_gpu_chunking, BackendError, BackendExecutionInput, BackendId, BackendInfo,
    CompiledArtifact, CostEstimate, DeviceInfo, ExecutionPlan, GpuChunkingConfig, ReproSupport,
    RuntimeBackend, SupportReport,
};
use crate::SupportLevel;

pub fn cuda_native_feature_enabled() -> bool {
    cfg!(feature = "cuda-native")
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

    fn supports(&self, plan: &ExecutionPlan, device: &DeviceInfo) -> SupportReport {
        if self.validate_device(device).is_err() {
            return SupportReport {
                backend_id: BackendId::NvidiaCuda,
                device_id: device.device_id.clone(),
                support_level: SupportLevel::Unsupported,
                unsupported_features: vec!["unknown_device".to_string()],
                warnings: vec![],
            };
        }

        let mut warnings = vec![
            "CUDA backend currently executes through delegated CPU fallback while native kernels are in progress"
                .to_string(),
        ];
        let mut unsupported_features = vec!["native_cuda_execution_not_implemented".to_string()];

        if !supports_first_cuda_kernel_shape(plan) {
            unsupported_features.push("first_cuda_kernel_shape_not_supported".to_string());
            warnings.push(
                "first staged CUDA kernel currently targets the narrow European-call stepwise workload"
                    .to_string(),
            );
        }

        if cuda_native_feature_enabled() {
            if probe_cuda_toolchain() {
                warnings.push(
                    "cuda-native feature enabled and CUDA toolchain detected; host-side kernel staging is active"
                        .to_string(),
                );
            } else {
                warnings.push(
                    "cuda-native feature enabled but CUDA toolchain was not detected on this machine"
                        .to_string(),
                );
            }
        } else {
            warnings.push(
                "enable the `cuda-native` feature to validate host-side native CUDA staging in CI or locally"
                    .to_string(),
            );
        }

        SupportReport {
            backend_id: BackendId::NvidiaCuda,
            device_id: device.device_id.clone(),
            support_level: SupportLevel::SupportedWithFallbacks,
            unsupported_features,
            warnings,
        }
    }

    fn estimate_cost(&self, plan: &ExecutionPlan, device: &DeviceInfo) -> CostEstimate {
        let op_scale = (plan.n_paths as f64) * (plan.n_steps as f64);
        let estimated_runtime_ms = (op_scale / 50_000_000.0).max(0.01);
        let estimated_compile_ms = if cuda_native_feature_enabled() {
            1.5
        } else {
            2.0
        };
        let chunking = plan_gpu_chunking(
            plan.n_paths,
            device.memory_total_mb,
            GpuChunkingConfig {
                bytes_per_path: super::estimate_gpu_bytes_per_path(plan),
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
            confidence: if cuda_native_feature_enabled() {
                "medium".to_string()
            } else {
                "low".to_string()
            },
        }
    }

    fn compile(
        &self,
        plan: &ExecutionPlan,
        device: &DeviceInfo,
    ) -> Result<CompiledArtifact, BackendError> {
        self.validate_device(device)?;

        let native_artifact = Some(make_native_artifact_metadata(
            "european_call_stepwise_v1",
            "mc_cuda_european_call_stepwise_v1",
            "crates/mc-core/src/backend/cuda.rs",
            "cuda_c_host_staging",
            "cuda-native",
            probe_cuda_toolchain(),
            cuda_kernel_notes(plan),
        ));

        Ok(compile_gpu_fallback_artifact(
            BackendId::NvidiaCuda,
            "cuda",
            plan,
            device,
            native_artifact,
        ))
    }

    fn execute(
        &self,
        artifact: &CompiledArtifact,
        input: &BackendExecutionInput,
    ) -> Result<super::RunOutput, BackendError> {
        execute_gpu_fallback(BackendId::NvidiaCuda, artifact, input)
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

pub(crate) fn discover_nvidia_devices() -> Vec<DeviceInfo> {
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

fn supports_first_cuda_kernel_shape(plan: &ExecutionPlan) -> bool {
    plan.features.conditional_expression_count == 0 && plan.features.reduction_count <= 1
}

fn cuda_kernel_notes(plan: &ExecutionPlan) -> Vec<String> {
    let mut notes = vec![
        "host-side CUDA kernel ABI is staged but runtime still executes through delegated CPU fallback"
            .to_string(),
        format!(
            "validated target shape: n_paths={} n_steps={} conditional_expressions={}",
            plan.n_paths, plan.n_steps, plan.features.conditional_expression_count
        ),
    ];

    if cuda_native_feature_enabled() {
        notes.push(
            "feature gate enabled; native launch plumbing can be validated at compile time"
                .to_string(),
        );
    } else {
        notes.push(
            "feature gate disabled; artifact remains a native-ready manifest only".to_string(),
        );
    }

    notes
}

fn probe_cuda_toolchain() -> bool {
    Command::new("nvcc")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
