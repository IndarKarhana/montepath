use mc_core::{
    builtin_backends, estimate_gpu_bytes_per_path, european_call_price_mc_cpu_stepwise,
    plan_gpu_chunking, AppleMetalBackend, ArtifactExecutionMode, BackendDecisionReport,
    BackendError, BackendExecutionInput, BackendId, DeviceInfo, EuropeanCallConfig, ExecutionPlan,
    FeatureSummary, GpuChunkingConfig, NvidiaCudaBackend, PlannerMode, RejectedBackend,
    RuntimeBackend, SupportLevel,
};
#[allow(unused_imports)]
use mc_core::{cuda_native_feature_enabled, metal_native_feature_enabled};

fn test_plan() -> ExecutionPlan {
    ExecutionPlan {
        backend: BackendId::CpuNative,
        planner_mode: PlannerMode::Balanced,
        n_paths: 200_000,
        n_steps: 64,
        features: FeatureSummary::default(),
        decision_report: BackendDecisionReport {
            selected_backend: BackendId::CpuNative,
            reasons: vec!["unit-test".to_string()],
            rejected_backends: vec![RejectedBackend {
                backend: BackendId::NvidiaCuda,
                reason: "unit-test".to_string(),
            }],
        },
    }
}

fn mock_cuda_device() -> DeviceInfo {
    DeviceInfo {
        device_id: "cuda:0".to_string(),
        backend_id: BackendId::NvidiaCuda,
        name: "Mock CUDA Device".to_string(),
        vendor: "nvidia".to_string(),
        memory_total_mb: Some(8_192),
        memory_free_mb: None,
        supports_float64: true,
        supports_unified_memory: false,
        max_threads_hint: 1024,
    }
}

fn mock_metal_device() -> DeviceInfo {
    DeviceInfo {
        device_id: "metal:0".to_string(),
        backend_id: BackendId::AppleMetal,
        name: "Mock Metal Device".to_string(),
        vendor: "apple".to_string(),
        memory_total_mb: Some(6_144),
        memory_free_mb: None,
        supports_float64: false,
        supports_unified_memory: true,
        max_threads_hint: 1024,
    }
}

#[test]
fn builtin_registry_contains_cpu_cuda_and_metal() {
    let mut ids = builtin_backends()
        .into_iter()
        .map(|backend| backend.backend_id())
        .collect::<Vec<_>>();
    ids.sort_by_key(|id| match id {
        BackendId::CpuNative => 0,
        BackendId::NvidiaCuda => 1,
        BackendId::AppleMetal => 2,
    });

    assert_eq!(
        ids,
        vec![
            BackendId::CpuNative,
            BackendId::NvidiaCuda,
            BackendId::AppleMetal
        ]
    );
}

#[test]
fn cuda_discovery_reports_consistent_device_metadata() {
    let backend = NvidiaCudaBackend::new();
    for device in backend.discover_devices() {
        assert_eq!(device.backend_id, BackendId::NvidiaCuda);
        assert!(device.device_id.starts_with("cuda:"));
    }
}

#[test]
fn metal_discovery_reports_consistent_device_metadata() {
    let backend = AppleMetalBackend::new();
    for device in backend.discover_devices() {
        assert_eq!(device.backend_id, BackendId::AppleMetal);
        assert!(device.device_id.starts_with("metal:"));
    }
}

#[test]
fn cuda_supports_reports_pending_execution_for_valid_device() {
    let backend = NvidiaCudaBackend::new();
    let report = backend.supports(&test_plan(), &mock_cuda_device());

    assert_eq!(report.support_level, SupportLevel::SupportedWithFallbacks);
    assert!(report
        .unsupported_features
        .contains(&"native_cuda_execution_not_implemented".to_string()));
}

#[test]
fn metal_supports_reports_native_v1_status_for_valid_device() {
    let backend = AppleMetalBackend::new();
    let report = backend.supports(&test_plan(), &mock_metal_device());

    assert_eq!(report.support_level, SupportLevel::SupportedWithFallbacks);
    if metal_native_feature_enabled() {
        assert!(!report
            .unsupported_features
            .contains(&"native_metal_execution_not_implemented".to_string()));
    } else {
        assert!(report
            .unsupported_features
            .contains(&"metal_native_feature_disabled".to_string()));
    }
}

