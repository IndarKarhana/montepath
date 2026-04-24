use std::{fs, path::PathBuf, process::Command, thread, time::Instant};

#[cfg(all(feature = "metal-native", target_os = "macos"))]
use std::path::Path;

#[cfg(all(feature = "metal-native", target_os = "macos"))]
use std::{cell::RefCell, mem::size_of};

#[cfg(all(feature = "metal-native", target_os = "macos"))]
use metal::{
    CommandBufferRef, CommandQueue, CompileOptions, ComputeCommandEncoderRef, ComputePipelineState,
    Device, Library, MTLCommandBufferStatus, MTLResourceOptions, MTLSize,
};
#[cfg(all(feature = "metal-native", target_os = "macos"))]
use objc::rc::autoreleasepool;

use super::{
    compile_gpu_fallback_artifact, execute_gpu_fallback, make_native_artifact_metadata,
    plan_gpu_chunking, BackendError, BackendExecutionInput, BackendId, BackendInfo,
    CompiledArtifact, CostEstimate, DeviceInfo, ExecutionPlan, GpuBufferBinding,
    GpuBufferDirection, GpuChunkingConfig, GpuKernelContract, GpuLaunchDimensions,
    GpuScalarBinding, GpuValueType, ReproSupport, RuntimeBackend, SupportReport,
};
use crate::{MonteCarloTechnique, SupportLevel};

pub fn metal_native_feature_enabled() -> bool {
    cfg!(feature = "metal-native")
}

const FIRST_METAL_KERNEL_ENTRY_POINT: &str = "mc_metal_european_call_stepwise_v1";
const FIRST_METAL_REDUCTION_ENTRY_POINT: &str = "mc_metal_reduce_sum_f32_v1";
const FIRST_METAL_KERNEL_FAMILY: &str = "european_call_stepwise_v1";
const FIRST_METAL_KERNEL_SOURCE_MODULE: &str =
    "crates/mc-core/src/backend/kernels/european_call_stepwise_v1.metal";
const FIRST_METAL_KERNEL_SOURCE: &str = include_str!("kernels/european_call_stepwise_v1.metal");
const METAL_THREADGROUP_WIDTH: usize = 256;

