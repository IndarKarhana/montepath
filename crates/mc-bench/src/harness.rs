use std::collections::BTreeMap;
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use mc_core::{
    arithmetic_asian_call_price_mc_cpu, european_call_price_mc_cpu_stepwise,
    european_call_price_mc_cpu_terminal, plan_execution, ArithmeticAsianCallConfig, BackendId,
    BackendPreference, BackendSupportReport, EuropeanCallConfig, MonteCarloTechnique, PlannerMode,
    RunConfig,
};
#[cfg(feature = "metal-native")]
use mc_core::{
    AppleMetalBackend, BackendDecisionReport, BackendExecutionInput, ExecutionPlan, FeatureSummary,
    RuntimeBackend,
};
use mc_schema::{
    validate_simulation_spec, AxisKind, AxisSpec, Expr, ObservationSpec, ParameterSpec,
    RandomVarSpec, ReductionSpec, SimulationSpec, StateUpdate, StateVarSpec, StepSpec,
};
use serde::Deserialize;

use crate::result::{BenchmarkReport, BenchmarkResult};

const MC_PATHS: usize = 100_000;
const MC_STEPS: usize = 64;
const MC_REPEATS: usize = 3;

pub fn run_default_benchmarks() -> BenchmarkReport {
    let spec = sample_spec(false);

    let mut results = vec![
        benchmark_schema_validation(&spec, 10_000),
        benchmark_planner_overhead(&spec, 10_000),
        benchmark_planner_choice_accuracy(),
        benchmark_planner_choice_accuracy_measured(),
        benchmark_mc_rust_cpu_stepwise(MC_REPEATS),
        benchmark_mc_rust_cpu_stepwise_antithetic(MC_REPEATS),
        benchmark_mc_rust_cpu_stepwise_antithetic_quality(),
        benchmark_mc_rust_cpu_stepwise_control_variate(MC_REPEATS),
        benchmark_mc_rust_cpu_stepwise_control_variate_quality(),
        benchmark_mc_rust_cpu_terminal(MC_REPEATS),
        benchmark_mc_rust_cpu_terminal_antithetic(MC_REPEATS),
        benchmark_mc_rust_cpu_terminal_antithetic_quality(),
        benchmark_mc_rust_cpu_terminal_control_variate(MC_REPEATS),
        benchmark_mc_rust_cpu_terminal_control_variate_quality(),
        benchmark_mc_rust_cpu_arithmetic_asian_stepwise(MC_REPEATS),
        benchmark_mc_rust_cpu_arithmetic_asian_stepwise_control_variate(MC_REPEATS),
        benchmark_mc_rust_cpu_arithmetic_asian_stepwise_control_variate_quality(),
    ];

    if let Some(metal_result) = benchmark_mc_native_metal_stepwise(MC_REPEATS) {
        results.push(metal_result);
    }
    if let Some(metal_result) = benchmark_mc_native_metal_stepwise_antithetic(MC_REPEATS) {
        results.push(metal_result);
    }
    if let Some(metal_quality) = benchmark_mc_native_metal_stepwise_antithetic_quality() {
        results.push(metal_quality);
    }
    if let Some(metal_result) = benchmark_mc_native_metal_stepwise_control_variate(MC_REPEATS) {
        results.push(metal_result);
    }
    if let Some(metal_quality) = benchmark_mc_native_metal_stepwise_control_variate_quality() {
        results.push(metal_quality);
    }
    if let Some(metal_result) = benchmark_mc_native_metal_arithmetic_asian_stepwise(MC_REPEATS) {
        results.push(metal_result);
    }
    if let Some(metal_result) =
        benchmark_mc_native_metal_arithmetic_asian_stepwise_control_variate(MC_REPEATS)
    {
        results.push(metal_result);
    }
    if let Some(metal_quality) =
        benchmark_mc_native_metal_arithmetic_asian_stepwise_control_variate_quality()
    {
        results.push(metal_quality);
    }

    results.extend(benchmark_python_competitors(
        MC_PATHS, MC_STEPS, MC_REPEATS, 42,
    ));

    BenchmarkReport {
        generated_at_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_millis(),
        results,
    }
}

pub fn build_competitiveness_plan(report: &BenchmarkReport) -> String {
    let rust = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_european_call_rust")
        .and_then(|r| r.runtime_ms())
        .unwrap_or(f64::INFINITY);

    let mut competitor_rows = report
        .results
        .iter()
        .filter(|r| {
            r.benchmark_name == "mc_cpu_european_call_numpy"
                || r.benchmark_name == "mc_cpu_european_call_numba"
        })
        .filter_map(|r| {
            r.runtime_ms().map(|runtime| {
                (
                    r.benchmark_name.clone(),
                    runtime,
                    if rust.is_finite() {
                        rust / runtime
                    } else {
                        0.0
                    },
                )
            })
        })
        .collect::<Vec<_>>();

    competitor_rows.sort_by(|a, b| a.1.total_cmp(&b.1));

    let mut out = String::new();
    out.push_str("# Competitiveness Plan\n\n");

    if !rust.is_finite() {
        out.push_str("Rust Monte Carlo benchmark result missing. Run benchmark harness first.\n");
        return out;
    }

    out.push_str(&format!(
        "Current Rust fair baseline (`mc_cpu_european_call_rust`, step-wise): `{:.3} ms`\n\n",
        rust
    ));

    let slower_than = competitor_rows
        .iter()
        .filter(|(_, runtime, _)| rust > *runtime)
        .collect::<Vec<_>>();

    if slower_than.is_empty() {
        out.push_str("Status: Rust currently leads available CPU baselines for this workload.\n\n");
        out.push_str("Maintain lead plan:\n");
        out.push_str("- Keep the step-wise benchmark as the primary competitive claim.\n");
        out.push_str("- Keep RNG and loop hot path allocation-free.\n");
        out.push_str("- Add release-mode benchmark gates for MC runtime.\n");
        out.push_str("- Expand competitor matrix to GPU baselines (JAX/CuPy/PyTorch) when hardware is available.\n");
        if let Some(terminal_runtime) = report
            .results
            .iter()
            .find(|r| r.benchmark_name == "mc_cpu_european_call_rust_terminal")
            .and_then(|r| r.runtime_ms())
        {
            out.push_str(&format!(
                "- Preserve the specialized terminal-distribution fast path (`{:.3} ms`) as a separate optimization track.\n",
                terminal_runtime
            ));
        }
        return out;
    }

    out.push_str("Status: Rust is slower than at least one available baseline.\n\n");
    out.push_str("Observed gaps:\n");
    for (name, runtime, ratio) in &slower_than {
        out.push_str(&format!(
            "- `{name}` is faster: `{:.3} ms` vs Rust `{:.3} ms` (Rust is `{:.2}x` slower)\n",
            runtime, rust, ratio
        ));
    }

    out.push_str("\nAction plan to close the gap:\n");
    out.push_str("- Optimize the fair step-wise kernel before tuning specialized fast paths.\n");
    out.push_str(
        "- Introduce SIMD-friendly normal generation and batched exponentials in CPU runtime.\n",
    );
    out.push_str(
        "- Keep deterministic multithreaded path partitioning with stable reduction order.\n",
    );
    out.push_str("- Benchmark release profile (`--release`) and optimize hottest functions with profiler evidence.\n");
    out.push_str("- Keep workload-specialized kernels as explicit secondary benchmarks, not as the sole competitiveness claim.\n");

    out
}

