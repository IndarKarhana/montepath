use std::f64::consts::PI;
use std::thread;

use serde::{Deserialize, Serialize};

const STANDARD_NORMAL_TWO_SIGMA_TAIL: f64 = 0.045_500_263_896_358_42;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EuropeanCallMethod {
    Auto,
    TerminalDistribution,
    StepwisePaths,
}

impl Default for EuropeanCallMethod {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MonteCarloTechnique {
    Standard,
    Antithetic,
    ControlVariate,
}

impl Default for MonteCarloTechnique {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SamplingMethod {
    Pseudorandom,
    RandomizedHalton,
    LatinHypercube,
    ScrambledSobol,
    ScrambledSobolBrownianBridge,
}

impl Default for SamplingMethod {
    fn default() -> Self {
        Self::Pseudorandom
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuredSamplingGuidance {
    pub sampling: SamplingMethod,
    pub n_points: usize,
    pub dimensions: usize,
    pub recommended_points: usize,
    pub is_power_of_two: bool,
    pub warnings: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StandardNormalDiagnostics {
    pub sample_count: usize,
    pub dimensions: usize,
    pub finite: bool,
    pub mean: f64,
    pub mean_abs: f64,
    pub variance: f64,
    pub variance_abs_error: f64,
    pub min: f64,
    pub max: f64,
    pub tail_2sigma_fraction: f64,
    pub tail_2sigma_abs_error: f64,
    pub max_axis_mean_abs: f64,
    pub max_axis_variance_abs_error: f64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PricingWorkloadFamily {
    EuropeanCall,
    ArithmeticAsianCall,
    DownAndOutCall,
    BasketCall,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PricingQualityComparison {
    pub workload: PricingWorkloadFamily,
    pub sampling: SamplingMethod,
    pub paths: usize,
    pub steps: usize,
    pub pseudorandom_price: f64,
    pub pseudorandom_stderr: f64,
    pub structured_price: f64,
    pub structured_stderr: f64,
    pub stderr_ratio_vs_pseudorandom: f64,
    pub price_delta: f64,
    pub price_delta_abs: f64,
    pub price_delta_stderr_units: f64,
    pub normal_diagnostics: StandardNormalDiagnostics,
    pub guidance: StructuredSamplingGuidance,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyticPricingComparison {
    pub workload: PricingWorkloadFamily,
    pub sampling: SamplingMethod,
    pub paths: usize,
    pub steps: usize,
    pub analytic_price: f64,
    pub pseudorandom_price: f64,
    pub pseudorandom_stderr: f64,
    pub pseudorandom_error: f64,
    pub pseudorandom_abs_error: f64,
    pub pseudorandom_error_stderr_units: f64,
    pub structured_price: f64,
    pub structured_stderr: f64,
    pub structured_error: f64,
    pub structured_abs_error: f64,
    pub structured_error_stderr_units: f64,
    pub abs_error_ratio_vs_pseudorandom: f64,
    pub abs_error_reduction_vs_pseudorandom: f64,
    pub normal_diagnostics: StandardNormalDiagnostics,
    pub guidance: StructuredSamplingGuidance,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct GaussianUncertaintyConfig {
    pub n_samples: usize,
    pub dimensions: usize,
    pub seed: u64,
    pub sampling: SamplingMethod,
}

impl Default for GaussianUncertaintyConfig {
    fn default() -> Self {
        Self {
            n_samples: 100_000,
            dimensions: 3,
            seed: 42,
            sampling: SamplingMethod::Pseudorandom,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct GaussianUncertaintyResult {
    pub mean: f64,
    pub stderr: f64,
    pub analytic_mean: f64,
    pub abs_error: f64,
}

pub fn structured_sampling_guidance_cpu(
    sampling: SamplingMethod,
    n_points: usize,
    dimensions: usize,
) -> StructuredSamplingGuidance {
    let is_power_of_two = n_points.is_power_of_two();
    let recommended_points = if is_power_of_two {
        n_points
    } else {
        n_points.next_power_of_two()
    };
    let mut warnings = Vec::new();
    let mut notes = Vec::new();

    if n_points == 0 {
        warnings.push("n_points must be positive for execution and diagnostics".to_string());
    }
    if dimensions == 0 {
        warnings.push("dimensions must be positive for execution and diagnostics".to_string());
    }

    match sampling {
        SamplingMethod::Pseudorandom => {
            notes.push(
                "pseudorandom sampling has no low-discrepancy sample-size requirement".to_string(),
            );
        }
        SamplingMethod::ScrambledSobol | SamplingMethod::ScrambledSobolBrownianBridge => {
            if !is_power_of_two {
                warnings.push(format!(
                    "Sobol balance is strongest at powers of two; use {recommended_points} points for the next balanced size"
                ));
            }
            if dimensions > sobol_burley::NUM_DIMENSIONS as usize {
                warnings.push(format!(
                    "requested dimensions exceed the Sobol table size {}; higher dimensions wrap with seed offsets",
                    sobol_burley::NUM_DIMENSIONS
                ));
            }
            notes.push("scrambled Sobol is best for smooth low-to-moderate-dimensional integrands and replicated scrambling for error estimates".to_string());
            if sampling == SamplingMethod::ScrambledSobolBrownianBridge {
                notes.push("Brownian bridge maps early Sobol dimensions to coarse path features and is intended for path construction".to_string());
            }
        }
        SamplingMethod::RandomizedHalton => {
            if dimensions > 32 {
                warnings.push(
                    "Halton quality can degrade in higher dimensions; prefer Sobol or workload-specific validation above roughly 32 dimensions"
                        .to_string(),
                );
            }
            notes.push("randomized Halton is useful for simple structured-sampling breadth and deterministic diagnostics".to_string());
        }
        SamplingMethod::LatinHypercube => {
            notes.push("Latin hypercube balances one-dimensional marginals; it is not a full low-discrepancy sequence for path ordering".to_string());
            if n_points < dimensions.saturating_mul(16) {
                warnings.push(
                    "Latin hypercube has few strata per effective dimension at this size; validate estimator quality before recommending it"
                        .to_string(),
                );
            }
        }
    }

    StructuredSamplingGuidance {
        sampling,
        n_points,
        dimensions,
        recommended_points,
        is_power_of_two,
        warnings,
        notes,
    }
}

pub fn diagnose_standard_normal_samples_cpu(
    samples: &[f64],
    dimensions: usize,
) -> StandardNormalDiagnostics {
    let sample_count = samples.len();
    let mut warnings = Vec::new();

    if sample_count == 0 {
        warnings.push("no samples were provided".to_string());
        return StandardNormalDiagnostics {
            sample_count,
            dimensions,
            finite: true,
            mean: 0.0,
            mean_abs: 0.0,
            variance: 0.0,
            variance_abs_error: 1.0,
            min: 0.0,
            max: 0.0,
            tail_2sigma_fraction: 0.0,
            tail_2sigma_abs_error: STANDARD_NORMAL_TWO_SIGMA_TAIL,
            max_axis_mean_abs: 0.0,
            max_axis_variance_abs_error: 0.0,
            warnings,
        };
    }

    if dimensions == 0 {
        warnings.push("dimensions should be positive for axis diagnostics".to_string());
    } else if !sample_count.is_multiple_of(dimensions) {
        warnings.push(
            "sample count is not an exact multiple of dimensions; trailing axis diagnostics are partial"
                .to_string(),
        );
    }

    let finite = samples.iter().all(|value| value.is_finite());
    if !finite {
        warnings.push("samples contain non-finite values".to_string());
    }

    let mean = samples.iter().sum::<f64>() / sample_count as f64;
    let variance = samples
        .iter()
        .map(|value| {
            let centered = value - mean;
            centered * centered
        })
        .sum::<f64>()
        / sample_count as f64;
    let min = samples.iter().copied().fold(f64::INFINITY, f64::min);
    let max = samples.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let tail_2sigma_fraction =
        samples.iter().filter(|value| value.abs() > 2.0).count() as f64 / sample_count as f64;

    let (max_axis_mean_abs, max_axis_variance_abs_error) = if dimensions == 0 {
        (0.0, 0.0)
    } else {
        max_axis_normal_moment_errors(samples, dimensions)
    };

    StandardNormalDiagnostics {
        sample_count,
        dimensions,
        finite,
        mean,
        mean_abs: mean.abs(),
        variance,
        variance_abs_error: (variance - 1.0).abs(),
        min,
        max,
        tail_2sigma_fraction,
        tail_2sigma_abs_error: (tail_2sigma_fraction - STANDARD_NORMAL_TWO_SIGMA_TAIL).abs(),
        max_axis_mean_abs,
        max_axis_variance_abs_error,
        warnings,
    }
}

pub fn diagnose_standard_normals_cpu(
    sampling: SamplingMethod,
    n_points: usize,
    dimensions: usize,
    seed: u64,
) -> StandardNormalDiagnostics {
    let samples = generate_standard_normals_cpu(sampling, n_points, dimensions, seed);
    diagnose_standard_normal_samples_cpu(&samples, dimensions)
}

pub fn compare_european_call_sampling_quality_cpu(
    cfg: &EuropeanCallConfig,
    sampling: SamplingMethod,
) -> PricingQualityComparison {
    let mut baseline_cfg = *cfg;
    baseline_cfg.sampling = SamplingMethod::Pseudorandom;
    baseline_cfg.technique = MonteCarloTechnique::Standard;

    let mut structured_cfg = baseline_cfg;
    structured_cfg.sampling = sampling;

    let baseline = european_call_price_mc_cpu_stepwise(&baseline_cfg);
    let structured = european_call_price_mc_cpu_stepwise(&structured_cfg);

    build_pricing_quality_comparison(
        PricingWorkloadFamily::EuropeanCall,
        sampling,
        cfg.n_paths,
        cfg.n_steps,
        cfg.seed,
        baseline,
        structured,
    )
}

pub fn compare_european_call_realized_error_cpu(
    cfg: &EuropeanCallConfig,
    sampling: SamplingMethod,
) -> AnalyticPricingComparison {
    let mut baseline_cfg = *cfg;
    baseline_cfg.sampling = SamplingMethod::Pseudorandom;
    baseline_cfg.technique = MonteCarloTechnique::Standard;

    let mut structured_cfg = baseline_cfg;
    structured_cfg.sampling = sampling;

    let baseline = european_call_price_mc_cpu_stepwise(&baseline_cfg);
    let structured = european_call_price_mc_cpu_stepwise(&structured_cfg);
    let analytic = black_scholes_european_call_price(cfg.s0, cfg.k, cfg.r, cfg.sigma, cfg.t);

    build_analytic_pricing_comparison(
        PricingWorkloadFamily::EuropeanCall,
        sampling,
        cfg.n_paths,
        cfg.n_steps,
        cfg.seed,
        analytic,
        baseline,
        structured,
    )
}

pub fn compare_arithmetic_asian_sampling_quality_cpu(
    cfg: &ArithmeticAsianCallConfig,
    sampling: SamplingMethod,
) -> PricingQualityComparison {
    let mut baseline_cfg = *cfg;
    baseline_cfg.sampling = SamplingMethod::Pseudorandom;
    baseline_cfg.technique = MonteCarloTechnique::Standard;

    let mut structured_cfg = baseline_cfg;
    structured_cfg.sampling = sampling;

    let baseline = arithmetic_asian_call_price_mc_cpu(&baseline_cfg);
    let structured = arithmetic_asian_call_price_mc_cpu(&structured_cfg);

    build_pricing_quality_comparison(
        PricingWorkloadFamily::ArithmeticAsianCall,
        sampling,
        cfg.n_paths,
        cfg.n_steps,
        cfg.seed,
        baseline,
        structured,
    )
}

pub fn compare_down_and_out_sampling_quality_cpu(
    cfg: &DownAndOutCallConfig,
    sampling: SamplingMethod,
) -> PricingQualityComparison {
    let mut baseline_cfg = *cfg;
    baseline_cfg.sampling = SamplingMethod::Pseudorandom;
    baseline_cfg.technique = MonteCarloTechnique::Standard;

    let mut structured_cfg = baseline_cfg;
    structured_cfg.sampling = sampling;

    let baseline = down_and_out_call_price_mc_cpu(&baseline_cfg);
    let structured = down_and_out_call_price_mc_cpu(&structured_cfg);

    build_pricing_quality_comparison(
        PricingWorkloadFamily::DownAndOutCall,
        sampling,
        cfg.n_paths,
        cfg.n_steps,
        cfg.seed,
        baseline,
        structured,
    )
}

pub fn compare_basket_call_sampling_quality_cpu(
    cfg: &BasketCallConfig,
    sampling: SamplingMethod,
) -> PricingQualityComparison {
    let mut baseline_cfg = *cfg;
    baseline_cfg.sampling = SamplingMethod::Pseudorandom;
    baseline_cfg.technique = MonteCarloTechnique::Standard;

    let mut structured_cfg = baseline_cfg;
    structured_cfg.sampling = sampling;

    let baseline = basket_call_price_mc_cpu(&baseline_cfg);
    let structured = basket_call_price_mc_cpu(&structured_cfg);

    build_pricing_quality_comparison(
        PricingWorkloadFamily::BasketCall,
        sampling,
        cfg.n_paths,
        2,
        cfg.seed,
        baseline,
        structured,
    )
}

pub fn gaussian_uncertainty_mean_cpu(cfg: &GaussianUncertaintyConfig) -> GaussianUncertaintyResult {
    assert!(cfg.n_samples > 0, "n_samples must be > 0");
    assert!(cfg.dimensions >= 3, "dimensions must be >= 3");

    let normals =
        generate_standard_normals_cpu(cfg.sampling, cfg.n_samples, cfg.dimensions, cfg.seed);
    let mut sum = 0.0;
    let mut sq_sum = 0.0;

    for row in normals.chunks_exact(cfg.dimensions) {
        let value = gaussian_uncertainty_response(row);
        sum += value;
        sq_sum += value * value;
    }

    let n = cfg.n_samples as f64;
    let mean = sum / n;
    let variance = if cfg.n_samples > 1 {
        ((sq_sum - (sum * sum / n)) / (n - 1.0)).max(0.0)
    } else {
        0.0
    };
    let stderr = (variance / n).sqrt();
    let analytic_mean = gaussian_uncertainty_analytic_mean();

    GaussianUncertaintyResult {
        mean,
        stderr,
        analytic_mean,
        abs_error: (mean - analytic_mean).abs(),
    }
}

fn gaussian_uncertainty_response(z: &[f64]) -> f64 {
    z[0] * z[0] + 0.5 * z[1] + (0.1 * z[2]).exp()
}

fn gaussian_uncertainty_analytic_mean() -> f64 {
    1.0 + 0.005f64.exp()
}

pub fn black_scholes_european_call_price(s0: f64, k: f64, r: f64, sigma: f64, t: f64) -> f64 {
    assert!(s0 > 0.0, "s0 must be > 0");
    assert!(k >= 0.0, "k must be >= 0");
    assert!(sigma >= 0.0, "sigma must be >= 0");
    assert!(t >= 0.0, "t must be >= 0");

    if k == 0.0 {
        return s0;
    }
    if t == 0.0 {
        return (s0 - k).max(0.0);
    }
    if sigma == 0.0 {
        return (s0 - k * (-r * t).exp()).max(0.0);
    }

    let sqrt_t = t.sqrt();
    let d1 = ((s0 / k).ln() + (r + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    s0 * standard_normal_cdf(d1) - k * (-r * t).exp() * standard_normal_cdf(d2)
}

fn standard_normal_cdf(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let z = x.abs() / 2f64.sqrt();
    let t = 1.0 / (1.0 + 0.3275911 * z);
    let a1 = 0.254_829_592;
    let a2 = -0.284_496_736;
    let a3 = 1.421_413_741;
    let a4 = -1.453_152_027;
    let a5 = 1.061_405_429;
    let poly = (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t;
    let erf_approx = 1.0 - poly * (-z * z).exp();
    0.5 * (1.0 + sign * erf_approx)
}

fn build_analytic_pricing_comparison(
    workload: PricingWorkloadFamily,
    sampling: SamplingMethod,
    paths: usize,
    steps: usize,
    seed: u64,
    analytic_price: f64,
    pseudorandom: EuropeanCallResult,
    structured: EuropeanCallResult,
) -> AnalyticPricingComparison {
    let pseudorandom_error = pseudorandom.price - analytic_price;
    let pseudorandom_abs_error = pseudorandom_error.abs();
    let structured_error = structured.price - analytic_price;
    let structured_abs_error = structured_error.abs();
    let denominator = pseudorandom_abs_error.max(f64::EPSILON);
    let abs_error_ratio_vs_pseudorandom = structured_abs_error / denominator;
    let abs_error_reduction_vs_pseudorandom = pseudorandom_abs_error - structured_abs_error;
    let pseudorandom_error_stderr_units = if pseudorandom.stderr == 0.0 {
        0.0
    } else {
        pseudorandom_error / pseudorandom.stderr
    };
    let structured_error_stderr_units = if structured.stderr == 0.0 {
        0.0
    } else {
        structured_error / structured.stderr
    };
    let normal_diagnostics = diagnose_standard_normals_cpu(sampling, paths, steps, seed);
    let guidance = structured_sampling_guidance_cpu(sampling, paths, steps);
    let mut warnings = Vec::new();

    if abs_error_ratio_vs_pseudorandom > 1.0 {
        warnings.push(format!(
            "structured realized error is {:.2}x the pseudorandom realized error for this analytic reference",
            abs_error_ratio_vs_pseudorandom
        ));
    }
    if structured_error_stderr_units.abs() > 3.0 {
        warnings.push(format!(
            "structured estimate is {:.2} standard errors from the analytic reference; validate with more paths or independent seeds",
            structured_error_stderr_units.abs()
        ));
    }
    warnings.extend(guidance.warnings.iter().cloned());
    warnings.extend(normal_diagnostics.warnings.iter().cloned());

    AnalyticPricingComparison {
        workload,
        sampling,
        paths,
        steps,
        analytic_price,
        pseudorandom_price: pseudorandom.price,
        pseudorandom_stderr: pseudorandom.stderr,
        pseudorandom_error,
        pseudorandom_abs_error,
        pseudorandom_error_stderr_units,
        structured_price: structured.price,
        structured_stderr: structured.stderr,
        structured_error,
        structured_abs_error,
        structured_error_stderr_units,
        abs_error_ratio_vs_pseudorandom,
        abs_error_reduction_vs_pseudorandom,
        normal_diagnostics,
        guidance,
        warnings,
    }
}

fn build_pricing_quality_comparison(
    workload: PricingWorkloadFamily,
    sampling: SamplingMethod,
    paths: usize,
    steps: usize,
    seed: u64,
    pseudorandom: EuropeanCallResult,
    structured: EuropeanCallResult,
) -> PricingQualityComparison {
    let price_delta = structured.price - pseudorandom.price;
    let combined_stderr =
        (pseudorandom.stderr * pseudorandom.stderr + structured.stderr * structured.stderr).sqrt();
    let price_delta_stderr_units = if combined_stderr == 0.0 {
        0.0
    } else {
        price_delta / combined_stderr
    };
    let stderr_ratio_vs_pseudorandom = if pseudorandom.stderr == 0.0 {
        1.0
    } else {
        structured.stderr / pseudorandom.stderr
    };
    let normal_diagnostics = diagnose_standard_normals_cpu(sampling, paths, steps, seed);
    let guidance = structured_sampling_guidance_cpu(sampling, paths, steps);
    let mut warnings = Vec::new();

    if price_delta_stderr_units.abs() > 3.0 {
        warnings.push(format!(
            "structured estimate differs from pseudorandom by {:.2} combined standard errors; validate with more paths or independent seeds",
            price_delta_stderr_units.abs()
        ));
    }
    if stderr_ratio_vs_pseudorandom > 1.0 {
        warnings.push(format!(
            "structured stderr is {:.2}x the pseudorandom stderr for this workload",
            stderr_ratio_vs_pseudorandom
        ));
    }
    warnings.extend(guidance.warnings.iter().cloned());
    warnings.extend(normal_diagnostics.warnings.iter().cloned());

    PricingQualityComparison {
        workload,
        sampling,
        paths,
        steps,
        pseudorandom_price: pseudorandom.price,
        pseudorandom_stderr: pseudorandom.stderr,
        structured_price: structured.price,
        structured_stderr: structured.stderr,
        stderr_ratio_vs_pseudorandom,
        price_delta,
        price_delta_abs: price_delta.abs(),
        price_delta_stderr_units,
        normal_diagnostics,
        guidance,
        warnings,
    }
}

fn max_axis_normal_moment_errors(samples: &[f64], dimensions: usize) -> (f64, f64) {
    let mut max_mean_abs = 0.0;
    let mut max_variance_abs_error = 0.0;

    for dim in 0..dimensions {
        let mut count = 0usize;
        let mut sum = 0.0;
        let mut sq_sum = 0.0;

        for value in samples.iter().skip(dim).step_by(dimensions) {
            count += 1;
            sum += *value;
            sq_sum += *value * *value;
        }

        if count == 0 {
            continue;
        }

        let mean = sum / count as f64;
        let variance = (sq_sum / count as f64) - mean * mean;
        let mean_abs = mean.abs();
        let variance_abs_error = (variance - 1.0).abs();
        if mean_abs > max_mean_abs {
            max_mean_abs = mean_abs;
        }
        if variance_abs_error > max_variance_abs_error {
            max_variance_abs_error = variance_abs_error;
        }
    }

    (max_mean_abs, max_variance_abs_error)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MonteCarloMethodCategory {
    Baseline,
    VarianceReduction,
    StructuredSampling,
    Multilevel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackendMethodSupport {
    Native,
    CpuReference,
    DelegatedCpuFallback,
    Planned,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MonteCarloMethodCapability {
    pub method_id: String,
    pub display_name: String,
    pub category: MonteCarloMethodCategory,
    pub cpu_native: BackendMethodSupport,
    pub apple_metal: BackendMethodSupport,
    pub nvidia_cuda: BackendMethodSupport,
    pub notes: Vec<String>,
}

pub fn monte_carlo_method_capabilities() -> Vec<MonteCarloMethodCapability> {
    vec![
        method_capability(
            "standard_mc",
            "Standard pseudorandom Monte Carlo",
            MonteCarloMethodCategory::Baseline,
            BackendMethodSupport::CpuReference,
            BackendMethodSupport::Native,
            BackendMethodSupport::DelegatedCpuFallback,
            &["baseline path simulation and terminal-distribution execution where applicable"],
        ),
        method_capability(
            "antithetic_variates",
            "Antithetic variates",
            MonteCarloMethodCategory::VarianceReduction,
            BackendMethodSupport::CpuReference,
            BackendMethodSupport::Native,
            BackendMethodSupport::DelegatedCpuFallback,
            &["native Metal support currently covers the European-call path family"],
        ),
        method_capability(
            "control_variates",
            "Control variates",
            MonteCarloMethodCategory::VarianceReduction,
            BackendMethodSupport::CpuReference,
            BackendMethodSupport::Native,
            BackendMethodSupport::DelegatedCpuFallback,
            &["current control uses discounted terminal stock with known expectation S0"],
        ),
        method_capability(
            "randomized_halton",
            "Randomized Halton sampling",
            MonteCarloMethodCategory::StructuredSampling,
            BackendMethodSupport::CpuReference,
            BackendMethodSupport::DelegatedCpuFallback,
            BackendMethodSupport::DelegatedCpuFallback,
            &["native GPU structured sampling is not implemented yet"],
        ),
        method_capability(
            "latin_hypercube",
            "Latin hypercube sampling",
            MonteCarloMethodCategory::StructuredSampling,
            BackendMethodSupport::CpuReference,
            BackendMethodSupport::DelegatedCpuFallback,
            BackendMethodSupport::DelegatedCpuFallback,
            &["implemented with deterministic per-dimension stratum permutations and seeded jitter"],
        ),
        method_capability(
            "scrambled_sobol",
            "Scrambled Sobol sampling",
            MonteCarloMethodCategory::StructuredSampling,
            BackendMethodSupport::CpuReference,
            BackendMethodSupport::DelegatedCpuFallback,
            BackendMethodSupport::DelegatedCpuFallback,
            &["CPU reference path uses Owen-scrambled Sobol sampling"],
        ),
        method_capability(
            "scrambled_sobol_brownian_bridge",
            "Scrambled Sobol with Brownian bridge path construction",
            MonteCarloMethodCategory::StructuredSampling,
            BackendMethodSupport::CpuReference,
            BackendMethodSupport::DelegatedCpuFallback,
            BackendMethodSupport::DelegatedCpuFallback,
            &["Brownian bridge is available for step-wise GBM path workloads"],
        ),
        method_capability(
            "multilevel_monte_carlo",
            "Multilevel Monte Carlo",
            MonteCarloMethodCategory::Multilevel,
            BackendMethodSupport::CpuReference,
            BackendMethodSupport::Planned,
            BackendMethodSupport::Planned,
            &["CPU reference support currently covers arithmetic Asian calls with coupled fine/coarse GBM paths"],
        ),
        method_capability(
            "multilevel_randomized_qmc",
            "Multilevel randomized quasi-Monte Carlo",
            MonteCarloMethodCategory::Multilevel,
            BackendMethodSupport::CpuReference,
            BackendMethodSupport::Planned,
            BackendMethodSupport::Planned,
            &["CPU reference support currently covers arithmetic Asian MLMC with scrambled Sobol increments"],
        ),
    ]
}

fn method_capability(
    method_id: &str,
    display_name: &str,
    category: MonteCarloMethodCategory,
    cpu_native: BackendMethodSupport,
    apple_metal: BackendMethodSupport,
    nvidia_cuda: BackendMethodSupport,
    notes: &[&str],
) -> MonteCarloMethodCapability {
    MonteCarloMethodCapability {
        method_id: method_id.to_string(),
        display_name: display_name.to_string(),
        category,
        cpu_native,
        apple_metal,
        nvidia_cuda,
        notes: notes.iter().map(|note| note.to_string()).collect(),
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct EuropeanCallConfig {
    pub s0: f64,
    pub k: f64,
    pub r: f64,
    pub sigma: f64,
    pub t: f64,
    pub n_paths: usize,
    pub n_steps: usize,
    pub seed: u64,
    pub n_threads: usize,
    pub technique: MonteCarloTechnique,
    pub sampling: SamplingMethod,
}

impl Default for EuropeanCallConfig {
    fn default() -> Self {
        Self {
            s0: 100.0,
            k: 100.0,
            r: 0.03,
            sigma: 0.2,
            t: 1.0,
            n_paths: 100_000,
            n_steps: 252,
            seed: 42,
            n_threads: 0,
            technique: MonteCarloTechnique::Standard,
            sampling: SamplingMethod::Pseudorandom,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct EuropeanCallResult {
    pub price: f64,
    pub stderr: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ArithmeticAsianCallConfig {
    pub s0: f64,
    pub k: f64,
    pub r: f64,
    pub sigma: f64,
    pub t: f64,
    pub n_paths: usize,
    pub n_steps: usize,
    pub seed: u64,
    pub n_threads: usize,
    pub technique: MonteCarloTechnique,
    pub sampling: SamplingMethod,
}

impl Default for ArithmeticAsianCallConfig {
    fn default() -> Self {
        Self {
            s0: 100.0,
            k: 100.0,
            r: 0.03,
            sigma: 0.2,
            t: 1.0,
            n_paths: 100_000,
            n_steps: 252,
            seed: 42,
            n_threads: 0,
            technique: MonteCarloTechnique::Standard,
            sampling: SamplingMethod::Pseudorandom,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArithmeticAsianMlmcConfig {
    pub s0: f64,
    pub k: f64,
    pub r: f64,
    pub sigma: f64,
    pub t: f64,
    pub base_steps: usize,
    pub levels: usize,
    pub refinement_factor: usize,
    pub paths_per_level: Vec<usize>,
    pub seed: u64,
    pub sampling: SamplingMethod,
    pub scramble_replicates: usize,
}

impl Default for ArithmeticAsianMlmcConfig {
    fn default() -> Self {
        Self {
            s0: 100.0,
            k: 100.0,
            r: 0.03,
            sigma: 0.2,
            t: 1.0,
            base_steps: 16,
            levels: 4,
            refinement_factor: 2,
            paths_per_level: vec![50_000, 25_000, 12_500, 6_250],
            seed: 42,
            sampling: SamplingMethod::Pseudorandom,
            scramble_replicates: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArithmeticAsianMlmcLevelResult {
    pub level: usize,
    pub paths: usize,
    pub fine_steps: usize,
    pub coarse_steps: Option<usize>,
    pub mean: f64,
    pub variance: f64,
    pub stderr: f64,
    pub cost_step_updates: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArithmeticAsianMlmcResult {
    pub price: f64,
    pub stderr: f64,
    pub total_paths: usize,
    pub total_step_updates: usize,
    pub scramble_replicates: usize,
    pub replicate_estimates: Vec<f64>,
    pub levels: Vec<ArithmeticAsianMlmcLevelResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArithmeticAsianMlmcAllocationLevel {
    pub level: usize,
    pub fine_steps: usize,
    pub coarse_steps: Option<usize>,
    pub pilot_paths: usize,
    pub pilot_variance: f64,
    pub cost_per_path: usize,
    pub recommended_paths: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArithmeticAsianMlmcAllocationPlan {
    pub target_step_updates: usize,
    pub estimated_step_updates: usize,
    pub paths_per_level: Vec<usize>,
    pub levels: Vec<ArithmeticAsianMlmcAllocationLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArithmeticAsianMlmcToleranceConfig {
    pub target_stderr: f64,
    pub pilot_paths_per_level: usize,
    pub min_step_updates: usize,
    pub max_step_updates: usize,
}

impl Default for ArithmeticAsianMlmcToleranceConfig {
    fn default() -> Self {
        Self {
            target_stderr: 0.05,
            pilot_paths_per_level: 2_048,
            min_step_updates: 100_000,
            max_step_updates: 10_000_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArithmeticAsianMlmcTolerancePlan {
    pub target_stderr: f64,
    pub estimated_stderr: f64,
    pub target_met: bool,
    pub max_step_updates_hit: bool,
    pub scramble_replicates: usize,
    pub estimated_step_updates: usize,
    pub paths_per_level: Vec<usize>,
    pub allocation: ArithmeticAsianMlmcAllocationPlan,
    pub recommended_config: ArithmeticAsianMlmcConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct DownAndOutCallConfig {
    pub s0: f64,
    pub k: f64,
    pub barrier: f64,
    pub r: f64,
    pub sigma: f64,
    pub t: f64,
    pub n_paths: usize,
    pub n_steps: usize,
    pub seed: u64,
    pub n_threads: usize,
    pub technique: MonteCarloTechnique,
    pub sampling: SamplingMethod,
}

impl Default for DownAndOutCallConfig {
    fn default() -> Self {
        Self {
            s0: 100.0,
            k: 100.0,
            barrier: 80.0,
            r: 0.03,
            sigma: 0.2,
            t: 1.0,
            n_paths: 100_000,
            n_steps: 252,
            seed: 42,
            n_threads: 0,
            technique: MonteCarloTechnique::Standard,
            sampling: SamplingMethod::Pseudorandom,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BasketCallConfig {
    pub s01: f64,
    pub s02: f64,
    pub k: f64,
    pub r: f64,
    pub sigma1: f64,
    pub sigma2: f64,
    pub rho: f64,
    pub weight1: f64,
    pub weight2: f64,
    pub t: f64,
    pub n_paths: usize,
    pub seed: u64,
    pub n_threads: usize,
    pub technique: MonteCarloTechnique,
    pub sampling: SamplingMethod,
}

impl Default for BasketCallConfig {
    fn default() -> Self {
        Self {
            s01: 100.0,
            s02: 95.0,
            k: 100.0,
            r: 0.03,
            sigma1: 0.2,
            sigma2: 0.25,
            rho: 0.35,
            weight1: 0.5,
            weight2: 0.5,
            t: 1.0,
            n_paths: 100_000,
            seed: 42,
            n_threads: 0,
            technique: MonteCarloTechnique::Standard,
            sampling: SamplingMethod::Pseudorandom,
        }
    }
}

pub type DownAndOutCallResult = EuropeanCallResult;

pub type ArithmeticAsianCallResult = EuropeanCallResult;

pub type BasketCallResult = EuropeanCallResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArithmeticAsianCallPricer {
    config: ArithmeticAsianCallConfig,
}

impl Default for ArithmeticAsianCallPricer {
    fn default() -> Self {
        Self::new()
    }
}

impl ArithmeticAsianCallPricer {
    pub fn new() -> Self {
        Self {
            config: ArithmeticAsianCallConfig::default(),
        }
    }

    pub fn from_config(config: ArithmeticAsianCallConfig) -> Self {
        Self { config }
    }

    pub fn s0(mut self, value: f64) -> Self {
        self.config.s0 = value;
        self
    }

    pub fn strike(mut self, value: f64) -> Self {
        self.config.k = value;
        self
    }

    pub fn rate(mut self, value: f64) -> Self {
        self.config.r = value;
        self
    }

    pub fn volatility(mut self, value: f64) -> Self {
        self.config.sigma = value;
        self
    }

    pub fn maturity(mut self, value: f64) -> Self {
        self.config.t = value;
        self
    }

    pub fn paths(mut self, value: usize) -> Self {
        self.config.n_paths = value;
        self
    }

    pub fn steps(mut self, value: usize) -> Self {
        self.config.n_steps = value;
        self
    }

    pub fn seed(mut self, value: u64) -> Self {
        self.config.seed = value;
        self
    }

    pub fn threads(mut self, value: usize) -> Self {
        self.config.n_threads = value;
        self
    }

    pub fn technique(mut self, value: MonteCarloTechnique) -> Self {
        self.config.technique = value;
        self
    }

    pub fn sampling(mut self, value: SamplingMethod) -> Self {
        self.config.sampling = value;
        self
    }

    pub fn standard(mut self) -> Self {
        self.config.technique = MonteCarloTechnique::Standard;
        self
    }

    pub fn antithetic(mut self) -> Self {
        self.config.technique = MonteCarloTechnique::Antithetic;
        self
    }

    pub fn control_variate(mut self) -> Self {
        self.config.technique = MonteCarloTechnique::ControlVariate;
        self
    }

    pub fn randomized_halton(mut self) -> Self {
        self.config.sampling = SamplingMethod::RandomizedHalton;
        self
    }

    pub fn latin_hypercube(mut self) -> Self {
        self.config.sampling = SamplingMethod::LatinHypercube;
        self
    }

    pub fn scrambled_sobol(mut self) -> Self {
        self.config.sampling = SamplingMethod::ScrambledSobol;
        self
    }

    pub fn scrambled_sobol_brownian_bridge(mut self) -> Self {
        self.config.sampling = SamplingMethod::ScrambledSobolBrownianBridge;
        self
    }

    pub fn pseudorandom(mut self) -> Self {
        self.config.sampling = SamplingMethod::Pseudorandom;
        self
    }

    pub fn config(&self) -> &ArithmeticAsianCallConfig {
        &self.config
    }

    pub fn price(&self) -> ArithmeticAsianCallResult {
        arithmetic_asian_call_price_mc_cpu(&self.config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArithmeticAsianMlmcPricer {
    config: ArithmeticAsianMlmcConfig,
}

impl Default for ArithmeticAsianMlmcPricer {
    fn default() -> Self {
        Self::new()
    }
}

impl ArithmeticAsianMlmcPricer {
    pub fn new() -> Self {
        Self {
            config: ArithmeticAsianMlmcConfig::default(),
        }
    }

    pub fn from_config(config: ArithmeticAsianMlmcConfig) -> Self {
        Self { config }
    }

    pub fn s0(mut self, value: f64) -> Self {
        self.config.s0 = value;
        self
    }

    pub fn strike(mut self, value: f64) -> Self {
        self.config.k = value;
        self
    }

    pub fn rate(mut self, value: f64) -> Self {
        self.config.r = value;
        self
    }

    pub fn volatility(mut self, value: f64) -> Self {
        self.config.sigma = value;
        self
    }

    pub fn maturity(mut self, value: f64) -> Self {
        self.config.t = value;
        self
    }

    pub fn base_steps(mut self, value: usize) -> Self {
        self.config.base_steps = value;
        self
    }

    pub fn levels(mut self, value: usize) -> Self {
        self.config.levels = value;
        self
    }

    pub fn refinement_factor(mut self, value: usize) -> Self {
        self.config.refinement_factor = value;
        self
    }

    pub fn paths_per_level(mut self, value: Vec<usize>) -> Self {
        self.config.paths_per_level = value;
        self
    }

    pub fn seed(mut self, value: u64) -> Self {
        self.config.seed = value;
        self
    }

    pub fn scramble_replicates(mut self, value: usize) -> Self {
        self.config.scramble_replicates = value;
        self
    }

    pub fn sampling(mut self, value: SamplingMethod) -> Self {
        self.config.sampling = value;
        self
    }

    pub fn pseudorandom(mut self) -> Self {
        self.config.sampling = SamplingMethod::Pseudorandom;
        self
    }

    pub fn scrambled_sobol(mut self) -> Self {
        self.config.sampling = SamplingMethod::ScrambledSobol;
        self
    }

    pub fn config(&self) -> &ArithmeticAsianMlmcConfig {
        &self.config
    }

    pub fn solve_tolerance(
        &self,
        tolerance: &ArithmeticAsianMlmcToleranceConfig,
    ) -> ArithmeticAsianMlmcTolerancePlan {
        solve_arithmetic_asian_mlmc_tolerance_cpu(&self.config, tolerance)
    }

    pub fn price(&self) -> ArithmeticAsianMlmcResult {
        arithmetic_asian_call_price_mlmc_cpu(&self.config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DownAndOutCallPricer {
    config: DownAndOutCallConfig,
}

impl Default for DownAndOutCallPricer {
    fn default() -> Self {
        Self::new()
    }
}

impl DownAndOutCallPricer {
    pub fn new() -> Self {
        Self {
            config: DownAndOutCallConfig::default(),
        }
    }

    pub fn from_config(config: DownAndOutCallConfig) -> Self {
        Self { config }
    }

    pub fn s0(mut self, value: f64) -> Self {
        self.config.s0 = value;
        self
    }

    pub fn strike(mut self, value: f64) -> Self {
        self.config.k = value;
        self
    }

    pub fn barrier(mut self, value: f64) -> Self {
        self.config.barrier = value;
        self
    }

    pub fn rate(mut self, value: f64) -> Self {
        self.config.r = value;
        self
    }

    pub fn volatility(mut self, value: f64) -> Self {
        self.config.sigma = value;
        self
    }

    pub fn maturity(mut self, value: f64) -> Self {
        self.config.t = value;
        self
    }

    pub fn paths(mut self, value: usize) -> Self {
        self.config.n_paths = value;
        self
    }

    pub fn steps(mut self, value: usize) -> Self {
        self.config.n_steps = value;
        self
    }

    pub fn seed(mut self, value: u64) -> Self {
        self.config.seed = value;
        self
    }

    pub fn threads(mut self, value: usize) -> Self {
        self.config.n_threads = value;
        self
    }

    pub fn technique(mut self, value: MonteCarloTechnique) -> Self {
        self.config.technique = value;
        self
    }

    pub fn sampling(mut self, value: SamplingMethod) -> Self {
        self.config.sampling = value;
        self
    }

    pub fn standard(mut self) -> Self {
        self.config.technique = MonteCarloTechnique::Standard;
        self
    }

    pub fn antithetic(mut self) -> Self {
        self.config.technique = MonteCarloTechnique::Antithetic;
        self
    }

    pub fn control_variate(mut self) -> Self {
        self.config.technique = MonteCarloTechnique::ControlVariate;
        self
    }

    pub fn randomized_halton(mut self) -> Self {
        self.config.sampling = SamplingMethod::RandomizedHalton;
        self
    }

    pub fn latin_hypercube(mut self) -> Self {
        self.config.sampling = SamplingMethod::LatinHypercube;
        self
    }

    pub fn scrambled_sobol(mut self) -> Self {
        self.config.sampling = SamplingMethod::ScrambledSobol;
        self
    }

    pub fn scrambled_sobol_brownian_bridge(mut self) -> Self {
        self.config.sampling = SamplingMethod::ScrambledSobolBrownianBridge;
        self
    }

    pub fn pseudorandom(mut self) -> Self {
        self.config.sampling = SamplingMethod::Pseudorandom;
        self
    }

    pub fn config(&self) -> &DownAndOutCallConfig {
        &self.config
    }

    pub fn price(&self) -> DownAndOutCallResult {
        down_and_out_call_price_mc_cpu(&self.config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BasketCallPricer {
    config: BasketCallConfig,
}

impl Default for BasketCallPricer {
    fn default() -> Self {
        Self::new()
    }
}

impl BasketCallPricer {
    pub fn new() -> Self {
        Self {
            config: BasketCallConfig::default(),
        }
    }

    pub fn from_config(config: BasketCallConfig) -> Self {
        Self { config }
    }

    pub fn spot1(mut self, value: f64) -> Self {
        self.config.s01 = value;
        self
    }

    pub fn spot2(mut self, value: f64) -> Self {
        self.config.s02 = value;
        self
    }

    pub fn strike(mut self, value: f64) -> Self {
        self.config.k = value;
        self
    }

    pub fn rate(mut self, value: f64) -> Self {
        self.config.r = value;
        self
    }

    pub fn volatility1(mut self, value: f64) -> Self {
        self.config.sigma1 = value;
        self
    }

    pub fn volatility2(mut self, value: f64) -> Self {
        self.config.sigma2 = value;
        self
    }

    pub fn correlation(mut self, value: f64) -> Self {
        self.config.rho = value;
        self
    }

    pub fn weights(mut self, weight1: f64, weight2: f64) -> Self {
        self.config.weight1 = weight1;
        self.config.weight2 = weight2;
        self
    }

    pub fn maturity(mut self, value: f64) -> Self {
        self.config.t = value;
        self
    }

    pub fn paths(mut self, value: usize) -> Self {
        self.config.n_paths = value;
        self
    }

    pub fn seed(mut self, value: u64) -> Self {
        self.config.seed = value;
        self
    }

    pub fn threads(mut self, value: usize) -> Self {
        self.config.n_threads = value;
        self
    }

    pub fn technique(mut self, value: MonteCarloTechnique) -> Self {
        self.config.technique = value;
        self
    }

    pub fn sampling(mut self, value: SamplingMethod) -> Self {
        self.config.sampling = value;
        self
    }

    pub fn standard(mut self) -> Self {
        self.config.technique = MonteCarloTechnique::Standard;
        self
    }

    pub fn antithetic(mut self) -> Self {
        self.config.technique = MonteCarloTechnique::Antithetic;
        self
    }

    pub fn control_variate(mut self) -> Self {
        self.config.technique = MonteCarloTechnique::ControlVariate;
        self
    }

    pub fn randomized_halton(mut self) -> Self {
        self.config.sampling = SamplingMethod::RandomizedHalton;
        self
    }

    pub fn latin_hypercube(mut self) -> Self {
        self.config.sampling = SamplingMethod::LatinHypercube;
        self
    }

    pub fn scrambled_sobol(mut self) -> Self {
        self.config.sampling = SamplingMethod::ScrambledSobol;
        self
    }

    pub fn scrambled_sobol_brownian_bridge(mut self) -> Self {
        self.config.sampling = SamplingMethod::ScrambledSobolBrownianBridge;
        self
    }

    pub fn pseudorandom(mut self) -> Self {
        self.config.sampling = SamplingMethod::Pseudorandom;
        self
    }

    pub fn config(&self) -> &BasketCallConfig {
        &self.config
    }

    pub fn price(&self) -> BasketCallResult {
        basket_call_price_mc_cpu(&self.config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EuropeanCallPricer {
    config: EuropeanCallConfig,
    method: EuropeanCallMethod,
}

impl Default for EuropeanCallPricer {
    fn default() -> Self {
        Self::new()
    }
}

impl EuropeanCallPricer {
    pub fn new() -> Self {
        Self {
            config: EuropeanCallConfig::default(),
            method: EuropeanCallMethod::Auto,
        }
    }

    pub fn from_config(config: EuropeanCallConfig) -> Self {
        Self {
            config,
            method: EuropeanCallMethod::Auto,
        }
    }

    pub fn s0(mut self, value: f64) -> Self {
        self.config.s0 = value;
        self
    }

    pub fn strike(mut self, value: f64) -> Self {
        self.config.k = value;
        self
    }

    pub fn rate(mut self, value: f64) -> Self {
        self.config.r = value;
        self
    }

    pub fn volatility(mut self, value: f64) -> Self {
        self.config.sigma = value;
        self
    }

    pub fn maturity(mut self, value: f64) -> Self {
        self.config.t = value;
        self
    }

    pub fn paths(mut self, value: usize) -> Self {
        self.config.n_paths = value;
        self
    }

    pub fn steps(mut self, value: usize) -> Self {
        self.config.n_steps = value;
        self
    }

    pub fn seed(mut self, value: u64) -> Self {
        self.config.seed = value;
        self
    }

    pub fn threads(mut self, value: usize) -> Self {
        self.config.n_threads = value;
        self
    }

    pub fn method(mut self, value: EuropeanCallMethod) -> Self {
        self.method = value;
        self
    }

    pub fn technique(mut self, value: MonteCarloTechnique) -> Self {
        self.config.technique = value;
        self
    }

    pub fn sampling(mut self, value: SamplingMethod) -> Self {
        self.config.sampling = value;
        self
    }

    pub fn terminal(mut self) -> Self {
        self.method = EuropeanCallMethod::TerminalDistribution;
        self
    }

    pub fn stepwise(mut self) -> Self {
        self.method = EuropeanCallMethod::StepwisePaths;
        self
    }

    pub fn antithetic(mut self) -> Self {
        self.config.technique = MonteCarloTechnique::Antithetic;
        self
    }

    pub fn control_variate(mut self) -> Self {
        self.config.technique = MonteCarloTechnique::ControlVariate;
        self
    }

    pub fn standard(mut self) -> Self {
        self.config.technique = MonteCarloTechnique::Standard;
        self
    }

    pub fn randomized_halton(mut self) -> Self {
        self.config.sampling = SamplingMethod::RandomizedHalton;
        self
    }

    pub fn latin_hypercube(mut self) -> Self {
        self.config.sampling = SamplingMethod::LatinHypercube;
        self
    }

    pub fn scrambled_sobol(mut self) -> Self {
        self.config.sampling = SamplingMethod::ScrambledSobol;
        self
    }

    pub fn scrambled_sobol_brownian_bridge(mut self) -> Self {
        self.config.sampling = SamplingMethod::ScrambledSobolBrownianBridge;
        self
    }

    pub fn pseudorandom(mut self) -> Self {
        self.config.sampling = SamplingMethod::Pseudorandom;
        self
    }

    pub fn config(&self) -> &EuropeanCallConfig {
        &self.config
    }

    pub fn methodology(&self) -> EuropeanCallMethod {
        self.method
    }

    pub fn price(&self) -> EuropeanCallResult {
        european_call_price_mc_cpu_with_method(&self.config, self.method)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MonteCarloRng {
    state: u64,
    cached_normal: Option<f64>,
}

impl MonteCarloRng {
    pub fn new(seed: u64) -> Self {
        let non_zero_seed = if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        };

        Self {
            state: non_zero_seed,
            cached_normal: None,
        }
    }

    fn next_u64(&mut self) -> u64 {
        // xorshift64* for a small deterministic PRNG with low overhead.
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn next_f64_open01(&mut self) -> f64 {
        // Use top 53 bits to produce a uniform in (0, 1).
        let raw = self.next_u64() >> 11;
        let value = (raw as f64) * (1.0 / ((1u64 << 53) as f64));
        value.max(f64::MIN_POSITIVE)
    }

    pub fn standard_normal(&mut self) -> f64 {
        if let Some(cached) = self.cached_normal.take() {
            return cached;
        }

        // Box-Muller transform. Cache one sample to halve transcendental calls.
        let u1 = self.next_f64_open01();
        let u2 = self.next_f64_open01();
        let radius = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * PI * u2;
        let z0 = radius * theta.cos();
        let z1 = radius * theta.sin();
        self.cached_normal = Some(z1);
        z0
    }
}

pub fn european_call_price_mc_cpu(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    european_call_price_mc_cpu_with_method(cfg, EuropeanCallMethod::Auto)
}

pub fn european_call_price_mc_cpu_with_method(
    cfg: &EuropeanCallConfig,
    method: EuropeanCallMethod,
) -> EuropeanCallResult {
    assert!(cfg.n_paths > 0, "n_paths must be > 0");
    assert!(cfg.n_steps > 0, "n_steps must be > 0");

    match method {
        EuropeanCallMethod::Auto | EuropeanCallMethod::TerminalDistribution => {
            european_call_price_mc_cpu_terminal(cfg)
        }
        EuropeanCallMethod::StepwisePaths => european_call_price_mc_cpu_stepwise(cfg),
    }
}

pub fn european_call_price_mc_cpu_terminal(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    if cfg.sampling != SamplingMethod::Pseudorandom {
        return simulate_terminal_qmc(cfg);
    }

    match cfg.technique {
        MonteCarloTechnique::Antithetic => return simulate_terminal_antithetic(cfg),
        MonteCarloTechnique::ControlVariate => return simulate_terminal_control_variate(cfg),
        MonteCarloTechnique::Standard => {}
    }

    // For European calls under GBM, we can sample terminal distribution directly:
    // S_T = S_0 * exp((r - 0.5*sigma^2)T + sigma*sqrt(T)*Z)
    // This is equivalent in distribution to step-by-step simulation and is much faster.
    let drift_t = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * cfg.t;
    let vol_t = cfg.sigma * cfg.t.sqrt();
    let discount = (-cfg.r * cfg.t).exp();

    let thread_count = resolved_thread_count(cfg.n_threads);
    let (payoff_sum, payoff_sq_sum) = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_terminal_chunk(
            cfg.seed,
            cfg.n_paths,
            cfg.s0,
            cfg.k,
            drift_t,
            vol_t,
            discount,
            MonteCarloTechnique::Standard,
        )
    } else {
        simulate_terminal_parallel(cfg, thread_count, drift_t, vol_t, discount)
    };

    summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
}

pub fn european_call_price_mc_cpu_stepwise(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    if cfg.sampling != SamplingMethod::Pseudorandom {
        return simulate_stepwise_qmc(cfg);
    }

    match cfg.technique {
        MonteCarloTechnique::Antithetic => return simulate_stepwise_antithetic(cfg),
        MonteCarloTechnique::ControlVariate => return simulate_stepwise_control_variate(cfg),
        MonteCarloTechnique::Standard => {}
    }

    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();

    let thread_count = resolved_thread_count(cfg.n_threads);
    let (payoff_sum, payoff_sq_sum) = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_stepwise_chunk(
            cfg.seed,
            cfg.n_paths,
            cfg.n_steps,
            cfg.s0,
            cfg.k,
            drift_dt,
            vol_dt,
            discount,
            MonteCarloTechnique::Standard,
        )
    } else {
        simulate_stepwise_parallel(cfg, thread_count, drift_dt, vol_dt, discount)
    };

    summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
}

pub fn arithmetic_asian_call_price_mc_cpu(
    cfg: &ArithmeticAsianCallConfig,
) -> ArithmeticAsianCallResult {
    assert!(cfg.n_paths > 0, "n_paths must be > 0");
    assert!(cfg.n_steps > 0, "n_steps must be > 0");

    if cfg.sampling != SamplingMethod::Pseudorandom {
        return simulate_asian_stepwise_qmc(cfg);
    }

    match cfg.technique {
        MonteCarloTechnique::Standard => arithmetic_asian_call_price_mc_stepwise_standard(cfg),
        MonteCarloTechnique::Antithetic => arithmetic_asian_call_price_mc_stepwise_antithetic(cfg),
        MonteCarloTechnique::ControlVariate => {
            arithmetic_asian_call_price_mc_stepwise_control_variate(cfg)
        }
    }
}

pub fn arithmetic_asian_call_price_mlmc_cpu(
    cfg: &ArithmeticAsianMlmcConfig,
) -> ArithmeticAsianMlmcResult {
    validate_arithmetic_asian_mlmc_config(cfg);

    if cfg.sampling != SamplingMethod::Pseudorandom && cfg.scramble_replicates > 1 {
        return arithmetic_asian_call_price_mlmc_replicated_cpu(cfg);
    }

    arithmetic_asian_call_price_mlmc_single_cpu(cfg)
}

pub fn tune_arithmetic_asian_mlmc_allocation_cpu(
    cfg: &ArithmeticAsianMlmcConfig,
    target_step_updates: usize,
    pilot_paths_per_level: usize,
) -> ArithmeticAsianMlmcAllocationPlan {
    validate_arithmetic_asian_mlmc_config(cfg);
    assert!(target_step_updates > 0, "target_step_updates must be > 0");
    assert!(
        pilot_paths_per_level > 1,
        "pilot_paths_per_level must be > 1"
    );

    let mut levels = Vec::with_capacity(cfg.levels);
    let mut denominator = 0.0;

    for level in 0..cfg.levels {
        let fine_steps = mlmc_fine_steps(cfg.base_steps, cfg.refinement_factor, level);
        let coarse_steps = if level == 0 {
            None
        } else {
            Some(fine_steps / cfg.refinement_factor)
        };
        let cost_per_path = fine_steps + coarse_steps.unwrap_or(0);
        let moments = simulate_asian_mlmc_level(cfg, level, pilot_paths_per_level, fine_steps);
        let n = pilot_paths_per_level as f64;
        let mean = moments.sum / n;
        let pilot_variance = ((moments.sq_sum / n) - mean * mean).max(1.0e-18);

        denominator += (pilot_variance * cost_per_path as f64).sqrt();
        levels.push(ArithmeticAsianMlmcAllocationLevel {
            level,
            fine_steps,
            coarse_steps,
            pilot_paths: pilot_paths_per_level,
            pilot_variance,
            cost_per_path,
            recommended_paths: 1,
        });
    }

    let mut estimated_step_updates = 0usize;
    for level in &mut levels {
        let weight = (level.pilot_variance / level.cost_per_path as f64).sqrt();
        let raw_paths = (target_step_updates as f64 * weight / denominator).ceil();
        level.recommended_paths = (raw_paths as usize).max(2);
        estimated_step_updates = estimated_step_updates
            .saturating_add(level.recommended_paths.saturating_mul(level.cost_per_path));
    }

    ArithmeticAsianMlmcAllocationPlan {
        target_step_updates,
        estimated_step_updates,
        paths_per_level: levels.iter().map(|level| level.recommended_paths).collect(),
        levels,
    }
}

pub fn solve_arithmetic_asian_mlmc_tolerance_cpu(
    cfg: &ArithmeticAsianMlmcConfig,
    tolerance: &ArithmeticAsianMlmcToleranceConfig,
) -> ArithmeticAsianMlmcTolerancePlan {
    validate_arithmetic_asian_mlmc_config(cfg);
    validate_arithmetic_asian_mlmc_tolerance_config(tolerance);

    let scramble_replicates = effective_mlmc_scramble_replicates(cfg);
    let mut pilot_cfg = cfg.clone();
    pilot_cfg.scramble_replicates = 1;
    let mut allocation = solve_arithmetic_asian_mlmc_target_variance_allocation(
        &pilot_cfg,
        tolerance.target_stderr * tolerance.target_stderr * scramble_replicates as f64,
        tolerance.pilot_paths_per_level,
    );

    let mut estimated_stderr =
        estimate_arithmetic_asian_mlmc_stderr(&allocation, scramble_replicates);
    let mut estimated_step_updates = allocation
        .estimated_step_updates
        .saturating_mul(scramble_replicates);

    if estimated_step_updates < tolerance.min_step_updates {
        let min_per_replicate = tolerance
            .min_step_updates
            .div_ceil(scramble_replicates)
            .max(1);
        allocation = tune_arithmetic_asian_mlmc_allocation_cpu(
            &pilot_cfg,
            min_per_replicate,
            tolerance.pilot_paths_per_level,
        );
        estimated_stderr = estimate_arithmetic_asian_mlmc_stderr(&allocation, scramble_replicates);
        estimated_step_updates = allocation
            .estimated_step_updates
            .saturating_mul(scramble_replicates);
    }

    let mut max_step_updates_hit = false;
    if estimated_step_updates > tolerance.max_step_updates {
        max_step_updates_hit = true;
        let max_per_replicate = (tolerance.max_step_updates / scramble_replicates).max(1);
        allocation = tune_arithmetic_asian_mlmc_allocation_cpu(
            &pilot_cfg,
            max_per_replicate,
            tolerance.pilot_paths_per_level,
        );
        estimated_stderr = estimate_arithmetic_asian_mlmc_stderr(&allocation, scramble_replicates);
        estimated_step_updates = allocation
            .estimated_step_updates
            .saturating_mul(scramble_replicates);
    }

    let mut recommended_config = cfg.clone();
    recommended_config.paths_per_level = allocation.paths_per_level.clone();
    if recommended_config.sampling == SamplingMethod::Pseudorandom {
        recommended_config.scramble_replicates = 1;
    }

    ArithmeticAsianMlmcTolerancePlan {
        target_stderr: tolerance.target_stderr,
        estimated_stderr,
        target_met: estimated_stderr <= tolerance.target_stderr
            && estimated_step_updates <= tolerance.max_step_updates,
        max_step_updates_hit,
        scramble_replicates,
        estimated_step_updates,
        paths_per_level: allocation.paths_per_level.clone(),
        allocation,
        recommended_config,
    }
}

fn validate_arithmetic_asian_mlmc_config(cfg: &ArithmeticAsianMlmcConfig) {
    assert!(cfg.base_steps > 0, "base_steps must be > 0");
    assert!(cfg.levels > 0, "levels must be > 0");
    assert!(cfg.refinement_factor >= 2, "refinement_factor must be >= 2");
    assert_eq!(
        cfg.paths_per_level.len(),
        cfg.levels,
        "paths_per_level must contain one positive path count per level"
    );
    assert!(
        cfg.paths_per_level.iter().all(|paths| *paths > 0),
        "paths_per_level values must be > 0"
    );
    assert!(
        cfg.scramble_replicates > 0,
        "scramble_replicates must be > 0"
    );
}

fn validate_arithmetic_asian_mlmc_tolerance_config(tolerance: &ArithmeticAsianMlmcToleranceConfig) {
    assert!(
        tolerance.target_stderr.is_finite() && tolerance.target_stderr > 0.0,
        "target_stderr must be finite and > 0"
    );
    assert!(
        tolerance.pilot_paths_per_level > 1,
        "pilot_paths_per_level must be > 1"
    );
    assert!(
        tolerance.min_step_updates > 0,
        "min_step_updates must be > 0"
    );
    assert!(
        tolerance.max_step_updates >= tolerance.min_step_updates,
        "max_step_updates must be >= min_step_updates"
    );
}

fn effective_mlmc_scramble_replicates(cfg: &ArithmeticAsianMlmcConfig) -> usize {
    if cfg.sampling == SamplingMethod::Pseudorandom {
        1
    } else {
        cfg.scramble_replicates.max(1)
    }
}

fn solve_arithmetic_asian_mlmc_target_variance_allocation(
    cfg: &ArithmeticAsianMlmcConfig,
    target_variance: f64,
    pilot_paths_per_level: usize,
) -> ArithmeticAsianMlmcAllocationPlan {
    let mut levels = measure_arithmetic_asian_mlmc_pilot_levels(cfg, pilot_paths_per_level);
    let denominator = levels
        .iter()
        .map(|level| (level.pilot_variance * level.cost_per_path as f64).sqrt())
        .sum::<f64>();
    let mut estimated_step_updates = 0usize;

    for level in &mut levels {
        let weight = (level.pilot_variance / level.cost_per_path as f64).sqrt();
        let raw_paths = (denominator * weight / target_variance).ceil();
        level.recommended_paths = (raw_paths as usize).max(2);
        estimated_step_updates = estimated_step_updates
            .saturating_add(level.recommended_paths.saturating_mul(level.cost_per_path));
    }

    ArithmeticAsianMlmcAllocationPlan {
        target_step_updates: estimated_step_updates,
        estimated_step_updates,
        paths_per_level: levels.iter().map(|level| level.recommended_paths).collect(),
        levels,
    }
}

fn measure_arithmetic_asian_mlmc_pilot_levels(
    cfg: &ArithmeticAsianMlmcConfig,
    pilot_paths_per_level: usize,
) -> Vec<ArithmeticAsianMlmcAllocationLevel> {
    let mut levels = Vec::with_capacity(cfg.levels);

    for level in 0..cfg.levels {
        let fine_steps = mlmc_fine_steps(cfg.base_steps, cfg.refinement_factor, level);
        let coarse_steps = if level == 0 {
            None
        } else {
            Some(fine_steps / cfg.refinement_factor)
        };
        let cost_per_path = fine_steps + coarse_steps.unwrap_or(0);
        let moments = simulate_asian_mlmc_level(cfg, level, pilot_paths_per_level, fine_steps);
        let n = pilot_paths_per_level as f64;
        let mean = moments.sum / n;
        let pilot_variance = ((moments.sq_sum / n) - mean * mean).max(1.0e-18);

        levels.push(ArithmeticAsianMlmcAllocationLevel {
            level,
            fine_steps,
            coarse_steps,
            pilot_paths: pilot_paths_per_level,
            pilot_variance,
            cost_per_path,
            recommended_paths: 1,
        });
    }

    levels
}

fn estimate_arithmetic_asian_mlmc_stderr(
    allocation: &ArithmeticAsianMlmcAllocationPlan,
    scramble_replicates: usize,
) -> f64 {
    let variance = allocation
        .levels
        .iter()
        .map(|level| level.pilot_variance / level.recommended_paths as f64)
        .sum::<f64>();
    (variance / scramble_replicates as f64).sqrt()
}

fn arithmetic_asian_call_price_mlmc_single_cpu(
    cfg: &ArithmeticAsianMlmcConfig,
) -> ArithmeticAsianMlmcResult {
    let mut price = 0.0;
    let mut variance_contribution = 0.0;
    let mut total_paths = 0usize;
    let mut total_step_updates = 0usize;
    let mut levels = Vec::with_capacity(cfg.levels);

    for level in 0..cfg.levels {
        let paths = cfg.paths_per_level[level];
        let fine_steps = mlmc_fine_steps(cfg.base_steps, cfg.refinement_factor, level);
        let coarse_steps = if level == 0 {
            None
        } else {
            Some(fine_steps / cfg.refinement_factor)
        };
        let moments = simulate_asian_mlmc_level(cfg, level, paths, fine_steps);
        let n = paths as f64;
        let mean = moments.sum / n;
        let variance = ((moments.sq_sum / n) - mean * mean).max(0.0);
        let stderr = (variance / n).sqrt();
        let cost_step_updates = paths.saturating_mul(fine_steps + coarse_steps.unwrap_or(0));

        price += mean;
        variance_contribution += variance / n;
        total_paths = total_paths.saturating_add(paths);
        total_step_updates = total_step_updates.saturating_add(cost_step_updates);
        levels.push(ArithmeticAsianMlmcLevelResult {
            level,
            paths,
            fine_steps,
            coarse_steps,
            mean,
            variance,
            stderr,
            cost_step_updates,
        });
    }

    ArithmeticAsianMlmcResult {
        price,
        stderr: variance_contribution.sqrt(),
        total_paths,
        total_step_updates,
        scramble_replicates: 1,
        replicate_estimates: vec![price],
        levels,
    }
}

fn arithmetic_asian_call_price_mlmc_replicated_cpu(
    cfg: &ArithmeticAsianMlmcConfig,
) -> ArithmeticAsianMlmcResult {
    let replicate_count = cfg.scramble_replicates;
    let mut replicate_results = Vec::with_capacity(replicate_count);

    for replicate in 0..replicate_count {
        let mut replicate_cfg = cfg.clone();
        replicate_cfg.seed = derive_chunk_seed(cfg.seed, replicate as u64);
        replicate_cfg.scramble_replicates = 1;
        replicate_results.push(arithmetic_asian_call_price_mlmc_single_cpu(&replicate_cfg));
    }

    let replicate_estimates = replicate_results
        .iter()
        .map(|result| result.price)
        .collect::<Vec<_>>();
    let price = replicate_estimates.iter().sum::<f64>() / replicate_count as f64;
    let stderr = if replicate_count > 1 {
        let sample_variance = replicate_estimates
            .iter()
            .map(|estimate| {
                let diff = estimate - price;
                diff * diff
            })
            .sum::<f64>()
            / (replicate_count - 1) as f64;
        (sample_variance / replicate_count as f64).sqrt()
    } else {
        replicate_results[0].stderr
    };

    let total_paths = replicate_results
        .iter()
        .map(|result| result.total_paths)
        .sum();
    let total_step_updates = replicate_results
        .iter()
        .map(|result| result.total_step_updates)
        .sum();

    let mut levels = Vec::with_capacity(cfg.levels);
    for level_idx in 0..cfg.levels {
        let first = &replicate_results[0].levels[level_idx];
        let mean = replicate_results
            .iter()
            .map(|result| result.levels[level_idx].mean)
            .sum::<f64>()
            / replicate_count as f64;
        let variance = if replicate_count > 1 {
            replicate_results
                .iter()
                .map(|result| {
                    let diff = result.levels[level_idx].mean - mean;
                    diff * diff
                })
                .sum::<f64>()
                / (replicate_count - 1) as f64
        } else {
            first.variance
        };
        let stderr = if replicate_count > 1 {
            (variance / replicate_count as f64).sqrt()
        } else {
            first.stderr
        };

        levels.push(ArithmeticAsianMlmcLevelResult {
            level: first.level,
            paths: first.paths.saturating_mul(replicate_count),
            fine_steps: first.fine_steps,
            coarse_steps: first.coarse_steps,
            mean,
            variance,
            stderr,
            cost_step_updates: first.cost_step_updates.saturating_mul(replicate_count),
        });
    }

    ArithmeticAsianMlmcResult {
        price,
        stderr,
        total_paths,
        total_step_updates,
        scramble_replicates: replicate_count,
        replicate_estimates,
        levels,
    }
}

pub fn down_and_out_call_price_mc_cpu(cfg: &DownAndOutCallConfig) -> DownAndOutCallResult {
    assert!(cfg.n_paths > 0, "n_paths must be > 0");
    assert!(cfg.n_steps > 0, "n_steps must be > 0");
    assert!(cfg.barrier > 0.0, "barrier must be > 0");

    if cfg.sampling != SamplingMethod::Pseudorandom {
        return simulate_down_and_out_stepwise_qmc(cfg);
    }

    match cfg.technique {
        MonteCarloTechnique::Standard => down_and_out_call_price_mc_stepwise_standard(cfg),
        MonteCarloTechnique::Antithetic => down_and_out_call_price_mc_stepwise_antithetic(cfg),
        MonteCarloTechnique::ControlVariate => {
            down_and_out_call_price_mc_stepwise_control_variate(cfg)
        }
    }
}

pub fn basket_call_price_mc_cpu(cfg: &BasketCallConfig) -> BasketCallResult {
    validate_basket_call_config(cfg);

    if cfg.sampling != SamplingMethod::Pseudorandom {
        return simulate_basket_qmc(cfg);
    }

    match cfg.technique {
        MonteCarloTechnique::Standard => simulate_basket_pseudorandom_standard(cfg),
        MonteCarloTechnique::Antithetic => simulate_basket_pseudorandom_antithetic(cfg),
        MonteCarloTechnique::ControlVariate => simulate_basket_pseudorandom_control_variate(cfg),
    }
}

fn validate_basket_call_config(cfg: &BasketCallConfig) {
    assert!(cfg.n_paths > 0, "n_paths must be > 0");
    assert!(cfg.s01 > 0.0, "s01 must be > 0");
    assert!(cfg.s02 > 0.0, "s02 must be > 0");
    assert!(cfg.k >= 0.0, "k must be >= 0");
    assert!(cfg.sigma1 >= 0.0, "sigma1 must be >= 0");
    assert!(cfg.sigma2 >= 0.0, "sigma2 must be >= 0");
    assert!(cfg.t > 0.0, "t must be > 0");
    assert!((-1.0..=1.0).contains(&cfg.rho), "rho must be in [-1, 1]");
}

#[allow(dead_code)]
pub(crate) fn generate_stepwise_standard_normals_f32(
    seed: u64,
    n_paths: usize,
    n_steps: usize,
) -> Vec<f32> {
    let mut rng = MonteCarloRng::new(seed);
    let mut normals = Vec::with_capacity(n_paths.saturating_mul(n_steps));

    for _ in 0..n_paths.saturating_mul(n_steps) {
        normals.push(rng.standard_normal() as f32);
    }

    normals
}

pub fn generate_standard_normals_cpu(
    sampling: SamplingMethod,
    n_points: usize,
    dimensions: usize,
    seed: u64,
) -> Vec<f64> {
    assert!(n_points > 0, "n_points must be > 0");
    assert!(dimensions > 0, "dimensions must be > 0");

    let total = n_points.saturating_mul(dimensions);
    let mut normals = Vec::with_capacity(total);

    if sampling == SamplingMethod::Pseudorandom {
        let mut rng = MonteCarloRng::new(seed);
        for _ in 0..total {
            normals.push(rng.standard_normal());
        }
        return normals;
    }

    if matches!(
        sampling,
        SamplingMethod::ScrambledSobol | SamplingMethod::ScrambledSobolBrownianBridge
    ) {
        fill_scrambled_sobol_standard_normals(n_points, dimensions, seed, &mut normals);
        return normals;
    }

    let sampler = StructuredNormalSampler::new(sampling, seed, n_points, dimensions);
    for point_idx in 0..n_points {
        for dim_idx in 0..dimensions {
            normals.push(sampler.standard_normal(point_idx, dim_idx));
        }
    }

    normals
}

fn fill_scrambled_sobol_standard_normals(
    n_points: usize,
    dimensions: usize,
    seed: u64,
    normals: &mut Vec<f64>,
) {
    let base_seed = seed as u32;
    for point_idx in 0..n_points {
        fill_scrambled_sobol_point_standard_normals(base_seed, point_idx, dimensions, normals);
    }
}

fn fill_scrambled_sobol_point_standard_normals(
    base_seed: u32,
    point_idx: usize,
    dimensions: usize,
    normals: &mut Vec<f64>,
) {
    let dimension_count = sobol_burley::NUM_DIMENSIONS as usize;
    let dimension_sets = sobol_burley::NUM_DIMENSION_SETS_4D as usize;
    let block = point_idx / ScrambledSobol::MAX_SEQUENCE_LEN;
    let local_index = (point_idx % ScrambledSobol::MAX_SEQUENCE_LEN) as u32;
    let block_seed = base_seed.wrapping_add((block as u32).wrapping_mul(0x9E37_79B9));
    let mut dim_idx = 0usize;

    while dim_idx + 4 <= dimensions && (dim_idx % dimension_count) + 3 < dimension_count {
        let seed_offset = (dim_idx / dimension_count) as u32;
        let dim_set = ((dim_idx % dimension_count) / 4) % dimension_sets;
        let sample = sobol_burley::sample_4d(
            local_index,
            dim_set as u32,
            block_seed.wrapping_add(seed_offset),
        );
        normals.push(inverse_standard_normal(sample[0] as f64));
        normals.push(inverse_standard_normal(sample[1] as f64));
        normals.push(inverse_standard_normal(sample[2] as f64));
        normals.push(inverse_standard_normal(sample[3] as f64));
        dim_idx += 4;
    }

    while dim_idx < dimensions {
        let seed_offset = (dim_idx / dimension_count) as u32;
        let dimension = dim_idx % dimension_count;
        let u = sobol_burley::sample(
            local_index,
            dimension as u32,
            block_seed.wrapping_add(seed_offset),
        ) as f64;
        normals.push(inverse_standard_normal(u));
        dim_idx += 1;
    }
}

#[allow(dead_code)]
pub(crate) fn generate_stepwise_stateless_normals_f32(
    seed: u64,
    n_paths: usize,
    n_steps: usize,
) -> Vec<f32> {
    let seed_u32 = seed as u32;
    let mut normals = Vec::with_capacity(n_paths.saturating_mul(n_steps));

    for path_idx in 0..n_paths {
        for step_idx in 0..n_steps {
            normals.push(stateless_standard_normal_f32(
                seed_u32,
                path_idx as u32,
                step_idx as u32,
            ));
        }
    }

    normals
}

#[allow(dead_code)]
pub(crate) fn european_call_price_mc_stepwise_from_f32_normals(
    cfg: &EuropeanCallConfig,
    normals: &[f32],
) -> EuropeanCallResult {
    let expected = cfg.n_paths.saturating_mul(cfg.n_steps);
    assert_eq!(
        normals.len(),
        expected,
        "stepwise normal buffer must contain n_paths * n_steps values"
    );

    let log_s0 = cfg.s0.ln() as f32;
    let strike = cfg.k as f32;
    let dt = (cfg.t / cfg.n_steps as f64) as f32;
    let drift_dt = ((cfg.r - 0.5 * cfg.sigma * cfg.sigma) as f32) * dt;
    let vol_dt = (cfg.sigma as f32) * dt.sqrt();
    let discount = ((-cfg.r * cfg.t).exp()) as f32;

    let mut payoff_sum = 0.0f64;
    let mut payoff_sq_sum = 0.0f64;

    for path_idx in 0..cfg.n_paths {
        let mut log_s_t = log_s0;
        let base_offset = path_idx * cfg.n_steps;
        for step_idx in 0..cfg.n_steps {
            let z = normals[base_offset + step_idx];
            log_s_t += drift_dt + vol_dt * z;
        }

        let s_t = log_s_t.exp();
        let payoff = ((s_t - strike).max(0.0) * discount) as f64;
        payoff_sum += payoff;
        payoff_sq_sum += payoff * payoff;
    }

    summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
}

#[allow(dead_code)]
pub(crate) fn arithmetic_asian_call_price_mc_stepwise_from_f32_normals(
    cfg: &ArithmeticAsianCallConfig,
    normals: &[f32],
) -> ArithmeticAsianCallResult {
    let expected = cfg.n_paths.saturating_mul(cfg.n_steps);
    assert_eq!(
        normals.len(),
        expected,
        "stepwise normal buffer must contain n_paths * n_steps values"
    );

    let log_s0 = cfg.s0.ln() as f32;
    let strike = cfg.k as f32;
    let dt = (cfg.t / cfg.n_steps as f64) as f32;
    let drift_dt = ((cfg.r - 0.5 * cfg.sigma * cfg.sigma) as f32) * dt;
    let vol_dt = (cfg.sigma as f32) * dt.sqrt();
    let discount = ((-cfg.r * cfg.t).exp()) as f32;
    let inv_steps = 1.0f32 / (cfg.n_steps as f32);

    let mut payoff_sum = 0.0f64;
    let mut payoff_sq_sum = 0.0f64;

    for path_idx in 0..cfg.n_paths {
        let mut log_s_t = log_s0;
        let mut arithmetic_sum = 0.0f32;
        let base_offset = path_idx * cfg.n_steps;
        for step_idx in 0..cfg.n_steps {
            let z = normals[base_offset + step_idx];
            log_s_t += drift_dt + vol_dt * z;
            arithmetic_sum += log_s_t.exp();
        }

        let arithmetic_average = arithmetic_sum * inv_steps;
        let payoff = ((arithmetic_average - strike).max(0.0) * discount) as f64;
        payoff_sum += payoff;
        payoff_sq_sum += payoff * payoff;
    }

    summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
}

#[allow(dead_code)]
pub(crate) fn down_and_out_call_price_mc_stepwise_from_f32_normals(
    cfg: &DownAndOutCallConfig,
    normals: &[f32],
) -> DownAndOutCallResult {
    let expected = cfg.n_paths.saturating_mul(cfg.n_steps);
    assert_eq!(
        normals.len(),
        expected,
        "stepwise normal buffer must contain n_paths * n_steps values"
    );

    let log_s0 = cfg.s0.ln() as f32;
    let strike = cfg.k as f32;
    let barrier = cfg.barrier as f32;
    let dt = (cfg.t / cfg.n_steps as f64) as f32;
    let drift_dt = ((cfg.r - 0.5 * cfg.sigma * cfg.sigma) as f32) * dt;
    let vol_dt = (cfg.sigma as f32) * dt.sqrt();
    let discount = ((-cfg.r * cfg.t).exp()) as f32;

    let mut payoff_sum = 0.0f64;
    let mut payoff_sq_sum = 0.0f64;

    for path_idx in 0..cfg.n_paths {
        let mut log_s_t = log_s0;
        let mut knocked_out = cfg.s0 <= cfg.barrier;
        let base_offset = path_idx * cfg.n_steps;
        for step_idx in 0..cfg.n_steps {
            let z = normals[base_offset + step_idx];
            log_s_t += drift_dt + vol_dt * z;
            if log_s_t.exp() <= barrier {
                knocked_out = true;
            }
        }

        let s_t = log_s_t.exp();
        let payoff = if knocked_out {
            0.0
        } else {
            ((s_t - strike).max(0.0) * discount) as f64
        };
        payoff_sum += payoff;
        payoff_sq_sum += payoff * payoff;
    }

    summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
}

#[allow(dead_code)]
pub(crate) fn stateless_standard_normal_f32(seed: u32, path_idx: u32, step_idx: u32) -> f32 {
    let u1 = stateless_open01_f32(seed, path_idx, step_idx, 0);
    let u2 = stateless_open01_f32(seed, path_idx, step_idx, 1);
    let radius = (-2.0f32 * u1.ln()).sqrt();
    let theta = 2.0f32 * (std::f32::consts::PI) * u2;
    radius * theta.cos()
}

#[allow(dead_code)]
fn stateless_open01_f32(seed: u32, path_idx: u32, step_idx: u32, lane: u32) -> f32 {
    let mixed = seed
        ^ path_idx.wrapping_mul(747_796_405)
        ^ step_idx.wrapping_mul(2_891_336_453)
        ^ lane.wrapping_mul(277_803_737);
    let hashed = hash_u32(mixed);
    (((hashed as f64) + 1.0) / 4_294_967_297.0).max(f32::MIN_POSITIVE as f64) as f32
}

#[allow(dead_code)]
fn hash_u32(mut x: u32) -> u32 {
    x = x.wrapping_add(0x9E37_79B9);
    x ^= x >> 16;
    x = x.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 13;
    x = x.wrapping_mul(0xC2B2_AE35);
    x ^ (x >> 16)
}

#[derive(Debug, Clone)]
struct ScrambledSobol {
    seed: u32,
}

impl ScrambledSobol {
    const MAX_SEQUENCE_LEN: usize = 1 << 16;

    fn new(seed: u64) -> Self {
        Self { seed: seed as u32 }
    }

    fn standard_normal(&self, point_index: usize, normal_index: usize) -> f64 {
        let block = point_index / Self::MAX_SEQUENCE_LEN;
        let local_index = point_index % Self::MAX_SEQUENCE_LEN;
        let dimension_count = sobol_burley::NUM_DIMENSIONS as usize;
        let dimension = normal_index % dimension_count;
        let seed = self
            .seed
            .wrapping_add((normal_index / dimension_count) as u32)
            .wrapping_add((block as u32).wrapping_mul(0x9E37_79B9));
        let u = sobol_burley::sample(local_index as u32, dimension as u32, seed) as f64;
        inverse_standard_normal(u.clamp(f64::MIN_POSITIVE, 1.0 - f64::EPSILON))
    }

    fn fill_standard_normals(&self, point_index: usize, dimensions: usize, out: &mut Vec<f64>) {
        out.clear();
        out.reserve(dimensions);
        fill_scrambled_sobol_point_standard_normals(self.seed, point_index, dimensions, out);
    }
}

#[derive(Debug, Clone)]
struct RandomizedHalton {
    primes: Vec<u32>,
    shifts: Vec<f64>,
}

impl RandomizedHalton {
    fn new(seed: u64, normal_dimensions: usize) -> Self {
        let dimension_count = normal_dimensions;
        let primes = first_n_primes(dimension_count.max(2));
        let mut shifts = Vec::with_capacity(dimension_count.max(2));
        for dim in 0..dimension_count.max(2) {
            shifts.push(open01_from_u64(splitmix64(
                seed.wrapping_add((dim as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)),
            )));
        }

        Self { primes, shifts }
    }

    fn standard_normal(&self, point_index: usize, normal_index: usize) -> f64 {
        let u = halton_with_shift(
            (point_index + 1) as u64,
            self.primes[normal_index],
            self.shifts[normal_index],
        );
        inverse_standard_normal(u)
    }

    fn fill_standard_normals(&self, point_index: usize, dimensions: usize, out: &mut Vec<f64>) {
        out.clear();
        out.reserve(dimensions);
        for normal_index in 0..dimensions {
            out.push(self.standard_normal(point_index, normal_index));
        }
    }
}

#[derive(Debug, Clone)]
struct LatinHypercube {
    n_paths: usize,
    params: Vec<LatinHypercubeDimension>,
}

#[derive(Debug, Clone, Copy)]
struct LatinHypercubeDimension {
    multiplier: usize,
    offset: usize,
    jitter_seed: u64,
}

impl LatinHypercube {
    fn new(seed: u64, n_paths: usize, normal_dimensions: usize) -> Self {
        let dimension_count = normal_dimensions.max(1);
        let mut params = Vec::with_capacity(dimension_count);

        for dim in 0..dimension_count {
            let mixed = splitmix64(
                seed ^ ((dim as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)) ^ 0xD1B5_4A32_D192_ED03,
            );
            let multiplier = coprime_multiplier(mixed, n_paths.max(1));
            let offset = if n_paths == 0 {
                0
            } else {
                (splitmix64(mixed ^ 0xA24B_AED4_963E_E407) as usize) % n_paths
            };
            let jitter_seed = splitmix64(mixed ^ 0x9FB2_1C65_1E98_DF25);

            params.push(LatinHypercubeDimension {
                multiplier,
                offset,
                jitter_seed,
            });
        }

        Self { n_paths, params }
    }

    fn standard_normal(&self, point_index: usize, normal_index: usize) -> f64 {
        let dim = self.params[normal_index];
        let stratum = if self.n_paths == 0 {
            0
        } else {
            (dim.multiplier
                .wrapping_mul(point_index)
                .wrapping_add(dim.offset))
                % self.n_paths
        };
        let jitter = open01_from_u64(splitmix64(
            dim.jitter_seed ^ (point_index as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9),
        ));
        let u = ((stratum as f64) + jitter) / (self.n_paths as f64);
        inverse_standard_normal(u.clamp(f64::MIN_POSITIVE, 1.0 - f64::EPSILON))
    }

    fn fill_standard_normals(&self, point_index: usize, dimensions: usize, out: &mut Vec<f64>) {
        out.clear();
        out.reserve(dimensions);
        for normal_index in 0..dimensions {
            out.push(self.standard_normal(point_index, normal_index));
        }
    }
}

#[derive(Debug, Clone)]
enum StructuredNormalSampler {
    RandomizedHalton(RandomizedHalton),
    LatinHypercube(LatinHypercube),
    ScrambledSobol(ScrambledSobol),
}

impl StructuredNormalSampler {
    fn new(method: SamplingMethod, seed: u64, n_paths: usize, normal_dimensions: usize) -> Self {
        match method {
            SamplingMethod::Pseudorandom => {
                unreachable!("pseudorandom sampling is handled by the PRNG runtime path")
            }
            SamplingMethod::RandomizedHalton => {
                Self::RandomizedHalton(RandomizedHalton::new(seed, normal_dimensions))
            }
            SamplingMethod::LatinHypercube => {
                Self::LatinHypercube(LatinHypercube::new(seed, n_paths, normal_dimensions))
            }
            SamplingMethod::ScrambledSobol | SamplingMethod::ScrambledSobolBrownianBridge => {
                Self::ScrambledSobol(ScrambledSobol::new(seed))
            }
        }
    }

    fn standard_normal(&self, point_index: usize, normal_index: usize) -> f64 {
        match self {
            Self::RandomizedHalton(sampler) => sampler.standard_normal(point_index, normal_index),
            Self::LatinHypercube(sampler) => sampler.standard_normal(point_index, normal_index),
            Self::ScrambledSobol(sampler) => sampler.standard_normal(point_index, normal_index),
        }
    }

    fn fill_standard_normals(&self, point_index: usize, dimensions: usize, out: &mut Vec<f64>) {
        match self {
            Self::RandomizedHalton(sampler) => {
                sampler.fill_standard_normals(point_index, dimensions, out)
            }
            Self::LatinHypercube(sampler) => {
                sampler.fill_standard_normals(point_index, dimensions, out)
            }
            Self::ScrambledSobol(sampler) => {
                sampler.fill_standard_normals(point_index, dimensions, out)
            }
        }
    }
}

fn uses_brownian_bridge(method: SamplingMethod) -> bool {
    matches!(method, SamplingMethod::ScrambledSobolBrownianBridge)
}

fn path_standard_normal(
    sampler: &StructuredNormalSampler,
    point_index: usize,
    step_index: usize,
    path_normals: Option<&[f64]>,
    bridge_normals: Option<&[f64]>,
) -> f64 {
    if let Some(normals) = bridge_normals {
        normals[step_index]
    } else if let Some(normals) = path_normals {
        normals[step_index]
    } else {
        sampler.standard_normal(point_index, step_index)
    }
}

#[derive(Debug, Clone)]
struct BrownianBridgeWorkspace {
    plan: Option<BrownianBridgePlan>,
    w: Vec<f64>,
    increments: Vec<f64>,
    bridge_inputs: Vec<f64>,
}

impl BrownianBridgeWorkspace {
    fn new(method: SamplingMethod, n_steps: usize, maturity: f64) -> Self {
        if uses_brownian_bridge(method) {
            Self {
                plan: Some(BrownianBridgePlan::new(n_steps, maturity)),
                w: vec![0.0; n_steps + 1],
                increments: vec![0.0; n_steps],
                bridge_inputs: Vec::with_capacity(n_steps),
            }
        } else {
            Self {
                plan: None,
                w: Vec::new(),
                increments: Vec::new(),
                bridge_inputs: Vec::new(),
            }
        }
    }

    fn step_normals(
        &mut self,
        sampler: &StructuredNormalSampler,
        point_index: usize,
    ) -> Option<&[f64]> {
        if let Some(plan) = &self.plan {
            sampler.fill_standard_normals(point_index, plan.n_steps, &mut self.bridge_inputs);
            plan.fill_step_normals_from_inputs(
                &self.bridge_inputs,
                &mut self.w,
                &mut self.increments,
            );
            Some(self.increments.as_slice())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
struct BrownianBridgePlan {
    n_steps: usize,
    maturity_sqrt: f64,
    inv_sqrt_dt: f64,
    ops: Vec<BrownianBridgeOp>,
}

#[derive(Debug, Clone, Copy)]
struct BrownianBridgeOp {
    left: usize,
    mid: usize,
    right: usize,
    left_weight: f64,
    right_weight: f64,
    stddev: f64,
    normal_index: usize,
}

impl BrownianBridgePlan {
    fn new(n_steps: usize, maturity: f64) -> Self {
        let dt = maturity / n_steps as f64;
        let mut intervals = vec![(0usize, n_steps)];
        let mut ops = Vec::with_capacity(n_steps.saturating_sub(1));
        let mut normal_index = 1usize;

        while let Some((left, right)) = intervals.pop() {
            if right <= left + 1 {
                continue;
            }

            let mid = (left + right) / 2;
            let t_left = left as f64 * dt;
            let t_mid = mid as f64 * dt;
            let t_right = right as f64 * dt;
            let denom = t_right - t_left;
            ops.push(BrownianBridgeOp {
                left,
                mid,
                right,
                left_weight: (t_right - t_mid) / denom,
                right_weight: (t_mid - t_left) / denom,
                stddev: ((t_mid - t_left) * (t_right - t_mid) / denom).sqrt(),
                normal_index,
            });
            normal_index += 1;

            intervals.push((mid, right));
            intervals.push((left, mid));
        }

        Self {
            n_steps,
            maturity_sqrt: maturity.sqrt(),
            inv_sqrt_dt: 1.0 / dt.sqrt(),
            ops,
        }
    }

    fn fill_step_normals_from_inputs(&self, inputs: &[f64], w: &mut [f64], increments: &mut [f64]) {
        debug_assert_eq!(w.len(), self.n_steps + 1);
        debug_assert_eq!(increments.len(), self.n_steps);
        debug_assert_eq!(inputs.len(), self.n_steps);

        w.fill(0.0);
        w[self.n_steps] = self.maturity_sqrt * inputs[0];

        for op in &self.ops {
            w[op.mid] = op.left_weight * w[op.left]
                + op.right_weight * w[op.right]
                + op.stddev * inputs[op.normal_index];
        }

        for step in 1..=self.n_steps {
            increments[step - 1] = (w[step] - w[step - 1]) * self.inv_sqrt_dt;
        }
    }
}

fn coprime_multiplier(seed: u64, modulus: usize) -> usize {
    if modulus <= 1 {
        return 1;
    }

    let mut candidate = ((seed as usize) % modulus).max(1);
    if candidate % 2 == 0 {
        candidate = candidate.saturating_add(1);
    }

    while gcd_usize(candidate, modulus) != 1 {
        candidate = candidate.saturating_add(2);
        if candidate >= modulus {
            candidate = 1;
        }
    }

    candidate
}

fn gcd_usize(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let next = a % b;
        a = b;
        b = next;
    }
    a
}

fn inverse_standard_normal(p: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
        4.374_664_141_464_968,
        2.938_163_982_698_783,
    ];
    const D: [f64; 4] = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996,
        3.754_408_661_907_416,
    ];
    const P_LOW: f64 = 0.024_25;
    const P_HIGH: f64 = 1.0 - P_LOW;

    let p = p.clamp(f64::MIN_POSITIVE, 1.0 - f64::EPSILON);
    if p < P_LOW {
        let q = (-2.0 * p.ln()).sqrt();
        return (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0);
    }
    if p <= P_HIGH {
        let q = p - 0.5;
        let r = q * q;
        return (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0);
    }

    let q = (-2.0 * (1.0 - p).ln()).sqrt();
    -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
        / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
}

fn halton_with_shift(index: u64, base: u32, shift: f64) -> f64 {
    let shifted = halton(index, base) + shift;
    shifted - shifted.floor()
}

fn halton(mut index: u64, base: u32) -> f64 {
    let mut f = 1.0 / base as f64;
    let mut value = 0.0;
    while index > 0 {
        value += f * (index % base as u64) as f64;
        index /= base as u64;
        f /= base as f64;
    }
    value.max(f64::MIN_POSITIVE)
}

fn first_n_primes(n: usize) -> Vec<u32> {
    let mut primes = Vec::with_capacity(n);
    let mut candidate = 2u32;
    while primes.len() < n {
        if is_prime(candidate, &primes) {
            primes.push(candidate);
        }
        candidate = candidate.saturating_add(1);
    }
    primes
}

fn is_prime(candidate: u32, known_primes: &[u32]) -> bool {
    let limit = (candidate as f64).sqrt() as u32;
    for &p in known_primes {
        if p > limit {
            break;
        }
        if candidate % p == 0 {
            return false;
        }
    }
    true
}

fn open01_from_u64(value: u64) -> f64 {
    let top = value >> 11;
    ((top as f64) * (1.0 / ((1u64 << 53) as f64))).max(f64::MIN_POSITIVE)
}

fn simulate_terminal_parallel(
    cfg: &EuropeanCallConfig,
    thread_count: usize,
    drift_t: f64,
    vol_t: f64,
    discount: f64,
) -> (f64, f64) {
    let base_chunk = cfg.n_paths / thread_count;
    let remainder = cfg.n_paths % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let n_paths_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        let s0 = cfg.s0;
        let k = cfg.k;
        let technique = cfg.technique;
        handles.push(thread::spawn(move || {
            simulate_terminal_chunk(
                seed,
                n_paths_chunk,
                s0,
                k,
                drift_t,
                vol_t,
                discount,
                technique,
            )
        }));
    }

    // Join in spawn order so reduction order is deterministic across runs.
    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;
    for handle in handles {
        let (chunk_sum, chunk_sq_sum) = handle
            .join()
            .expect("CPU Monte Carlo worker thread panicked");
        payoff_sum += chunk_sum;
        payoff_sq_sum += chunk_sq_sum;
    }

    (payoff_sum, payoff_sq_sum)
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ControlVariateMoments {
    pub(crate) sample_count: usize,
    pub(crate) payoff_sum: f64,
    pub(crate) payoff_sq_sum: f64,
    pub(crate) control_sum: f64,
    pub(crate) control_sq_sum: f64,
    pub(crate) payoff_control_cross_sum: f64,
}

impl ControlVariateMoments {
    pub(crate) fn record(&mut self, payoff: f64, control: f64) {
        self.sample_count += 1;
        self.payoff_sum += payoff;
        self.payoff_sq_sum += payoff * payoff;
        self.control_sum += control;
        self.control_sq_sum += control * control;
        self.payoff_control_cross_sum += payoff * control;
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.sample_count += other.sample_count;
        self.payoff_sum += other.payoff_sum;
        self.payoff_sq_sum += other.payoff_sq_sum;
        self.control_sum += other.control_sum;
        self.control_sq_sum += other.control_sq_sum;
        self.payoff_control_cross_sum += other.payoff_control_cross_sum;
    }
}

fn simulate_stepwise_parallel(
    cfg: &EuropeanCallConfig,
    thread_count: usize,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> (f64, f64) {
    let base_chunk = cfg.n_paths / thread_count;
    let remainder = cfg.n_paths % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let n_paths_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        let s0 = cfg.s0;
        let k = cfg.k;
        let n_steps = cfg.n_steps;
        let technique = cfg.technique;
        handles.push(thread::spawn(move || {
            simulate_stepwise_chunk(
                seed,
                n_paths_chunk,
                n_steps,
                s0,
                k,
                drift_dt,
                vol_dt,
                discount,
                technique,
            )
        }));
    }

    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;
    for handle in handles {
        let (chunk_sum, chunk_sq_sum) = handle
            .join()
            .expect("CPU Monte Carlo worker thread panicked");
        payoff_sum += chunk_sum;
        payoff_sq_sum += chunk_sq_sum;
    }

    (payoff_sum, payoff_sq_sum)
}

fn simulate_terminal_control_variate_parallel(
    cfg: &EuropeanCallConfig,
    thread_count: usize,
    drift_t: f64,
    vol_t: f64,
    discount: f64,
) -> ControlVariateMoments {
    let base_chunk = cfg.n_paths / thread_count;
    let remainder = cfg.n_paths % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let n_paths_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        let s0 = cfg.s0;
        let k = cfg.k;
        handles.push(thread::spawn(move || {
            simulate_terminal_control_variate_chunk(
                seed,
                n_paths_chunk,
                s0,
                k,
                drift_t,
                vol_t,
                discount,
            )
        }));
    }

    let mut moments = ControlVariateMoments::default();
    for handle in handles {
        let chunk = handle
            .join()
            .expect("CPU Monte Carlo worker thread panicked");
        moments.merge(chunk);
    }

    moments
}

fn simulate_stepwise_control_variate_parallel(
    cfg: &EuropeanCallConfig,
    thread_count: usize,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> ControlVariateMoments {
    let base_chunk = cfg.n_paths / thread_count;
    let remainder = cfg.n_paths % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let n_paths_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        let s0 = cfg.s0;
        let k = cfg.k;
        let n_steps = cfg.n_steps;
        handles.push(thread::spawn(move || {
            simulate_stepwise_control_variate_chunk(
                seed,
                n_paths_chunk,
                n_steps,
                s0,
                k,
                drift_dt,
                vol_dt,
                discount,
            )
        }));
    }

    let mut moments = ControlVariateMoments::default();
    for handle in handles {
        let chunk = handle
            .join()
            .expect("CPU Monte Carlo worker thread panicked");
        moments.merge(chunk);
    }

    moments
}

fn simulate_terminal_chunk(
    seed: u64,
    n_paths: usize,
    s0: f64,
    k: f64,
    drift_t: f64,
    vol_t: f64,
    discount: f64,
    technique: MonteCarloTechnique,
) -> (f64, f64) {
    let mut rng = MonteCarloRng::new(seed);
    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;

    match technique {
        MonteCarloTechnique::Standard => {
            for _ in 0..n_paths {
                let z = rng.standard_normal();
                let payoff = european_call_payoff_from_shock(s0, k, drift_t, vol_t, z, discount);
                payoff_sum += payoff;
                payoff_sq_sum += payoff * payoff;
            }
        }
        MonteCarloTechnique::Antithetic => {
            let pair_count = n_paths / 2;
            for _ in 0..pair_count {
                let z = rng.standard_normal();
                let payoff_a = european_call_payoff_from_shock(s0, k, drift_t, vol_t, z, discount);
                let payoff_b = european_call_payoff_from_shock(s0, k, drift_t, vol_t, -z, discount);
                payoff_sum += payoff_a + payoff_b;
                payoff_sq_sum += payoff_a * payoff_a + payoff_b * payoff_b;
            }

            if n_paths % 2 != 0 {
                let z = rng.standard_normal();
                let payoff = european_call_payoff_from_shock(s0, k, drift_t, vol_t, z, discount);
                payoff_sum += payoff;
                payoff_sq_sum += payoff * payoff;
            }
        }
        MonteCarloTechnique::ControlVariate => {
            unreachable!("control variate terminal path uses dedicated accumulator kernel");
        }
    }

    (payoff_sum, payoff_sq_sum)
}

fn simulate_stepwise_chunk(
    seed: u64,
    n_paths: usize,
    n_steps: usize,
    s0: f64,
    k: f64,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
    technique: MonteCarloTechnique,
) -> (f64, f64) {
    let mut rng = MonteCarloRng::new(seed);
    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;

    match technique {
        MonteCarloTechnique::Standard => {
            for _ in 0..n_paths {
                let mut log_s_t = s0.ln();
                for _ in 0..n_steps {
                    let z = rng.standard_normal();
                    log_s_t += drift_dt + vol_dt * z;
                }

                let payoff = (log_s_t.exp() - k).max(0.0) * discount;
                payoff_sum += payoff;
                payoff_sq_sum += payoff * payoff;
            }
        }
        MonteCarloTechnique::Antithetic => {
            let pair_count = n_paths / 2;
            for _ in 0..pair_count {
                let mut log_a = s0.ln();
                let mut log_b = s0.ln();
                for _ in 0..n_steps {
                    let z = rng.standard_normal();
                    log_a += drift_dt + vol_dt * z;
                    log_b += drift_dt - vol_dt * z;
                }

                let payoff_a = (log_a.exp() - k).max(0.0) * discount;
                let payoff_b = (log_b.exp() - k).max(0.0) * discount;
                payoff_sum += payoff_a + payoff_b;
                payoff_sq_sum += payoff_a * payoff_a + payoff_b * payoff_b;
            }

            if n_paths % 2 != 0 {
                let mut log_s_t = s0.ln();
                for _ in 0..n_steps {
                    let z = rng.standard_normal();
                    log_s_t += drift_dt + vol_dt * z;
                }

                let payoff = (log_s_t.exp() - k).max(0.0) * discount;
                payoff_sum += payoff;
                payoff_sq_sum += payoff * payoff;
            }
        }
        MonteCarloTechnique::ControlVariate => {
            unreachable!("control variate stepwise path uses dedicated accumulator kernel");
        }
    }

    (payoff_sum, payoff_sq_sum)
}

fn simulate_terminal_control_variate_chunk(
    seed: u64,
    n_paths: usize,
    s0: f64,
    k: f64,
    drift_t: f64,
    vol_t: f64,
    discount: f64,
) -> ControlVariateMoments {
    let mut rng = MonteCarloRng::new(seed);
    let mut moments = ControlVariateMoments::default();

    for _ in 0..n_paths {
        let z = rng.standard_normal();
        let s_t = s0 * (drift_t + vol_t * z).exp();
        let control = discount * s_t;
        let payoff = (s_t - k).max(0.0) * discount;
        moments.record(payoff, control);
    }

    moments
}

fn simulate_stepwise_control_variate_chunk(
    seed: u64,
    n_paths: usize,
    n_steps: usize,
    s0: f64,
    k: f64,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> ControlVariateMoments {
    let mut rng = MonteCarloRng::new(seed);
    let mut moments = ControlVariateMoments::default();

    for _ in 0..n_paths {
        let mut log_s_t = s0.ln();
        for _ in 0..n_steps {
            let z = rng.standard_normal();
            log_s_t += drift_dt + vol_dt * z;
        }

        let s_t = log_s_t.exp();
        let control = discount * s_t;
        let payoff = (s_t - k).max(0.0) * discount;
        moments.record(payoff, control);
    }

    moments
}

fn simulate_asian_stepwise_chunk(
    seed: u64,
    n_paths: usize,
    n_steps: usize,
    s0: f64,
    k: f64,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> (f64, f64) {
    let mut rng = MonteCarloRng::new(seed);
    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;

    for _ in 0..n_paths {
        let mut log_s_t = s0.ln();
        let mut arithmetic_sum = 0.0;
        for _ in 0..n_steps {
            let z = rng.standard_normal();
            log_s_t += drift_dt + vol_dt * z;
            arithmetic_sum += log_s_t.exp();
        }

        let arithmetic_average = arithmetic_sum / n_steps as f64;
        let payoff = (arithmetic_average - k).max(0.0) * discount;
        payoff_sum += payoff;
        payoff_sq_sum += payoff * payoff;
    }

    (payoff_sum, payoff_sq_sum)
}

fn simulate_asian_stepwise_control_variate_chunk(
    seed: u64,
    n_paths: usize,
    n_steps: usize,
    s0: f64,
    k: f64,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> ControlVariateMoments {
    let mut rng = MonteCarloRng::new(seed);
    let mut moments = ControlVariateMoments::default();

    for _ in 0..n_paths {
        let mut log_s_t = s0.ln();
        let mut arithmetic_sum = 0.0;
        for _ in 0..n_steps {
            let z = rng.standard_normal();
            log_s_t += drift_dt + vol_dt * z;
            arithmetic_sum += log_s_t.exp();
        }

        let s_t = log_s_t.exp();
        let arithmetic_average = arithmetic_sum / n_steps as f64;
        let control = discount * s_t;
        let payoff = (arithmetic_average - k).max(0.0) * discount;
        moments.record(payoff, control);
    }

    moments
}

#[derive(Debug, Clone, Copy, Default)]
struct MlmcMoments {
    sum: f64,
    sq_sum: f64,
}

fn mlmc_fine_steps(base_steps: usize, refinement_factor: usize, level: usize) -> usize {
    let mut steps = base_steps;
    for _ in 0..level {
        steps = steps
            .checked_mul(refinement_factor)
            .expect("MLMC fine step count overflowed usize");
    }
    steps
}

fn simulate_asian_mlmc_level(
    cfg: &ArithmeticAsianMlmcConfig,
    level: usize,
    paths: usize,
    fine_steps: usize,
) -> MlmcMoments {
    if cfg.sampling != SamplingMethod::Pseudorandom {
        return simulate_asian_mlmc_structured_level(cfg, level, paths, fine_steps);
    }

    let mut rng = MonteCarloRng::new(derive_chunk_seed(cfg.seed, level as u64));
    let mut moments = MlmcMoments::default();

    if level == 0 {
        let dt = cfg.t / fine_steps as f64;
        let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
        let vol_dt = cfg.sigma * dt.sqrt();
        let discount = (-cfg.r * cfg.t).exp();

        for _ in 0..paths {
            let payoff = simulate_asian_payoff_with_rng(
                &mut rng, cfg.s0, cfg.k, fine_steps, drift_dt, vol_dt, discount,
            );
            moments.sum += payoff;
            moments.sq_sum += payoff * payoff;
        }

        return moments;
    }

    let refinement = cfg.refinement_factor;
    let coarse_steps = fine_steps / refinement;
    let fine_dt = cfg.t / fine_steps as f64;
    let coarse_dt = cfg.t / coarse_steps as f64;
    let fine_drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * fine_dt;
    let fine_vol_dt = cfg.sigma * fine_dt.sqrt();
    let coarse_drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * coarse_dt;
    let coarse_vol_dt = cfg.sigma * coarse_dt.sqrt();
    let sqrt_refinement = (refinement as f64).sqrt();
    let discount = (-cfg.r * cfg.t).exp();

    for _ in 0..paths {
        let diff = simulate_asian_mlmc_coupled_difference(
            &mut rng,
            cfg.s0,
            cfg.k,
            coarse_steps,
            refinement,
            fine_drift_dt,
            fine_vol_dt,
            coarse_drift_dt,
            coarse_vol_dt,
            sqrt_refinement,
            discount,
        );
        moments.sum += diff;
        moments.sq_sum += diff * diff;
    }

    moments
}

fn simulate_asian_mlmc_structured_level(
    cfg: &ArithmeticAsianMlmcConfig,
    level: usize,
    paths: usize,
    fine_steps: usize,
) -> MlmcMoments {
    let sampler = StructuredNormalSampler::new(
        cfg.sampling,
        derive_chunk_seed(cfg.seed, level as u64),
        paths,
        fine_steps,
    );
    let mut moments = MlmcMoments::default();

    if level == 0 {
        let dt = cfg.t / fine_steps as f64;
        let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
        let vol_dt = cfg.sigma * dt.sqrt();
        let discount = (-cfg.r * cfg.t).exp();
        let mut path_normals = Vec::with_capacity(fine_steps);

        for path_idx in 0..paths {
            sampler.fill_standard_normals(path_idx, fine_steps, &mut path_normals);
            let payoff = simulate_asian_structured_payoff_from_normals(
                &path_normals,
                cfg.s0,
                cfg.k,
                drift_dt,
                vol_dt,
                discount,
            );
            moments.sum += payoff;
            moments.sq_sum += payoff * payoff;
        }

        return moments;
    }

    let refinement = cfg.refinement_factor;
    let coarse_steps = fine_steps / refinement;
    let fine_dt = cfg.t / fine_steps as f64;
    let coarse_dt = cfg.t / coarse_steps as f64;
    let fine_drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * fine_dt;
    let fine_vol_dt = cfg.sigma * fine_dt.sqrt();
    let coarse_drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * coarse_dt;
    let coarse_vol_dt = cfg.sigma * coarse_dt.sqrt();
    let sqrt_refinement = (refinement as f64).sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let mut path_normals = Vec::with_capacity(fine_steps);

    for path_idx in 0..paths {
        sampler.fill_standard_normals(path_idx, fine_steps, &mut path_normals);
        let diff = simulate_asian_mlmc_structured_coupled_difference_from_normals(
            &path_normals,
            cfg.s0,
            cfg.k,
            coarse_steps,
            refinement,
            fine_drift_dt,
            fine_vol_dt,
            coarse_drift_dt,
            coarse_vol_dt,
            sqrt_refinement,
            discount,
        );
        moments.sum += diff;
        moments.sq_sum += diff * diff;
    }

    moments
}

fn simulate_asian_payoff_with_rng(
    rng: &mut MonteCarloRng,
    s0: f64,
    k: f64,
    n_steps: usize,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> f64 {
    let mut log_s_t = s0.ln();
    let mut arithmetic_sum = 0.0;
    for _ in 0..n_steps {
        let z = rng.standard_normal();
        log_s_t += drift_dt + vol_dt * z;
        arithmetic_sum += log_s_t.exp();
    }

    let arithmetic_average = arithmetic_sum / n_steps as f64;
    (arithmetic_average - k).max(0.0) * discount
}

fn simulate_asian_structured_payoff_from_normals(
    normals: &[f64],
    s0: f64,
    k: f64,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> f64 {
    let mut log_s_t = s0.ln();
    let mut arithmetic_sum = 0.0;
    for z in normals {
        log_s_t += drift_dt + vol_dt * *z;
        arithmetic_sum += log_s_t.exp();
    }

    let arithmetic_average = arithmetic_sum / normals.len() as f64;
    (arithmetic_average - k).max(0.0) * discount
}

#[allow(clippy::too_many_arguments)]
fn simulate_asian_mlmc_coupled_difference(
    rng: &mut MonteCarloRng,
    s0: f64,
    k: f64,
    coarse_steps: usize,
    refinement: usize,
    fine_drift_dt: f64,
    fine_vol_dt: f64,
    coarse_drift_dt: f64,
    coarse_vol_dt: f64,
    sqrt_refinement: f64,
    discount: f64,
) -> f64 {
    let mut fine_log_s = s0.ln();
    let mut coarse_log_s = s0.ln();
    let mut fine_sum = 0.0;
    let mut coarse_sum = 0.0;

    for _ in 0..coarse_steps {
        let mut coarse_z_sum = 0.0;
        for _ in 0..refinement {
            let z = rng.standard_normal();
            coarse_z_sum += z;
            fine_log_s += fine_drift_dt + fine_vol_dt * z;
            fine_sum += fine_log_s.exp();
        }

        let coarse_z = coarse_z_sum / sqrt_refinement;
        coarse_log_s += coarse_drift_dt + coarse_vol_dt * coarse_z;
        coarse_sum += coarse_log_s.exp();
    }

    let fine_steps = coarse_steps * refinement;
    let fine_payoff = (fine_sum / fine_steps as f64 - k).max(0.0) * discount;
    let coarse_payoff = (coarse_sum / coarse_steps as f64 - k).max(0.0) * discount;
    fine_payoff - coarse_payoff
}

#[allow(clippy::too_many_arguments)]
fn simulate_asian_mlmc_structured_coupled_difference_from_normals(
    normals: &[f64],
    s0: f64,
    k: f64,
    coarse_steps: usize,
    refinement: usize,
    fine_drift_dt: f64,
    fine_vol_dt: f64,
    coarse_drift_dt: f64,
    coarse_vol_dt: f64,
    sqrt_refinement: f64,
    discount: f64,
) -> f64 {
    let mut fine_log_s = s0.ln();
    let mut coarse_log_s = s0.ln();
    let mut fine_sum = 0.0;
    let mut coarse_sum = 0.0;
    let mut normal_iter = normals.iter();

    for _ in 0..coarse_steps {
        let mut coarse_z_sum = 0.0;
        for _ in 0..refinement {
            let z = *normal_iter
                .next()
                .expect("normal count must match MLMC fine step count");
            coarse_z_sum += z;
            fine_log_s += fine_drift_dt + fine_vol_dt * z;
            fine_sum += fine_log_s.exp();
        }

        let coarse_z = coarse_z_sum / sqrt_refinement;
        coarse_log_s += coarse_drift_dt + coarse_vol_dt * coarse_z;
        coarse_sum += coarse_log_s.exp();
    }

    let fine_steps = coarse_steps * refinement;
    let fine_payoff = (fine_sum / fine_steps as f64 - k).max(0.0) * discount;
    let coarse_payoff = (coarse_sum / coarse_steps as f64 - k).max(0.0) * discount;
    fine_payoff - coarse_payoff
}

fn simulate_asian_stepwise_parallel(
    cfg: &ArithmeticAsianCallConfig,
    thread_count: usize,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> (f64, f64) {
    let base_chunk = cfg.n_paths / thread_count;
    let remainder = cfg.n_paths % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let n_paths_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        let s0 = cfg.s0;
        let k = cfg.k;
        let n_steps = cfg.n_steps;
        handles.push(thread::spawn(move || {
            simulate_asian_stepwise_chunk(
                seed,
                n_paths_chunk,
                n_steps,
                s0,
                k,
                drift_dt,
                vol_dt,
                discount,
            )
        }));
    }

    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;
    for handle in handles {
        let (chunk_sum, chunk_sq_sum) = handle
            .join()
            .expect("CPU Monte Carlo worker thread panicked");
        payoff_sum += chunk_sum;
        payoff_sq_sum += chunk_sq_sum;
    }

    (payoff_sum, payoff_sq_sum)
}

fn simulate_asian_stepwise_control_variate_parallel(
    cfg: &ArithmeticAsianCallConfig,
    thread_count: usize,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> ControlVariateMoments {
    let base_chunk = cfg.n_paths / thread_count;
    let remainder = cfg.n_paths % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let n_paths_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        let s0 = cfg.s0;
        let k = cfg.k;
        let n_steps = cfg.n_steps;
        handles.push(thread::spawn(move || {
            simulate_asian_stepwise_control_variate_chunk(
                seed,
                n_paths_chunk,
                n_steps,
                s0,
                k,
                drift_dt,
                vol_dt,
                discount,
            )
        }));
    }

    let mut moments = ControlVariateMoments::default();
    for handle in handles {
        let chunk = handle
            .join()
            .expect("CPU Monte Carlo worker thread panicked");
        moments.merge(chunk);
    }

    moments
}

fn simulate_asian_stepwise_antithetic_chunk(
    seed: u64,
    pair_count: usize,
    n_steps: usize,
    s0: f64,
    k: f64,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> (f64, f64) {
    let mut rng = MonteCarloRng::new(seed);
    let mut block_sum = 0.0;
    let mut block_sq_sum = 0.0;

    for _ in 0..pair_count {
        let mut log_a = s0.ln();
        let mut log_b = s0.ln();
        let mut arithmetic_sum_a = 0.0;
        let mut arithmetic_sum_b = 0.0;
        for _ in 0..n_steps {
            let z = rng.standard_normal();
            log_a += drift_dt + vol_dt * z;
            log_b += drift_dt - vol_dt * z;
            arithmetic_sum_a += log_a.exp();
            arithmetic_sum_b += log_b.exp();
        }

        let payoff_a = (arithmetic_sum_a / n_steps as f64 - k).max(0.0) * discount;
        let payoff_b = (arithmetic_sum_b / n_steps as f64 - k).max(0.0) * discount;
        let block_estimate = 0.5 * (payoff_a + payoff_b);
        block_sum += block_estimate;
        block_sq_sum += block_estimate * block_estimate;
    }

    (block_sum, block_sq_sum)
}

fn simulate_asian_stepwise_antithetic_parallel(
    cfg: &ArithmeticAsianCallConfig,
    thread_count: usize,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> (f64, f64) {
    let pair_count = cfg.n_paths.div_ceil(2);
    let base_chunk = pair_count / thread_count;
    let remainder = pair_count % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let pair_count_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        let s0 = cfg.s0;
        let k = cfg.k;
        let n_steps = cfg.n_steps;
        handles.push(thread::spawn(move || {
            simulate_asian_stepwise_antithetic_chunk(
                seed,
                pair_count_chunk,
                n_steps,
                s0,
                k,
                drift_dt,
                vol_dt,
                discount,
            )
        }));
    }

    let mut block_sum = 0.0;
    let mut block_sq_sum = 0.0;
    for handle in handles {
        let (chunk_sum, chunk_sq_sum) = handle
            .join()
            .expect("CPU Monte Carlo worker thread panicked");
        block_sum += chunk_sum;
        block_sq_sum += chunk_sq_sum;
    }

    (block_sum, block_sq_sum)
}

fn simulate_terminal_qmc(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    let drift_t = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * cfg.t;
    let vol_t = cfg.sigma * cfg.t.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let qmc = StructuredNormalSampler::new(cfg.sampling, cfg.seed, cfg.n_paths, 1);

    match cfg.technique {
        MonteCarloTechnique::Standard => {
            let mut payoff_sum = 0.0;
            let mut payoff_sq_sum = 0.0;
            for path_idx in 0..cfg.n_paths {
                let z = qmc.standard_normal(path_idx, 0);
                let payoff =
                    european_call_payoff_from_shock(cfg.s0, cfg.k, drift_t, vol_t, z, discount);
                payoff_sum += payoff;
                payoff_sq_sum += payoff * payoff;
            }
            summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
        }
        MonteCarloTechnique::Antithetic => {
            let pair_count = cfg.n_paths.div_ceil(2);
            let mut block_sum = 0.0;
            let mut block_sq_sum = 0.0;
            for pair_idx in 0..pair_count {
                let z = qmc.standard_normal(pair_idx, 0);
                let payoff_a =
                    european_call_payoff_from_shock(cfg.s0, cfg.k, drift_t, vol_t, z, discount);
                let payoff_b =
                    european_call_payoff_from_shock(cfg.s0, cfg.k, drift_t, vol_t, -z, discount);
                let block_estimate = 0.5 * (payoff_a + payoff_b);
                block_sum += block_estimate;
                block_sq_sum += block_estimate * block_estimate;
            }
            summarize_block_estimates(pair_count, block_sum, block_sq_sum)
        }
        MonteCarloTechnique::ControlVariate => {
            let mut moments = ControlVariateMoments::default();
            for path_idx in 0..cfg.n_paths {
                let z = qmc.standard_normal(path_idx, 0);
                let s_t = cfg.s0 * (drift_t + vol_t * z).exp();
                let control = discount * s_t;
                let payoff = (s_t - cfg.k).max(0.0) * discount;
                moments.record(payoff, control);
            }
            summarize_control_variate(moments, cfg.s0)
        }
    }
}

fn simulate_stepwise_qmc(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let qmc = StructuredNormalSampler::new(cfg.sampling, cfg.seed, cfg.n_paths, cfg.n_steps);
    let mut bridge_workspace = BrownianBridgeWorkspace::new(cfg.sampling, cfg.n_steps, cfg.t);

    match cfg.technique {
        MonteCarloTechnique::Standard => {
            let mut payoff_sum = 0.0;
            let mut payoff_sq_sum = 0.0;
            let mut path_normals = Vec::with_capacity(cfg.n_steps);
            for path_idx in 0..cfg.n_paths {
                let mut log_s_t = cfg.s0.ln();
                let bridge = bridge_workspace.step_normals(&qmc, path_idx);
                let normals = if bridge.is_none() {
                    qmc.fill_standard_normals(path_idx, cfg.n_steps, &mut path_normals);
                    Some(path_normals.as_slice())
                } else {
                    None
                };
                for step_idx in 0..cfg.n_steps {
                    let z = path_standard_normal(&qmc, path_idx, step_idx, normals, bridge);
                    log_s_t += drift_dt + vol_dt * z;
                }
                let payoff = (log_s_t.exp() - cfg.k).max(0.0) * discount;
                payoff_sum += payoff;
                payoff_sq_sum += payoff * payoff;
            }
            summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
        }
        MonteCarloTechnique::Antithetic => {
            let pair_count = cfg.n_paths.div_ceil(2);
            let mut block_sum = 0.0;
            let mut block_sq_sum = 0.0;
            let mut path_normals = Vec::with_capacity(cfg.n_steps);
            for pair_idx in 0..pair_count {
                let mut log_a = cfg.s0.ln();
                let mut log_b = cfg.s0.ln();
                let bridge = bridge_workspace.step_normals(&qmc, pair_idx);
                let normals = if bridge.is_none() {
                    qmc.fill_standard_normals(pair_idx, cfg.n_steps, &mut path_normals);
                    Some(path_normals.as_slice())
                } else {
                    None
                };
                for step_idx in 0..cfg.n_steps {
                    let z = path_standard_normal(&qmc, pair_idx, step_idx, normals, bridge);
                    log_a += drift_dt + vol_dt * z;
                    log_b += drift_dt - vol_dt * z;
                }
                let payoff_a = (log_a.exp() - cfg.k).max(0.0) * discount;
                let payoff_b = (log_b.exp() - cfg.k).max(0.0) * discount;
                let block_estimate = 0.5 * (payoff_a + payoff_b);
                block_sum += block_estimate;
                block_sq_sum += block_estimate * block_estimate;
            }
            summarize_block_estimates(pair_count, block_sum, block_sq_sum)
        }
        MonteCarloTechnique::ControlVariate => {
            let mut moments = ControlVariateMoments::default();
            let mut path_normals = Vec::with_capacity(cfg.n_steps);
            for path_idx in 0..cfg.n_paths {
                let mut log_s_t = cfg.s0.ln();
                let bridge = bridge_workspace.step_normals(&qmc, path_idx);
                let normals = if bridge.is_none() {
                    qmc.fill_standard_normals(path_idx, cfg.n_steps, &mut path_normals);
                    Some(path_normals.as_slice())
                } else {
                    None
                };
                for step_idx in 0..cfg.n_steps {
                    let z = path_standard_normal(&qmc, path_idx, step_idx, normals, bridge);
                    log_s_t += drift_dt + vol_dt * z;
                }
                let s_t = log_s_t.exp();
                let control = discount * s_t;
                let payoff = (s_t - cfg.k).max(0.0) * discount;
                moments.record(payoff, control);
            }
            summarize_control_variate(moments, cfg.s0)
        }
    }
}

fn simulate_asian_stepwise_qmc(cfg: &ArithmeticAsianCallConfig) -> ArithmeticAsianCallResult {
    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let qmc = StructuredNormalSampler::new(cfg.sampling, cfg.seed, cfg.n_paths, cfg.n_steps);
    let mut bridge_workspace = BrownianBridgeWorkspace::new(cfg.sampling, cfg.n_steps, cfg.t);

    match cfg.technique {
        MonteCarloTechnique::Standard => {
            let mut payoff_sum = 0.0;
            let mut payoff_sq_sum = 0.0;
            let mut path_normals = Vec::with_capacity(cfg.n_steps);
            for path_idx in 0..cfg.n_paths {
                let mut log_s_t = cfg.s0.ln();
                let mut arithmetic_sum = 0.0;
                let bridge = bridge_workspace.step_normals(&qmc, path_idx);
                let normals = if bridge.is_none() {
                    qmc.fill_standard_normals(path_idx, cfg.n_steps, &mut path_normals);
                    Some(path_normals.as_slice())
                } else {
                    None
                };
                for step_idx in 0..cfg.n_steps {
                    let z = path_standard_normal(&qmc, path_idx, step_idx, normals, bridge);
                    log_s_t += drift_dt + vol_dt * z;
                    arithmetic_sum += log_s_t.exp();
                }
                let payoff = (arithmetic_sum / cfg.n_steps as f64 - cfg.k).max(0.0) * discount;
                payoff_sum += payoff;
                payoff_sq_sum += payoff * payoff;
            }
            summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
        }
        MonteCarloTechnique::Antithetic => {
            let pair_count = cfg.n_paths.div_ceil(2);
            let mut block_sum = 0.0;
            let mut block_sq_sum = 0.0;
            let mut path_normals = Vec::with_capacity(cfg.n_steps);
            for pair_idx in 0..pair_count {
                let mut log_a = cfg.s0.ln();
                let mut log_b = cfg.s0.ln();
                let mut arithmetic_sum_a = 0.0;
                let mut arithmetic_sum_b = 0.0;
                let bridge = bridge_workspace.step_normals(&qmc, pair_idx);
                let normals = if bridge.is_none() {
                    qmc.fill_standard_normals(pair_idx, cfg.n_steps, &mut path_normals);
                    Some(path_normals.as_slice())
                } else {
                    None
                };
                for step_idx in 0..cfg.n_steps {
                    let z = path_standard_normal(&qmc, pair_idx, step_idx, normals, bridge);
                    log_a += drift_dt + vol_dt * z;
                    log_b += drift_dt - vol_dt * z;
                    arithmetic_sum_a += log_a.exp();
                    arithmetic_sum_b += log_b.exp();
                }
                let payoff_a = (arithmetic_sum_a / cfg.n_steps as f64 - cfg.k).max(0.0) * discount;
                let payoff_b = (arithmetic_sum_b / cfg.n_steps as f64 - cfg.k).max(0.0) * discount;
                let block_estimate = 0.5 * (payoff_a + payoff_b);
                block_sum += block_estimate;
                block_sq_sum += block_estimate * block_estimate;
            }
            summarize_block_estimates(pair_count, block_sum, block_sq_sum)
        }
        MonteCarloTechnique::ControlVariate => {
            let mut moments = ControlVariateMoments::default();
            let mut path_normals = Vec::with_capacity(cfg.n_steps);
            for path_idx in 0..cfg.n_paths {
                let mut log_s_t = cfg.s0.ln();
                let mut arithmetic_sum = 0.0;
                let bridge = bridge_workspace.step_normals(&qmc, path_idx);
                let normals = if bridge.is_none() {
                    qmc.fill_standard_normals(path_idx, cfg.n_steps, &mut path_normals);
                    Some(path_normals.as_slice())
                } else {
                    None
                };
                for step_idx in 0..cfg.n_steps {
                    let z = path_standard_normal(&qmc, path_idx, step_idx, normals, bridge);
                    log_s_t += drift_dt + vol_dt * z;
                    arithmetic_sum += log_s_t.exp();
                }
                let s_t = log_s_t.exp();
                let payoff = (arithmetic_sum / cfg.n_steps as f64 - cfg.k).max(0.0) * discount;
                moments.record(payoff, discount * s_t);
            }
            summarize_control_variate(moments, cfg.s0)
        }
    }
}

fn simulate_down_and_out_stepwise_qmc(cfg: &DownAndOutCallConfig) -> DownAndOutCallResult {
    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let qmc = StructuredNormalSampler::new(cfg.sampling, cfg.seed, cfg.n_paths, cfg.n_steps);
    let mut bridge_workspace = BrownianBridgeWorkspace::new(cfg.sampling, cfg.n_steps, cfg.t);

    match cfg.technique {
        MonteCarloTechnique::Standard => {
            let mut payoff_sum = 0.0;
            let mut payoff_sq_sum = 0.0;
            let mut path_normals = Vec::with_capacity(cfg.n_steps);
            for path_idx in 0..cfg.n_paths {
                let mut log_s_t = cfg.s0.ln();
                let mut knocked_out = cfg.s0 <= cfg.barrier;
                let bridge = bridge_workspace.step_normals(&qmc, path_idx);
                let normals = if bridge.is_none() {
                    qmc.fill_standard_normals(path_idx, cfg.n_steps, &mut path_normals);
                    Some(path_normals.as_slice())
                } else {
                    None
                };
                for step_idx in 0..cfg.n_steps {
                    let z = path_standard_normal(&qmc, path_idx, step_idx, normals, bridge);
                    log_s_t += drift_dt + vol_dt * z;
                    if log_s_t.exp() <= cfg.barrier {
                        knocked_out = true;
                    }
                }
                let payoff = if knocked_out {
                    0.0
                } else {
                    (log_s_t.exp() - cfg.k).max(0.0) * discount
                };
                payoff_sum += payoff;
                payoff_sq_sum += payoff * payoff;
            }
            summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
        }
        MonteCarloTechnique::Antithetic => {
            let pair_count = cfg.n_paths.div_ceil(2);
            let mut block_sum = 0.0;
            let mut block_sq_sum = 0.0;
            let mut path_normals = Vec::with_capacity(cfg.n_steps);
            for pair_idx in 0..pair_count {
                let mut log_a = cfg.s0.ln();
                let mut log_b = cfg.s0.ln();
                let mut knock_a = cfg.s0 <= cfg.barrier;
                let mut knock_b = cfg.s0 <= cfg.barrier;
                let bridge = bridge_workspace.step_normals(&qmc, pair_idx);
                let normals = if bridge.is_none() {
                    qmc.fill_standard_normals(pair_idx, cfg.n_steps, &mut path_normals);
                    Some(path_normals.as_slice())
                } else {
                    None
                };
                for step_idx in 0..cfg.n_steps {
                    let z = path_standard_normal(&qmc, pair_idx, step_idx, normals, bridge);
                    log_a += drift_dt + vol_dt * z;
                    log_b += drift_dt - vol_dt * z;
                    if log_a.exp() <= cfg.barrier {
                        knock_a = true;
                    }
                    if log_b.exp() <= cfg.barrier {
                        knock_b = true;
                    }
                }
                let payoff_a = if knock_a {
                    0.0
                } else {
                    (log_a.exp() - cfg.k).max(0.0) * discount
                };
                let payoff_b = if knock_b {
                    0.0
                } else {
                    (log_b.exp() - cfg.k).max(0.0) * discount
                };
                let block_estimate = 0.5 * (payoff_a + payoff_b);
                block_sum += block_estimate;
                block_sq_sum += block_estimate * block_estimate;
            }
            summarize_block_estimates(pair_count, block_sum, block_sq_sum)
        }
        MonteCarloTechnique::ControlVariate => {
            let mut moments = ControlVariateMoments::default();
            let mut path_normals = Vec::with_capacity(cfg.n_steps);
            for path_idx in 0..cfg.n_paths {
                let mut log_s_t = cfg.s0.ln();
                let mut knocked_out = cfg.s0 <= cfg.barrier;
                let bridge = bridge_workspace.step_normals(&qmc, path_idx);
                let normals = if bridge.is_none() {
                    qmc.fill_standard_normals(path_idx, cfg.n_steps, &mut path_normals);
                    Some(path_normals.as_slice())
                } else {
                    None
                };
                for step_idx in 0..cfg.n_steps {
                    let z = path_standard_normal(&qmc, path_idx, step_idx, normals, bridge);
                    log_s_t += drift_dt + vol_dt * z;
                    if log_s_t.exp() <= cfg.barrier {
                        knocked_out = true;
                    }
                }
                let s_t = log_s_t.exp();
                let payoff = if knocked_out {
                    0.0
                } else {
                    (s_t - cfg.k).max(0.0) * discount
                };
                moments.record(payoff, discount * s_t);
            }
            summarize_control_variate(moments, cfg.s0)
        }
    }
}

fn simulate_basket_pseudorandom_standard(cfg: &BasketCallConfig) -> BasketCallResult {
    let params = BasketTerminalParams::new(cfg);
    let mut rng = MonteCarloRng::new(cfg.seed);
    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;

    for _ in 0..cfg.n_paths {
        let z1 = rng.standard_normal();
        let z2 = rng.standard_normal();
        let payoff = basket_terminal_payoff(cfg, &params, z1, z2).0;
        payoff_sum += payoff;
        payoff_sq_sum += payoff * payoff;
    }

    summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
}

fn simulate_basket_pseudorandom_antithetic(cfg: &BasketCallConfig) -> BasketCallResult {
    let params = BasketTerminalParams::new(cfg);
    let pair_count = cfg.n_paths.div_ceil(2);
    let mut rng = MonteCarloRng::new(cfg.seed);
    let mut block_sum = 0.0;
    let mut block_sq_sum = 0.0;

    for _ in 0..pair_count {
        let z1 = rng.standard_normal();
        let z2 = rng.standard_normal();
        let payoff_a = basket_terminal_payoff(cfg, &params, z1, z2).0;
        let payoff_b = basket_terminal_payoff(cfg, &params, -z1, -z2).0;
        let block_estimate = 0.5 * (payoff_a + payoff_b);
        block_sum += block_estimate;
        block_sq_sum += block_estimate * block_estimate;
    }

    summarize_block_estimates(pair_count, block_sum, block_sq_sum)
}

fn simulate_basket_pseudorandom_control_variate(cfg: &BasketCallConfig) -> BasketCallResult {
    let params = BasketTerminalParams::new(cfg);
    let mut rng = MonteCarloRng::new(cfg.seed);
    let mut moments = ControlVariateMoments::default();

    for _ in 0..cfg.n_paths {
        let z1 = rng.standard_normal();
        let z2 = rng.standard_normal();
        let (payoff, control) = basket_terminal_payoff(cfg, &params, z1, z2);
        moments.record(payoff, control);
    }

    summarize_control_variate(moments, basket_control_expectation(cfg))
}

fn simulate_basket_qmc(cfg: &BasketCallConfig) -> BasketCallResult {
    let params = BasketTerminalParams::new(cfg);
    let qmc = StructuredNormalSampler::new(cfg.sampling, cfg.seed, cfg.n_paths, 2);
    let mut normals = Vec::with_capacity(2);

    match cfg.technique {
        MonteCarloTechnique::Standard => {
            let mut payoff_sum = 0.0;
            let mut payoff_sq_sum = 0.0;
            for path_idx in 0..cfg.n_paths {
                qmc.fill_standard_normals(path_idx, 2, &mut normals);
                let payoff = basket_terminal_payoff(cfg, &params, normals[0], normals[1]).0;
                payoff_sum += payoff;
                payoff_sq_sum += payoff * payoff;
            }
            summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
        }
        MonteCarloTechnique::Antithetic => {
            let pair_count = cfg.n_paths.div_ceil(2);
            let mut block_sum = 0.0;
            let mut block_sq_sum = 0.0;
            for pair_idx in 0..pair_count {
                qmc.fill_standard_normals(pair_idx, 2, &mut normals);
                let payoff_a = basket_terminal_payoff(cfg, &params, normals[0], normals[1]).0;
                let payoff_b = basket_terminal_payoff(cfg, &params, -normals[0], -normals[1]).0;
                let block_estimate = 0.5 * (payoff_a + payoff_b);
                block_sum += block_estimate;
                block_sq_sum += block_estimate * block_estimate;
            }
            summarize_block_estimates(pair_count, block_sum, block_sq_sum)
        }
        MonteCarloTechnique::ControlVariate => {
            let mut moments = ControlVariateMoments::default();
            for path_idx in 0..cfg.n_paths {
                qmc.fill_standard_normals(path_idx, 2, &mut normals);
                let (payoff, control) =
                    basket_terminal_payoff(cfg, &params, normals[0], normals[1]);
                moments.record(payoff, control);
            }
            summarize_control_variate(moments, basket_control_expectation(cfg))
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct BasketTerminalParams {
    drift1: f64,
    drift2: f64,
    vol1: f64,
    vol2: f64,
    rho_orthogonal_scale: f64,
    discount: f64,
}

impl BasketTerminalParams {
    fn new(cfg: &BasketCallConfig) -> Self {
        Self {
            drift1: (cfg.r - 0.5 * cfg.sigma1 * cfg.sigma1) * cfg.t,
            drift2: (cfg.r - 0.5 * cfg.sigma2 * cfg.sigma2) * cfg.t,
            vol1: cfg.sigma1 * cfg.t.sqrt(),
            vol2: cfg.sigma2 * cfg.t.sqrt(),
            rho_orthogonal_scale: (1.0 - cfg.rho * cfg.rho).max(0.0).sqrt(),
            discount: (-cfg.r * cfg.t).exp(),
        }
    }
}

fn basket_terminal_payoff(
    cfg: &BasketCallConfig,
    params: &BasketTerminalParams,
    z1: f64,
    z2_independent: f64,
) -> (f64, f64) {
    let z2 = cfg.rho * z1 + params.rho_orthogonal_scale * z2_independent;
    let s1_t = cfg.s01 * (params.drift1 + params.vol1 * z1).exp();
    let s2_t = cfg.s02 * (params.drift2 + params.vol2 * z2).exp();
    let basket_terminal = cfg.weight1 * s1_t + cfg.weight2 * s2_t;
    let payoff = (basket_terminal - cfg.k).max(0.0) * params.discount;
    let control = basket_terminal * params.discount;
    (payoff, control)
}

fn basket_control_expectation(cfg: &BasketCallConfig) -> f64 {
    cfg.weight1 * cfg.s01 + cfg.weight2 * cfg.s02
}

pub(crate) fn summarize_payoffs(
    n_paths: usize,
    payoff_sum: f64,
    payoff_sq_sum: f64,
) -> EuropeanCallResult {
    let n = n_paths as f64;
    let price = payoff_sum / n;
    let variance = (payoff_sq_sum / n) - (price * price);
    let stderr = variance.max(0.0).sqrt() / n.sqrt();

    EuropeanCallResult { price, stderr }
}

pub(crate) fn summarize_block_estimates(
    block_count: usize,
    block_sum: f64,
    block_sq_sum: f64,
) -> EuropeanCallResult {
    let n = block_count as f64;
    let price = block_sum / n;
    let variance = (block_sq_sum / n) - (price * price);
    let stderr = variance.max(0.0).sqrt() / n.sqrt();

    EuropeanCallResult { price, stderr }
}

pub(crate) fn summarize_control_variate(
    moments: ControlVariateMoments,
    control_expectation: f64,
) -> EuropeanCallResult {
    let n = moments.sample_count as f64;
    let payoff_mean = moments.payoff_sum / n;
    let control_mean = moments.control_sum / n;
    let control_var = (moments.control_sq_sum / n) - (control_mean * control_mean);

    if control_var <= f64::EPSILON {
        return summarize_payoffs(
            moments.sample_count,
            moments.payoff_sum,
            moments.payoff_sq_sum,
        );
    }

    let payoff_control_cov = (moments.payoff_control_cross_sum / n) - (payoff_mean * control_mean);
    let beta = payoff_control_cov / control_var;
    let adjusted_mean = payoff_mean - beta * (control_mean - control_expectation);
    let adjusted_sq_mean = (moments.payoff_sq_sum / n)
        - (2.0 * beta)
            * ((moments.payoff_control_cross_sum / n) - control_expectation * payoff_mean)
        + (beta * beta)
            * ((moments.control_sq_sum / n) - (2.0 * control_expectation * control_mean)
                + (control_expectation * control_expectation));
    let adjusted_var = (adjusted_sq_mean - (adjusted_mean * adjusted_mean)).max(0.0);
    let stderr = adjusted_var.sqrt() / n.sqrt();

    EuropeanCallResult {
        price: adjusted_mean,
        stderr,
    }
}

fn european_call_payoff_from_shock(
    s0: f64,
    k: f64,
    drift_t: f64,
    vol_t: f64,
    z: f64,
    discount: f64,
) -> f64 {
    let s_t = s0 * (drift_t + vol_t * z).exp();
    (s_t - k).max(0.0) * discount
}

fn simulate_terminal_antithetic(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    let drift_t = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * cfg.t;
    let vol_t = cfg.sigma * cfg.t.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let pair_count = cfg.n_paths.div_ceil(2);

    let mut rng = MonteCarloRng::new(cfg.seed);
    let mut block_sum = 0.0;
    let mut block_sq_sum = 0.0;

    for _ in 0..pair_count {
        let z = rng.standard_normal();
        let payoff_a = european_call_payoff_from_shock(cfg.s0, cfg.k, drift_t, vol_t, z, discount);
        let payoff_b = european_call_payoff_from_shock(cfg.s0, cfg.k, drift_t, vol_t, -z, discount);
        let block_estimate = 0.5 * (payoff_a + payoff_b);
        block_sum += block_estimate;
        block_sq_sum += block_estimate * block_estimate;
    }

    summarize_block_estimates(pair_count, block_sum, block_sq_sum)
}

fn simulate_stepwise_antithetic(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let pair_count = cfg.n_paths.div_ceil(2);

    let mut rng = MonteCarloRng::new(cfg.seed);
    let mut block_sum = 0.0;
    let mut block_sq_sum = 0.0;

    for _ in 0..pair_count {
        let mut log_a = cfg.s0.ln();
        let mut log_b = cfg.s0.ln();
        for _ in 0..cfg.n_steps {
            let z = rng.standard_normal();
            log_a += drift_dt + vol_dt * z;
            log_b += drift_dt - vol_dt * z;
        }

        let payoff_a = (log_a.exp() - cfg.k).max(0.0) * discount;
        let payoff_b = (log_b.exp() - cfg.k).max(0.0) * discount;
        let block_estimate = 0.5 * (payoff_a + payoff_b);
        block_sum += block_estimate;
        block_sq_sum += block_estimate * block_estimate;
    }

    summarize_block_estimates(pair_count, block_sum, block_sq_sum)
}

fn simulate_terminal_control_variate(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    let drift_t = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * cfg.t;
    let vol_t = cfg.sigma * cfg.t.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let thread_count = resolved_thread_count(cfg.n_threads);

    let moments = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_terminal_control_variate_chunk(
            cfg.seed,
            cfg.n_paths,
            cfg.s0,
            cfg.k,
            drift_t,
            vol_t,
            discount,
        )
    } else {
        simulate_terminal_control_variate_parallel(cfg, thread_count, drift_t, vol_t, discount)
    };

    summarize_control_variate(moments, cfg.s0)
}

fn simulate_stepwise_control_variate(cfg: &EuropeanCallConfig) -> EuropeanCallResult {
    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let thread_count = resolved_thread_count(cfg.n_threads);

    let moments = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_stepwise_control_variate_chunk(
            cfg.seed,
            cfg.n_paths,
            cfg.n_steps,
            cfg.s0,
            cfg.k,
            drift_dt,
            vol_dt,
            discount,
        )
    } else {
        simulate_stepwise_control_variate_parallel(cfg, thread_count, drift_dt, vol_dt, discount)
    };

    summarize_control_variate(moments, cfg.s0)
}

fn arithmetic_asian_call_price_mc_stepwise_standard(
    cfg: &ArithmeticAsianCallConfig,
) -> ArithmeticAsianCallResult {
    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let thread_count = resolved_thread_count(cfg.n_threads);

    let (payoff_sum, payoff_sq_sum) = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_asian_stepwise_chunk(
            cfg.seed,
            cfg.n_paths,
            cfg.n_steps,
            cfg.s0,
            cfg.k,
            drift_dt,
            vol_dt,
            discount,
        )
    } else {
        simulate_asian_stepwise_parallel(cfg, thread_count, drift_dt, vol_dt, discount)
    };

    summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
}

fn arithmetic_asian_call_price_mc_stepwise_antithetic(
    cfg: &ArithmeticAsianCallConfig,
) -> ArithmeticAsianCallResult {
    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let pair_count = cfg.n_paths.div_ceil(2);
    let thread_count = resolved_thread_count(cfg.n_threads);

    let (block_sum, block_sq_sum) = if thread_count <= 1 || pair_count < thread_count * 2_000 {
        simulate_asian_stepwise_antithetic_chunk(
            cfg.seed,
            pair_count,
            cfg.n_steps,
            cfg.s0,
            cfg.k,
            drift_dt,
            vol_dt,
            discount,
        )
    } else {
        simulate_asian_stepwise_antithetic_parallel(cfg, thread_count, drift_dt, vol_dt, discount)
    };

    summarize_block_estimates(pair_count, block_sum, block_sq_sum)
}

fn arithmetic_asian_call_price_mc_stepwise_control_variate(
    cfg: &ArithmeticAsianCallConfig,
) -> ArithmeticAsianCallResult {
    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let thread_count = resolved_thread_count(cfg.n_threads);

    let moments = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_asian_stepwise_control_variate_chunk(
            cfg.seed,
            cfg.n_paths,
            cfg.n_steps,
            cfg.s0,
            cfg.k,
            drift_dt,
            vol_dt,
            discount,
        )
    } else {
        simulate_asian_stepwise_control_variate_parallel(
            cfg,
            thread_count,
            drift_dt,
            vol_dt,
            discount,
        )
    };

    summarize_control_variate(moments, cfg.s0)
}

fn simulate_down_and_out_stepwise_chunk(
    seed: u64,
    n_paths: usize,
    n_steps: usize,
    s0: f64,
    k: f64,
    barrier: f64,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> (f64, f64) {
    let mut rng = MonteCarloRng::new(seed);
    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;

    for _ in 0..n_paths {
        let mut log_s_t = s0.ln();
        let mut knocked_out = s0 <= barrier;
        for _ in 0..n_steps {
            let z = rng.standard_normal();
            log_s_t += drift_dt + vol_dt * z;
            if log_s_t.exp() <= barrier {
                knocked_out = true;
            }
        }

        let payoff = if knocked_out {
            0.0
        } else {
            (log_s_t.exp() - k).max(0.0) * discount
        };
        payoff_sum += payoff;
        payoff_sq_sum += payoff * payoff;
    }

    (payoff_sum, payoff_sq_sum)
}

fn simulate_down_and_out_stepwise_control_variate_chunk(
    seed: u64,
    n_paths: usize,
    n_steps: usize,
    s0: f64,
    k: f64,
    barrier: f64,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> ControlVariateMoments {
    let mut rng = MonteCarloRng::new(seed);
    let mut moments = ControlVariateMoments::default();

    for _ in 0..n_paths {
        let mut log_s_t = s0.ln();
        let mut knocked_out = s0 <= barrier;
        for _ in 0..n_steps {
            let z = rng.standard_normal();
            log_s_t += drift_dt + vol_dt * z;
            if log_s_t.exp() <= barrier {
                knocked_out = true;
            }
        }

        let s_t = log_s_t.exp();
        let payoff = if knocked_out {
            0.0
        } else {
            (s_t - k).max(0.0) * discount
        };
        moments.record(payoff, discount * s_t);
    }

    moments
}

fn simulate_down_and_out_stepwise_antithetic_chunk(
    seed: u64,
    pair_count: usize,
    n_steps: usize,
    s0: f64,
    k: f64,
    barrier: f64,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> (f64, f64) {
    let mut rng = MonteCarloRng::new(seed);
    let mut block_sum = 0.0;
    let mut block_sq_sum = 0.0;

    for _ in 0..pair_count {
        let mut log_a = s0.ln();
        let mut log_b = s0.ln();
        let mut knock_a = s0 <= barrier;
        let mut knock_b = s0 <= barrier;
        for _ in 0..n_steps {
            let z = rng.standard_normal();
            log_a += drift_dt + vol_dt * z;
            log_b += drift_dt - vol_dt * z;
            if log_a.exp() <= barrier {
                knock_a = true;
            }
            if log_b.exp() <= barrier {
                knock_b = true;
            }
        }

        let payoff_a = if knock_a {
            0.0
        } else {
            (log_a.exp() - k).max(0.0) * discount
        };
        let payoff_b = if knock_b {
            0.0
        } else {
            (log_b.exp() - k).max(0.0) * discount
        };
        let block_estimate = 0.5 * (payoff_a + payoff_b);
        block_sum += block_estimate;
        block_sq_sum += block_estimate * block_estimate;
    }

    (block_sum, block_sq_sum)
}

fn simulate_down_and_out_stepwise_parallel(
    cfg: &DownAndOutCallConfig,
    thread_count: usize,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> (f64, f64) {
    let base_chunk = cfg.n_paths / thread_count;
    let remainder = cfg.n_paths % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let n_paths_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        let s0 = cfg.s0;
        let k = cfg.k;
        let barrier = cfg.barrier;
        let n_steps = cfg.n_steps;
        handles.push(thread::spawn(move || {
            simulate_down_and_out_stepwise_chunk(
                seed,
                n_paths_chunk,
                n_steps,
                s0,
                k,
                barrier,
                drift_dt,
                vol_dt,
                discount,
            )
        }));
    }

    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;
    for handle in handles {
        let (chunk_sum, chunk_sq_sum) = handle
            .join()
            .expect("CPU Monte Carlo worker thread panicked");
        payoff_sum += chunk_sum;
        payoff_sq_sum += chunk_sq_sum;
    }

    (payoff_sum, payoff_sq_sum)
}

fn simulate_down_and_out_stepwise_control_variate_parallel(
    cfg: &DownAndOutCallConfig,
    thread_count: usize,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> ControlVariateMoments {
    let base_chunk = cfg.n_paths / thread_count;
    let remainder = cfg.n_paths % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let n_paths_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        let s0 = cfg.s0;
        let k = cfg.k;
        let barrier = cfg.barrier;
        let n_steps = cfg.n_steps;
        handles.push(thread::spawn(move || {
            simulate_down_and_out_stepwise_control_variate_chunk(
                seed,
                n_paths_chunk,
                n_steps,
                s0,
                k,
                barrier,
                drift_dt,
                vol_dt,
                discount,
            )
        }));
    }

    let mut moments = ControlVariateMoments::default();
    for handle in handles {
        let chunk = handle
            .join()
            .expect("CPU Monte Carlo worker thread panicked");
        moments.merge(chunk);
    }

    moments
}

fn simulate_down_and_out_stepwise_antithetic_parallel(
    cfg: &DownAndOutCallConfig,
    thread_count: usize,
    drift_dt: f64,
    vol_dt: f64,
    discount: f64,
) -> (f64, f64) {
    let pair_count = cfg.n_paths.div_ceil(2);
    let base_chunk = pair_count / thread_count;
    let remainder = pair_count % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let pair_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        let s0 = cfg.s0;
        let k = cfg.k;
        let barrier = cfg.barrier;
        let n_steps = cfg.n_steps;
        handles.push(thread::spawn(move || {
            simulate_down_and_out_stepwise_antithetic_chunk(
                seed, pair_chunk, n_steps, s0, k, barrier, drift_dt, vol_dt, discount,
            )
        }));
    }

    let mut block_sum = 0.0;
    let mut block_sq_sum = 0.0;
    for handle in handles {
        let (chunk_sum, chunk_sq_sum) = handle
            .join()
            .expect("CPU Monte Carlo worker thread panicked");
        block_sum += chunk_sum;
        block_sq_sum += chunk_sq_sum;
    }

    (block_sum, block_sq_sum)
}

fn down_and_out_call_price_mc_stepwise_standard(
    cfg: &DownAndOutCallConfig,
) -> DownAndOutCallResult {
    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let thread_count = resolved_thread_count(cfg.n_threads);

    let (payoff_sum, payoff_sq_sum) = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_down_and_out_stepwise_chunk(
            cfg.seed,
            cfg.n_paths,
            cfg.n_steps,
            cfg.s0,
            cfg.k,
            cfg.barrier,
            drift_dt,
            vol_dt,
            discount,
        )
    } else {
        simulate_down_and_out_stepwise_parallel(cfg, thread_count, drift_dt, vol_dt, discount)
    };

    summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
}

fn down_and_out_call_price_mc_stepwise_antithetic(
    cfg: &DownAndOutCallConfig,
) -> DownAndOutCallResult {
    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let pair_count = cfg.n_paths.div_ceil(2);
    let thread_count = resolved_thread_count(cfg.n_threads);

    let (block_sum, block_sq_sum) = if thread_count <= 1 || pair_count < thread_count * 2_000 {
        simulate_down_and_out_stepwise_antithetic_chunk(
            cfg.seed,
            pair_count,
            cfg.n_steps,
            cfg.s0,
            cfg.k,
            cfg.barrier,
            drift_dt,
            vol_dt,
            discount,
        )
    } else {
        simulate_down_and_out_stepwise_antithetic_parallel(
            cfg,
            thread_count,
            drift_dt,
            vol_dt,
            discount,
        )
    };

    summarize_block_estimates(pair_count, block_sum, block_sq_sum)
}

fn down_and_out_call_price_mc_stepwise_control_variate(
    cfg: &DownAndOutCallConfig,
) -> DownAndOutCallResult {
    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let thread_count = resolved_thread_count(cfg.n_threads);

    let moments = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_down_and_out_stepwise_control_variate_chunk(
            cfg.seed,
            cfg.n_paths,
            cfg.n_steps,
            cfg.s0,
            cfg.k,
            cfg.barrier,
            drift_dt,
            vol_dt,
            discount,
        )
    } else {
        simulate_down_and_out_stepwise_control_variate_parallel(
            cfg,
            thread_count,
            drift_dt,
            vol_dt,
            discount,
        )
    };

    summarize_control_variate(moments, cfg.s0)
}

fn resolved_thread_count(requested_threads: usize) -> usize {
    if requested_threads > 0 {
        return requested_threads;
    }

    thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

fn derive_chunk_seed(base_seed: u64, chunk_index: u64) -> u64 {
    splitmix64(base_seed.wrapping_add(chunk_index.wrapping_mul(0x9E37_79B9_7F4A_7C15)))
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