#[cfg(all(feature = "metal-native", target_os = "macos"))]
thread_local! {
    static METAL_RUNTIME_CACHE: RefCell<Option<MetalRuntimeCache>> = const { RefCell::new(None) };
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
            "Apple Metal backend now has a narrow native v1 path for standard European-call stepwise execution"
                .to_string(),
        ];
        let mut unsupported_features = Vec::new();

        if !supports_first_metal_kernel_shape(plan) {
            unsupported_features.push("first_metal_kernel_shape_not_supported".to_string());
            warnings.push(
                "plans outside the staged European-call stepwise workload still fall back to CPU execution"
                    .to_string(),
            );
        }

        if metal_native_feature_enabled() {
            if cfg!(target_os = "macos") {
                warnings.push(
                    "metal-native feature enabled; execution uses in-process Metal runtime with cached pipelines on macOS"
                        .to_string(),
                );
            } else {
                warnings.push(
                    "metal-native feature enabled on a non-macOS target; native execution is unavailable here"
                        .to_string(),
                );
            }

            if probe_metal_toolchain() {
                warnings.push(
                    "Metal developer tools detected; artifact staging can precompile metallib outputs"
                        .to_string(),
                );
            } else {
                warnings.push(
                    "Metal developer tools were not detected; runtime source compilation remains available on macOS"
                        .to_string(),
                );
            }
        } else {
            unsupported_features.push("metal_native_feature_disabled".to_string());
            warnings.push(
                "enable the `metal-native` feature to activate native Apple GPU execution instead of fallback"
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
        let compile_status = stage_native_metal_kernel(plan);
        let notes = metal_kernel_notes(plan, &compile_status);

        let native_artifact = Some(make_native_artifact_metadata(
            FIRST_METAL_KERNEL_FAMILY,
            FIRST_METAL_KERNEL_ENTRY_POINT,
            FIRST_METAL_KERNEL_SOURCE_MODULE,
            "metal_shading_language",
            "metal-native",
            compile_status.toolchain_available,
            compile_status.compile_requested,
            compile_status.compile_succeeded,
            compile_status.compiled_module_path,
            Some(first_metal_kernel_contract(plan)),
            notes,
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
        if let Ok(run_output) = execute_native_metal_if_possible(artifact, input) {
            return Ok(run_output);
        }

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

fn first_metal_kernel_contract(plan: &ExecutionPlan) -> GpuKernelContract {
    let threadgroups_x = reduction_group_count(plan.n_paths) as u32;
    GpuKernelContract {
        kernel_family: FIRST_METAL_KERNEL_FAMILY.to_string(),
        entry_point: FIRST_METAL_KERNEL_ENTRY_POINT.to_string(),
        buffers: vec![
            GpuBufferBinding {
                binding_index: 0,
                name: "partial_sums".to_string(),
                direction: GpuBufferDirection::Output,
                value_type: GpuValueType::Float32,
                element_count: threadgroups_x as usize,
            },
            GpuBufferBinding {
                binding_index: 1,
                name: "partial_sq_sums".to_string(),
                direction: GpuBufferDirection::Output,
                value_type: GpuValueType::Float32,
                element_count: threadgroups_x as usize,
            },
        ],
        scalars: vec![
            GpuScalarBinding {
                binding_index: 2,
                name: "n_paths".to_string(),
                value_type: GpuValueType::Int32,
            },
            GpuScalarBinding {
                binding_index: 3,
                name: "n_steps".to_string(),
                value_type: GpuValueType::Int32,
            },
            GpuScalarBinding {
                binding_index: 4,
                name: "log_s0".to_string(),
                value_type: GpuValueType::Float32,
            },
            GpuScalarBinding {
                binding_index: 5,
                name: "strike".to_string(),
                value_type: GpuValueType::Float32,
            },
            GpuScalarBinding {
                binding_index: 6,
                name: "drift_dt".to_string(),
                value_type: GpuValueType::Float32,
            },
            GpuScalarBinding {
                binding_index: 7,
                name: "vol_dt".to_string(),
                value_type: GpuValueType::Float32,
            },
            GpuScalarBinding {
                binding_index: 8,
                name: "discount".to_string(),
                value_type: GpuValueType::Float32,
            },
            GpuScalarBinding {
                binding_index: 9,
                name: "seed".to_string(),
                value_type: GpuValueType::Int32,
            },
        ],
        launch: GpuLaunchDimensions {
            logical_threads: plan.n_paths,
            threads_per_group_x: METAL_THREADGROUP_WIDTH as u32,
            threadgroups_x,
        },
    }
}

#[derive(Debug, Clone)]
struct MetalKernelStageStatus {
    toolchain_available: bool,
    compile_requested: bool,
    compile_succeeded: bool,
    compiled_module_path: Option<String>,
    diagnostics: Vec<String>,
}

fn metal_kernel_notes(
    plan: &ExecutionPlan,
    compile_status: &MetalKernelStageStatus,
) -> Vec<String> {
    let mut notes = vec![
        "native Metal v1 runtime is available for the staged European-call stepwise workload; other shapes still fall back"
            .to_string(),
        "native execution caches the Metal device, command queue, and compute pipeline state inside the Rust process"
            .to_string(),
        "main pricing kernel now performs RNG and first-stage reductions on-device; follow-up reduction passes continue on-device until a single aggregate remains"
            .to_string(),
        format!(
            "validated target shape: n_paths={} n_steps={} conditional_expressions={}",
            plan.n_paths, plan.n_steps, plan.features.conditional_expression_count
        ),
        format!("kernel_entry_point={FIRST_METAL_KERNEL_ENTRY_POINT}"),
        format!("reduction_entry_point={FIRST_METAL_REDUCTION_ENTRY_POINT}"),
    ];

    if metal_native_feature_enabled() {
        notes.push(
            "feature gate enabled; native Metal launch path is active on macOS and validated in tests"
                .to_string(),
        );
    } else {
        notes.push(
            "feature gate disabled; artifact remains a native-ready manifest only".to_string(),
        );
    }

    notes.extend(compile_status.diagnostics.iter().cloned());

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

fn stage_native_metal_kernel(plan: &ExecutionPlan) -> MetalKernelStageStatus {
    let toolchain_available = probe_metal_toolchain();
    let compile_requested = metal_native_feature_enabled();

    if !compile_requested {
        return MetalKernelStageStatus {
            toolchain_available,
            compile_requested,
            compile_succeeded: false,
            compiled_module_path: None,
            diagnostics: vec![
                "native Metal compile skipped because the `metal-native` feature is disabled"
                    .to_string(),
            ],
        };
    }

    if !toolchain_available {
        return MetalKernelStageStatus {
            toolchain_available,
            compile_requested,
            compile_succeeded: false,
            compiled_module_path: None,
            diagnostics: vec![
                "native Metal precompile requested but Metal developer tools were not available; runtime source compilation will still work on macOS"
                    .to_string(),
            ],
        };
    }

    match compile_metal_kernel_to_metallib(plan) {
        Ok(metallib_path) => MetalKernelStageStatus {
            toolchain_available,
            compile_requested,
            compile_succeeded: true,
            compiled_module_path: Some(metallib_path.display().to_string()),
            diagnostics: vec![format!(
                "native Metal library compilation succeeded for staged kernel: {}",
                metallib_path.display()
            )],
        },
        Err(error) => MetalKernelStageStatus {
            toolchain_available,
            compile_requested,
            compile_succeeded: false,
            compiled_module_path: None,
            diagnostics: vec![format!("native Metal library compilation failed: {error}")],
        },
    }
}

fn execute_native_metal_if_possible(
    artifact: &CompiledArtifact,
    input: &BackendExecutionInput,
) -> Result<super::RunOutput, BackendError> {
    if artifact.backend_id != BackendId::AppleMetal {
        return Err(BackendError::IncompatibleExecutionInput);
    }

    if !metal_native_feature_enabled() || !cfg!(target_os = "macos") {
        return Err(BackendError::UnsupportedFeature(
            "native Metal execution requires the metal-native feature on macOS".to_string(),
        ));
    }

    let cfg = match input {
        BackendExecutionInput::EuropeanCall(cfg) => cfg,
    };

    if cfg.n_paths != artifact.n_paths || cfg.n_steps != artifact.n_steps {
        return Err(BackendError::IncompatibleExecutionInput);
    }

    if cfg.technique != MonteCarloTechnique::Standard {
        return Err(BackendError::UnsupportedFeature(
            "native Metal execution currently supports standard stepwise European call only"
                .to_string(),
        ));
    }

    let started = Instant::now();
    let compiled_module_path = artifact
        .native_artifact
        .as_ref()
        .and_then(|native| native.compiled_module_path.as_deref());
    let result = execute_metal_stepwise_kernel(cfg, compiled_module_path)?;

    Ok(super::RunOutput {
        price: result.price,
        stderr: result.stderr,
        runtime_ms: started.elapsed().as_secs_f64() * 1_000.0,
    })
}

fn execute_metal_stepwise_kernel(
    cfg: &crate::EuropeanCallConfig,
    compiled_module_path: Option<&str>,
) -> Result<crate::EuropeanCallResult, BackendError> {
    #[cfg(not(all(feature = "metal-native", target_os = "macos")))]
    {
        let _ = (cfg, compiled_module_path);
        Err(BackendError::UnsupportedFeature(
            "native Metal execution is unavailable on this target".to_string(),
        ))
    }

    #[cfg(all(feature = "metal-native", target_os = "macos"))]
    {
        let runtime_key = runtime_cache_key(compiled_module_path);
        METAL_RUNTIME_CACHE.with(|slot| {
            let mut cache = slot.borrow_mut();
            let needs_refresh = cache
                .as_ref()
                .map(|runtime| runtime.cache_key != runtime_key)
                .unwrap_or(true);
            if needs_refresh {
                *cache = Some(MetalRuntimeCache::new(&runtime_key, compiled_module_path)?);
            }

            cache
                .as_ref()
                .expect("metal runtime cache should be initialized")
                .execute(cfg)
        })
    }
}

#[cfg(all(feature = "metal-native", target_os = "macos"))]
#[derive(Debug)]
struct MetalRuntimeCache {
    cache_key: String,
    device: Device,
    command_queue: CommandQueue,
    pricing_pipeline: ComputePipelineState,
    reduction_pipeline: ComputePipelineState,
}

#[cfg(all(feature = "metal-native", target_os = "macos"))]
impl MetalRuntimeCache {
    fn new(cache_key: &str, compiled_module_path: Option<&str>) -> Result<Self, BackendError> {
        autoreleasepool(|| {
            let device = Device::system_default().ok_or_else(|| {
                BackendError::UnsupportedFeature(
                    "unable to create default Metal device for native execution".to_string(),
                )
            })?;
            let command_queue = device.new_command_queue();
            let library = load_metal_library(&device, compiled_module_path)?;
            let pricing_pipeline =
                create_compute_pipeline(&device, &library, FIRST_METAL_KERNEL_ENTRY_POINT)?;
            let reduction_pipeline =
                create_compute_pipeline(&device, &library, FIRST_METAL_REDUCTION_ENTRY_POINT)?;

            let pricing_limit = pricing_pipeline.max_total_threads_per_threadgroup() as usize;
            let reduction_limit = reduction_pipeline.max_total_threads_per_threadgroup() as usize;
            if pricing_limit < METAL_THREADGROUP_WIDTH || reduction_limit < METAL_THREADGROUP_WIDTH
            {
                return Err(BackendError::UnsupportedFeature(format!(
                    "Metal device does not support required threadgroup width {} (pricing limit {}, reduction limit {})",
                    METAL_THREADGROUP_WIDTH, pricing_limit, reduction_limit
                )));
            }

            Ok(Self {
                cache_key: cache_key.to_string(),
                device,
                command_queue,
                pricing_pipeline,
                reduction_pipeline,
            })
        })
    }

    fn execute(
        &self,
        cfg: &crate::EuropeanCallConfig,
    ) -> Result<crate::EuropeanCallResult, BackendError> {
        autoreleasepool(|| {
            let main_group_count = reduction_group_count(cfg.n_paths);
            let partial_buffer_len = main_group_count.max(1);
            let partial_buffer_bytes = (partial_buffer_len * size_of::<f32>()) as u64;

            let partial_sum_a = self
                .device
                .new_buffer(partial_buffer_bytes, MTLResourceOptions::StorageModeShared);
            let partial_sum_b = self
                .device
                .new_buffer(partial_buffer_bytes, MTLResourceOptions::StorageModeShared);
            let partial_sq_sum_a = self
                .device
                .new_buffer(partial_buffer_bytes, MTLResourceOptions::StorageModeShared);
            let partial_sq_sum_b = self
                .device
                .new_buffer(partial_buffer_bytes, MTLResourceOptions::StorageModeShared);

            let command_buffer = self.command_queue.new_command_buffer();
            encode_pricing_pass(
                command_buffer,
                &self.pricing_pipeline,
                &partial_sum_a,
                &partial_sq_sum_a,
                cfg,
            );

            let mut current_count = main_group_count;
            let mut final_data_in_a = true;

            while current_count > 1 {
                let next_count = reduction_group_count(current_count);
                if final_data_in_a {
                    encode_reduction_pass(
                        command_buffer,
                        &self.reduction_pipeline,
                        &partial_sum_a,
                        &partial_sum_b,
                        current_count,
                    );
                    encode_reduction_pass(
                        command_buffer,
                        &self.reduction_pipeline,
                        &partial_sq_sum_a,
                        &partial_sq_sum_b,
                        current_count,
                    );
                } else {
                    encode_reduction_pass(
                        command_buffer,
                        &self.reduction_pipeline,
                        &partial_sum_b,
                        &partial_sum_a,
                        current_count,
                    );
                    encode_reduction_pass(
                        command_buffer,
                        &self.reduction_pipeline,
                        &partial_sq_sum_b,
                        &partial_sq_sum_a,
                        current_count,
                    );
                }

                final_data_in_a = !final_data_in_a;
                current_count = next_count;
            }

            command_buffer.commit();
            command_buffer.wait_until_completed();

            if command_buffer.status() != MTLCommandBufferStatus::Completed {
                return Err(BackendError::UnsupportedFeature(
                    "Metal command buffer did not complete successfully".to_string(),
                ));
            }

            let final_sum_buffer = if final_data_in_a {
                &partial_sum_a
            } else {
                &partial_sum_b
            };
            let final_sq_sum_buffer = if final_data_in_a {
                &partial_sq_sum_a
            } else {
                &partial_sq_sum_b
            };

            let payoff_sum = unsafe { *final_sum_buffer.contents().cast::<f32>() as f64 };
            let payoff_sq_sum = unsafe { *final_sq_sum_buffer.contents().cast::<f32>() as f64 };
            let n = cfg.n_paths as f64;
            let price = payoff_sum / n;
            let variance = ((payoff_sq_sum / n) - (price * price)).max(0.0);
            let stderr = variance.sqrt() / n.sqrt();

            Ok(crate::EuropeanCallResult { price, stderr })
        })
    }
}

#[cfg(all(feature = "metal-native", target_os = "macos"))]
fn runtime_cache_key(compiled_module_path: Option<&str>) -> String {
    match compiled_module_path {
        Some(path) if Path::new(path).exists() => format!("metallib::{path}"),
        Some(path) => format!("source-fallback::{path}"),
        None => "source-runtime".to_string(),
    }
}

#[cfg(all(feature = "metal-native", target_os = "macos"))]
fn load_metal_library(
    device: &Device,
    compiled_module_path: Option<&str>,
) -> Result<Library, BackendError> {
    if let Some(path) = compiled_module_path {
        let path_ref = Path::new(path);
        if path_ref.exists() {
            return device.new_library_with_file(path_ref).map_err(|error| {
                BackendError::UnsupportedFeature(format!(
                    "unable to load staged Metal library from {}: {error}",
                    path_ref.display()
                ))
            });
        }
    }

    let options = CompileOptions::new();
    options.set_fast_math_enabled(true);
    device
        .new_library_with_source(FIRST_METAL_KERNEL_SOURCE, &options)
        .map_err(|error| {
            BackendError::UnsupportedFeature(format!(
                "unable to compile Metal source in-process: {error}"
            ))
        })
}

#[cfg(all(feature = "metal-native", target_os = "macos"))]
fn create_compute_pipeline(
    device: &Device,
    library: &Library,
    entry_point: &str,
) -> Result<ComputePipelineState, BackendError> {
    let function = library.get_function(entry_point, None).map_err(|error| {
        BackendError::UnsupportedFeature(format!(
            "unable to locate Metal kernel `{entry_point}`: {error}"
        ))
    })?;

    device
        .new_compute_pipeline_state_with_function(&function)
        .map_err(|error| {
            BackendError::UnsupportedFeature(format!(
                "unable to create Metal compute pipeline for `{entry_point}`: {error}"
            ))
        })
}

#[cfg(all(feature = "metal-native", target_os = "macos"))]
fn encode_pricing_pass(
    command_buffer: &CommandBufferRef,
    pipeline: &ComputePipelineState,
    partial_sum_buffer: &metal::Buffer,
    partial_sq_sum_buffer: &metal::Buffer,
    cfg: &crate::EuropeanCallConfig,
) {
    let encoder = command_buffer.new_compute_command_encoder();
    encoder.set_compute_pipeline_state(pipeline);
    encoder.set_buffer(0, Some(partial_sum_buffer), 0);
    encoder.set_buffer(1, Some(partial_sq_sum_buffer), 0);

    let n_paths = cfg.n_paths as i32;
    let n_steps = cfg.n_steps as i32;
    let log_s0 = cfg.s0.ln() as f32;
    let strike = cfg.k as f32;
    let dt = (cfg.t / cfg.n_steps as f64) as f32;
    let drift_dt = ((cfg.r - 0.5 * cfg.sigma * cfg.sigma) as f32) * dt;
    let vol_dt = (cfg.sigma as f32) * dt.sqrt();
    let discount = ((-cfg.r * cfg.t).exp()) as f32;
    let seed = cfg.seed as u32;

    set_scalar_bytes(encoder, 2, &n_paths);
    set_scalar_bytes(encoder, 3, &n_steps);
    set_scalar_bytes(encoder, 4, &log_s0);
    set_scalar_bytes(encoder, 5, &strike);
    set_scalar_bytes(encoder, 6, &drift_dt);
    set_scalar_bytes(encoder, 7, &vol_dt);
    set_scalar_bytes(encoder, 8, &discount);
    set_scalar_bytes(encoder, 9, &seed);

    let threads_per_group = MTLSize {
        width: METAL_THREADGROUP_WIDTH as u64,
        height: 1,
        depth: 1,
    };
    let thread_groups = MTLSize {
        width: reduction_group_count(cfg.n_paths) as u64,
        height: 1,
        depth: 1,
    };
    encoder.dispatch_thread_groups(thread_groups, threads_per_group);
    encoder.end_encoding();
}

#[cfg(all(feature = "metal-native", target_os = "macos"))]
fn encode_reduction_pass(
    command_buffer: &CommandBufferRef,
    pipeline: &ComputePipelineState,
    input_buffer: &metal::Buffer,
    output_buffer: &metal::Buffer,
    n_values: usize,
) {
    let encoder = command_buffer.new_compute_command_encoder();
    encoder.set_compute_pipeline_state(pipeline);
    encoder.set_buffer(0, Some(input_buffer), 0);
    encoder.set_buffer(1, Some(output_buffer), 0);
    let n_values_i32 = n_values as i32;
    set_scalar_bytes(encoder, 2, &n_values_i32);

    let threads_per_group = MTLSize {
        width: METAL_THREADGROUP_WIDTH as u64,
        height: 1,
        depth: 1,
    };
    let thread_groups = MTLSize {
        width: reduction_group_count(n_values) as u64,
        height: 1,
        depth: 1,
    };
    encoder.dispatch_thread_groups(thread_groups, threads_per_group);
    encoder.end_encoding();
}

#[cfg(all(feature = "metal-native", target_os = "macos"))]
fn set_scalar_bytes<T>(encoder: &ComputeCommandEncoderRef, index: u64, value: &T) {
    encoder.set_bytes(index, size_of::<T>() as u64, (value as *const T).cast());
}

fn reduction_group_count(n_values: usize) -> usize {
    n_values.div_ceil(METAL_THREADGROUP_WIDTH).max(1)
}

#[cfg(all(test, feature = "metal-native", target_os = "macos"))]
mod tests {
    use super::*;
    use crate::{
        runtime::cpu::{
            european_call_price_mc_stepwise_from_f32_normals,
            generate_stepwise_stateless_normals_f32,
        },
        EuropeanCallConfig,
    };

    #[test]
    fn native_metal_stepwise_kernel_matches_cpu_reference_from_same_normals() {
        let cfg = EuropeanCallConfig {
            n_paths: 4_096,
            n_steps: 32,
            seed: 404,
            n_threads: 1,
            technique: MonteCarloTechnique::Standard,
            ..EuropeanCallConfig::default()
        };
        let metal = execute_metal_stepwise_kernel(&cfg, None)
            .expect("native Metal stepwise kernel should execute successfully");
        let normals = generate_stepwise_stateless_normals_f32(cfg.seed, cfg.n_paths, cfg.n_steps);
        let cpu = european_call_price_mc_stepwise_from_f32_normals(&cfg, &normals);

        let price_error = (metal.price - cpu.price).abs();
        let stderr_error = (metal.stderr - cpu.stderr).abs();
        assert!(
            price_error <= 1e-3,
            "price mismatch too large: {price_error}"
        );
        assert!(
            stderr_error <= 1e-4,
            "stderr mismatch too large: {stderr_error}"
        );
    }

    #[test]
    fn native_metal_runtime_cache_supports_repeated_execution() {
        let cfg = EuropeanCallConfig {
            n_paths: 2_048,
            n_steps: 16,
            seed: 808,
            n_threads: 1,
            technique: MonteCarloTechnique::Standard,
            ..EuropeanCallConfig::default()
        };

        let first = execute_metal_stepwise_kernel(&cfg, None)
            .expect("first native Metal execution should succeed");
        let second = execute_metal_stepwise_kernel(&cfg, None)
            .expect("second native Metal execution should reuse cache and succeed");

        assert!((first.price - second.price).abs() <= 1e-6);
        assert!((first.stderr - second.stderr).abs() <= 1e-6);
    }
}

fn compile_metal_kernel_to_metallib(plan: &ExecutionPlan) -> Result<PathBuf, String> {
    let output_dir = staged_metal_output_dir();
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("unable to create Metal staging directory: {error}"))?;

    let source_path = output_dir.join(format!(
        "{FIRST_METAL_KERNEL_FAMILY}_{}paths_{}steps.metal",
        plan.n_paths, plan.n_steps
    ));
    let air_path = output_dir.join(format!(
        "{FIRST_METAL_KERNEL_FAMILY}_{}paths_{}steps.air",
        plan.n_paths, plan.n_steps
    ));
    let metallib_path = output_dir.join(format!(
        "{FIRST_METAL_KERNEL_FAMILY}_{}paths_{}steps.metallib",
        plan.n_paths, plan.n_steps
    ));

    fs::write(&source_path, FIRST_METAL_KERNEL_SOURCE)
        .map_err(|error| format!("unable to write staged Metal source: {error}"))?;

    let metal_output = Command::new("xcrun")
        .args([
            "-sdk",
            "macosx",
            "metal",
            source_path.to_string_lossy().as_ref(),
            "-o",
            air_path.to_string_lossy().as_ref(),
        ])
        .output()
        .map_err(|error| format!("failed to spawn metal compiler: {error}"))?;

    if !metal_output.status.success() {
        let stderr = String::from_utf8_lossy(&metal_output.stderr);
        return Err(stderr.trim().to_string());
    }

    let metallib_output = Command::new("xcrun")
        .args([
            "-sdk",
            "macosx",
            "metallib",
            air_path.to_string_lossy().as_ref(),
            "-o",
            metallib_path.to_string_lossy().as_ref(),
        ])
        .output()
        .map_err(|error| format!("failed to spawn metallib: {error}"))?;

    if !metallib_output.status.success() {
        let stderr = String::from_utf8_lossy(&metallib_output.stderr);
        return Err(stderr.trim().to_string());
    }

    Ok(metallib_path)
}

fn staged_metal_output_dir() -> PathBuf {
    std::env::temp_dir().join("mc-library").join("metal")
}