#[test]
fn cuda_compile_succeeds_with_fallback_artifact() {
    let backend = NvidiaCudaBackend::new();
    let artifact = backend
        .compile(&test_plan(), &mock_cuda_device())
        .expect("cuda fallback compile should succeed");

    assert_eq!(artifact.backend_id, BackendId::NvidiaCuda);
    assert!(artifact.artifact_id.starts_with("cuda-fallback:"));
    assert_eq!(artifact.execution_mode, ArtifactExecutionMode::GpuFallback);
    let native = artifact
        .native_artifact
        .expect("cuda compile should produce native staging metadata");
    assert_eq!(native.kernel_family, "european_call_stepwise_v1");
    assert_eq!(native.entry_point, "mc_cuda_european_call_stepwise_v1");
    assert_eq!(native.feature_gate, "cuda-native");
    assert_eq!(
        native.source_module,
        "crates/mc-core/src/backend/kernels/european_call_stepwise_v1.cu"
    );
    if cuda_native_feature_enabled() {
        assert!(native.compile_requested);
        if native.compile_succeeded {
            assert!(native.compiled_module_path.is_some());
        }
    } else {
        assert!(!native.compile_requested);
        assert!(!native.compile_succeeded);
        assert!(native.compiled_module_path.is_none());
    }
}

#[test]
fn metal_compile_succeeds_with_fallback_artifact() {
    let backend = AppleMetalBackend::new();
    let artifact = backend
        .compile(&test_plan(), &mock_metal_device())
        .expect("metal fallback compile should succeed");

    assert_eq!(artifact.backend_id, BackendId::AppleMetal);
    assert!(artifact.artifact_id.starts_with("metal-fallback:"));
    assert_eq!(artifact.execution_mode, ArtifactExecutionMode::GpuFallback);
    let native = artifact
        .native_artifact
        .expect("metal compile should produce native staging metadata");
    assert_eq!(native.kernel_family, "european_call_stepwise_v1");
    assert_eq!(native.entry_point, "mc_metal_european_call_stepwise_v1");
    assert_eq!(native.feature_gate, "metal-native");
    assert_eq!(
        native.source_module,
        "crates/mc-core/src/backend/kernels/european_call_stepwise_v1.metal"
    );
    if metal_native_feature_enabled() {
        assert!(native.compile_requested);
        if native.compile_succeeded {
            assert!(native.compiled_module_path.is_some());
        }
    } else {
        assert!(!native.compile_requested);
        assert!(!native.compile_succeeded);
        assert!(native.compiled_module_path.is_none());
    }
}

#[test]
fn cuda_execute_runs_with_deterministic_fallback() {
    let backend = NvidiaCudaBackend::new();
    let artifact = backend
        .compile(&test_plan(), &mock_cuda_device())
        .expect("cuda fallback compile should succeed");
    let input = BackendExecutionInput::EuropeanCall(EuropeanCallConfig {
        n_paths: artifact.n_paths,
        n_steps: artifact.n_steps,
        seed: 77,
        n_threads: 2,
        ..EuropeanCallConfig::default()
    });

    let run1 = backend
        .execute(&artifact, &input)
        .expect("cuda fallback execute should succeed");
    let run2 = backend
        .execute(&artifact, &input)
        .expect("cuda fallback execute should succeed");

    assert_eq!(run1.price, run2.price);
    assert_eq!(run1.stderr, run2.stderr);
}

#[test]
fn metal_execute_runs_with_deterministic_results() {
    let backend = AppleMetalBackend::new();
    let artifact = backend
        .compile(&test_plan(), &mock_metal_device())
        .expect("metal fallback compile should succeed");
    let input = BackendExecutionInput::EuropeanCall(EuropeanCallConfig {
        n_paths: artifact.n_paths,
        n_steps: artifact.n_steps,
        seed: 88,
        n_threads: 2,
        ..EuropeanCallConfig::default()
    });

    let run1 = backend
        .execute(&artifact, &input)
        .expect("metal fallback execute should succeed");
    let run2 = backend
        .execute(&artifact, &input)
        .expect("metal fallback execute should succeed");

    assert_eq!(run1.price, run2.price);
    assert_eq!(run1.stderr, run2.stderr);
}

#[test]
fn gpu_fallback_execute_rejects_mismatched_workload_shape() {
    let backend = NvidiaCudaBackend::new();
    let artifact = backend
        .compile(&test_plan(), &mock_cuda_device())
        .expect("cuda fallback compile should succeed");

    let err = backend
        .execute(
            &artifact,
            &BackendExecutionInput::EuropeanCall(EuropeanCallConfig {
                n_paths: artifact.n_paths + 1,
                n_steps: artifact.n_steps,
                ..EuropeanCallConfig::default()
            }),
        )
        .expect_err("mismatched workload should be rejected");

    assert!(matches!(err, BackendError::IncompatibleExecutionInput));
}

