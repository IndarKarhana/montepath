use std::{process::Command, thread};

use super::{
    compile_gpu_fallback_artifact, execute_gpu_fallback, make_native_artifact_metadata,
    plan_gpu_chunking, BackendError, BackendExecutionInput, BackendId, BackendInfo,
    CompiledArtifact, CostEstimate, DeviceInfo, ExecutionPlan, GpuChunkingConfig, ReproSupport,
    RuntimeBackend, SupportReport,
};
use crate::SupportLevel;

pub fn metal_native_feature_enabled() -> bool {
    cfg!(feature = "metal-native")
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

    fn supports(&self, plan: &ExecutionPlan, device: &DeviceInfo) -> SupportReport {
        if self.validate_device(device).is_err() {
            return SupportReport {
                backend_id: BackendId::AppleMetal,
                device_id: device.device_id.clone(),
                support_level: SupportLevel::Unsupported,
                unsupported_features: vec!["unknown_device".to_string()],
                warnings: vec![],
            };
        }

        let mut warnings = vec![
            "Apple Metal backend currently executes through delegated CPU fallback while native kernels are in progress"
                .to_string(),
        ];
        let mut unsupported_features = vec!["native_metal_execution_not_implemented".to_string()];

        if !supports_first_metal_kernel_shape(plan) {
            unsupported_features.push("first_metal_kernel_shape_not_supported".to_string());
            warnings.push(
                "first staged Metal kernel currently targets the narrow European-call stepwise workload"
                    .to_string(),
            );
        }

        if metal_native_feature_enabled() {
            if probe_metal_toolchain() {
                warnings.push(
                    "metal-native feature enabled and Metal toolchain detected; host-side shader staging is active"
                        .to_string(),
                );
            } else {
                warnings.push(
                    "metal-native feature enabled but Metal toolchain was not detected on this machine"
                        .to_string(),
                );
            }
        } else {
            warnings.push(
                "enable the `metal-native` feature to validate host-side Metal staging in CI or locally"
                    .to_string(),
            );
        }

        SupportReport {
            backend_id: BackendId::AppleMetal,
            device_id: device.device_id.clone(),
            support_level: SupportLevel::SupportedWithFallbacks,
            unsupported_features,
            warnings,
        }
    }

    fn estimate_cost(&self, plan: &ExecutionPlan, device: &DeviceInfo) -> CostEstimate {
        let op_scale = (plan.n_paths as f64) * (plan.n_steps as f64);
        let estimated_runtime_ms = (op_scale / 35_000_000.0).max(0.01);
        let estimated_compile_ms = if metal_native_feature_enabled() {
            1.0
        } else {
            1.5
        };
        let chunking = plan_gpu_chunking(
            plan.n_paths,
            device.memory_total_mb,
            GpuChunkingConfig {
                bytes_per_path: super::estimate_gpu_bytes_per_path(plan),
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
            confidence: if metal_native_feature_enabled() {
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
            "mc_metal_european_call_stepwise_v1",
            "crates/mc-core/src/backend/metal.rs",
            "metal_shading_language_host_staging",
            "metal-native",
            probe_metal_toolchain(),
            metal_kernel_notes(plan),
        ));

        Ok(compile_gpu_fallback_artifact(
            BackendId::AppleMetal,
            "metal",
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
        execute_gpu_fallback(BackendId::AppleMetal, artifact, input)
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

pub(crate) fn discover_apple_metal_devices() -> Vec<DeviceInfo> {
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

fn supports_first_metal_kernel_shape(plan: &ExecutionPlan) -> bool {
    plan.features.conditional_expression_count == 0 && plan.features.reduction_count <= 1
}

fn metal_kernel_notes(plan: &ExecutionPlan) -> Vec<String> {
    let mut notes = vec![
        "host-side Metal shader ABI is staged but runtime still executes through delegated CPU fallback"
            .to_string(),
        format!(
            "validated target shape: n_paths={} n_steps={} conditional_expressions={}",
            plan.n_paths, plan.n_steps, plan.features.conditional_expression_count
        ),
    ];

    if metal_native_feature_enabled() {
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

fn probe_metal_toolchain() -> bool {
    if !cfg!(target_os = "macos") {
        return false;
    }

    Command::new("xcrun")
        .args(["-sdk", "macosx", "metal", "-v"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
