use std::collections::BTreeMap;
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use mc_core::{
    american_put_price_lsm_cpu, arithmetic_asian_call_price_mc_cpu,
    arithmetic_asian_call_price_mlmc_cpu, basket_call_price_mc_cpu, bermudan_put_price_lsm_cpu,
    black_scholes_european_call_greeks, compare_american_put_lsm_binomial_reference_cpu,
    compare_arithmetic_asian_mlmc_reference_cpu, compare_arithmetic_asian_sampling_quality_cpu,
    compare_basket_call_sampling_quality_cpu, compare_bermudan_put_lsm_binomial_reference_cpu,
    compare_down_and_out_sampling_quality_cpu, compare_european_call_realized_error_cpu,
    compare_european_call_sampling_quality_cpu, compare_heston_black_scholes_limit_cpu,
    compare_lookback_call_sampling_quality_cpu, down_and_out_call_price_mc_cpu,
    european_call_greeks_cpu, european_call_price_mc_cpu_stepwise,
    european_call_price_mc_cpu_terminal, gaussian_uncertainty_mean_cpu,
    gaussian_uncertainty_moments_cpu, generate_standard_normals_cpu,
    heston_european_call_greeks_cpu, heston_european_call_price_mc_cpu, lookback_call_price_mc_cpu,
    merton_jump_diffusion_call_price_mc_cpu, merton_jump_diffusion_call_reference_price,
    plan_execution, price_all_current_greeks_bump_and_revalue_cpu,
    price_european_call_parameter_sweep_cpu, solve_arithmetic_asian_mlmc_tolerance_cpu,
    AmericanPutConfig, ArithmeticAsianCallConfig, ArithmeticAsianMlmcConfig,
    ArithmeticAsianMlmcToleranceConfig, BackendId, BackendPreference, BackendSupportReport,
    BasketCallConfig, BermudanPutConfig, DownAndOutCallConfig, EuropeanCallConfig,
    EuropeanCallMethod, EuropeanCallParameterSweepConfig, EuropeanCallSweepScenario,
    GaussianUncertaintyConfig, Greek, GreekEstimator, HestonEuropeanCallConfig, LookbackCallConfig,
    MertonJumpDiffusionCallConfig, MonteCarloTechnique, PlannerMode, RunConfig, SamplingMethod,
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
const GREEK_PATHS: usize = 50_000;
const GREEK_STEPS: usize = 64;
const QMC_GENERATION_POINTS: usize = 100_000;
const QMC_GENERATION_DIMENSIONS: usize = 64;
const ASIAN_MLMC_MIN_STEP_UPDATES: usize = 100_000;
const ASIAN_MLMC_MAX_STEP_UPDATES: usize = 2_000_000;
const ASIAN_MLMC_PILOT_PATHS: usize = 2_048;
const ASIAN_MLMC_TARGET_STDERR: f64 = 0.05;
const ASIAN_MLQMC_REPLICATES: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkSuite {
    Full,
    Compact,
}

pub fn run_default_benchmarks() -> BenchmarkReport {
    run_benchmarks(BenchmarkSuite::Full)
}

pub fn run_compact_benchmarks() -> BenchmarkReport {
    run_benchmarks(BenchmarkSuite::Compact)
}

pub fn run_benchmarks(suite: BenchmarkSuite) -> BenchmarkReport {
    match suite {
        BenchmarkSuite::Full => run_full_benchmarks(),
        BenchmarkSuite::Compact => run_compact_benchmarks_inner(),
    }
}

fn run_full_benchmarks() -> BenchmarkReport {
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
        benchmark_mc_rust_cpu_european_parameter_sweep(MC_REPEATS),
        benchmark_mc_rust_cpu_terminal_antithetic(MC_REPEATS),
        benchmark_mc_rust_cpu_terminal_antithetic_quality(),
        benchmark_mc_rust_cpu_terminal_control_variate(MC_REPEATS),
        benchmark_mc_rust_cpu_terminal_control_variate_quality(),
        benchmark_mc_rust_cpu_arithmetic_asian_stepwise(MC_REPEATS),
        benchmark_mc_rust_cpu_arithmetic_asian_stepwise_control_variate(MC_REPEATS),
        benchmark_mc_rust_cpu_arithmetic_asian_stepwise_control_variate_quality(),
        benchmark_mc_rust_cpu_arithmetic_asian_mlmc(MC_REPEATS),
        benchmark_mc_rust_cpu_arithmetic_asian_mlmc_quality(),
        benchmark_mc_rust_cpu_arithmetic_asian_mlmc_reference_calibration(),
        benchmark_mc_rust_cpu_arithmetic_asian_mlqmc(MC_REPEATS),
        benchmark_mc_rust_cpu_arithmetic_asian_mlqmc_quality(),
        benchmark_mc_rust_cpu_arithmetic_asian_mlqmc_reference_calibration(),
        benchmark_mc_rust_cpu_european_call_randomized_halton(MC_REPEATS),
        benchmark_mc_rust_cpu_european_call_randomized_halton_control_variate_quality(),
        benchmark_mc_rust_cpu_european_call_latin_hypercube(MC_REPEATS),
        benchmark_mc_rust_cpu_european_call_latin_hypercube_control_variate_quality(),
        benchmark_mc_rust_cpu_european_structured(
            MC_REPEATS,
            SamplingMethod::ScrambledSobol,
            "mc_cpu_european_call_rust_scrambled_sobol",
            "stepwise_paths_scrambled_sobol",
        ),
        benchmark_mc_rust_cpu_european_structured_control_variate_quality(
            SamplingMethod::ScrambledSobol,
            "mc_cpu_european_call_rust_scrambled_sobol_control_variate_quality",
            "stepwise_paths_scrambled_sobol_control_variate",
        ),
        benchmark_mc_rust_cpu_european_structured(
            MC_REPEATS,
            SamplingMethod::ScrambledSobolBrownianBridge,
            "mc_cpu_european_call_rust_scrambled_sobol_brownian_bridge",
            "stepwise_paths_scrambled_sobol_brownian_bridge",
        ),
        benchmark_mc_rust_cpu_european_structured_control_variate_quality(
            SamplingMethod::ScrambledSobolBrownianBridge,
            "mc_cpu_european_call_rust_scrambled_sobol_brownian_bridge_control_variate_quality",
            "stepwise_paths_scrambled_sobol_brownian_bridge_control_variate",
        ),
        benchmark_mc_rust_cpu_european_pricing_quality(
            SamplingMethod::RandomizedHalton,
            "mc_cpu_qmc_quality_european_randomized_halton",
            "pricing_quality_european_randomized_halton",
        ),
        benchmark_mc_rust_cpu_european_pricing_quality(
            SamplingMethod::LatinHypercube,
            "mc_cpu_qmc_quality_european_latin_hypercube",
            "pricing_quality_european_latin_hypercube",
        ),
        benchmark_mc_rust_cpu_european_pricing_quality(
            SamplingMethod::ScrambledSobol,
            "mc_cpu_qmc_quality_european_scrambled_sobol",
            "pricing_quality_european_scrambled_sobol",
        ),
        benchmark_mc_rust_cpu_european_pricing_quality(
            SamplingMethod::ScrambledSobolBrownianBridge,
            "mc_cpu_qmc_quality_european_scrambled_sobol_brownian_bridge",
            "pricing_quality_european_scrambled_sobol_brownian_bridge",
        ),
        benchmark_mc_rust_cpu_european_realized_error(
            SamplingMethod::RandomizedHalton,
            "mc_cpu_qmc_realized_error_european_randomized_halton",
            "realized_error_european_randomized_halton_black_scholes",
        ),
        benchmark_mc_rust_cpu_european_realized_error(
            SamplingMethod::LatinHypercube,
            "mc_cpu_qmc_realized_error_european_latin_hypercube",
            "realized_error_european_latin_hypercube_black_scholes",
        ),
        benchmark_mc_rust_cpu_european_realized_error(
            SamplingMethod::ScrambledSobol,
            "mc_cpu_qmc_realized_error_european_scrambled_sobol",
            "realized_error_european_scrambled_sobol_black_scholes",
        ),
        benchmark_mc_rust_cpu_european_realized_error(
            SamplingMethod::ScrambledSobolBrownianBridge,
            "mc_cpu_qmc_realized_error_european_scrambled_sobol_brownian_bridge",
            "realized_error_european_scrambled_sobol_brownian_bridge_black_scholes",
        ),
        benchmark_mc_rust_cpu_down_and_out_stepwise(MC_REPEATS),
        benchmark_mc_rust_cpu_down_and_out_stepwise_control_variate(MC_REPEATS),
        benchmark_mc_rust_cpu_down_and_out_stepwise_control_variate_quality(),
        benchmark_mc_rust_cpu_lookback_stepwise(MC_REPEATS),
        benchmark_mc_rust_cpu_lookback_stepwise_control_variate(MC_REPEATS),
        benchmark_mc_rust_cpu_lookback_pricing_quality(
            SamplingMethod::LatinHypercube,
            "mc_cpu_qmc_quality_lookback_latin_hypercube",
            "pricing_quality_lookback_latin_hypercube",
        ),
        benchmark_mc_rust_cpu_american_put_lsm(MC_REPEATS),
        benchmark_mc_rust_cpu_american_put_lsm_binomial_reference(),
        benchmark_mc_rust_cpu_bermudan_put_lsm(MC_REPEATS),
        benchmark_mc_rust_cpu_bermudan_put_lsm_binomial_reference(),
        benchmark_mc_rust_cpu_heston_european_stepwise(MC_REPEATS),
        benchmark_mc_rust_cpu_heston_black_scholes_limit(),
        benchmark_mc_rust_cpu_merton_jump_diffusion_call(MC_REPEATS),
        benchmark_mc_rust_cpu_merton_jump_diffusion_reference_quality(),
        benchmark_mc_rust_cpu_european_greeks(GreekEstimator::BumpAndRevalue),
        benchmark_mc_rust_cpu_european_greeks(GreekEstimator::Pathwise),
        benchmark_mc_rust_cpu_european_greeks(GreekEstimator::LikelihoodRatio),
        benchmark_mc_rust_cpu_heston_black_scholes_limit_greeks(),
        benchmark_mc_rust_cpu_all_workload_bump_greeks(),
        benchmark_mc_rust_cpu_asian_structured(
            MC_REPEATS,
            SamplingMethod::RandomizedHalton,
            "mc_cpu_arithmetic_asian_call_rust_randomized_halton",
            "arithmetic_asian_stepwise_randomized_halton",
        ),
        benchmark_mc_rust_cpu_asian_structured_control_variate_quality(
            SamplingMethod::RandomizedHalton,
            "mc_cpu_arithmetic_asian_call_rust_randomized_halton_control_variate_quality",
            "arithmetic_asian_stepwise_randomized_halton_control_variate",
        ),
        benchmark_mc_rust_cpu_asian_structured(
            MC_REPEATS,
            SamplingMethod::LatinHypercube,
            "mc_cpu_arithmetic_asian_call_rust_latin_hypercube",
            "arithmetic_asian_stepwise_latin_hypercube",
        ),
        benchmark_mc_rust_cpu_asian_structured_control_variate_quality(
            SamplingMethod::LatinHypercube,
            "mc_cpu_arithmetic_asian_call_rust_latin_hypercube_control_variate_quality",
            "arithmetic_asian_stepwise_latin_hypercube_control_variate",
        ),
        benchmark_mc_rust_cpu_asian_structured(
            MC_REPEATS,
            SamplingMethod::ScrambledSobolBrownianBridge,
            "mc_cpu_arithmetic_asian_call_rust_scrambled_sobol_brownian_bridge",
            "arithmetic_asian_stepwise_scrambled_sobol_brownian_bridge",
        ),
        benchmark_mc_rust_cpu_asian_structured_control_variate_quality(
            SamplingMethod::ScrambledSobolBrownianBridge,
            "mc_cpu_arithmetic_asian_call_rust_scrambled_sobol_brownian_bridge_control_variate_quality",
            "arithmetic_asian_stepwise_scrambled_sobol_brownian_bridge_control_variate",
        ),
        benchmark_mc_rust_cpu_asian_pricing_quality(
            SamplingMethod::RandomizedHalton,
            "mc_cpu_qmc_quality_arithmetic_asian_randomized_halton",
            "pricing_quality_arithmetic_asian_randomized_halton",
        ),
        benchmark_mc_rust_cpu_asian_pricing_quality(
            SamplingMethod::LatinHypercube,
            "mc_cpu_qmc_quality_arithmetic_asian_latin_hypercube",
            "pricing_quality_arithmetic_asian_latin_hypercube",
        ),
        benchmark_mc_rust_cpu_asian_pricing_quality(
            SamplingMethod::ScrambledSobolBrownianBridge,
            "mc_cpu_qmc_quality_arithmetic_asian_scrambled_sobol_brownian_bridge",
            "pricing_quality_arithmetic_asian_scrambled_sobol_brownian_bridge",
        ),
        benchmark_mc_rust_cpu_down_and_out_structured(
            MC_REPEATS,
            SamplingMethod::RandomizedHalton,
            "mc_cpu_down_and_out_call_rust_randomized_halton",
            "down_and_out_stepwise_randomized_halton",
        ),
        benchmark_mc_rust_cpu_down_and_out_structured_control_variate_quality(
            SamplingMethod::RandomizedHalton,
            "mc_cpu_down_and_out_call_rust_randomized_halton_control_variate_quality",
            "down_and_out_stepwise_randomized_halton_control_variate",
        ),
        benchmark_mc_rust_cpu_down_and_out_structured(
            MC_REPEATS,
            SamplingMethod::LatinHypercube,
            "mc_cpu_down_and_out_call_rust_latin_hypercube",
            "down_and_out_stepwise_latin_hypercube",
        ),
        benchmark_mc_rust_cpu_down_and_out_structured_control_variate_quality(
            SamplingMethod::LatinHypercube,
            "mc_cpu_down_and_out_call_rust_latin_hypercube_control_variate_quality",
            "down_and_out_stepwise_latin_hypercube_control_variate",
        ),
        benchmark_mc_rust_cpu_down_and_out_structured(
            MC_REPEATS,
            SamplingMethod::ScrambledSobolBrownianBridge,
            "mc_cpu_down_and_out_call_rust_scrambled_sobol_brownian_bridge",
            "down_and_out_stepwise_scrambled_sobol_brownian_bridge",
        ),
        benchmark_mc_rust_cpu_down_and_out_structured_control_variate_quality(
            SamplingMethod::ScrambledSobolBrownianBridge,
            "mc_cpu_down_and_out_call_rust_scrambled_sobol_brownian_bridge_control_variate_quality",
            "down_and_out_stepwise_scrambled_sobol_brownian_bridge_control_variate",
        ),
        benchmark_mc_rust_cpu_down_and_out_pricing_quality(
            SamplingMethod::RandomizedHalton,
            "mc_cpu_qmc_quality_down_and_out_randomized_halton",
            "pricing_quality_down_and_out_randomized_halton",
        ),
        benchmark_mc_rust_cpu_down_and_out_pricing_quality(
            SamplingMethod::LatinHypercube,
            "mc_cpu_qmc_quality_down_and_out_latin_hypercube",
            "pricing_quality_down_and_out_latin_hypercube",
        ),
        benchmark_mc_rust_cpu_down_and_out_pricing_quality(
            SamplingMethod::ScrambledSobolBrownianBridge,
            "mc_cpu_qmc_quality_down_and_out_scrambled_sobol_brownian_bridge",
            "pricing_quality_down_and_out_scrambled_sobol_brownian_bridge",
        ),
        benchmark_mc_rust_cpu_basket(
            MC_REPEATS,
            SamplingMethod::Pseudorandom,
            "mc_cpu_basket_call_rust",
            "basket_terminal_pseudorandom",
        ),
        benchmark_mc_rust_cpu_basket(
            MC_REPEATS,
            SamplingMethod::RandomizedHalton,
            "mc_cpu_basket_call_rust_randomized_halton",
            "basket_terminal_randomized_halton",
        ),
        benchmark_mc_rust_cpu_basket(
            MC_REPEATS,
            SamplingMethod::LatinHypercube,
            "mc_cpu_basket_call_rust_latin_hypercube",
            "basket_terminal_latin_hypercube",
        ),
        benchmark_mc_rust_cpu_basket(
            MC_REPEATS,
            SamplingMethod::ScrambledSobol,
            "mc_cpu_basket_call_rust_scrambled_sobol",
            "basket_terminal_scrambled_sobol",
        ),
        benchmark_mc_rust_cpu_basket_pricing_quality(
            SamplingMethod::RandomizedHalton,
            "mc_cpu_qmc_quality_basket_randomized_halton",
            "pricing_quality_basket_randomized_halton",
        ),
        benchmark_mc_rust_cpu_basket_pricing_quality(
            SamplingMethod::LatinHypercube,
            "mc_cpu_qmc_quality_basket_latin_hypercube",
            "pricing_quality_basket_latin_hypercube",
        ),
        benchmark_mc_rust_cpu_basket_pricing_quality(
            SamplingMethod::ScrambledSobol,
            "mc_cpu_qmc_quality_basket_scrambled_sobol",
            "pricing_quality_basket_scrambled_sobol",
        ),
        benchmark_mc_rust_qmc_generation(
            MC_REPEATS,
            SamplingMethod::ScrambledSobol,
            "mc_cpu_qmc_rust_scrambled_sobol_generation",
            "standard_normal_generation_scrambled_sobol",
        ),
        benchmark_mc_rust_qmc_generation(
            MC_REPEATS,
            SamplingMethod::RandomizedHalton,
            "mc_cpu_qmc_rust_randomized_halton_generation",
            "standard_normal_generation_randomized_halton",
        ),
        benchmark_mc_rust_qmc_generation(
            MC_REPEATS,
            SamplingMethod::LatinHypercube,
            "mc_cpu_qmc_rust_latin_hypercube_generation",
            "standard_normal_generation_latin_hypercube",
        ),
        benchmark_mc_rust_gaussian_uncertainty(
            MC_REPEATS,
            SamplingMethod::Pseudorandom,
            "mc_cpu_gaussian_uncertainty_rust_pseudorandom",
            "gaussian_uncertainty_pseudorandom",
        ),
        benchmark_mc_rust_gaussian_uncertainty(
            MC_REPEATS,
            SamplingMethod::RandomizedHalton,
            "mc_cpu_gaussian_uncertainty_rust_randomized_halton",
            "gaussian_uncertainty_randomized_halton",
        ),
        benchmark_mc_rust_gaussian_uncertainty(
            MC_REPEATS,
            SamplingMethod::LatinHypercube,
            "mc_cpu_gaussian_uncertainty_rust_latin_hypercube",
            "gaussian_uncertainty_latin_hypercube",
        ),
        benchmark_mc_rust_gaussian_uncertainty(
            MC_REPEATS,
            SamplingMethod::ScrambledSobol,
            "mc_cpu_gaussian_uncertainty_rust_scrambled_sobol",
            "gaussian_uncertainty_scrambled_sobol",
        ),
        benchmark_mc_rust_gaussian_uncertainty_moments(
            MC_REPEATS,
            SamplingMethod::LatinHypercube,
            "mc_cpu_gaussian_uncertainty_moments_rust_latin_hypercube",
            "gaussian_uncertainty_moments_latin_hypercube",
        ),
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
    if let Some(metal_result) = benchmark_mc_native_metal_down_and_out_stepwise(MC_REPEATS) {
        results.push(metal_result);
    }
    if let Some(metal_result) =
        benchmark_mc_native_metal_down_and_out_stepwise_control_variate(MC_REPEATS)
    {
        results.push(metal_result);
    }
    if let Some(metal_quality) =
        benchmark_mc_native_metal_down_and_out_stepwise_control_variate_quality()
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

fn run_compact_benchmarks_inner() -> BenchmarkReport {
    let spec = sample_spec(false);

    let results = vec![
        benchmark_schema_validation(&spec, 1_000),
        benchmark_planner_overhead(&spec, 1_000),
        benchmark_planner_choice_accuracy(),
        benchmark_planner_choice_accuracy_measured(),
        benchmark_mc_rust_cpu_stepwise(1),
        benchmark_mc_rust_cpu_stepwise_antithetic_quality(),
        benchmark_mc_rust_cpu_terminal(1),
        benchmark_mc_rust_cpu_european_parameter_sweep(1),
        benchmark_mc_rust_cpu_arithmetic_asian_mlmc(1),
        benchmark_mc_rust_cpu_arithmetic_asian_mlmc_quality(),
        benchmark_mc_rust_cpu_arithmetic_asian_mlmc_reference_calibration(),
        benchmark_mc_rust_cpu_arithmetic_asian_mlqmc(1),
        benchmark_mc_rust_cpu_arithmetic_asian_mlqmc_reference_calibration(),
        benchmark_mc_rust_cpu_down_and_out_stepwise(1),
        benchmark_mc_rust_cpu_lookback_stepwise(1),
        benchmark_mc_rust_cpu_american_put_lsm(1),
        benchmark_mc_rust_cpu_american_put_lsm_binomial_reference(),
        benchmark_mc_rust_cpu_bermudan_put_lsm(1),
        benchmark_mc_rust_cpu_bermudan_put_lsm_binomial_reference(),
        benchmark_mc_rust_cpu_heston_european_stepwise(1),
        benchmark_mc_rust_cpu_heston_black_scholes_limit(),
        benchmark_mc_rust_cpu_merton_jump_diffusion_call(1),
        benchmark_mc_rust_cpu_merton_jump_diffusion_reference_quality(),
        benchmark_mc_rust_cpu_european_greeks(GreekEstimator::BumpAndRevalue),
        benchmark_mc_rust_cpu_european_greeks(GreekEstimator::Pathwise),
        benchmark_mc_rust_cpu_european_greeks(GreekEstimator::LikelihoodRatio),
        benchmark_mc_rust_cpu_heston_black_scholes_limit_greeks(),
        benchmark_mc_rust_cpu_all_workload_bump_greeks(),
        benchmark_mc_rust_cpu_european_call_randomized_halton_control_variate_quality(),
        benchmark_mc_rust_cpu_european_call_latin_hypercube(1),
        benchmark_mc_rust_cpu_european_call_latin_hypercube_control_variate_quality(),
        benchmark_mc_rust_cpu_european_pricing_quality(
            SamplingMethod::ScrambledSobol,
            "mc_cpu_qmc_quality_european_scrambled_sobol",
            "pricing_quality_european_scrambled_sobol",
        ),
        benchmark_mc_rust_cpu_european_realized_error(
            SamplingMethod::LatinHypercube,
            "mc_cpu_qmc_realized_error_european_latin_hypercube",
            "realized_error_european_latin_hypercube_black_scholes",
        ),
        benchmark_mc_rust_cpu_asian_pricing_quality(
            SamplingMethod::LatinHypercube,
            "mc_cpu_qmc_quality_arithmetic_asian_latin_hypercube",
            "pricing_quality_arithmetic_asian_latin_hypercube",
        ),
        benchmark_mc_rust_cpu_down_and_out_pricing_quality(
            SamplingMethod::RandomizedHalton,
            "mc_cpu_qmc_quality_down_and_out_randomized_halton",
            "pricing_quality_down_and_out_randomized_halton",
        ),
        benchmark_mc_rust_cpu_lookback_pricing_quality(
            SamplingMethod::LatinHypercube,
            "mc_cpu_qmc_quality_lookback_latin_hypercube",
            "pricing_quality_lookback_latin_hypercube",
        ),
        benchmark_mc_rust_cpu_basket(
            1,
            SamplingMethod::ScrambledSobol,
            "mc_cpu_basket_call_rust_scrambled_sobol",
            "basket_terminal_scrambled_sobol",
        ),
        benchmark_mc_rust_cpu_basket_pricing_quality(
            SamplingMethod::LatinHypercube,
            "mc_cpu_qmc_quality_basket_latin_hypercube",
            "pricing_quality_basket_latin_hypercube",
        ),
        benchmark_mc_rust_gaussian_uncertainty(
            1,
            SamplingMethod::ScrambledSobol,
            "mc_cpu_gaussian_uncertainty_rust_scrambled_sobol",
            "gaussian_uncertainty_scrambled_sobol",
        ),
        benchmark_mc_rust_gaussian_uncertainty_moments(
            1,
            SamplingMethod::LatinHypercube,
            "mc_cpu_gaussian_uncertainty_moments_rust_latin_hypercube",
            "gaussian_uncertainty_moments_latin_hypercube",
        ),
    ];

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
    let metal = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_metal_european_call_native")
        .and_then(|r| r.runtime_ms());
    let down_and_out_cpu = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_down_and_out_call_rust")
        .and_then(|r| r.runtime_ms());
    let down_and_out_metal = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_metal_down_and_out_call_native")
        .and_then(|r| r.runtime_ms());
    let lookback_cpu = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_lookback_call_rust")
        .and_then(|r| r.runtime_ms());
    let american_put_lsm = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_american_put_lsm_rust")
        .and_then(|r| r.runtime_ms());
    let american_put_binomial_abs_error = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_american_put_lsm_binomial_reference_quality")
        .and_then(|r| r.metric_value);
    let bermudan_put_lsm = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_bermudan_put_lsm_rust")
        .and_then(|r| r.runtime_ms());
    let bermudan_put_binomial_abs_error = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_bermudan_put_lsm_binomial_reference_quality")
        .and_then(|r| r.metric_value);
    let heston_cpu = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_heston_european_call_rust")
        .and_then(|r| r.runtime_ms());
    let european_greek_pathwise = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_european_call_greeks_pathwise_rust")
        .and_then(|r| r.runtime_ms());
    let european_greek_pathwise_error = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_european_call_greeks_pathwise_rust")
        .and_then(|r| r.metric_value);
    let all_greek_count = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_all_workload_greeks_bump_rust")
        .and_then(|r| r.metric_value);
    let measured_planner_accuracy = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "planner_choice_accuracy_measured")
        .and_then(|r| r.metric_value);
    let halton_runtime = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_european_call_rust_randomized_halton")
        .and_then(|r| r.runtime_ms());
    let halton_cv_ratio = report
        .results
        .iter()
        .find(|r| {
            r.benchmark_name
                == "mc_cpu_european_call_rust_randomized_halton_control_variate_quality"
        })
        .and_then(|r| r.metric_value);
    let lhs_runtime = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_european_call_rust_latin_hypercube")
        .and_then(|r| r.runtime_ms());
    let lhs_cv_ratio = report
        .results
        .iter()
        .find(|r| {
            r.benchmark_name == "mc_cpu_european_call_rust_latin_hypercube_control_variate_quality"
        })
        .and_then(|r| r.metric_value);
    let sobol_runtime = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_european_call_rust_scrambled_sobol")
        .and_then(|r| r.runtime_ms());
    let sobol_bridge_runtime = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_european_call_rust_scrambled_sobol_brownian_bridge")
        .and_then(|r| r.runtime_ms());
    let sobol_bridge_cv_ratio = report
        .results
        .iter()
        .find(|r| {
            r.benchmark_name
                == "mc_cpu_european_call_rust_scrambled_sobol_brownian_bridge_control_variate_quality"
        })
        .and_then(|r| r.metric_value);
    let rust_sobol_generation = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_qmc_rust_scrambled_sobol_generation")
        .and_then(|r| r.runtime_ms());
    let scipy_sobol_generation = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_qmc_scipy_qmc_sobol_generation")
        .and_then(|r| r.runtime_ms());
    let asian_mlmc_runtime = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_arithmetic_asian_call_rust_mlmc")
        .and_then(|r| r.runtime_ms());
    let asian_mlmc_ratio = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_arithmetic_asian_call_rust_mlmc_quality")
        .and_then(|r| r.metric_value);
    let asian_mlqmc_runtime = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_arithmetic_asian_call_rust_mlqmc")
        .and_then(|r| r.runtime_ms());
    let asian_mlqmc_ratio = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_arithmetic_asian_call_rust_mlqmc_quality")
        .and_then(|r| r.metric_value);
    let gaussian_lhs_runtime = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_gaussian_uncertainty_rust_latin_hypercube")
        .and_then(|r| r.runtime_ms());
    let gaussian_lhs_abs_error = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_gaussian_uncertainty_rust_latin_hypercube")
        .and_then(|r| r.metric_value);
    let gaussian_pseudorandom_abs_error = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_gaussian_uncertainty_rust_pseudorandom")
        .and_then(|r| r.metric_value);
    let basket_sobol_runtime = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_basket_call_rust_scrambled_sobol")
        .and_then(|r| r.runtime_ms());
    let basket_lhs_ratio = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_qmc_quality_basket_latin_hypercube")
        .and_then(|r| r.metric_value);
    let european_lhs_realized_error = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_qmc_realized_error_european_latin_hypercube")
        .and_then(|r| r.metric_value);
    let european_sobol_realized_error = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_qmc_realized_error_european_scrambled_sobol")
        .and_then(|r| r.metric_value);
    let european_bridge_realized_error = report
        .results
        .iter()
        .find(|r| {
            r.benchmark_name == "mc_cpu_qmc_realized_error_european_scrambled_sobol_brownian_bridge"
        })
        .and_then(|r| r.metric_value);
    let quantlib_unavailable = report.results.iter().any(|r| {
        r.benchmark_name == "mc_cpu_european_call_quantlib_unavailable"
            || r.benchmark_name == "mc_cpu_european_call_quantlib_terminal_unavailable"
            || r.benchmark_name == "mc_cpu_lookback_call_quantlib_unavailable"
            || r.benchmark_name == "mc_cpu_heston_european_call_quantlib_unavailable"
    });

    let mut competitor_rows = report
        .results
        .iter()
        .filter(|r| {
            r.benchmark_name == "mc_cpu_european_call_numpy"
                || r.benchmark_name == "mc_cpu_european_call_numba"
                || r.benchmark_name == "mc_cpu_european_call_quantlib"
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

    out.push_str("Current tracked leaders:\n");
    out.push_str(&format!(
        "- Rust fair CPU baseline (`mc_cpu_european_call_rust`, step-wise): `{:.3} ms`\n",
        rust
    ));
    if let Some(runtime) = metal {
        out.push_str(&format!(
            "- Native Metal GBM baseline (`mc_metal_european_call_native`): `{:.3} ms`\n",
            runtime
        ));
    }
    if let (Some(cpu), Some(metal)) = (down_and_out_cpu, down_and_out_metal) {
        out.push_str(&format!(
            "- Down-and-out breadth check: CPU `{:.3} ms`, Metal `{:.3} ms`\n",
            cpu, metal
        ));
    }
    if let Some(runtime) = lookback_cpu {
        out.push_str(&format!(
            "- Fixed-strike lookback CPU breadth is live (`{:.3} ms`) with explicit QuantLib comparison reporting.\n",
            runtime
        ));
    }
    if let Some(runtime) = american_put_lsm {
        out.push_str(&format!(
            "- American put LSM CPU breadth is live (`{:.3} ms`) with CRR binomial-tree reference tracking.\n",
            runtime
        ));
    }
    if let Some(abs_error) = american_put_binomial_abs_error {
        out.push_str(&format!(
            "- American put LSM binomial-tree quality is tracked with absolute price error `{:.6}` vs CRR reference.\n",
            abs_error
        ));
    }
    if let Some(runtime) = bermudan_put_lsm {
        out.push_str(&format!(
            "- Bermudan put custom-schedule LSM CPU breadth is live (`{:.3} ms`) with explicit exercise-date metadata.\n",
            runtime
        ));
    }
    if let Some(abs_error) = bermudan_put_binomial_abs_error {
        out.push_str(&format!(
            "- Bermudan put LSM binomial-tree quality is tracked with absolute price error `{:.6}` vs mapped CRR exercise schedule.\n",
            abs_error
        ));
    }
    if let Some(runtime) = heston_cpu {
        out.push_str(&format!(
            "- Heston European CPU path simulation is live (`{:.3} ms`) with Black-Scholes-limit validation.\n",
            runtime
        ));
    }
    if let (Some(runtime), Some(abs_error)) =
        (european_greek_pathwise, european_greek_pathwise_error)
    {
        out.push_str(&format!(
            "- European pathwise Greeks are live (`{:.3} ms`) with Delta abs error `{:.6}` vs Black-Scholes.\n",
            runtime, abs_error
        ));
    }
    if let Some(accuracy) = measured_planner_accuracy {
        out.push_str(&format!(
            "- Measured planner choice accuracy: `{:.1}%`\n",
            accuracy
        ));
    }
    out.push('\n');

    let slower_than = competitor_rows
        .iter()
        .filter(|(_, runtime, _)| rust > *runtime)
        .collect::<Vec<_>>();

    if slower_than.is_empty() {
        out.push_str(
            "Status: Rust currently leads the available CPU baselines for the tracked fair European workload.\n\n",
        );
        out.push_str("Maintain lead plan:\n");
        out.push_str("- Keep the step-wise benchmark as the primary competitive claim.\n");
        out.push_str("- Keep RNG and loop hot path allocation-free.\n");
        out.push_str("- Keep breadth claims tied to the workloads we have actually benchmarked: European, Heston European, arithmetic Asian, down-and-out, lookback, American/Bermudan put LSM, basket, Greeks, and Gaussian UQ.\n");
        if let Some(count) = all_greek_count {
            out.push_str(&format!(
                "- Greek breadth is live with `{:.0}` bump-and-revalue estimates across current CPU workload families.\n",
                count
            ));
        }
        if quantlib_unavailable {
            out.push_str("- QuantLib comparison lane is wired but currently unavailable in this environment; install QuantLib-Python to populate the selected-workload scoreboard.\n");
        }
        out.push_str("- Expand competitor matrix to QuantLib exotic workloads and GPU baselines (JAX/CuPy/PyTorch/CUDA-native) when packages and hardware are available.\n");
        if let Some(runtime) = halton_runtime {
            out.push_str(&format!(
                "- First randomized-QMC pricing surface is live via randomized Halton (`{:.3} ms`), but it is currently a quality-first pricing path rather than a speed leader.\n",
                runtime
            ));
        }
        if let Some(runtime) = lhs_runtime {
            out.push_str(&format!(
                "- Latin hypercube pricing is live (`{:.3} ms`) as the first non-QMC structured-sampling breadth path.\n",
                runtime
            ));
        }
        if let Some(runtime) = sobol_runtime {
            out.push_str(&format!(
                "- Scrambled Sobol pricing is live (`{:.3} ms`) as the stronger QMC breadth path.\n",
                runtime
            ));
        }
        if let Some(runtime) = sobol_bridge_runtime {
            out.push_str(&format!(
                "- Scrambled Sobol with Brownian bridge pricing is live (`{:.3} ms`) for path construction experiments.\n",
                runtime
            ));
        }
        if let (Some(rust_runtime), Some(scipy_runtime)) =
            (rust_sobol_generation, scipy_sobol_generation)
        {
            let speedup = scipy_runtime / rust_runtime;
            out.push_str(&format!(
                "- QMC generation scoreboard is live: Rust scrambled Sobol generation `{:.3} ms`, SciPy scrambled Sobol generation `{:.3} ms` (`{:.2}x` Rust/SciPy speedup).\n",
                rust_runtime, scipy_runtime, speedup
            ));
        }
        if let Some(runtime) = asian_mlmc_runtime {
            out.push_str(&format!(
                "- Arithmetic Asian MLMC is live (`{:.3} ms`) with adaptive tolerance planning as the first multilevel CPU reference path.\n",
                runtime
            ));
        }
        if let Some(runtime) = asian_mlqmc_runtime {
            out.push_str(&format!(
                "- Arithmetic Asian MLQMC is live (`{:.3} ms`) with replicated scrambling and adaptive tolerance planning.\n",
                runtime
            ));
        }
        if let (Some(runtime), Some(lhs_error), Some(random_error)) = (
            gaussian_lhs_runtime,
            gaussian_lhs_abs_error,
            gaussian_pseudorandom_abs_error,
        ) {
            out.push_str(&format!(
                "- Gaussian UQ benchmark is live: Latin hypercube `{:.3} ms`, abs error `{:.6}` vs pseudorandom abs error `{:.6}`.\n",
                runtime, lhs_error, random_error
            ));
        }
        if let (Some(runtime), Some(ratio)) = (basket_sobol_runtime, basket_lhs_ratio) {
            out.push_str(&format!(
                "- Basket-call QMC breadth is live: scrambled Sobol basket pricing `{:.3} ms`, Latin-hypercube stderr ratio vs pseudorandom `{:.3}`.\n",
                runtime, ratio
            ));
        }
        if let (Some(lhs_ratio), Some(sobol_ratio), Some(bridge_ratio)) = (
            european_lhs_realized_error,
            european_sobol_realized_error,
            european_bridge_realized_error,
        ) {
            out.push_str(&format!(
                "- European realized-error study is live against Black-Scholes: Latin hypercube abs-error ratio vs pseudorandom `{:.3}`, scrambled Sobol ratio `{:.3}`, Sobol Brownian-bridge ratio `{:.3}`.\n",
                lhs_ratio, sobol_ratio, bridge_ratio
            ));
        }
        if let Some(ratio) = halton_cv_ratio {
            out.push_str(&format!(
                "- Preserve the randomized-QMC quality gain (`stderr_ratio_vs_standard = {:.3}`) while optimizing sequence generation and path construction.\n",
                ratio
            ));
        }
        if let Some(ratio) = lhs_cv_ratio {
            out.push_str(&format!(
                "- Preserve the Latin-hypercube quality gain (`stderr_ratio_vs_standard = {:.3}`) while benchmarking it across more workload families.\n",
                ratio
            ));
        }
        if let Some(ratio) = sobol_bridge_cv_ratio {
            out.push_str(&format!(
                "- Preserve the Sobol Brownian-bridge quality gain (`stderr_ratio_vs_standard = {:.3}`) while optimizing its current runtime overhead.\n",
                ratio
            ));
        }
        if let Some(ratio) = asian_mlmc_ratio {
            out.push_str(&format!(
                "- Track arithmetic Asian MLMC quality (`stderr_ratio_vs_standard = {:.3}`) and calibrate tolerance defaults before claiming it as a default winner.\n",
                ratio
            ));
        }
        if let Some(ratio) = asian_mlqmc_ratio {
            out.push_str(&format!(
                "- Preserve arithmetic Asian replicated MLQMC quality (`stderr_ratio_vs_standard = {:.3}`) while reducing its runtime overhead and increasing replicate coverage.\n",
                ratio
            ));
        }
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
        if let Some(accuracy) = measured_planner_accuracy {
            out.push_str(&format!(
                "- Improve planner calibration beyond the current measured accuracy of `{:.1}%` as workload breadth increases.\n",
                accuracy
            ));
        }
        return out;
    }

    out.push_str("Status: Rust is slower than at least one available CPU baseline on the tracked fair workload.\n\n");
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
    out.push_str("- Keep breadth work going in parallel so we do not win a single benchmark and lose the library surface.\n");
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
            spec: sample_spec(false),
            run_config: RunConfig {
                n_paths: 100_000,
                n_steps: 64,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            support: local_support(metal_supported),
            measured_winner: measured_winner_for_local_backends_barrier(
                100_000,
                64,
                MonteCarloTechnique::ControlVariate,
                9_006,
                metal_supported,
            ),
        },
        Scenario {
            spec: sample_spec(false),
            run_config: RunConfig {
                n_paths: 65_536,
                n_steps: 64,
                planner_mode: PlannerMode::Balanced,
                backend_preference: BackendPreference::Auto,
            },
            support: vec![
                BackendSupportReport::supported(BackendId::CpuNative),
                BackendSupportReport::unsupported(
                    BackendId::NvidiaCuda,
                    "no measured CUDA data on this machine",
                ),
                BackendSupportReport::unsupported(
                    BackendId::AppleMetal,
                    "native Metal randomized QMC is not available yet",
                ),
            ],
            measured_winner: BackendId::CpuNative,
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
        Some(_) => BackendId::CpuNative,
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

fn measured_winner_for_local_backends_barrier(
    n_paths: usize,
    n_steps: usize,
    technique: MonteCarloTechnique,
    seed: u64,
    metal_supported: bool,
) -> BackendId {
    let cpu_runtime_ms = measure_cpu_barrier_runtime_ms(n_paths, n_steps, technique, seed);

    if !metal_supported {
        return BackendId::CpuNative;
    }

    match measure_metal_barrier_runtime_ms(n_paths, n_steps, technique, seed) {
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

fn measure_cpu_barrier_runtime_ms(
    n_paths: usize,
    n_steps: usize,
    technique: MonteCarloTechnique,
    seed: u64,
) -> f64 {
    let cfg = DownAndOutCallConfig {
        n_paths,
        n_steps,
        seed,
        technique,
        ..DownAndOutCallConfig::default()
    };

    let started = Instant::now();
    let _ = down_and_out_call_price_mc_cpu(&cfg);
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
        let started = Instant::now();
        let _ = backend
            .execute(&artifact, &BackendExecutionInput::EuropeanCall(cfg))
            .ok()?;
        Some(started.elapsed().as_secs_f64() * 1_000.0)
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
        let started = Instant::now();
        let _ = backend
            .execute(&artifact, &BackendExecutionInput::ArithmeticAsianCall(cfg))
            .ok()?;
        Some(started.elapsed().as_secs_f64() * 1_000.0)
    }
}

fn measure_metal_barrier_runtime_ms(
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
        let warmup_cfg = DownAndOutCallConfig {
            n_paths,
            n_steps,
            seed: seed.saturating_sub(1),
            technique,
            ..DownAndOutCallConfig::default()
        };
        let _ = backend.execute(
            &artifact,
            &BackendExecutionInput::DownAndOutCall(warmup_cfg),
        );

        let cfg = DownAndOutCallConfig {
            n_paths,
            n_steps,
            seed,
            technique,
            ..DownAndOutCallConfig::default()
        };
        let started = Instant::now();
        let _ = backend
            .execute(&artifact, &BackendExecutionInput::DownAndOutCall(cfg))
            .ok()?;
        Some(started.elapsed().as_secs_f64() * 1_000.0)
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

fn benchmark_mc_rust_cpu_european_parameter_sweep(repeats: usize) -> BenchmarkResult {
    let cfg = EuropeanCallParameterSweepConfig {
        base_config: EuropeanCallConfig {
            n_paths: 25_000,
            n_steps: MC_STEPS,
            technique: MonteCarloTechnique::ControlVariate,
            sampling: SamplingMethod::LatinHypercube,
            ..EuropeanCallConfig::default()
        },
        method: EuropeanCallMethod::TerminalDistribution,
        seed_stride: 10_000,
        scenarios: vec![
            EuropeanCallSweepScenario {
                scenario_id: "atm_base".to_string(),
                ..EuropeanCallSweepScenario::default()
            },
            EuropeanCallSweepScenario {
                scenario_id: "down_10pct".to_string(),
                s0: Some(90.0),
                ..EuropeanCallSweepScenario::default()
            },
            EuropeanCallSweepScenario {
                scenario_id: "up_10pct_high_vol".to_string(),
                s0: Some(110.0),
                sigma: Some(0.35),
                ..EuropeanCallSweepScenario::default()
            },
            EuropeanCallSweepScenario {
                scenario_id: "long_tenor_low_vol".to_string(),
                sigma: Some(0.15),
                t: Some(2.0),
                ..EuropeanCallSweepScenario::default()
            },
        ],
    };

    let scenario_count = cfg.scenarios.len();
    let total_paths_per_iteration = cfg.base_config.n_paths * scenario_count;
    let mut runtimes = Vec::with_capacity(repeats);
    let mut max_abs_error = 0.0_f64;

    for i in 0..repeats {
        let mut cfg_i = cfg.clone();
        cfg_i.base_config.seed = cfg.base_config.seed + i as u64;
        let started = Instant::now();
        let result = price_european_call_parameter_sweep_cpu(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
        runtimes.push(runtime_ms);
        max_abs_error = max_abs_error.max(
            result
                .rows
                .iter()
                .map(|row| row.abs_error_vs_black_scholes)
                .fold(0.0_f64, f64::max),
        );
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;

    BenchmarkResult {
        benchmark_name: "mc_cpu_european_call_parameter_sweep_rust".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::price_european_call_parameter_sweep_cpu"
            .to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("european_terminal_parameter_sweep_lhs_control_variate".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: repeats,
        total_runtime_ms: avg_runtime_ms * repeats as f64,
        per_iteration_us: avg_runtime_ms * 1_000.0,
        throughput_per_sec: if avg_runtime_ms == 0.0 {
            total_paths_per_iteration as f64
        } else {
            total_paths_per_iteration as f64 / (avg_runtime_ms / 1_000.0)
        },
        metric_name: Some("max_abs_error_vs_black_scholes".to_string()),
        metric_value: Some(max_abs_error),
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

fn benchmark_mc_rust_cpu_arithmetic_asian_mlmc(repeats: usize) -> BenchmarkResult {
    benchmark_arithmetic_asian_mlmc_config(
        repeats,
        adaptive_arithmetic_asian_mlmc_config(SamplingMethod::Pseudorandom, 1, 42),
        "mc_cpu_arithmetic_asian_call_rust_mlmc",
        "arithmetic_asian_multilevel_coupled_adaptive_tolerance",
    )
}

fn benchmark_mc_rust_cpu_arithmetic_asian_mlqmc(repeats: usize) -> BenchmarkResult {
    benchmark_arithmetic_asian_mlmc_config(
        repeats,
        adaptive_arithmetic_asian_mlmc_config(
            SamplingMethod::ScrambledSobol,
            ASIAN_MLQMC_REPLICATES,
            42,
        ),
        "mc_cpu_arithmetic_asian_call_rust_mlqmc",
        "arithmetic_asian_multilevel_scrambled_sobol_replicated_adaptive_tolerance",
    )
}

fn adaptive_arithmetic_asian_mlmc_config(
    sampling: SamplingMethod,
    scramble_replicates: usize,
    seed: u64,
) -> ArithmeticAsianMlmcConfig {
    let pilot_cfg = ArithmeticAsianMlmcConfig {
        base_steps: 8,
        levels: 4,
        refinement_factor: 2,
        paths_per_level: vec![ASIAN_MLMC_PILOT_PATHS; 4],
        seed,
        sampling,
        scramble_replicates: 1,
        ..ArithmeticAsianMlmcConfig::default()
    };

    let tolerance = ArithmeticAsianMlmcToleranceConfig {
        target_stderr: ASIAN_MLMC_TARGET_STDERR,
        pilot_paths_per_level: ASIAN_MLMC_PILOT_PATHS,
        min_step_updates: ASIAN_MLMC_MIN_STEP_UPDATES,
        max_step_updates: ASIAN_MLMC_MAX_STEP_UPDATES,
    };
    let mut cfg = ArithmeticAsianMlmcConfig {
        scramble_replicates,
        ..pilot_cfg
    };
    let plan = solve_arithmetic_asian_mlmc_tolerance_cpu(&cfg, &tolerance);

    cfg.paths_per_level = plan.paths_per_level;
    cfg
}

fn benchmark_arithmetic_asian_mlmc_config(
    repeats: usize,
    cfg: ArithmeticAsianMlmcConfig,
    benchmark_name: &str,
    methodology: &str,
) -> BenchmarkResult {
    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);
    let mut step_updates = 0usize;

    for i in 0..repeats {
        let mut cfg_i = cfg.clone();
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = arithmetic_asian_call_price_mlmc_cpu(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

        step_updates = result.total_step_updates;
        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: benchmark_name.to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::arithmetic_asian_call_price_mlmc_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(methodology.to_string()),
        planner_mode: "n/a".to_string(),
        iterations: repeats,
        total_runtime_ms: avg_runtime_ms * repeats as f64,
        per_iteration_us: avg_runtime_ms * 1_000.0,
        throughput_per_sec: if avg_runtime_ms == 0.0 {
            step_updates as f64
        } else {
            (step_updates as f64) / (avg_runtime_ms / 1_000.0)
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

fn benchmark_mc_rust_cpu_arithmetic_asian_mlmc_quality() -> BenchmarkResult {
    let standard_cfg = ArithmeticAsianCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 7_112,
        ..ArithmeticAsianCallConfig::default()
    };
    let mlmc_cfg = adaptive_arithmetic_asian_mlmc_config(SamplingMethod::Pseudorandom, 1, 7_112);

    let standard = arithmetic_asian_call_price_mc_cpu(&standard_cfg);
    let mlmc = arithmetic_asian_call_price_mlmc_cpu(&mlmc_cfg);
    let stderr_ratio = if standard.stderr == 0.0 {
        1.0
    } else {
        mlmc.stderr / standard.stderr
    };

    BenchmarkResult {
        benchmark_name: "mc_cpu_arithmetic_asian_call_rust_mlmc_quality".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::arithmetic_asian_call_price_mlmc_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("arithmetic_asian_multilevel_coupled_adaptive_tolerance".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: 0.0,
        per_iteration_us: 0.0,
        throughput_per_sec: mlmc.total_step_updates as f64,
        metric_name: Some("stderr_ratio_vs_standard".to_string()),
        metric_value: Some(stderr_ratio),
    }
}

fn benchmark_mc_rust_cpu_arithmetic_asian_mlmc_reference_calibration() -> BenchmarkResult {
    let reference_cfg = ArithmeticAsianCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 7_114,
        ..ArithmeticAsianCallConfig::default()
    };
    let mlmc_cfg = adaptive_arithmetic_asian_mlmc_config(SamplingMethod::Pseudorandom, 1, 7_114);

    let started = Instant::now();
    let comparison = compare_arithmetic_asian_mlmc_reference_cpu(&mlmc_cfg, &reference_cfg);
    let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

    BenchmarkResult {
        benchmark_name: "mc_cpu_arithmetic_asian_call_rust_mlmc_reference_calibration".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::compare_arithmetic_asian_mlmc_reference_cpu"
            .to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(
            "arithmetic_asian_mlmc_realized_error_vs_high_budget_standard_mc".to_string(),
        ),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: runtime_ms,
        per_iteration_us: runtime_ms * 1_000.0,
        throughput_per_sec: if runtime_ms == 0.0 {
            (comparison.estimate_step_updates
                + comparison.reference_paths * comparison.reference_steps) as f64
        } else {
            (comparison.estimate_step_updates
                + comparison.reference_paths * comparison.reference_steps) as f64
                / (runtime_ms / 1_000.0)
        },
        metric_name: Some("abs_error_vs_standard_reference".to_string()),
        metric_value: Some(comparison.abs_error),
    }
}

fn benchmark_mc_rust_cpu_arithmetic_asian_mlqmc_quality() -> BenchmarkResult {
    let standard_cfg = ArithmeticAsianCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 7_113,
        ..ArithmeticAsianCallConfig::default()
    };
    let mlqmc_cfg = adaptive_arithmetic_asian_mlmc_config(
        SamplingMethod::ScrambledSobol,
        ASIAN_MLQMC_REPLICATES,
        7_113,
    );

    let standard = arithmetic_asian_call_price_mc_cpu(&standard_cfg);
    let mlqmc = arithmetic_asian_call_price_mlmc_cpu(&mlqmc_cfg);
    let stderr_ratio = if standard.stderr == 0.0 {
        1.0
    } else {
        mlqmc.stderr / standard.stderr
    };

    BenchmarkResult {
        benchmark_name: "mc_cpu_arithmetic_asian_call_rust_mlqmc_quality".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::arithmetic_asian_call_price_mlmc_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(
            "arithmetic_asian_multilevel_scrambled_sobol_replicated_adaptive_tolerance".to_string(),
        ),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: 0.0,
        per_iteration_us: 0.0,
        throughput_per_sec: mlqmc.total_step_updates as f64,
        metric_name: Some("stderr_ratio_vs_standard".to_string()),
        metric_value: Some(stderr_ratio),
    }
}

fn benchmark_mc_rust_cpu_european_call_randomized_halton(repeats: usize) -> BenchmarkResult {
    let cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        sampling: SamplingMethod::RandomizedHalton,
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
        benchmark_name: "mc_cpu_european_call_rust_randomized_halton".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::european_call_price_mc_cpu_stepwise".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("stepwise_paths_randomized_halton".to_string()),
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

fn benchmark_mc_rust_qmc_generation(
    repeats: usize,
    sampling: SamplingMethod,
    benchmark_name: &str,
    methodology: &str,
) -> BenchmarkResult {
    let mut runtimes = Vec::with_capacity(repeats);
    let mut mean_abs = Vec::with_capacity(repeats);

    for rep in 0..repeats {
        let started = Instant::now();
        let samples = generate_standard_normals_cpu(
            sampling,
            QMC_GENERATION_POINTS,
            QMC_GENERATION_DIMENSIONS,
            42 + rep as u64,
        );
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
        let mean = samples.iter().sum::<f64>() / samples.len() as f64;

        runtimes.push(runtime_ms);
        mean_abs.push(mean.abs());
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let generated_values = QMC_GENERATION_POINTS.saturating_mul(QMC_GENERATION_DIMENSIONS);

    BenchmarkResult {
        benchmark_name: benchmark_name.to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::generate_standard_normals_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(methodology.to_string()),
        planner_mode: "n/a".to_string(),
        iterations: repeats,
        total_runtime_ms: avg_runtime_ms * repeats as f64,
        per_iteration_us: avg_runtime_ms * 1_000.0,
        throughput_per_sec: if avg_runtime_ms == 0.0 {
            generated_values as f64
        } else {
            generated_values as f64 / (avg_runtime_ms / 1_000.0)
        },
        metric_name: Some("normal_mean_abs".to_string()),
        metric_value: Some(mean_abs.iter().sum::<f64>() / mean_abs.len() as f64),
    }
}

fn benchmark_mc_rust_gaussian_uncertainty(
    repeats: usize,
    sampling: SamplingMethod,
    benchmark_name: &str,
    methodology: &str,
) -> BenchmarkResult {
    let cfg = GaussianUncertaintyConfig {
        n_samples: MC_PATHS,
        dimensions: 3,
        seed: 42,
        sampling,
    };
    let mut runtimes = Vec::with_capacity(repeats);
    let mut abs_errors = Vec::with_capacity(repeats);

    for rep in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + rep as u64;
        let started = Instant::now();
        let result = gaussian_uncertainty_mean_cpu(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
        runtimes.push(runtime_ms);
        abs_errors.push(result.abs_error);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_abs_error = abs_errors.iter().sum::<f64>() / abs_errors.len() as f64;

    BenchmarkResult {
        benchmark_name: benchmark_name.to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::gaussian_uncertainty_mean_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(methodology.to_string()),
        planner_mode: "n/a".to_string(),
        iterations: repeats,
        total_runtime_ms: avg_runtime_ms * repeats as f64,
        per_iteration_us: avg_runtime_ms * 1_000.0,
        throughput_per_sec: if avg_runtime_ms == 0.0 {
            cfg.n_samples as f64
        } else {
            cfg.n_samples as f64 / (avg_runtime_ms / 1_000.0)
        },
        metric_name: Some("abs_error_vs_analytic_mean".to_string()),
        metric_value: Some(avg_abs_error),
    }
}

fn benchmark_mc_rust_gaussian_uncertainty_moments(
    repeats: usize,
    sampling: SamplingMethod,
    benchmark_name: &str,
    methodology: &str,
) -> BenchmarkResult {
    let cfg = GaussianUncertaintyConfig {
        n_samples: MC_PATHS,
        dimensions: 3,
        seed: 43,
        sampling,
    };
    let mut runtimes = Vec::with_capacity(repeats);
    let mut variance_abs_errors = Vec::with_capacity(repeats);

    for rep in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + rep as u64;
        let started = Instant::now();
        let result = gaussian_uncertainty_moments_cpu(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
        runtimes.push(runtime_ms);
        variance_abs_errors.push(result.variance_abs_error);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_variance_abs_error =
        variance_abs_errors.iter().sum::<f64>() / variance_abs_errors.len() as f64;

    BenchmarkResult {
        benchmark_name: benchmark_name.to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::gaussian_uncertainty_moments_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(methodology.to_string()),
        planner_mode: "n/a".to_string(),
        iterations: repeats,
        total_runtime_ms: avg_runtime_ms * repeats as f64,
        per_iteration_us: avg_runtime_ms * 1_000.0,
        throughput_per_sec: if avg_runtime_ms == 0.0 {
            cfg.n_samples as f64
        } else {
            cfg.n_samples as f64 / (avg_runtime_ms / 1_000.0)
        },
        metric_name: Some("abs_error_vs_analytic_variance".to_string()),
        metric_value: Some(avg_variance_abs_error),
    }
}

fn benchmark_mc_rust_cpu_arithmetic_asian_mlqmc_reference_calibration() -> BenchmarkResult {
    let reference_cfg = ArithmeticAsianCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 7_115,
        ..ArithmeticAsianCallConfig::default()
    };
    let mlqmc_cfg = adaptive_arithmetic_asian_mlmc_config(
        SamplingMethod::ScrambledSobol,
        ASIAN_MLQMC_REPLICATES,
        7_115,
    );

    let started = Instant::now();
    let comparison = compare_arithmetic_asian_mlmc_reference_cpu(&mlqmc_cfg, &reference_cfg);
    let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

    BenchmarkResult {
        benchmark_name: "mc_cpu_arithmetic_asian_call_rust_mlqmc_reference_calibration".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::compare_arithmetic_asian_mlmc_reference_cpu"
            .to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(
            "arithmetic_asian_mlqmc_realized_error_vs_high_budget_standard_mc".to_string(),
        ),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: runtime_ms,
        per_iteration_us: runtime_ms * 1_000.0,
        throughput_per_sec: if runtime_ms == 0.0 {
            (comparison.estimate_step_updates
                + comparison.reference_paths * comparison.reference_steps) as f64
        } else {
            (comparison.estimate_step_updates
                + comparison.reference_paths * comparison.reference_steps) as f64
                / (runtime_ms / 1_000.0)
        },
        metric_name: Some("abs_error_vs_standard_reference".to_string()),
        metric_value: Some(comparison.abs_error),
    }
}

fn benchmark_mc_rust_cpu_european_call_randomized_halton_control_variate_quality() -> BenchmarkResult
{
    let standard_cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 7_211,
        sampling: SamplingMethod::RandomizedHalton,
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
        benchmark_name: "mc_cpu_european_call_rust_randomized_halton_control_variate_quality"
            .to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::european_call_price_mc_cpu_stepwise".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("stepwise_paths_randomized_halton_control_variate".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: 0.0,
        per_iteration_us: 0.0,
        throughput_per_sec: 0.0,
        metric_name: Some("stderr_ratio_vs_standard".to_string()),
        metric_value: Some(stderr_ratio),
    }
}

fn benchmark_mc_rust_cpu_european_call_latin_hypercube(repeats: usize) -> BenchmarkResult {
    let cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        sampling: SamplingMethod::LatinHypercube,
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
        benchmark_name: "mc_cpu_european_call_rust_latin_hypercube".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::european_call_price_mc_cpu_stepwise".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("stepwise_paths_latin_hypercube".to_string()),
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

fn benchmark_mc_rust_cpu_european_call_latin_hypercube_control_variate_quality() -> BenchmarkResult
{
    let standard_cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 7_251,
        sampling: SamplingMethod::LatinHypercube,
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
        benchmark_name: "mc_cpu_european_call_rust_latin_hypercube_control_variate_quality"
            .to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::european_call_price_mc_cpu_stepwise".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("stepwise_paths_latin_hypercube_control_variate".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: 0.0,
        per_iteration_us: 0.0,
        throughput_per_sec: 0.0,
        metric_name: Some("stderr_ratio_vs_standard".to_string()),
        metric_value: Some(stderr_ratio),
    }
}

fn benchmark_mc_rust_cpu_european_structured(
    repeats: usize,
    sampling: SamplingMethod,
    benchmark_name: &str,
    methodology: &str,
) -> BenchmarkResult {
    let cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        sampling,
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
        benchmark_name: benchmark_name.to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::european_call_price_mc_cpu_stepwise".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(methodology.to_string()),
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

fn benchmark_mc_rust_cpu_european_structured_control_variate_quality(
    sampling: SamplingMethod,
    benchmark_name: &str,
    methodology: &str,
) -> BenchmarkResult {
    let standard_cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 7_261,
        sampling,
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
        benchmark_name: benchmark_name.to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::european_call_price_mc_cpu_stepwise".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(methodology.to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: 0.0,
        per_iteration_us: 0.0,
        throughput_per_sec: 0.0,
        metric_name: Some("stderr_ratio_vs_standard".to_string()),
        metric_value: Some(stderr_ratio),
    }
}

fn benchmark_mc_rust_cpu_european_pricing_quality(
    sampling: SamplingMethod,
    benchmark_name: &str,
    methodology: &str,
) -> BenchmarkResult {
    let cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 8_101,
        ..EuropeanCallConfig::default()
    };

    let started = Instant::now();
    let comparison = compare_european_call_sampling_quality_cpu(&cfg, sampling);
    let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

    BenchmarkResult {
        benchmark_name: benchmark_name.to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::compare_european_call_sampling_quality_cpu"
            .to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(methodology.to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: runtime_ms,
        per_iteration_us: runtime_ms * 1_000.0,
        throughput_per_sec: if runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            cfg.n_paths as f64 / (runtime_ms / 1_000.0)
        },
        metric_name: Some("stderr_ratio_vs_pseudorandom".to_string()),
        metric_value: Some(comparison.stderr_ratio_vs_pseudorandom),
    }
}

fn benchmark_mc_rust_cpu_european_realized_error(
    sampling: SamplingMethod,
    benchmark_name: &str,
    methodology: &str,
) -> BenchmarkResult {
    let cfg = EuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 8_501,
        ..EuropeanCallConfig::default()
    };

    let started = Instant::now();
    let comparison = compare_european_call_realized_error_cpu(&cfg, sampling);
    let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

    BenchmarkResult {
        benchmark_name: benchmark_name.to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::compare_european_call_realized_error_cpu"
            .to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(methodology.to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: runtime_ms,
        per_iteration_us: runtime_ms * 1_000.0,
        throughput_per_sec: if runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            cfg.n_paths as f64 / (runtime_ms / 1_000.0)
        },
        metric_name: Some("abs_error_ratio_vs_pseudorandom".to_string()),
        metric_value: Some(comparison.abs_error_ratio_vs_pseudorandom),
    }
}

fn benchmark_mc_rust_cpu_down_and_out_stepwise(repeats: usize) -> BenchmarkResult {
    let cfg = DownAndOutCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        ..DownAndOutCallConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = down_and_out_call_price_mc_cpu(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: "mc_cpu_down_and_out_call_rust".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::down_and_out_call_price_mc_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("down_and_out_stepwise".to_string()),
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

fn benchmark_mc_rust_cpu_down_and_out_stepwise_control_variate(repeats: usize) -> BenchmarkResult {
    let cfg = DownAndOutCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        technique: MonteCarloTechnique::ControlVariate,
        ..DownAndOutCallConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = down_and_out_call_price_mc_cpu(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: "mc_cpu_down_and_out_call_rust_control_variate".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::down_and_out_call_price_mc_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("down_and_out_stepwise_control_variate".to_string()),
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

fn benchmark_mc_rust_cpu_down_and_out_stepwise_control_variate_quality() -> BenchmarkResult {
    let standard_cfg = DownAndOutCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 7_311,
        ..DownAndOutCallConfig::default()
    };
    let control_cfg = DownAndOutCallConfig {
        technique: MonteCarloTechnique::ControlVariate,
        ..standard_cfg
    };

    let standard = down_and_out_call_price_mc_cpu(&standard_cfg);
    let control = down_and_out_call_price_mc_cpu(&control_cfg);
    let stderr_ratio = if standard.stderr == 0.0 {
        1.0
    } else {
        control.stderr / standard.stderr
    };

    BenchmarkResult {
        benchmark_name: "mc_cpu_down_and_out_call_rust_control_variate_quality".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::down_and_out_call_price_mc_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("down_and_out_stepwise_control_variate".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: 0.0,
        per_iteration_us: 0.0,
        throughput_per_sec: 0.0,
        metric_name: Some("stderr_ratio_vs_standard".to_string()),
        metric_value: Some(stderr_ratio),
    }
}

fn benchmark_mc_rust_cpu_lookback_stepwise(repeats: usize) -> BenchmarkResult {
    let cfg = LookbackCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        ..LookbackCallConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = lookback_call_price_mc_cpu(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: "mc_cpu_lookback_call_rust".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::lookback_call_price_mc_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("lookback_fixed_strike_stepwise".to_string()),
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

fn benchmark_mc_rust_cpu_lookback_stepwise_control_variate(repeats: usize) -> BenchmarkResult {
    let cfg = LookbackCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        technique: MonteCarloTechnique::ControlVariate,
        ..LookbackCallConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = lookback_call_price_mc_cpu(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: "mc_cpu_lookback_call_rust_control_variate".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::lookback_call_price_mc_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("lookback_fixed_strike_stepwise_control_variate".to_string()),
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

fn benchmark_mc_rust_cpu_american_put_lsm(repeats: usize) -> BenchmarkResult {
    let cfg = AmericanPutConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        ..AmericanPutConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = american_put_price_lsm_cpu(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: "mc_cpu_american_put_lsm_rust".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::american_put_price_lsm_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("american_put_longstaff_schwartz_laguerre".to_string()),
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

fn benchmark_mc_rust_cpu_bermudan_put_lsm(repeats: usize) -> BenchmarkResult {
    let cfg = BermudanPutConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        exercise_steps: vec![16, 32, 48, 64],
        ..BermudanPutConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg.clone();
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = bermudan_put_price_lsm_cpu(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: "mc_cpu_bermudan_put_lsm_rust".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::bermudan_put_price_lsm_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("bermudan_put_longstaff_schwartz_laguerre_custom_schedule".to_string()),
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

fn benchmark_mc_rust_cpu_american_put_lsm_binomial_reference() -> BenchmarkResult {
    let cfg = AmericanPutConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 14_007,
        ..AmericanPutConfig::default()
    };
    let reference_steps = 512;

    let started = Instant::now();
    let comparison = compare_american_put_lsm_binomial_reference_cpu(&cfg, reference_steps);
    let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

    BenchmarkResult {
        benchmark_name: "mc_cpu_american_put_lsm_binomial_reference_quality".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::compare_american_put_lsm_binomial_reference_cpu"
            .to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("american_put_lsm_vs_crr_binomial_reference".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: runtime_ms,
        per_iteration_us: runtime_ms * 1_000.0,
        throughput_per_sec: if runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            cfg.n_paths as f64 / (runtime_ms / 1_000.0)
        },
        metric_name: Some("abs_error_vs_binomial_reference".to_string()),
        metric_value: Some(comparison.abs_error),
    }
}

fn benchmark_mc_rust_cpu_bermudan_put_lsm_binomial_reference() -> BenchmarkResult {
    let cfg = BermudanPutConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        exercise_steps: vec![16, 32, 48, 64],
        seed: 14_008,
        ..BermudanPutConfig::default()
    };
    let reference_steps = 512;

    let started = Instant::now();
    let comparison = compare_bermudan_put_lsm_binomial_reference_cpu(&cfg, reference_steps);
    let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

    BenchmarkResult {
        benchmark_name: "mc_cpu_bermudan_put_lsm_binomial_reference_quality".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::compare_bermudan_put_lsm_binomial_reference_cpu"
            .to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("bermudan_put_lsm_vs_crr_binomial_reference".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: runtime_ms,
        per_iteration_us: runtime_ms * 1_000.0,
        throughput_per_sec: if runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            cfg.n_paths as f64 / (runtime_ms / 1_000.0)
        },
        metric_name: Some("abs_error_vs_binomial_reference".to_string()),
        metric_value: Some(comparison.abs_error),
    }
}

fn benchmark_mc_rust_cpu_lookback_pricing_quality(
    sampling: SamplingMethod,
    benchmark_name: &str,
    methodology: &str,
) -> BenchmarkResult {
    let cfg = LookbackCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 8_901,
        ..LookbackCallConfig::default()
    };

    let started = Instant::now();
    let comparison = compare_lookback_call_sampling_quality_cpu(&cfg, sampling);
    let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

    BenchmarkResult {
        benchmark_name: benchmark_name.to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::compare_lookback_call_sampling_quality_cpu"
            .to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(methodology.to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: runtime_ms,
        per_iteration_us: runtime_ms * 1_000.0,
        throughput_per_sec: if runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            cfg.n_paths as f64 / (runtime_ms / 1_000.0)
        },
        metric_name: Some("stderr_ratio_vs_pseudorandom".to_string()),
        metric_value: Some(comparison.stderr_ratio_vs_pseudorandom),
    }
}

fn benchmark_mc_rust_cpu_heston_european_stepwise(repeats: usize) -> BenchmarkResult {
    let cfg = HestonEuropeanCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        ..HestonEuropeanCallConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = heston_european_call_price_mc_cpu(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: "mc_cpu_heston_european_call_rust".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::heston_european_call_price_mc_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("heston_full_truncation_euler_stepwise".to_string()),
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

fn benchmark_mc_rust_cpu_heston_black_scholes_limit() -> BenchmarkResult {
    let sigma = 0.2_f64;
    let variance = sigma * sigma;
    let cfg = HestonEuropeanCallConfig {
        v0: variance,
        theta: variance,
        vol_of_vol: 0.0,
        rho: 0.0,
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 9_301,
        ..HestonEuropeanCallConfig::default()
    };

    let started = Instant::now();
    let comparison = compare_heston_black_scholes_limit_cpu(&cfg);
    let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

    BenchmarkResult {
        benchmark_name: "mc_cpu_heston_black_scholes_limit_quality".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::compare_heston_black_scholes_limit_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("heston_black_scholes_vol_of_vol_zero_limit".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: runtime_ms,
        per_iteration_us: runtime_ms * 1_000.0,
        throughput_per_sec: if runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            cfg.n_paths as f64 / (runtime_ms / 1_000.0)
        },
        metric_name: Some("abs_error_vs_black_scholes".to_string()),
        metric_value: Some(comparison.abs_error),
    }
}

fn benchmark_mc_rust_cpu_merton_jump_diffusion_call(repeats: usize) -> BenchmarkResult {
    let cfg = MertonJumpDiffusionCallConfig {
        n_paths: MC_PATHS,
        seed: 9_303,
        ..MertonJumpDiffusionCallConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = merton_jump_diffusion_call_price_mc_cpu(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: "mc_cpu_merton_jump_diffusion_call_rust".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::merton_jump_diffusion_call_price_mc_cpu"
            .to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("merton_jump_diffusion_terminal_poisson_lognormal".to_string()),
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

fn benchmark_mc_rust_cpu_merton_jump_diffusion_reference_quality() -> BenchmarkResult {
    let cfg = MertonJumpDiffusionCallConfig {
        n_paths: MC_PATHS,
        seed: 9_304,
        ..MertonJumpDiffusionCallConfig::default()
    };
    let reference = merton_jump_diffusion_call_reference_price(&cfg, 96, 1.0e-12);

    let started = Instant::now();
    let result = merton_jump_diffusion_call_price_mc_cpu(&cfg);
    let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
    let abs_error = (result.price - reference).abs();

    BenchmarkResult {
        benchmark_name: "mc_cpu_merton_jump_diffusion_reference_quality".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::merton_jump_diffusion_call_price_mc_cpu"
            .to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("merton_jump_diffusion_merton_series_reference".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: runtime_ms,
        per_iteration_us: runtime_ms * 1_000.0,
        throughput_per_sec: if runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            cfg.n_paths as f64 / (runtime_ms / 1_000.0)
        },
        metric_name: Some("abs_error_vs_merton_series".to_string()),
        metric_value: Some(abs_error),
    }
}

fn benchmark_mc_rust_cpu_european_greeks(estimator: GreekEstimator) -> BenchmarkResult {
    let cfg = EuropeanCallConfig {
        n_paths: GREEK_PATHS,
        n_steps: GREEK_STEPS,
        seed: 9_401,
        ..EuropeanCallConfig::default()
    };
    let analytic = black_scholes_european_call_greeks(cfg.s0, cfg.k, cfg.r, cfg.sigma, cfg.t);

    let started = Instant::now();
    let report = european_call_greeks_cpu(&cfg, estimator);
    let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
    let delta = report
        .estimate(Greek::Delta)
        .expect("European Greek benchmark should emit Delta");
    let abs_error = (delta.value - analytic.delta).abs();
    let estimator_name = greek_estimator_name(estimator);

    BenchmarkResult {
        benchmark_name: format!("mc_cpu_european_call_greeks_{estimator_name}_rust"),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::european_call_greeks_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(format!("european_call_greeks_{estimator_name}")),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: runtime_ms,
        per_iteration_us: runtime_ms * 1_000.0,
        throughput_per_sec: if runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            cfg.n_paths as f64 / (runtime_ms / 1_000.0)
        },
        metric_name: Some("abs_delta_error_vs_black_scholes".to_string()),
        metric_value: Some(abs_error),
    }
}

fn benchmark_mc_rust_cpu_heston_black_scholes_limit_greeks() -> BenchmarkResult {
    let sigma = 0.2_f64;
    let variance = sigma * sigma;
    let cfg = HestonEuropeanCallConfig {
        v0: variance,
        theta: variance,
        vol_of_vol: 0.0,
        rho: 0.0,
        n_paths: GREEK_PATHS,
        n_steps: GREEK_STEPS,
        seed: 9_402,
        ..HestonEuropeanCallConfig::default()
    };
    let analytic = black_scholes_european_call_greeks(cfg.s0, cfg.k, cfg.r, sigma, cfg.t);

    let started = Instant::now();
    let report = heston_european_call_greeks_cpu(&cfg, GreekEstimator::BumpAndRevalue);
    let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
    let delta = report
        .estimate(Greek::Delta)
        .expect("Heston Greek benchmark should emit Delta");
    let abs_error = (delta.value - analytic.delta).abs();

    BenchmarkResult {
        benchmark_name: "mc_cpu_heston_greeks_black_scholes_limit_delta_quality".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::heston_european_call_greeks_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("heston_greeks_bump_black_scholes_limit".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: runtime_ms,
        per_iteration_us: runtime_ms * 1_000.0,
        throughput_per_sec: if runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            cfg.n_paths as f64 / (runtime_ms / 1_000.0)
        },
        metric_name: Some("abs_delta_error_vs_black_scholes".to_string()),
        metric_value: Some(abs_error),
    }
}

fn benchmark_mc_rust_cpu_all_workload_bump_greeks() -> BenchmarkResult {
    let started = Instant::now();
    let reports =
        price_all_current_greeks_bump_and_revalue_cpu(GREEK_PATHS / 2, GREEK_STEPS, 9_403);
    let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
    let estimate_count = reports
        .iter()
        .map(|report| report.estimates.len())
        .sum::<usize>();

    BenchmarkResult {
        benchmark_name: "mc_cpu_all_workload_greeks_bump_rust".to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::price_all_current_greeks_bump_and_revalue_cpu"
            .to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some("all_current_workload_greeks_bump_and_revalue".to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: runtime_ms,
        per_iteration_us: runtime_ms * 1_000.0,
        throughput_per_sec: if runtime_ms == 0.0 {
            estimate_count as f64
        } else {
            estimate_count as f64 / (runtime_ms / 1_000.0)
        },
        metric_name: Some("greek_estimate_count".to_string()),
        metric_value: Some(estimate_count as f64),
    }
}

fn greek_estimator_name(estimator: GreekEstimator) -> &'static str {
    match estimator {
        GreekEstimator::BumpAndRevalue => "bump",
        GreekEstimator::Pathwise => "pathwise",
        GreekEstimator::LikelihoodRatio => "likelihood_ratio",
    }
}

fn benchmark_mc_rust_cpu_asian_structured(
    repeats: usize,
    sampling: SamplingMethod,
    benchmark_name: &str,
    methodology: &str,
) -> BenchmarkResult {
    let cfg = ArithmeticAsianCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        sampling,
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
        benchmark_name: benchmark_name.to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::arithmetic_asian_call_price_mc_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(methodology.to_string()),
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

fn benchmark_mc_rust_cpu_asian_structured_control_variate_quality(
    sampling: SamplingMethod,
    benchmark_name: &str,
    methodology: &str,
) -> BenchmarkResult {
    let standard_cfg = ArithmeticAsianCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 7_321,
        sampling,
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
        benchmark_name: benchmark_name.to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::arithmetic_asian_call_price_mc_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(methodology.to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: 0.0,
        per_iteration_us: 0.0,
        throughput_per_sec: 0.0,
        metric_name: Some("stderr_ratio_vs_standard".to_string()),
        metric_value: Some(stderr_ratio),
    }
}

fn benchmark_mc_rust_cpu_asian_pricing_quality(
    sampling: SamplingMethod,
    benchmark_name: &str,
    methodology: &str,
) -> BenchmarkResult {
    let cfg = ArithmeticAsianCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 8_201,
        ..ArithmeticAsianCallConfig::default()
    };

    let started = Instant::now();
    let comparison = compare_arithmetic_asian_sampling_quality_cpu(&cfg, sampling);
    let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

    BenchmarkResult {
        benchmark_name: benchmark_name.to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::compare_arithmetic_asian_sampling_quality_cpu"
            .to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(methodology.to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: runtime_ms,
        per_iteration_us: runtime_ms * 1_000.0,
        throughput_per_sec: if runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            cfg.n_paths as f64 / (runtime_ms / 1_000.0)
        },
        metric_name: Some("stderr_ratio_vs_pseudorandom".to_string()),
        metric_value: Some(comparison.stderr_ratio_vs_pseudorandom),
    }
}

fn benchmark_mc_rust_cpu_down_and_out_structured(
    repeats: usize,
    sampling: SamplingMethod,
    benchmark_name: &str,
    methodology: &str,
) -> BenchmarkResult {
    let cfg = DownAndOutCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        sampling,
        ..DownAndOutCallConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = down_and_out_call_price_mc_cpu(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: benchmark_name.to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::down_and_out_call_price_mc_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(methodology.to_string()),
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

fn benchmark_mc_rust_cpu_down_and_out_structured_control_variate_quality(
    sampling: SamplingMethod,
    benchmark_name: &str,
    methodology: &str,
) -> BenchmarkResult {
    let standard_cfg = DownAndOutCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 7_421,
        sampling,
        ..DownAndOutCallConfig::default()
    };
    let control_cfg = DownAndOutCallConfig {
        technique: MonteCarloTechnique::ControlVariate,
        ..standard_cfg
    };

    let standard = down_and_out_call_price_mc_cpu(&standard_cfg);
    let control = down_and_out_call_price_mc_cpu(&control_cfg);
    let stderr_ratio = if standard.stderr == 0.0 {
        1.0
    } else {
        control.stderr / standard.stderr
    };

    BenchmarkResult {
        benchmark_name: benchmark_name.to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::down_and_out_call_price_mc_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(methodology.to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: 0.0,
        per_iteration_us: 0.0,
        throughput_per_sec: 0.0,
        metric_name: Some("stderr_ratio_vs_standard".to_string()),
        metric_value: Some(stderr_ratio),
    }
}

fn benchmark_mc_rust_cpu_down_and_out_pricing_quality(
    sampling: SamplingMethod,
    benchmark_name: &str,
    methodology: &str,
) -> BenchmarkResult {
    let cfg = DownAndOutCallConfig {
        n_paths: MC_PATHS,
        n_steps: MC_STEPS,
        seed: 8_301,
        ..DownAndOutCallConfig::default()
    };

    let started = Instant::now();
    let comparison = compare_down_and_out_sampling_quality_cpu(&cfg, sampling);
    let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

    BenchmarkResult {
        benchmark_name: benchmark_name.to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::compare_down_and_out_sampling_quality_cpu"
            .to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(methodology.to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: runtime_ms,
        per_iteration_us: runtime_ms * 1_000.0,
        throughput_per_sec: if runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            cfg.n_paths as f64 / (runtime_ms / 1_000.0)
        },
        metric_name: Some("stderr_ratio_vs_pseudorandom".to_string()),
        metric_value: Some(comparison.stderr_ratio_vs_pseudorandom),
    }
}

fn benchmark_mc_rust_cpu_basket(
    repeats: usize,
    sampling: SamplingMethod,
    benchmark_name: &str,
    methodology: &str,
) -> BenchmarkResult {
    let cfg = BasketCallConfig {
        n_paths: MC_PATHS,
        sampling,
        ..BasketCallConfig::default()
    };

    let mut runtimes = Vec::with_capacity(repeats);
    let mut prices = Vec::with_capacity(repeats);

    for i in 0..repeats {
        let mut cfg_i = cfg;
        cfg_i.seed = cfg.seed + i as u64;
        let started = Instant::now();
        let result = basket_call_price_mc_cpu(&cfg_i);
        let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;
        runtimes.push(runtime_ms);
        prices.push(result.price);
    }

    let avg_runtime_ms = runtimes.iter().sum::<f64>() / runtimes.len() as f64;
    let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;

    BenchmarkResult {
        benchmark_name: benchmark_name.to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::basket_call_price_mc_cpu".to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(methodology.to_string()),
        planner_mode: "n/a".to_string(),
        iterations: repeats,
        total_runtime_ms: avg_runtime_ms * repeats as f64,
        per_iteration_us: avg_runtime_ms * 1_000.0,
        throughput_per_sec: if avg_runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            cfg.n_paths as f64 / (avg_runtime_ms / 1_000.0)
        },
        metric_name: Some("price_estimate".to_string()),
        metric_value: Some(avg_price),
    }
}

fn benchmark_mc_rust_cpu_basket_pricing_quality(
    sampling: SamplingMethod,
    benchmark_name: &str,
    methodology: &str,
) -> BenchmarkResult {
    let cfg = BasketCallConfig {
        n_paths: MC_PATHS,
        seed: 8_401,
        ..BasketCallConfig::default()
    };

    let started = Instant::now();
    let comparison = compare_basket_call_sampling_quality_cpu(&cfg, sampling);
    let runtime_ms = started.elapsed().as_secs_f64() * 1_000.0;

    BenchmarkResult {
        benchmark_name: benchmark_name.to_string(),
        benchmark_version: "0.1".to_string(),
        implementation: "mc-core::runtime::cpu::compare_basket_call_sampling_quality_cpu"
            .to_string(),
        backend: "cpu_native".to_string(),
        methodology: Some(methodology.to_string()),
        planner_mode: "n/a".to_string(),
        iterations: 1,
        total_runtime_ms: runtime_ms,
        per_iteration_us: runtime_ms * 1_000.0,
        throughput_per_sec: if runtime_ms == 0.0 {
            cfg.n_paths as f64
        } else {
            cfg.n_paths as f64 / (runtime_ms / 1_000.0)
        },
        metric_name: Some("stderr_ratio_vs_pseudorandom".to_string()),
        metric_value: Some(comparison.stderr_ratio_vs_pseudorandom),
    }
}

fn benchmark_mc_native_metal_down_and_out_stepwise(_repeats: usize) -> Option<BenchmarkResult> {
    benchmark_mc_native_metal_barrier_variant(
        _repeats,
        MonteCarloTechnique::Standard,
        "mc_metal_down_and_out_call_native",
        "down_and_out_stepwise_native_metal",
    )
}

fn benchmark_mc_native_metal_down_and_out_stepwise_control_variate(
    _repeats: usize,
) -> Option<BenchmarkResult> {
    benchmark_mc_native_metal_barrier_variant(
        _repeats,
        MonteCarloTechnique::ControlVariate,
        "mc_metal_down_and_out_call_native_control_variate",
        "down_and_out_stepwise_native_metal_control_variate",
    )
}

fn benchmark_mc_native_metal_down_and_out_stepwise_control_variate_quality(
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
        let standard_cfg = DownAndOutCallConfig {
            n_paths: MC_PATHS,
            n_steps: MC_STEPS,
            seed: 8_211,
            ..DownAndOutCallConfig::default()
        };
        let control_cfg = DownAndOutCallConfig {
            technique: MonteCarloTechnique::ControlVariate,
            ..standard_cfg
        };

        let standard = backend
            .execute(
                &artifact,
                &BackendExecutionInput::DownAndOutCall(standard_cfg),
            )
            .ok()?;
        let adjusted = backend
            .execute(
                &artifact,
                &BackendExecutionInput::DownAndOutCall(control_cfg),
            )
            .ok()?;
        let stderr_ratio = if standard.stderr == 0.0 {
            1.0
        } else {
            adjusted.stderr / standard.stderr
        };

        Some(BenchmarkResult {
            benchmark_name: "mc_metal_down_and_out_call_native_control_variate_quality".to_string(),
            benchmark_version: "0.1".to_string(),
            implementation: "mc-core::backend::metal::AppleMetalBackend::execute".to_string(),
            backend: "apple_metal".to_string(),
            methodology: Some("down_and_out_stepwise_native_metal_control_variate".to_string()),
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

fn benchmark_mc_native_metal_barrier_variant(
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

        let warmup_cfg = DownAndOutCallConfig {
            n_paths: MC_PATHS,
            n_steps: MC_STEPS,
            seed: 4_399,
            technique,
            ..DownAndOutCallConfig::default()
        };
        backend
            .execute(
                &artifact,
                &BackendExecutionInput::DownAndOutCall(warmup_cfg),
            )
            .ok()?;

        for i in 0.._repeats {
            let cfg = DownAndOutCallConfig {
                n_paths: MC_PATHS,
                n_steps: MC_STEPS,
                seed: 4_400 + i as u64,
                technique,
                ..DownAndOutCallConfig::default()
            };

            let result = backend
                .execute(&artifact, &BackendExecutionInput::DownAndOutCall(cfg))
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
                let benchmark_name = if methodology.starts_with("standard_normal_generation") {
                    format!("mc_cpu_qmc_{}_generation", entry.library)
                } else if methodology.starts_with("terminal_distribution_gpu") {
                    format!("mc_gpu_european_call_{}", entry.library)
                } else if methodology.starts_with("lookback_fixed_strike") {
                    format!("mc_cpu_lookback_call_{}", entry.library)
                } else if methodology.starts_with("american_put_lsm") {
                    format!("mc_cpu_american_put_lsm_{}", entry.library)
                } else if methodology.starts_with("bermudan_put_lsm") {
                    format!("mc_cpu_bermudan_put_lsm_{}", entry.library)
                } else if methodology.starts_with("heston_") {
                    format!("mc_cpu_heston_european_call_{}", entry.library)
                } else {
                    match methodology.as_str() {
                        "terminal_distribution" => {
                            format!("mc_cpu_european_call_{}_terminal", entry.library)
                        }
                        _ => format!("mc_cpu_european_call_{}", entry.library),
                    }
                };
                let work_units = if methodology.starts_with("standard_normal_generation") {
                    n_paths.saturating_mul(n_steps) as f64
                } else {
                    n_paths as f64
                };
                BenchmarkResult {
                    benchmark_name,
                    benchmark_version: "0.1".to_string(),
                    implementation: format!("python::{0}", entry.library),
                    backend: if methodology.starts_with("terminal_distribution_gpu") {
                        "gpu_external".to_string()
                    } else {
                        "cpu_external".to_string()
                    },
                    methodology: Some(methodology),
                    planner_mode: "n/a".to_string(),
                    iterations: repeats,
                    total_runtime_ms: runtime_ms * repeats as f64,
                    per_iteration_us: runtime_ms * 1_000.0,
                    throughput_per_sec: if runtime_ms == 0.0 {
                        work_units
                    } else {
                        work_units / (runtime_ms / 1_000.0)
                    },
                    metric_name: Some(
                        entry
                            .metric_name
                            .unwrap_or_else(|| "price_estimate".to_string()),
                    ),
                    metric_value: entry.metric_value.or(entry.price),
                }
            } else {
                let methodology = entry.methodology.clone();
                BenchmarkResult {
                    benchmark_name: if let Some(ref methodology) = methodology {
                        if methodology.starts_with("standard_normal_generation") {
                            format!("mc_cpu_qmc_{}_generation_unavailable", entry.library)
                        } else if methodology.starts_with("terminal_distribution_gpu") {
                            format!("mc_gpu_european_call_{}_unavailable", entry.library)
                        } else if methodology.starts_with("lookback_fixed_strike") {
                            format!("mc_cpu_lookback_call_{}_unavailable", entry.library)
                        } else if methodology.starts_with("american_put_lsm") {
                            format!("mc_cpu_american_put_lsm_{}_unavailable", entry.library)
                        } else if methodology.starts_with("bermudan_put_lsm") {
                            format!("mc_cpu_bermudan_put_lsm_{}_unavailable", entry.library)
                        } else if methodology.starts_with("heston_") {
                            format!("mc_cpu_heston_european_call_{}_unavailable", entry.library)
                        } else if methodology == "terminal_distribution" {
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
                    backend: if methodology
                        .as_deref()
                        .is_some_and(|value| value.starts_with("terminal_distribution_gpu"))
                    {
                        "gpu_external".to_string()
                    } else {
                        "cpu_external".to_string()
                    },
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
    metric_name: Option<String>,
    metric_value: Option<f64>,
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