#[test]
fn cuda_fallback_matches_cpu_stepwise_reference() {
    let backend = NvidiaCudaBackend::new();
    let artifact = backend
        .compile(&test_plan(), &mock_cuda_device())
        .expect("cuda fallback compile should succeed");
    let cfg = EuropeanCallConfig {
        n_paths: artifact.n_paths,
        n_steps: artifact.n_steps,
        seed: 5_001,
        n_threads: 4,
        ..EuropeanCallConfig::default()
    };

    let backend_run = backend
        .execute(&artifact, &BackendExecutionInput::EuropeanCall(cfg))
        .expect("cuda fallback execute should succeed");
    let cpu_run = european_call_price_mc_cpu_stepwise(&cfg);

    let error = (backend_run.price - cpu_run.price).abs();
    let tolerance = 6.0 * backend_run.stderr.max(cpu_run.stderr);
    assert!(
        error <= tolerance + 1e-12,
        "cuda fallback should remain statistically consistent with cpu reference: backend={} cpu={} tolerance={}",
        backend_run.price,
        cpu_run.price,
        tolerance
    );
}

#[test]
fn metal_fallback_matches_cpu_stepwise_reference() {
    let backend = AppleMetalBackend::new();
    let artifact = backend
        .compile(&test_plan(), &mock_metal_device())
        .expect("metal fallback compile should succeed");
    let cfg = EuropeanCallConfig {
        n_paths: artifact.n_paths,
        n_steps: artifact.n_steps,
        seed: 5_002,
        n_threads: 4,
        ..EuropeanCallConfig::default()
    };

    let backend_run = backend
        .execute(&artifact, &BackendExecutionInput::EuropeanCall(cfg))
        .expect("metal fallback execute should succeed");
    let cpu_run = european_call_price_mc_cpu_stepwise(&cfg);

    let error = (backend_run.price - cpu_run.price).abs();
    let tolerance = 6.0 * backend_run.stderr.max(cpu_run.stderr);
    assert!(
        error <= tolerance + 1e-12,
        "metal fallback should remain statistically consistent with cpu reference: backend={} cpu={} tolerance={}",
        backend_run.price,
        cpu_run.price,
        tolerance
    );
}

#[test]
fn metal_backend_antithetic_matches_cpu_stepwise_reference() {
    let backend = AppleMetalBackend::new();
    let artifact = backend
        .compile(&test_plan(), &mock_metal_device())
        .expect("metal compile should succeed");
    let cfg = EuropeanCallConfig {
        n_paths: artifact.n_paths,
        n_steps: artifact.n_steps,
        seed: 5_102,
        n_threads: 4,
        technique: mc_core::MonteCarloTechnique::Antithetic,
        ..EuropeanCallConfig::default()
    };

    let backend_run = backend
        .execute(&artifact, &BackendExecutionInput::EuropeanCall(cfg))
        .expect("metal execute should succeed");
    let cpu_run = european_call_price_mc_cpu_stepwise(&cfg);

    let error = (backend_run.price - cpu_run.price).abs();
    let tolerance = 6.0 * backend_run.stderr.max(cpu_run.stderr);
    assert!(error <= tolerance + 1e-12);
}

#[test]
fn metal_backend_control_variate_matches_cpu_stepwise_reference() {
    let backend = AppleMetalBackend::new();
    let artifact = backend
        .compile(&test_plan(), &mock_metal_device())
        .expect("metal compile should succeed");
    let cfg = EuropeanCallConfig {
        n_paths: artifact.n_paths,
        n_steps: artifact.n_steps,
        seed: 5_103,
        n_threads: 4,
        technique: mc_core::MonteCarloTechnique::ControlVariate,
        ..EuropeanCallConfig::default()
    };

    let backend_run = backend
        .execute(&artifact, &BackendExecutionInput::EuropeanCall(cfg))
        .expect("metal execute should succeed");
    let cpu_run = european_call_price_mc_cpu_stepwise(&cfg);

    let error = (backend_run.price - cpu_run.price).abs();
    let tolerance = 6.0 * backend_run.stderr.max(cpu_run.stderr);
    assert!(error <= tolerance + 1e-12);
}

