use mc_core::{
    builtin_backends, estimate_gpu_bytes_per_path, plan_gpu_chunking, AppleMetalBackend,
    BackendDecisionReport, BackendError, BackendExecutionInput, BackendId, CompiledArtifact,
    DeviceInfo, EuropeanCallConfig, ExecutionPlan, FeatureSummary, GpuChunkingConfig,
    NvidiaCudaBackend, PlannerMode, RejectedBackend, RuntimeBackend, SupportLevel,
};

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

    assert_eq!(report.support_level, SupportLevel::Unsupported);
    assert!(report
        .unsupported_features
        .contains(&"execution_not_implemented".to_string()));
}

#[test]
fn metal_supports_reports_pending_execution_for_valid_device() {
    let backend = AppleMetalBackend::new();
    let report = backend.supports(&test_plan(), &mock_metal_device());

    assert_eq!(report.support_level, SupportLevel::Unsupported);
    assert!(report
        .unsupported_features
        .contains(&"execution_not_implemented".to_string()));
}

#[test]
fn cuda_compile_returns_not_implemented_error() {
    let backend = NvidiaCudaBackend::new();
    let err = backend
        .compile(&test_plan(), &mock_cuda_device())
        .expect_err("cuda compile should return unsupported while kernels are not implemented");

    assert!(matches!(err, BackendError::UnsupportedFeature(_)));
}

#[test]
fn metal_compile_returns_not_implemented_error() {
    let backend = AppleMetalBackend::new();
    let err = backend
        .compile(&test_plan(), &mock_metal_device())
        .expect_err("metal compile should return unsupported while kernels are not implemented");

    assert!(matches!(err, BackendError::UnsupportedFeature(_)));
}

#[test]
fn cuda_execute_returns_not_implemented_error() {
    let backend = NvidiaCudaBackend::new();
    let artifact = CompiledArtifact {
        artifact_id: "cuda-dummy".to_string(),
        backend_id: BackendId::NvidiaCuda,
        device_id: "cuda:0".to_string(),
        n_paths: 10,
        n_steps: 10,
        planner_mode: PlannerMode::Balanced,
    };

    let err = backend
        .execute(
            &artifact,
            &BackendExecutionInput::EuropeanCall(EuropeanCallConfig::default()),
        )
        .expect_err("cuda execute should return unsupported while kernels are not implemented");

    assert!(matches!(err, BackendError::UnsupportedFeature(_)));
}

#[test]
fn metal_execute_returns_not_implemented_error() {
    let backend = AppleMetalBackend::new();
    let artifact = CompiledArtifact {
        artifact_id: "metal-dummy".to_string(),
        backend_id: BackendId::AppleMetal,
        device_id: "metal:0".to_string(),
        n_paths: 10,
        n_steps: 10,
        planner_mode: PlannerMode::Balanced,
    };

    let err = backend
        .execute(
            &artifact,
            &BackendExecutionInput::EuropeanCall(EuropeanCallConfig::default()),
        )
        .expect_err("metal execute should return unsupported while kernels are not implemented");

    assert!(matches!(err, BackendError::UnsupportedFeature(_)));
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