fn benchmark_schema_validation(spec: &SimulationSpec, iterations: usize) -> BenchmarkResult {
    let started = Instant::now();

    for _ in 0..iterations {
        let diagnostics = validate_simulation_spec(spec);
        if !diagnostics.is_empty() {
            panic!("expected no diagnostics in validation benchmark: {diagnostics:?}");
        }
    }

    let elapsed = started.elapsed();

    BenchmarkResult {
        benchmark_name: "schema_validation".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-schema::validate_simulation_spec".to_string(),
        backend: "cpu_native".to_string(),
        methodology: None,
        planner_mode: "n/a".to_string(),
        iterations,
        total_runtime_ms: elapsed.as_secs_f64() * 1_000.0,
        per_iteration_us: elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64,
        throughput_per_sec: throughput(iterations, elapsed.as_secs_f64()),
        metric_name: None,
        metric_value: None,
    }
}

fn benchmark_planner_overhead(spec: &SimulationSpec, iterations: usize) -> BenchmarkResult {
    let support = vec![
        BackendSupportReport::supported(BackendId::CpuNative),
        BackendSupportReport::supported(BackendId::NvidiaCuda),
        BackendSupportReport::supported(BackendId::AppleMetal),
    ];

    let started = Instant::now();

    for _ in 0..iterations {
        let plan = plan_execution(
            spec,
            RunConfig {
                n_paths: 1_000_000,
                n_steps: 252,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            &support,
        )
        .expect("planner benchmark should produce an execution plan");

        if plan.backend != BackendId::NvidiaCuda {
            panic!("expected planner to choose nvidia in benchmark scenario");
        }
    }

    let elapsed = started.elapsed();

    BenchmarkResult {
        benchmark_name: "planner_overhead_auto".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::plan_execution".to_string(),
        backend: "planner".to_string(),
        methodology: None,
        planner_mode: "balanced".to_string(),
        iterations,
        total_runtime_ms: elapsed.as_secs_f64() * 1_000.0,
        per_iteration_us: elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64,
        throughput_per_sec: throughput(iterations, elapsed.as_secs_f64()),
        metric_name: None,
        metric_value: None,
    }
}

fn benchmark_planner_choice_accuracy() -> BenchmarkResult {
    #[derive(Clone)]
    struct Scenario {
        spec: SimulationSpec,
        run_config: RunConfig,
        support: Vec<BackendSupportReport>,
        expected: BackendId,
    }

    let scenarios = vec![
        Scenario {
            spec: sample_spec(false),
            run_config: RunConfig {
                n_paths: 10_000,
                n_steps: 50,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            support: vec![
                BackendSupportReport::supported(BackendId::CpuNative),
                BackendSupportReport::supported(BackendId::NvidiaCuda),
                BackendSupportReport::supported(BackendId::AppleMetal),
            ],
            expected: BackendId::CpuNative,
        },
        Scenario {
            spec: sample_spec(false),
            run_config: RunConfig {
                n_paths: 100_000,
                n_steps: 64,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            support: vec![
                BackendSupportReport::supported(BackendId::CpuNative),
                BackendSupportReport::supported(BackendId::NvidiaCuda),
                BackendSupportReport::supported(BackendId::AppleMetal),
            ],
            expected: BackendId::AppleMetal,
        },
        Scenario {
            spec: sample_spec(false),
            run_config: RunConfig {
                n_paths: 1_000_000,
                n_steps: 252,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            support: vec![
                BackendSupportReport::supported(BackendId::CpuNative),
                BackendSupportReport::supported(BackendId::NvidiaCuda),
                BackendSupportReport::supported(BackendId::AppleMetal),
            ],
            expected: BackendId::NvidiaCuda,
        },
        Scenario {
            spec: sample_spec(true),
            run_config: RunConfig {
                n_paths: 1_000_000,
                n_steps: 252,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            support: vec![
                BackendSupportReport::supported(BackendId::CpuNative),
                BackendSupportReport::supported(BackendId::NvidiaCuda),
                BackendSupportReport::supported(BackendId::AppleMetal),
            ],
            expected: BackendId::CpuNative,
        },
        Scenario {
            spec: sample_spec(false),
            run_config: RunConfig {
                n_paths: 1_000_000,
                n_steps: 252,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            support: vec![
                BackendSupportReport::supported(BackendId::CpuNative),
                BackendSupportReport::unsupported(BackendId::NvidiaCuda, "cuda unavailable"),
                BackendSupportReport::supported(BackendId::AppleMetal),
            ],
            expected: BackendId::AppleMetal,
        },
    ];

    let iterations = scenarios.len();
    let started = Instant::now();
    let mut correct = 0usize;

    for scenario in &scenarios {
        let plan = plan_execution(
            &scenario.spec,
            scenario.run_config.clone(),
            &scenario.support,
        )
        .expect("planner scenario should produce execution plan");
        if plan.backend == scenario.expected {
            correct += 1;
        }
    }

    let elapsed = started.elapsed();

    BenchmarkResult {
        benchmark_name: "planner_choice_accuracy".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::plan_execution".to_string(),
        backend: "planner".to_string(),
        methodology: None,
        planner_mode: "balanced".to_string(),
        iterations,
        total_runtime_ms: elapsed.as_secs_f64() * 1_000.0,
        per_iteration_us: elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64,
        throughput_per_sec: throughput(iterations, elapsed.as_secs_f64()),
        metric_name: Some("accuracy_pct".to_string()),
        metric_value: Some((correct as f64 / iterations as f64) * 100.0),
    }
}

fn benchmark_planner_choice_accuracy_measured() -> BenchmarkResult {
    #[derive(Clone)]
    struct Scenario {
        spec: SimulationSpec,
        run_config: RunConfig,
        support: Vec<BackendSupportReport>,
        measured_winner: BackendId,
    }

    let metal_supported = measured_metal_is_available();
    let local_support = |metal_supported: bool| -> Vec<BackendSupportReport> {
        vec![
            BackendSupportReport::supported(BackendId::CpuNative),
            BackendSupportReport::unsupported(
                BackendId::NvidiaCuda,
                "no measured CUDA data on this machine",
            ),
            if metal_supported {
                BackendSupportReport::supported(BackendId::AppleMetal)
            } else {
                BackendSupportReport::unsupported(
                    BackendId::AppleMetal,
                    "native Metal benchmark path unavailable on this machine",
                )
            },
        ]
    };

    let conditional_support = vec![
        BackendSupportReport::supported(BackendId::CpuNative),
        BackendSupportReport::unsupported(
            BackendId::NvidiaCuda,
            "no measured CUDA data on this machine",
        ),
        BackendSupportReport::unsupported(
            BackendId::AppleMetal,
            "conditional-heavy workload not yet calibrated for native Metal",
        ),
    ];

    let scenarios = vec![
        Scenario {
            spec: sample_spec(false),
            run_config: RunConfig {
                n_paths: 10_000,
                n_steps: 50,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            support: local_support(metal_supported),
            measured_winner: measured_winner_for_local_backends(
                10_000,
                50,
                MonteCarloTechnique::Standard,
                9_001,
                metal_supported,
            ),
        },
        Scenario {
            spec: sample_spec(false),
            run_config: RunConfig {
                n_paths: 100_000,
                n_steps: 64,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            support: local_support(metal_supported),
            measured_winner: measured_winner_for_local_backends(
                100_000,
                64,
                MonteCarloTechnique::Standard,
                9_002,
                metal_supported,
            ),
        },
        Scenario {
            spec: sample_spec(false),
            run_config: RunConfig {
                n_paths: 100_000,
                n_steps: 64,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            support: local_support(metal_supported),
            measured_winner: measured_winner_for_local_backends(
                100_000,
                64,
                MonteCarloTechnique::Antithetic,
                9_003,
                metal_supported,
            ),
        },
        Scenario {
            spec: sample_spec(false),
            run_config: RunConfig {
                n_paths: 100_000,
                n_steps: 64,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            support: local_support(metal_supported),
            measured_winner: measured_winner_for_local_backends(
                100_000,
                64,
                MonteCarloTechnique::ControlVariate,
                9_004,
                metal_supported,
            ),
        },
        Scenario {
            spec: sample_spec(false),
            run_config: RunConfig {
                n_paths: 100_000,
                n_steps: 64,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            support: local_support(metal_supported),
            measured_winner: measured_winner_for_local_backends_asian(
                100_000,
                64,
                MonteCarloTechnique::ControlVariate,
                9_005,
                metal_supported,
            ),
        },
        Scenario {
            spec: sample_spec(true),
            run_config: RunConfig {
                n_paths: 100_000,
                n_steps: 64,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            support: conditional_support,
            measured_winner: BackendId::CpuNative,
        },
    ];

    let iterations = scenarios.len();
    let started = Instant::now();
    let mut correct = 0usize;

    for scenario in &scenarios {
        let plan = plan_execution(
            &scenario.spec,
            scenario.run_config.clone(),
            &scenario.support,
        )
        .expect("measured planner scenario should produce execution plan");

        if plan.backend == scenario.measured_winner {
            correct += 1;
        }
    }

    let elapsed = started.elapsed();

    BenchmarkResult {
        benchmark_name: "planner_choice_accuracy_measured".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::plan_execution".to_string(),
        backend: "planner".to_string(),
        methodology: Some("measured_local_backend_winners".to_string()),
        planner_mode: "balanced".to_string(),
        iterations,
        total_runtime_ms: elapsed.as_secs_f64() * 1_000.0,
        per_iteration_us: elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64,
        throughput_per_sec: throughput(iterations, elapsed.as_secs_f64()),
        metric_name: Some("accuracy_pct".to_string()),
        metric_value: Some((correct as f64 / iterations as f64) * 100.0),
    }
}

fn measured_winner_for_local_backends(
    n_paths: usize,
    n_steps: usize,
    technique: MonteCarloTechnique,
    seed: u64,
    metal_supported: bool,
) -> BackendId {
    let cpu_runtime_ms = measure_cpu_stepwise_runtime_ms(n_paths, n_steps, technique, seed);

    if !metal_supported {
        return BackendId::CpuNative;
    }

    match measure_metal_stepwise_runtime_ms(n_paths, n_steps, technique, seed) {
        Some(metal_runtime_ms) if metal_runtime_ms < cpu_runtime_ms => BackendId::AppleMetal,
        _ => BackendId::CpuNative,
    }
}

fn measure_cpu_stepwise_runtime_ms(
    n_paths: usize,
    n_steps: usize,
    technique: MonteCarloTechnique,
    seed: u64,
) -> f64 {
    let cfg = EuropeanCallConfig {
        n_paths,
        n_steps,
        seed,
        technique,
        ..EuropeanCallConfig::default()
    };

    let started = Instant::now();
    let _ = european_call_price_mc_cpu_stepwise(&cfg);
    started.elapsed().as_secs_f64() * 1_000.0
}

fn measured_winner_for_local_backends_asian(
    n_paths: usize,
    n_steps: usize,
    technique: MonteCarloTechnique,
    seed: u64,
    metal_supported: bool,
) -> BackendId {
    let cpu_runtime_ms = measure_cpu_asian_runtime_ms(n_paths, n_steps, technique, seed);

    if !metal_supported {
        return BackendId::CpuNative;
    }

    match measure_metal_asian_runtime_ms(n_paths, n_steps, technique, seed) {
        Some(metal_runtime_ms) if metal_runtime_ms < cpu_runtime_ms => BackendId::AppleMetal,
        _ => BackendId::CpuNative,
    }
}

fn measure_cpu_asian_runtime_ms(
    n_paths: usize,
    n_steps: usize,
    technique: MonteCarloTechnique,
    seed: u64,
) -> f64 {
    let cfg = ArithmeticAsianCallConfig {
        n_paths,
        n_steps,
        seed,
        technique,
        ..ArithmeticAsianCallConfig::default()
    };

    let started = Instant::now();
    let _ = arithmetic_asian_call_price_mc_cpu(&cfg);
    started.elapsed().as_secs_f64() * 1_000.0
}

fn measured_metal_is_available() -> bool {
    #[cfg(not(feature = "metal-native"))]
    {
        false
    }

    #[cfg(feature = "metal-native")]
    {
        let backend = AppleMetalBackend::new();
        !backend.discover_devices().is_empty()
    }
}

fn measure_metal_stepwise_runtime_ms(
    n_paths: usize,
    n_steps: usize,
    technique: MonteCarloTechnique,
    seed: u64,
) -> Option<f64> {
    #[cfg(not(feature = "metal-native"))]
    {
        let _ = (n_paths, n_steps, technique, seed);
        None
    }

    #[cfg(feature = "metal-native")]
    {
        let backend = AppleMetalBackend::new();
        let mut devices = backend.discover_devices();
        let device = devices.pop()?;
        let plan = ExecutionPlan {
            backend: BackendId::AppleMetal,
            planner_mode: PlannerMode::Balanced,
            n_paths,
            n_steps,
            features: FeatureSummary::default(),
            decision_report: BackendDecisionReport {
                selected_backend: BackendId::AppleMetal,
                reasons: vec!["measured planner calibration".to_string()],
                rejected_backends: Vec::new(),
            },
        };
        let artifact = backend.compile(&plan, &device).ok()?;
        let warmup_cfg = EuropeanCallConfig {
            n_paths,
            n_steps,
            seed: seed.saturating_sub(1),
            technique,
            ..EuropeanCallConfig::default()
        };
        let _ = backend.execute(&artifact, &BackendExecutionInput::EuropeanCall(warmup_cfg));

        let cfg = EuropeanCallConfig {
            n_paths,
            n_steps,
            seed,
            technique,
            ..EuropeanCallConfig::default()
        };
        let result = backend
            .execute(&artifact, &BackendExecutionInput::EuropeanCall(cfg))
            .ok()?;
        Some(result.runtime_ms)
    }
}

fn measure_metal_asian_runtime_ms(
    n_paths: usize,
    n_steps: usize,
    technique: MonteCarloTechnique,
    seed: u64,
) -> Option<f64> {
    #[cfg(not(feature = "metal-native"))]
    {
        let _ = (n_paths, n_steps, technique, seed);
        None
    }

    #[cfg(feature = "metal-native")]
    {
        let backend = AppleMetalBackend::new();
        let mut devices = backend.discover_devices();
        let device = devices.pop()?;
        let plan = ExecutionPlan {
            backend: BackendId::AppleMetal,
            planner_mode: PlannerMode::Balanced,
            n_paths,
            n_steps,
            features: FeatureSummary::default(),
            decision_report: BackendDecisionReport {
                selected_backend: BackendId::AppleMetal,
                reasons: vec!["measured planner calibration".to_string()],
                rejected_backends: Vec::new(),
            },
        };
        let artifact = backend.compile(&plan, &device).ok()?;
        let warmup_cfg = ArithmeticAsianCallConfig {
            n_paths,
            n_steps,
            seed: seed.saturating_sub(1),
            technique,
            ..ArithmeticAsianCallConfig::default()
        };
        let _ = backend.execute(
            &artifact,
            &BackendExecutionInput::ArithmeticAsianCall(warmup_cfg),
        );

        let cfg = ArithmeticAsianCallConfig {
            n_paths,
            n_steps,
            seed,
            technique,
            ..ArithmeticAsianCallConfig::default()
        };
        let result = backend
            .execute(&artifact, &BackendExecutionInput::ArithmeticAsianCall(cfg))
            .ok()?;
        Some(result.runtime_ms)
    }
}

fn benchmark_mc_rust_cpu_stepwise(repeats: usize) -> BenchmarkResult {
    let cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        ..EuropeanCallConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = european_call_price_mc_cpu_stepwise(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: "mc_cpu_european_call_rust".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::european_call_price_mc_cpu_stepwise".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("stepwise_paths".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: repeats,
        total_runtime_ms: avg_runtime_ms * repeats as f64,
        per_iteration_us: avg_runtime_ms * 1_000.0,
        throughput_per_sec: if avg_runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            (cfg.n_paths as f64) / (avg_runtime_ms / 1_000.0)
        },
        metric_name: Some("price_estimate".to_string()),
        metric_value: Some(avg_price),
    }
}

fn benchmark_mc_native_metal_stepwise(_repeats: usize) -> Option<BenchmarkResult> {
    benchmark_mc_native_metal_variant(
        _repeats,
        MonteCarloTechnique::Standard,
        "mc_metal_european_call_native",
        "stepwise_paths_native_metal",
    )
}

fn benchmark_mc_native_metal_stepwise_antithetic(_repeats: usize) -> Option<BenchmarkResult> {
    benchmark_mc_native_metal_variant(
        _repeats,
        MonteCarloTechnique::Antithetic,
        "mc_metal_european_call_native_antithetic",
        "stepwise_paths_native_metal_antithetic",
    )
}

fn benchmark_mc_native_metal_stepwise_control_variate(_repeats: usize) -> Option<BenchmarkResult> {
    benchmark_mc_native_metal_variant(
        _repeats,
        MonteCarloTechnique::ControlVariate,
        "mc_metal_european_call_native_control_variate",
        "stepwise_paths_native_metal_control_variate",
    )
}

fn benchmark_mc_native_metal_variant(
    _repeats: usize,
    technique: MonteCarloTechnique,
    benchmark_name: &str,
    methodology: &str,
) -> Option<BenchmarkResult> {
    #[cfg(not(feature = "metal-native"))]
    {
        let _ = (_repeats, technique, benchmark_name, methodology);
        return None;
    }

    #[cfg(feature = "metal-native")]
    {
        let backend = AppleMetalBackend::new();
        let mut devices = backend.discover_devices();
        let device = devices.pop()?;

        let plan = ExecutionPlan {
            backend: BackendId::AppleMetal,
            planner_mode: PlannerMode::Balanced,
            n_paths: MC_PATHS,
            n_steps: MC_STEPS,
            features: FeatureSummary::default(),
            decision_report: BackendDecisionReport {
                selected_backend: BackendId::AppleMetal,
                reasons: vec!["native metal benchmark".to_string()],
                rejected_backends: Vec::new(),
            },
        };

        let artifact = backend.compile(&plan, &device).ok()?;
        let mut runtimes = Vec::with_capacity(_repeats);
        let mut prices = Vec::with_capacity(_repeats);

        let warmup_cfg = EuropeanCallConfig {
            n_paths: MC_PATHS,
            n_steps: MC_STEPS,
            seed: 4_199,
            technique,
            ..EuropeanCallConfig::default()
        };
        backend
            .execute(&artifact, &BackendExecutionInput::EuropeanCall(warmup_cfg))
            .ok()?;

        for i in 0.._repeats {
            let cfg = EuropeanCallConfig {
                n_paths: MC_PATHS,
                n_steps: MC_STEPS,
                seed: 4_200 + i as u64,
                technique,
                ..EuropeanCallConfig::default()
            };

            let result = backend
                .execute(&artifact, &BackendExecutionInput::EuropeanCall(cfg))
                .ok()?;
            runtimes.push(result.runtime_ms);
            prices.push(result.price);
        }

        let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
        let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

        Some(BenchmarkResult {
            benchmark_name: benchmark_name.to_string(),
            benchmark_version: "0.1".to_string(),
            implementation: "mc-core::backend::metal::AppleMetalBackend::execute".to_string(),
            backend: "apple_metal".to_string(),
            methodology: Some(methodology.to_string()),
            planner_mode: "n/a".to_string(),
            iterations: _repeats,
            total_runtime_ms: avg_runtime_ms * _repeats as f64,
            per_iteration_us: avg_runtime_ms * 1_000.0,
            throughput_per_sec: if avg_runtime_ms == 0.0 {
                MC_PATHS as f64
            } else {
                (MC_PATHS as f64) / (avg_runtime_ms / 1_000.0)
            },
            metric_name: Some("price_estimate".to_string()),
            metric_value: Some(avg_price),
        })
    }
}

fn benchmark_mc_native_metal_stepwise_antithetic_quality() -> Option<BenchmarkResult> {
    benchmark_mc_native_metal_quality(
        MonteCarloTechnique::Antithetic,
        "mc_metal_european_call_native_antithetic_quality",
        "stepwise_paths_native_metal_antithetic",
        8_101,
    )
}

fn benchmark_mc_native_metal_stepwise_control_variate_quality() -> Option<BenchmarkResult> {
    benchmark_mc_native_metal_quality(
        MonteCarloTechnique::ControlVariate,
        "mc_metal_european_call_native_control_variate_quality",
        "stepwise_paths_native_metal_control_variate",
        8_102,
    )
}

fn benchmark_mc_native_metal_quality(
    technique: MonteCarloTechnique,
    benchmark_name: &str,
    methodology: &str,
    seed: u64,
) -> Option<BenchmarkResult> {
    #[cfg(not(feature = "metal-native"))]
    {
        let _ = (technique, benchmark_name, methodology, seed);
        return None;
    }

    #[cfg(feature = "metal-native")]
    {
        let backend = AppleMetalBackend::new();
        let mut devices = backend.discover_devices();
        let device = devices.pop()?;

        let plan = ExecutionPlan {
            backend: BackendId::AppleMetal,
            planner_mode: PlannerMode::Balanced,
            n_paths: MC_PATHS,
            n_steps: MC_STEPS,
            features: FeatureSummary::default(),
            decision_report: BackendDecisionReport {
                selected_backend: BackendId::AppleMetal,
                reasons: vec!["native metal benchmark".to_string()],
                rejected_backends: Vec::new(),
            },
        };

        let artifact = backend.compile(&plan, &device).ok()?;
        let standard_cfg = EuropeanCallConfig {
            n_paths: MC_PATHS,
            n_steps: MC_STEPS,
            seed,
            technique: MonteCarloTechnique::Standard,
            ..EuropeanCallConfig::default()
        };
        let technique_cfg = EuropeanCallConfig {
            technique,
            ..standard_cfg
        };

        let standard = backend
            .execute(
                &artifact,
                &BackendExecutionInput::EuropeanCall(standard_cfg),
            )
            .ok()?;
        let adjusted = backend
            .execute(
                &artifact,
                &BackendExecutionInput::EuropeanCall(technique_cfg),
            )
            .ok()?;
        let stderr_ratio = if standard.stderr == 0.0 {
            1.0
        } else {
            adjusted.stderr / standard.stderr
        };

        Some(BenchmarkResult {
            benchmark_name: benchmark_name.to_string(),
            benchmark_version: "0.1".to_string(),
            implementation: "mc-core::backend::metal::AppleMetalBackend::execute".to_string(),
            backend: "apple_metal".to_string(),
            methodology: Some(methodology.to_string()),
            planner_mode: "n/a".to_string(),
            iterations: 1,
            total_runtime_ms: 0.0,
            per_iteration_us: 0.0,
            throughput_per_sec: 0.0,
            metric_name: Some("stderr_ratio_vs_standard".to_string()),
            metric_value: Some(stderr_ratio),
        })
    }
}

fn benchmark_mc_native_metal_arithmetic_asian_stepwise(_repeats: usize) -> Option<BenchmarkResult> {
    benchmark_mc_native_metal_asian_variant(
        _repeats,
        MonteCarloTechnique::Standard,
        "mc_metal_arithmetic_asian_call_native",
        "arithmetic_asian_stepwise_native_metal",
    )
}

fn benchmark_mc_native_metal_arithmetic_asian_stepwise_control_variate(
    _repeats: usize,
) -> Option<BenchmarkResult> {
    benchmark_mc_native_metal_asian_variant(
        _repeats,
        MonteCarloTechnique::ControlVariate,
        "mc_metal_arithmetic_asian_call_native_control_variate",
        "arithmetic_asian_stepwise_native_metal_control_variate",
    )
}

fn benchmark_mc_native_metal_arithmetic_asian_stepwise_control_variate_quality(
) -> Option<BenchmarkResult> {
    #[cfg(not(feature = "metal-native"))]
    {
        return None;
    }

    #[cfg(feature = "metal-native")]
    {
        let backend = AppleMetalBackend::new();
        let mut devices = backend.discover_devices();
        let device = devices.pop()?;

        let plan = ExecutionPlan {
            backend: BackendId::AppleMetal,
            planner_mode: PlannerMode::Balanced,
            n_paths: MC_PATHS,
            n_steps: MC_STEPS,
            features: FeatureSummary::default(),
            decision_report: BackendDecisionReport {
                selected_backend: BackendId::AppleMetal,
                reasons: vec!["native metal benchmark".to_string()],
                rejected_backends: Vec::new(),
            },
        };

        let artifact = backend.compile(&plan, &device).ok()?;
        let standard_cfg = ArithmeticAsianCallConfig {
            n_paths: MC_PATHS,
            n_steps: MC_STEPS,
            seed: 8_111,
            technique: MonteCarloTechnique::Standard,
            ..ArithmeticAsianCallConfig::default()
        };
        let control_cfg = ArithmeticAsianCallConfig {
            technique: MonteCarloTechnique::ControlVariate,
            ..standard_cfg
        };

        let standard = backend
            .execute(
                &artifact,
                &BackendExecutionInput::ArithmeticAsianCall(standard_cfg),
            )
            .ok()?;
        let adjusted = backend
            .execute(
                &artifact,
                &BackendExecutionInput::ArithmeticAsianCall(control_cfg),
            )
            .ok()?;
        let stderr_ratio = if standard.stderr == 0.0 {
            1.0
        } else {
            adjusted.stderr / standard.stderr
        };

        Some(BenchmarkResult {
            benchmark_name: "mc_metal_arithmetic_asian_call_native_control_variate_quality"
                .to_string(),
            benchmark_version: "0.1".to_string(),
            implementation: "mc-core::backend::metal::AppleMetalBackend::execute".to_string(),
            backend: "apple_metal".to_string(),
            methodology: Some("arithmetic_asian_stepwise_native_metal_control_variate".to_string()),
            planner_mode: "n/a".to_string(),
            iterations: 1,
            total_runtime_ms: 0.0,
            per_iteration_us: 0.0,
            throughput_per_sec: 0.0,
            metric_name: Some("stderr_ratio_vs_standard".to_string()),
            metric_value: Some(stderr_ratio),
        })
    }
}

fn benchmark_mc_native_metal_asian_variant(
    _repeats: usize,
    technique: MonteCarloTechnique,
    benchmark_name: &str,
    methodology: &str,
) -> Option<BenchmarkResult> {
    #[cfg(not(feature = "metal-native"))]
    {
        let _ = (_repeats, technique, benchmark_name, methodology);
        return None;
    }

    #[cfg(feature = "metal-native")]
    {
        let backend = AppleMetalBackend::new();
        let mut devices = backend.discover_devices();
        let device = devices.pop()?;

        let plan = ExecutionPlan {
            backend: BackendId::AppleMetal,
            planner_mode: PlannerMode::Balanced,
            n_paths: MC_PATHS,
            n_steps: MC_STEPS,
            features: FeatureSummary::default(),
            decision_report: BackendDecisionReport {
                selected_backend: BackendId::AppleMetal,
                reasons: vec!["native metal benchmark".to_string()],
                rejected_backends: Vec::new(),
            },
        };

        let artifact = backend.compile(&plan, &device).ok()?;
        let mut runtimes = Vec::with_capacity(_repeats);
        let mut prices = Vec::with_capacity(_repeats);

        let warmup_cfg = ArithmeticAsianCallConfig {
            n_paths: MC_PATHS,
            n_steps: MC_STEPS,
            seed: 4_299,
            technique,
            ..ArithmeticAsianCallConfig::default()
        };
        backend
            .execute(
                &artifact,
                &BackendExecutionInput::ArithmeticAsianCall(warmup_cfg),
            )
            .ok()?;

        for i in 0.._repeats {
            let cfg = ArithmeticAsianCallConfig {
                n_paths: MC_PATHS,
                n_steps: MC_STEPS,
                seed: 4_300 + i as u64,
                technique,
                ..ArithmeticAsianCallConfig::default()
            };

            let result = backend
                .execute(&artifact, &BackendExecutionInput::ArithmeticAsianCall(cfg))
                .ok()?;
            runtimes.push(result.runtime_ms);
            prices.push(result.price);
        }

        let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
        let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

        Some(BenchmarkResult {
            benchmark_name: benchmark_name.to_string(),
            benchmark_version: "0.1".to_string(),
            implementation: "mc-core::backend::metal::AppleMetalBackend::execute".to_string(),
            backend: "apple_metal".to_string(),
            methodology: Some(methodology.to_string()),
            planner_mode: "n/a".to_string(),
            iterations: _repeats,
            total_runtime_ms: avg_runtime_ms * _repeats as f64,
            per_iteration_us: avg_runtime_ms * 1_000.0,
            throughput_per_sec: if avg_runtime_ms == 0.0 {
                MC_PATHS as f64
            } else {
                (MC_PATHS as f64) / (avg_runtime_ms / 1_000.0)
            },
            metric_name: Some("price_estimate".to_string()),
            metric_value: Some(avg_price),
        })
    }
}

fn benchmark_mc_rust_cpu_terminal(repeats: usize) -> BenchmarkResult {
    let cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        ..EuropeanCallConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = european_call_price_mc_cpu_terminal(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: "mc_cpu_european_call_rust_terminal".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::european_call_price_mc_cpu_terminal".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("terminal_distribution".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: repeats,
        total_runtime_ms: avg_runtime_ms * repeats as f64,
        per_iteration_us: avg_runtime_ms * 1_000.0,
        throughput_per_sec: if avg_runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            (cfg.n_paths as f64) / (avg_runtime_ms / 1_000.0)
        },
        metric_name: Some("price_estimate".to_string()),
        metric_value: Some(avg_price),
    }
}

fn benchmark_mc_rust_cpu_stepwise_control_variate(repeats: usize) -> BenchmarkResult {
    let cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        technique: MonteCarloTechnique::ControlVariate,
        ..EuropeanCallConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = european_call_price_mc_cpu_stepwise(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: "mc_cpu_european_call_rust_control_variate".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::european_call_price_mc_cpu_stepwise".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("stepwise_paths_control_variate".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: repeats,
        total_runtime_ms: avg_runtime_ms * repeats as f64,
        per_iteration_us: avg_runtime_ms * 1_000.0,
        throughput_per_sec: if avg_runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            (cfg.n_paths as f64) / (avg_runtime_ms / 1_000.0)
        },
        metric_name: Some("price_estimate".to_string()),
        metric_value: Some(avg_price),
    }
}

fn benchmark_mc_rust_cpu_stepwise_antithetic(repeats: usize) -> BenchmarkResult {
    let cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        technique: MonteCarloTechnique::Antithetic,
        ..EuropeanCallConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = european_call_price_mc_cpu_stepwise(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: "mc_cpu_european_call_rust_antithetic".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::european_call_price_mc_cpu_stepwise".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("stepwise_paths_antithetic".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: repeats,
        total_runtime_ms: avg_runtime_ms * repeats as f64,
        per_iteration_us: avg_runtime_ms * 1_000.0,
        throughput_per_sec: if avg_runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            (cfg.n_paths as f64) / (avg_runtime_ms / 1_000.0)
        },
        metric_name: Some("price_estimate".to_string()),
        metric_value: Some(avg_price),
    }
}

fn benchmark_mc_rust_cpu_terminal_antithetic(repeats: usize) -> BenchmarkResult {
    let cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        technique: MonteCarloTechnique::Antithetic,
        ..EuropeanCallConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = european_call_price_mc_cpu_terminal(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: "mc_cpu_european_call_rust_terminal_antithetic".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::european_call_price_mc_cpu_terminal".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("terminal_distribution_antithetic".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: repeats,
        total_runtime_ms: avg_runtime_ms * repeats as f64,
        per_iteration_us: avg_runtime_ms * 1_000.0,
        throughput_per_sec: if avg_runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            (cfg.n_paths as f64) / (avg_runtime_ms / 1_000.0)
        },
        metric_name: Some("price_estimate".to_string()),
        metric_value: Some(avg_price),
    }
}

fn benchmark_mc_rust_cpu_terminal_control_variate(repeats: usize) -> BenchmarkResult {
    let cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        technique: MonteCarloTechnique::ControlVariate,
        ..EuropeanCallConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = european_call_price_mc_cpu_terminal(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: "mc_cpu_european_call_rust_terminal_control_variate".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::european_call_price_mc_cpu_terminal".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("terminal_distribution_control_variate".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: repeats,
        total_runtime_ms: avg_runtime_ms * repeats as f64,
        per_iteration_us: avg_runtime_ms * 1_000.0,
        throughput_per_sec: if avg_runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            (cfg.n_paths as f64) / (avg_runtime_ms / 1_000.0)
        },
        metric_name: Some("price_estimate".to_string()),
        metric_value: Some(avg_price),
    }
}

fn benchmark_mc_rust_cpu_arithmetic_asian_stepwise(repeats: usize) -> BenchmarkResult {
    let cfg = ArithmeticAsianCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        ..ArithmeticAsianCallConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = arithmetic_asian_call_price_mc_cpu(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: "mc_cpu_arithmetic_asian_call_rust".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::arithmetic_asian_call_price_mc_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("arithmetic_asian_stepwise".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: repeats,
        total_runtime_ms: avg_runtime_ms * repeats as f64,
        per_iteration_us: avg_runtime_ms * 1_000.0,
        throughput_per_sec: if avg_runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            (cfg.n_paths as f64) / (avg_runtime_ms / 1_000.0)
        },
        metric_name: Some("price_estimate".to_string()),
        metric_value: Some(avg_price),
    }
}

fn benchmark_mc_rust_cpu_arithmetic_asian_stepwise_control_variate(
    repeats: usize,
) -> BenchmarkResult {
    let cfg = ArithmeticAsianCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        technique: MonteCarloTechnique::ControlVariate,
        ..ArithmeticAsianCallConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = arithmetic_asian_call_price_mc_cpu(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: "mc_cpu_arithmetic_asian_call_rust_control_variate".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::arithmetic_asian_call_price_mc_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("arithmetic_asian_stepwise_control_variate".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: repeats,
        total_runtime_ms: avg_runtime_ms * repeats as f64,
        per_iteration_us: avg_runtime_ms * 1_000.0,
        throughput_per_sec: if avg_runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            (cfg.n_paths as f64) / (avg_runtime_ms / 1_000.0)
        },
        metric_name: Some("price_estimate".to_string()),
        metric_value: Some(avg_price),
    }
}

fn benchmark_mc_rust_cpu_stepwise_antithetic_quality() -> BenchmarkResult {
    let standard_cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 7_001,
        ..EuropeanCallConfig::default()
    };
    let antithetic_cfg = EuropeanCallConfig {
        technique: MonteCarloTechnique::Antithetic,
        ..standard_cfg
    };

    let standard = european_call_price_mc_cpu_stepwise(&standard_cfg);
    let antithetic = european_call_price_mc_cpu_stepwise(&antithetic_cfg);
    let stderr_ratio = if standard.stderr == 0.0 {
        1.0
    } else {
        antithetic.stderr / standard.stderr
    };

    BenchmarkResult {
        benchmark_name: "mc_cpu_european_call_rust_antithetic_quality".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::european_call_price_mc_cpu_stepwise".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("stepwise_paths_antithetic".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: 0.0,
        per_iteration_us: 0.0,
        throughput_per_sec: 0.0,
        metric_name: Some("stderr_ratio_vs_standard".to_string()),
        metric_value: Some(stderr_ratio),
    }
}

fn benchmark_mc_rust_cpu_terminal_antithetic_quality() -> BenchmarkResult {
    let standard_cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 7_002,
        ..EuropeanCallConfig::default()
    };
    let antithetic_cfg = EuropeanCallConfig {
        technique: MonteCarloTechnique::Antithetic,
        ..standard_cfg
    };

    let standard = european_call_price_mc_cpu_terminal(&standard_cfg);
    let antithetic = european_call_price_mc_cpu_terminal(&antithetic_cfg);
    let stderr_ratio = if standard.stderr == 0.0 {
        1.0
    } else {
        antithetic.stderr / standard.stderr
    };

    BenchmarkResult {
        benchmark_name: "mc_cpu_european_call_rust_terminal_antithetic_quality".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::european_call_price_mc_cpu_terminal".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("terminal_distribution_antithetic".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: 0.0,
        per_iteration_us: 0.0,
        throughput_per_sec: 0.0,
        metric_name: Some("stderr_ratio_vs_standard".to_string()),
        metric_value: Some(stderr_ratio),
    }
}

fn benchmark_mc_rust_cpu_stepwise_control_variate_quality() -> BenchmarkResult {
    let standard_cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 7_003,
        ..EuropeanCallConfig::default()
    };
    let control_cfg = EuropeanCallConfig {
        technique: MonteCarloTechnique::ControlVariate,
        ..standard_cfg
    };

    let standard = european_call_price_mc_cpu_stepwise(&standard_cfg);
    let control = european_call_price_mc_cpu_stepwise(&control_cfg);
    let stderr_ratio = if standard.stderr == 0.0 {
        1.0
    } else {
        control.stderr / standard.stderr
    };

    BenchmarkResult {
        benchmark_name: "mc_cpu_european_call_rust_control_variate_quality".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::european_call_price_mc_cpu_stepwise".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("stepwise_paths_control_variate".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: 0.0,
        per_iteration_us: 0.0,
        throughput_per_sec: 0.0,
        metric_name: Some("stderr_ratio_vs_standard".to_string()),
        metric_value: Some(stderr_ratio),
    }
}

fn benchmark_mc_rust_cpu_terminal_control_variate_quality() -> BenchmarkResult {
    let standard_cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 7_004,
        ..EuropeanCallConfig::default()
    };
    let control_cfg = EuropeanCallConfig {
        technique: MonteCarloTechnique::ControlVariate,
        ..standard_cfg
    };

    let standard = european_call_price_mc_cpu_terminal(&standard_cfg);
    let control = european_call_price_mc_cpu_terminal(&control_cfg);
    let stderr_ratio = if standard.stderr == 0.0 {
        1.0
    } else {
        control.stderr / standard.stderr
    };

    BenchmarkResult {
        benchmark_name: "mc_cpu_european_call_rust_terminal_control_variate_quality".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::european_call_price_mc_cpu_terminal".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("terminal_distribution_control_variate".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: 0.0,
        per_iteration_us: 0.0,
        throughput_per_sec: 0.0,
        metric_name: Some("stderr_ratio_vs_standard".to_string()),
        metric_value: Some(stderr_ratio),
    }
}

fn benchmark_mc_rust_cpu_arithmetic_asian_stepwise_control_variate_quality() -> BenchmarkResult {
    let standard_cfg = ArithmeticAsianCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 7_111,
        ..ArithmeticAsianCallConfig::default()
    };
    let control_cfg = ArithmeticAsianCallConfig {
        technique: MonteCarloTechnique::ControlVariate,
        ..standard_cfg
    };

    let standard = arithmetic_asian_call_price_mc_cpu(&standard_cfg);
    let control = arithmetic_asian_call_price_mc_cpu(&control_cfg);
    let stderr_ratio = if standard.stderr == 0.0 {
        1.0
    } else {
        control.stderr / standard.stderr
    };

    BenchmarkResult {
        benchmark_name: "mc_cpu_arithmetic_asian_call_rust_control_variate_quality".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::arithmetic_asian_call_price_mc_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("arithmetic_asian_stepwise_control_variate".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: 0.0,
        per_iteration_us: 0.0,
        throughput_per_sec: 0.0,
        metric_name: Some("stderr_ratio_vs_standard".to_string()),
        metric_value: Some(stderr_ratio),
    }
}

fn benchmark_python_competitors(
    n_paths: usize,
    n_steps: usize,
    repeats: usize,
    seed: u64,
) -> Vec<BenchmarkResult> {
    let output = Command::new("python3")
        .arg("benchmarks/competitors/python_cpu_baselines.py")
        .arg("--paths")
        .arg(n_paths.to_string())
        .arg("--steps")
        .arg(n_steps.to_string())
        .arg("--repeats")
        .arg(repeats.to_string())
        .arg("--seed")
        .arg(seed.to_string())
        .output();

    let Ok(output) = output else {
        return vec![BenchmarkResult {
            benchmark_name: "mc_cpu_european_call_competitors".to_string(),
            benchmark_version: "0.1".to_string(),
            implementation: "python_cpu_baselines.py".to_string(),
            backend: "external".to_string(),
            methodology: None,
            planner_mode: "n/a".to_string(),
            iterations: 1,
            total_runtime_ms: 0.0,
            per_iteration_us: 0.0,
            throughput_per_sec: 0.0,
            metric_name: Some("error".to_string()),
            metric_value: None,
        }];
    };

    if !output.status.success() {
        return vec![BenchmarkResult {
            benchmark_name: "mc_cpu_european_call_competitors".to_string(),
            benchmark_version: "0.1".to_string(),
            implementation: "python_cpu_baselines.py".to_string(),
            backend: "external".to_string(),
            methodology: None,
            planner_mode: "n/a".to_string(),
            iterations: 1,
            total_runtime_ms: 0.0,
            per_iteration_us: 0.0,
            throughput_per_sec: 0.0,
            metric_name: Some("script_failed".to_string()),
            metric_value: None,
        }];
    }

    let parsed = serde_json::from_slice::<PythonBenchmarkPayload>(&output.stdout);
    let Ok(parsed) = parsed else {
        return vec![BenchmarkResult {
            benchmark_name: "mc_cpu_european_call_competitors".to_string(),
            benchmark_version: "0.1".to_string(),
            implementation: "python_cpu_baselines.py".to_string(),
            backend: "external".to_string(),
            methodology: None,
            planner_mode: "n/a".to_string(),
            iterations: 1,
            total_runtime_ms: 0.0,
            per_iteration_us: 0.0,
            throughput_per_sec: 0.0,
            metric_name: Some("parse_failed".to_string()),
            metric_value: None,
        }];
    };

    parsed
        .results
        .into_iter()
        .map(|entry| {
            if entry.available {
                let runtime_ms = entry.runtime_ms.unwrap_or(0.0);
                let methodology = entry
                    .methodology
                    .unwrap_or_else(|| "stepwise_paths".to_string());
                let benchmark_name = match methodology.as_str() {
                    "terminal_distribution" => {
                        format!("mc_cpu_european_call_{}_terminal", entry.library)
                    }
                    _ => format!("mc_cpu_european_call_{}", entry.library),
                };
                BenchmarkResult {
                    benchmark_name,
                    benchmark_version: "0.1".to_string(),
                    implementation: format!("python::{0}", entry.library),
                    backend: "cpu_external".to_string(),
                    methodology: Some(methodology),
                    planner_mode: "n/a".to_string(),
                    iterations: repeats,
                    total_runtime_ms: runtime_ms * repeats as f64,
                    per_iteration_us: runtime_ms * 1_000.0,
                    throughput_per_sec: if runtime_ms == 0.0 {
                        n_paths as f64
                    } else {
                        n_paths as f64 / (runtime_ms / 1_000.0)
                    },
                    metric_name: Some("price_estimate".to_string()),
                    metric_value: entry.price,
                }
            } else {
                let methodology = entry.methodology.clone();
                BenchmarkResult {
                    benchmark_name: if let Some(ref methodology) = methodology {
                        if methodology == "terminal_distribution" {
                            format!(
                                "mc_cpu_european_call_{}_terminal_unavailable",
                                entry.library
                            )
                        } else {
                            format!("mc_cpu_european_call_{}_unavailable", entry.library)
                        }
                    } else {
                        format!("mc_cpu_european_call_{}_unavailable", entry.library)
                    },
                    benchmark_version: "0.1".to_string(),
                    implementation: format!("python::{}", entry.library),
                    backend: "cpu_external".to_string(),
                    methodology,
                    planner_mode: "n/a".to_string(),
                    iterations: 1,
                    total_runtime_ms: 0.0,
                    per_iteration_us: 0.0,
                    throughput_per_sec: 0.0,
                    metric_name: Some("unavailable".to_string()),
                    metric_value: None,
                }
            }
        })
        .collect()
}

fn throughput(iterations: usize, elapsed_seconds: f64) -> f64 {
    if elapsed_seconds == 0.0 {
        iterations as f64
    } else {
        iterations as f64 / elapsed_seconds
    }
}

#[derive(Debug, Deserialize)]
struct PythonBenchmarkPayload {
    #[allow(dead_code)]
    environment: BTreeMap<String, serde_json::Value>,
    results: Vec<PythonLibraryResult>,
}

#[derive(Debug, Deserialize)]
struct PythonLibraryResult {
    library: String,
    available: bool,
    methodology: Option<String>,
    runtime_ms: Option<f64>,
    price: Option<f64>,
    #[allow(dead_code)]
    stderr: Option<f64>,
    #[allow(dead_code)]
    note: Option<String>,
}

fn sample_spec(with_conditional: bool) -> SimulationSpec {
    let mut axes = BTreeMap::new();
    axes.insert(
        "path".to_string(),
        AxisSpec {
            name: "path".to_string(),
            kind: AxisKind::Runtime,
            size: None,
            parallel: true,
            ordered: false,
        },
    );
    axes.insert(
        "step".to_string(),
        AxisSpec {
            name: "step".to_string(),
            kind: AxisKind::Runtime,
            size: None,
            parallel: false,
            ordered: true,
        },
    );

    let update_expr = if with_conditional {
        Expr::BinaryOp {
            op: "gt".to_string(),
            lhs: Box::new(Expr::StateRef {
                value: "price".to_string(),
            }),
            rhs: Box::new(Expr::Literal { value: 0.0 }),
        }
    } else {
        Expr::StateRef {
            value: "price".to_string(),
        }
    };

    SimulationSpec {
        schema_version: "0.1".to_string(),
        name: "benchmark_case".to_string(),
        version: "0.1.0".to_string(),
        parameters: vec![ParameterSpec {
            name: "s0".to_string(),
            dtype: "float64".to_string(),
        }],
        axes,
        random_variables: vec![RandomVarSpec {
            name: "z".to_string(),
            distribution: "normal".to_string(),
            dtype: "float32".to_string(),
            axes: vec!["step".to_string()],
        }],
        state_variables: vec![StateVarSpec {
            name: "price".to_string(),
            dtype: "float32".to_string(),
            init: Expr::ParameterRef {
                value: "s0".to_string(),
            },
        }],
        steps: vec![StepSpec {
            name: "advance".to_string(),
            axis: "step".to_string(),
            updates: vec![StateUpdate {
                target: "price".to_string(),
                expr: update_expr,
            }],
        }],
        observations: vec![ObservationSpec {
            name: "payoff".to_string(),
            expr: Expr::StateRef {
                value: "price".to_string(),
            },
        }],
        reductions: vec![ReductionSpec {
            name: "expected_payoff".to_string(),
            op: "mean".to_string(),
            source: "payoff".to_string(),
            axes: vec!["path".to_string()],
        }],
    }
}

trait ResultExt {
    fn runtime_ms(&self) -> Option<f64>;
}

impl ResultExt for BenchmarkResult {
    fn runtime_ms(&self) -> Option<f64> {
        if self.iterations == 0 {
            None
        } else {
            Some(self.total_runtime_ms / self.iterations as f64)
        }
    }
}