#[test]
fn gpu_chunk_plan_respects_memory_budget() {
    let plan = plan_gpu_chunking(
        1_000_000,
        Some(2_048),
        GpuChunkingConfig {
            bytes_per_path: 4_096,
            target_utilization: 0.75,
            minimum_paths_per_chunk: 16_384,
            fallback_budget_mb: 4_096,
        },
    );

    assert_eq!(plan.total_paths, 1_000_000);
    assert!(plan.chunk_count > 1);
    assert!(plan.estimated_peak_memory_mb <= 2_048);
}

#[test]
fn estimate_gpu_bytes_per_path_scales_with_steps() {
    let mut small = test_plan();
    small.n_steps = 16;
    let mut large = test_plan();
    large.n_steps = 512;

    let small_bytes = estimate_gpu_bytes_per_path(&small);
    let large_bytes = estimate_gpu_bytes_per_path(&large);

    assert!(small_bytes > 0);
    assert!(large_bytes >= small_bytes);
}

#[test]
fn cuda_and_metal_kernel_contracts_share_launch_shape() {
    let cuda_backend = NvidiaCudaBackend::new();
    let metal_backend = AppleMetalBackend::new();
    let cuda_artifact = cuda_backend
        .compile(&test_plan(), &mock_cuda_device())
        .expect("cuda compile should succeed");
    let metal_artifact = metal_backend
        .compile(&test_plan(), &mock_metal_device())
        .expect("metal compile should succeed");

    let cuda_contract = cuda_artifact
        .native_artifact
        .expect("cuda native metadata missing")
        .kernel_contract
        .expect("cuda kernel contract missing");
    let metal_contract = metal_artifact
        .native_artifact
        .expect("metal native metadata missing")
        .kernel_contract
        .expect("metal kernel contract missing");

    assert_eq!(cuda_contract.kernel_family, metal_contract.kernel_family);
    assert_eq!(cuda_contract.launch.logical_threads, test_plan().n_paths);
    assert_eq!(metal_contract.launch.logical_threads, test_plan().n_paths);
    assert_eq!(
        cuda_contract.launch.threadgroups_x,
        metal_contract.launch.threadgroups_x
    );
}

#[test]
fn gpu_kernel_contracts_match_expected_buffer_roles() {
    let cuda_backend = NvidiaCudaBackend::new();
    let artifact = cuda_backend
        .compile(&test_plan(), &mock_cuda_device())
        .expect("cuda compile should succeed");
    let contract = artifact
        .native_artifact
        .expect("cuda native metadata missing")
        .kernel_contract
        .expect("cuda kernel contract missing");

    assert_eq!(contract.buffers.len(), 2);
    assert_eq!(contract.buffers[0].name, "normals");
    assert_eq!(
        contract.buffers[0].element_count,
        test_plan().n_paths * test_plan().n_steps
    );
    assert_eq!(contract.buffers[1].name, "payoffs");
    assert_eq!(contract.buffers[1].element_count, test_plan().n_paths);
    assert_eq!(contract.scalars.len(), 7);
    assert_eq!(contract.scalars[0].name, "n_paths");
    assert_eq!(contract.scalars[1].name, "n_steps");
}

#[cfg(not(any(feature = "cuda-native", feature = "metal-native")))]
#[test]
fn default_build_reports_native_feature_gates_disabled() {
    assert!(!cuda_native_feature_enabled());
    assert!(!metal_native_feature_enabled());
}

#[cfg(feature = "cuda-native")]
#[test]
fn cuda_native_feature_gate_reports_enabled_when_requested() {
    assert!(cuda_native_feature_enabled());
}

#[cfg(feature = "cuda-native")]
#[test]
fn cuda_native_compile_records_requested_status() {
    let backend = NvidiaCudaBackend::new();
    let artifact = backend
        .compile(&test_plan(), &mock_cuda_device())
        .expect("cuda compile should succeed with native staging metadata");
    let native = artifact
        .native_artifact
        .expect("cuda compile should include native metadata");

    assert!(native.compile_requested);
    if native.compile_succeeded {
        assert!(native.compiled_module_path.is_some());
    }
}

#[cfg(feature = "metal-native")]
#[test]
fn metal_native_feature_gate_reports_enabled_when_requested() {
    assert!(metal_native_feature_enabled());
}

#[cfg(feature = "metal-native")]
#[test]
fn metal_native_compile_records_requested_status() {
    let backend = AppleMetalBackend::new();
    let artifact = backend
        .compile(&test_plan(), &mock_metal_device())
        .expect("metal compile should succeed with native staging metadata");
    let native = artifact
        .native_artifact
        .expect("metal compile should include native metadata");

    assert!(native.compile_requested);
    if native.compile_succeeded {
        assert!(native.compiled_module_path.is_some());
    }
}
