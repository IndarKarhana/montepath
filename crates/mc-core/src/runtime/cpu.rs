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
    MertonJumpDiffusionCall,
    HestonEuropeanCall,
    ArithmeticAsianCall,
    DownAndOutCall,
    BasketCall,
    LookbackCall,
    AmericanPut,
    BermudanPut,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Greek {
    Delta,
    Delta2,
    Vega,
    Vega2,
    Rho,
    Theta,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GreekEstimator {
    BumpAndRevalue,
    Pathwise,
    LikelihoodRatio,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GreekEstimate {
    pub greek: Greek,
    pub estimator: GreekEstimator,
    pub value: f64,
    pub stderr: Option<f64>,
    pub bump_size: Option<f64>,
    pub base_price: f64,
    pub bumped_up_price: Option<f64>,
    pub bumped_down_price: Option<f64>,
    pub stderr_estimate_kind: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GreekReport {
    pub workload: PricingWorkloadFamily,
    pub estimator: GreekEstimator,
    pub paths: usize,
    pub steps: usize,
    pub seed: u64,
    pub base_price: f64,
    pub base_stderr: f64,
    pub estimated_runtime_ms: Option<f64>,
    pub estimates: Vec<GreekEstimate>,
    pub warnings: Vec<String>,
}

impl GreekReport {
    pub fn estimate(&self, greek: Greek) -> Option<&GreekEstimate> {
        self.estimates
            .iter()
            .find(|estimate| estimate.greek == greek)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BlackScholesGreeks {
    pub price: f64,
    pub delta: f64,
    pub vega: f64,
    pub rho: f64,
    pub theta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HestonReferenceComparison {
    pub reference_name: String,
    pub paths: usize,
    pub steps: usize,
    pub reference_price: f64,
    pub simulated_price: f64,
    pub stderr: f64,
    pub error: f64,
    pub abs_error: f64,
    pub error_stderr_units: f64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EarlyExerciseReferenceComparison {
    pub workload: PricingWorkloadFamily,
    pub reference_name: String,
    pub paths: usize,
    pub steps: usize,
    pub seed: u64,
    pub reference_steps: usize,
    pub exercise_schedule: Vec<usize>,
    pub reference_price: f64,
    pub simulated_price: f64,
    pub stderr: f64,
    pub error: f64,
    pub abs_error: f64,
    pub error_stderr_units: f64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MlmcReferenceComparison {
    pub workload: PricingWorkloadFamily,
    pub reference_name: String,
    pub sampling: SamplingMethod,
    pub levels: usize,
    pub estimate_price: f64,
    pub estimate_stderr: f64,
    pub estimate_step_updates: usize,
    pub reference_price: f64,
    pub reference_stderr: f64,
    pub reference_paths: usize,
    pub reference_steps: usize,
    pub error: f64,
    pub abs_error: f64,
    pub combined_stderr: f64,
    pub error_combined_stderr_units: f64,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GaussianUncertaintyMomentResult {
    pub samples: usize,
    pub dimensions: usize,
    pub sampling: SamplingMethod,
    pub mean: f64,
    pub variance: f64,
    pub stderr_mean: f64,
    pub analytic_mean: f64,
    pub analytic_variance: f64,
    pub mean_abs_error: f64,
    pub variance_abs_error: f64,
    pub mean_ci_95_low: f64,
    pub mean_ci_95_high: f64,
    pub warnings: Vec<String>,
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

pub fn compare_heston_black_scholes_limit_cpu(
    cfg: &HestonEuropeanCallConfig,
) -> HestonReferenceComparison {
    let result = heston_european_call_price_mc_cpu(cfg);
    let reference_vol = cfg.v0.max(0.0).sqrt();
    let reference_price =
        black_scholes_european_call_price(cfg.s0, cfg.k, cfg.r, reference_vol, cfg.t);
    let error = result.price - reference_price;
    let abs_error = error.abs();
    let error_stderr_units = if result.stderr > 0.0 {
        error / result.stderr
    } else if error == 0.0 {
        0.0
    } else {
        f64::INFINITY.copysign(error)
    };
    let mut warnings = Vec::new();

    if cfg.vol_of_vol != 0.0 {
        warnings.push("Black-Scholes reference is exact only when vol_of_vol is zero".to_string());
    }
    if (cfg.theta - cfg.v0).abs() > f64::EPSILON {
        warnings.push(
            "Black-Scholes reference assumes theta equals v0 so variance is constant".to_string(),
        );
    }

    HestonReferenceComparison {
        reference_name: "black_scholes_vol_of_vol_zero_limit".to_string(),
        paths: cfg.n_paths,
        steps: cfg.n_steps,
        reference_price,
        simulated_price: result.price,
        stderr: result.stderr,
        error,
        abs_error,
        error_stderr_units,
        warnings,
    }
}

pub fn american_put_binomial_reference_price(
    cfg: &AmericanPutConfig,
    reference_steps: usize,
) -> f64 {
    validate_american_put_config(cfg);
    assert!(reference_steps > 0, "reference_steps must be > 0");
    let schedule = (0..=reference_steps).collect::<Vec<_>>();
    binomial_put_reference_price(
        cfg.s0,
        cfg.k,
        cfg.r,
        cfg.sigma,
        cfg.t,
        reference_steps,
        &schedule,
    )
}

pub fn bermudan_put_binomial_reference_price(
    cfg: &BermudanPutConfig,
    reference_steps: usize,
) -> f64 {
    validate_bermudan_put_config(cfg);
    assert!(reference_steps > 0, "reference_steps must be > 0");
    let (schedule, _) = normalize_bermudan_exercise_schedule(&cfg.exercise_steps, cfg.n_steps);
    let reference_schedule =
        map_exercise_schedule_to_reference_steps(&schedule, cfg.n_steps, reference_steps);
    binomial_put_reference_price(
        cfg.s0,
        cfg.k,
        cfg.r,
        cfg.sigma,
        cfg.t,
        reference_steps,
        &reference_schedule,
    )
}

pub fn compare_american_put_lsm_binomial_reference_cpu(
    cfg: &AmericanPutConfig,
    reference_steps: usize,
) -> EarlyExerciseReferenceComparison {
    let result = american_put_price_lsm_cpu(cfg);
    let reference_price = american_put_binomial_reference_price(cfg, reference_steps);
    build_early_exercise_reference_comparison(
        PricingWorkloadFamily::AmericanPut,
        "crr_binomial_american_put",
        cfg.n_paths,
        cfg.n_steps,
        cfg.seed,
        reference_steps,
        (1..=cfg.n_steps).collect(),
        reference_price,
        result.price,
        result.stderr,
        vec![
            "Binomial reference uses a Cox-Ross-Rubinstein tree with exercise at every reference step.".to_string(),
            "LSM and binomial references use different discretization grids, so compare with Monte Carlo error and discretization tolerance.".to_string(),
        ],
    )
}

pub fn compare_bermudan_put_lsm_binomial_reference_cpu(
    cfg: &BermudanPutConfig,
    reference_steps: usize,
) -> EarlyExerciseReferenceComparison {
    let result = bermudan_put_price_lsm_cpu(cfg);
    let reference_price = bermudan_put_binomial_reference_price(cfg, reference_steps);
    build_early_exercise_reference_comparison(
        PricingWorkloadFamily::BermudanPut,
        "crr_binomial_bermudan_put",
        cfg.n_paths,
        cfg.n_steps,
        cfg.seed,
        reference_steps,
        result.exercise_schedule,
        reference_price,
        result.price,
        result.stderr,
        vec![
            "Binomial reference maps simulation-grid exercise steps onto the requested reference tree.".to_string(),
            "LSM and binomial references use different discretization grids, so compare with Monte Carlo error and discretization tolerance.".to_string(),
        ],
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

pub fn compare_lookback_call_sampling_quality_cpu(
    cfg: &LookbackCallConfig,
    sampling: SamplingMethod,
) -> PricingQualityComparison {
    let mut baseline_cfg = *cfg;
    baseline_cfg.sampling = SamplingMethod::Pseudorandom;
    baseline_cfg.technique = MonteCarloTechnique::Standard;

    let mut structured_cfg = baseline_cfg;
    structured_cfg.sampling = sampling;

    let baseline = lookback_call_price_mc_cpu(&baseline_cfg);
    let structured = lookback_call_price_mc_cpu(&structured_cfg);

    build_pricing_quality_comparison(
        PricingWorkloadFamily::LookbackCall,
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

pub fn gaussian_uncertainty_moments_cpu(
    cfg: &GaussianUncertaintyConfig,
) -> GaussianUncertaintyMomentResult {
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
    let stderr_mean = (variance / n).sqrt();
    let analytic_mean = gaussian_uncertainty_analytic_mean();
    let analytic_variance = gaussian_uncertainty_analytic_variance();
    let mut warnings = Vec::new();

    if cfg.n_samples < 1_024 {
        warnings.push(
            "moment estimates are noisy below roughly 1024 samples; use as smoke evidence only"
                .to_string(),
        );
    }

    GaussianUncertaintyMomentResult {
        samples: cfg.n_samples,
        dimensions: cfg.dimensions,
        sampling: cfg.sampling,
        mean,
        variance,
        stderr_mean,
        analytic_mean,
        analytic_variance,
        mean_abs_error: (mean - analytic_mean).abs(),
        variance_abs_error: (variance - analytic_variance).abs(),
        mean_ci_95_low: mean - 1.96 * stderr_mean,
        mean_ci_95_high: mean + 1.96 * stderr_mean,
        warnings,
    }
}

fn gaussian_uncertainty_response(z: &[f64]) -> f64 {
    z[0] * z[0] + 0.5 * z[1] + (0.1 * z[2]).exp()
}

fn gaussian_uncertainty_analytic_mean() -> f64 {
    1.0 + 0.005f64.exp()
}

fn gaussian_uncertainty_analytic_variance() -> f64 {
    2.0 + 0.25 + (0.02f64.exp() - 0.01f64.exp())
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

pub fn black_scholes_european_put_price(s0: f64, k: f64, r: f64, sigma: f64, t: f64) -> f64 {
    let call = black_scholes_european_call_price(s0, k, r, sigma, t);
    call + k * (-r * t).exp() - s0
}

pub fn black_scholes_european_call_greeks(
    s0: f64,
    k: f64,
    r: f64,
    sigma: f64,
    t: f64,
) -> BlackScholesGreeks {
    assert!(s0 > 0.0, "s0 must be > 0");
    assert!(k > 0.0, "k must be > 0");
    assert!(sigma > 0.0, "sigma must be > 0");
    assert!(t > 0.0, "t must be > 0");

    let sqrt_t = t.sqrt();
    let d1 = ((s0 / k).ln() + (r + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    let discount = (-r * t).exp();
    let price = s0 * standard_normal_cdf(d1) - k * discount * standard_normal_cdf(d2);
    let pdf_d1 = standard_normal_pdf(d1);

    BlackScholesGreeks {
        price,
        delta: standard_normal_cdf(d1),
        vega: s0 * pdf_d1 * sqrt_t,
        rho: k * t * discount * standard_normal_cdf(d2),
        theta: -(s0 * pdf_d1 * sigma) / (2.0 * sqrt_t) - r * k * discount * standard_normal_cdf(d2),
    }
}

fn standard_normal_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * PI).sqrt()
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

fn discounted_lognormal_call_price(
    log_mean: f64,
    log_variance: f64,
    k: f64,
    r: f64,
    t: f64,
) -> f64 {
    if log_variance == 0.0 {
        return (-r * t).exp() * (log_mean.exp() - k).max(0.0);
    }

    let log_stddev = log_variance.sqrt();
    let log_k = k.ln();
    let d2 = (log_mean - log_k) / log_stddev;
    let d1 = d2 + log_stddev;
    (-r * t).exp()
        * ((log_mean + 0.5 * log_variance).exp() * standard_normal_cdf(d1)
            - k * standard_normal_cdf(d2))
}

fn sample_poisson(rng: &mut MonteCarloRng, mean: f64) -> usize {
    debug_assert!(mean.is_finite());
    debug_assert!(mean >= 0.0);

    if mean == 0.0 {
        return 0;
    }

    let u = rng.next_f64_open01();
    let mut probability = (-mean).exp();
    let mut cumulative = probability;
    let mut k = 0usize;

    while u > cumulative {
        k += 1;
        probability *= mean / k as f64;
        cumulative += probability;
        assert!(k < 4_096, "Poisson sampler did not converge");
    }

    k
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
    Sensitivity,
    EarlyExercise,
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
        method_capability(
            "longstaff_schwartz_lsm",
            "Longstaff-Schwartz least-squares Monte Carlo",
            MonteCarloMethodCategory::EarlyExercise,
            BackendMethodSupport::CpuReference,
            BackendMethodSupport::Planned,
            BackendMethodSupport::Planned,
            &[
                "CPU reference support currently covers American puts and Bermudan custom-schedule puts under GBM with Laguerre continuation-value regression",
                "European put lower-bound validation is committed; high-precision American/Bermudan fixtures and external comparison lanes are pending",
            ],
        ),
        method_capability(
            "bump_and_revalue_greeks",
            "Bump-and-revalue Greeks",
            MonteCarloMethodCategory::Sensitivity,
            BackendMethodSupport::CpuReference,
            BackendMethodSupport::Planned,
            BackendMethodSupport::Planned,
            &["CPU reference support covers current European, arithmetic Asian, down-and-out, lookback, basket, and Heston workload families"],
        ),
        method_capability(
            "pathwise_greeks",
            "Pathwise Greeks",
            MonteCarloMethodCategory::Sensitivity,
            BackendMethodSupport::CpuReference,
            BackendMethodSupport::Planned,
            BackendMethodSupport::Planned,
            &["CPU reference support currently covers terminal GBM European-call Delta, Vega, Rho, and Theta"],
        ),
        method_capability(
            "likelihood_ratio_greeks",
            "Likelihood-ratio Greeks",
            MonteCarloMethodCategory::Sensitivity,
            BackendMethodSupport::CpuReference,
            BackendMethodSupport::Planned,
            BackendMethodSupport::Planned,
            &["CPU reference support currently covers terminal GBM European-call Delta, Vega, and Rho"],
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EuropeanCallSweepScenario {
    pub scenario_id: String,
    pub s0: Option<f64>,
    pub k: Option<f64>,
    pub r: Option<f64>,
    pub sigma: Option<f64>,
    pub t: Option<f64>,
    pub n_paths: Option<usize>,
    pub n_steps: Option<usize>,
    pub seed: Option<u64>,
    pub sampling: Option<SamplingMethod>,
    pub technique: Option<MonteCarloTechnique>,
    pub method: Option<EuropeanCallMethod>,
}

impl Default for EuropeanCallSweepScenario {
    fn default() -> Self {
        Self {
            scenario_id: "base".to_string(),
            s0: None,
            k: None,
            r: None,
            sigma: None,
            t: None,
            n_paths: None,
            n_steps: None,
            seed: None,
            sampling: None,
            technique: None,
            method: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EuropeanCallParameterSweepConfig {
    pub base_config: EuropeanCallConfig,
    pub method: EuropeanCallMethod,
    pub seed_stride: u64,
    pub scenarios: Vec<EuropeanCallSweepScenario>,
}

impl Default for EuropeanCallParameterSweepConfig {
    fn default() -> Self {
        Self {
            base_config: EuropeanCallConfig::default(),
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
                    scenario_id: "high_vol".to_string(),
                    sigma: Some(0.35),
                    ..EuropeanCallSweepScenario::default()
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EuropeanCallParameterSweepRow {
    pub scenario_index: usize,
    pub scenario_id: String,
    pub method: EuropeanCallMethod,
    pub config: EuropeanCallConfig,
    pub result: EuropeanCallResult,
    pub analytic_price: f64,
    pub signed_error_vs_black_scholes: f64,
    pub abs_error_vs_black_scholes: f64,
    pub abs_error_stderr_units: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EuropeanCallParameterSweepResult {
    pub schema_version: String,
    pub workload: PricingWorkloadFamily,
    pub method: EuropeanCallMethod,
    pub scenario_count: usize,
    pub total_paths: usize,
    pub base_seed: u64,
    pub seed_stride: u64,
    pub rows: Vec<EuropeanCallParameterSweepRow>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct HestonEuropeanCallConfig {
    pub s0: f64,
    pub k: f64,
    pub r: f64,
    pub v0: f64,
    pub kappa: f64,
    pub theta: f64,
    pub vol_of_vol: f64,
    pub rho: f64,
    pub t: f64,
    pub n_paths: usize,
    pub n_steps: usize,
    pub seed: u64,
    pub n_threads: usize,
    pub technique: MonteCarloTechnique,
}

impl Default for HestonEuropeanCallConfig {
    fn default() -> Self {
        Self {
            s0: 100.0,
            k: 100.0,
            r: 0.03,
            v0: 0.04,
            kappa: 1.5,
            theta: 0.04,
            vol_of_vol: 0.3,
            rho: -0.6,
            t: 1.0,
            n_paths: 100_000,
            n_steps: 252,
            seed: 42,
            n_threads: 0,
            technique: MonteCarloTechnique::Standard,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MertonJumpDiffusionCallConfig {
    pub s0: f64,
    pub k: f64,
    pub r: f64,
    pub sigma: f64,
    pub jump_intensity: f64,
    pub jump_mean: f64,
    pub jump_volatility: f64,
    pub t: f64,
    pub n_paths: usize,
    pub seed: u64,
    pub n_threads: usize,
}

impl Default for MertonJumpDiffusionCallConfig {
    fn default() -> Self {
        Self {
            s0: 100.0,
            k: 100.0,
            r: 0.03,
            sigma: 0.2,
            jump_intensity: 0.4,
            jump_mean: -0.08,
            jump_volatility: 0.25,
            t: 1.0,
            n_paths: 100_000,
            seed: 42,
            n_threads: 0,
        }
    }
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
pub struct LookbackCallConfig {
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

impl Default for LookbackCallConfig {
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct AmericanPutConfig {
    pub s0: f64,
    pub k: f64,
    pub r: f64,
    pub sigma: f64,
    pub t: f64,
    pub n_paths: usize,
    pub n_steps: usize,
    pub seed: u64,
    pub n_threads: usize,
    pub basis_degree: usize,
}

impl Default for AmericanPutConfig {
    fn default() -> Self {
        Self {
            s0: 100.0,
            k: 100.0,
            r: 0.03,
            sigma: 0.2,
            t: 1.0,
            n_paths: 100_000,
            n_steps: 64,
            seed: 42,
            n_threads: 0,
            basis_degree: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BermudanPutConfig {
    pub s0: f64,
    pub k: f64,
    pub r: f64,
    pub sigma: f64,
    pub t: f64,
    pub n_paths: usize,
    pub n_steps: usize,
    pub seed: u64,
    pub n_threads: usize,
    pub basis_degree: usize,
    pub exercise_steps: Vec<usize>,
}

impl Default for BermudanPutConfig {
    fn default() -> Self {
        Self {
            s0: 100.0,
            k: 100.0,
            r: 0.03,
            sigma: 0.2,
            t: 1.0,
            n_paths: 100_000,
            n_steps: 64,
            seed: 42,
            n_threads: 0,
            basis_degree: 2,
            exercise_steps: vec![16, 32, 48, 64],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AmericanPutResult {
    pub workload: PricingWorkloadFamily,
    pub price: f64,
    pub stderr: f64,
    pub paths: usize,
    pub steps: usize,
    pub seed: u64,
    pub early_exercise_count: usize,
    pub maturity_exercise_count: usize,
    pub regression_steps: usize,
    pub regression_basis: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BermudanPutResult {
    pub workload: PricingWorkloadFamily,
    pub price: f64,
    pub stderr: f64,
    pub paths: usize,
    pub steps: usize,
    pub seed: u64,
    pub exercise_schedule: Vec<usize>,
    pub exercise_date_count: usize,
    pub early_exercise_count: usize,
    pub maturity_exercise_count: usize,
    pub regression_steps: usize,
    pub regression_basis: String,
    pub warnings: Vec<String>,
}

pub type DownAndOutCallResult = EuropeanCallResult;

pub type HestonEuropeanCallResult = EuropeanCallResult;

pub type MertonJumpDiffusionCallResult = EuropeanCallResult;

pub type LookbackCallResult = EuropeanCallResult;

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

    pub fn greeks(&self, estimator: GreekEstimator) -> GreekReport {
        arithmetic_asian_call_greeks_cpu(&self.config, estimator)
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

    pub fn greeks(&self, estimator: GreekEstimator) -> GreekReport {
        down_and_out_call_greeks_cpu(&self.config, estimator)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LookbackCallPricer {
    config: LookbackCallConfig,
}

impl Default for LookbackCallPricer {
    fn default() -> Self {
        Self::new()
    }
}

impl LookbackCallPricer {
    pub fn new() -> Self {
        Self {
            config: LookbackCallConfig::default(),
        }
    }

    pub fn from_config(config: LookbackCallConfig) -> Self {
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

    pub fn config(&self) -> &LookbackCallConfig {
        &self.config
    }

    pub fn price(&self) -> LookbackCallResult {
        lookback_call_price_mc_cpu(&self.config)
    }

    pub fn greeks(&self, estimator: GreekEstimator) -> GreekReport {
        lookback_call_greeks_cpu(&self.config, estimator)
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

    pub fn greeks(&self, estimator: GreekEstimator) -> GreekReport {
        basket_call_greeks_cpu(&self.config, estimator)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AmericanPutPricer {
    config: AmericanPutConfig,
}

impl Default for AmericanPutPricer {
    fn default() -> Self {
        Self::new()
    }
}

impl AmericanPutPricer {
    pub fn new() -> Self {
        Self {
            config: AmericanPutConfig::default(),
        }
    }

    pub fn from_config(config: AmericanPutConfig) -> Self {
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

    pub fn basis_degree(mut self, value: usize) -> Self {
        self.config.basis_degree = value;
        self
    }

    pub fn config(&self) -> &AmericanPutConfig {
        &self.config
    }

    pub fn price(&self) -> AmericanPutResult {
        american_put_price_lsm_cpu(&self.config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BermudanPutPricer {
    config: BermudanPutConfig,
}

impl Default for BermudanPutPricer {
    fn default() -> Self {
        Self::new()
    }
}

impl BermudanPutPricer {
    pub fn new() -> Self {
        Self {
            config: BermudanPutConfig::default(),
        }
    }

    pub fn from_config(config: BermudanPutConfig) -> Self {
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

    pub fn exercise_steps(mut self, value: Vec<usize>) -> Self {
        self.config.exercise_steps = value;
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

    pub fn basis_degree(mut self, value: usize) -> Self {
        self.config.basis_degree = value;
        self
    }

    pub fn config(&self) -> &BermudanPutConfig {
        &self.config
    }

    pub fn price(&self) -> BermudanPutResult {
        bermudan_put_price_lsm_cpu(&self.config)
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

    pub fn greeks(&self, estimator: GreekEstimator) -> GreekReport {
        european_call_greeks_cpu(&self.config, estimator)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HestonEuropeanCallPricer {
    config: HestonEuropeanCallConfig,
}

impl Default for HestonEuropeanCallPricer {
    fn default() -> Self {
        Self::new()
    }
}

impl HestonEuropeanCallPricer {
    pub fn new() -> Self {
        Self {
            config: HestonEuropeanCallConfig::default(),
        }
    }

    pub fn from_config(config: HestonEuropeanCallConfig) -> Self {
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

    pub fn initial_variance(mut self, value: f64) -> Self {
        self.config.v0 = value;
        self
    }

    pub fn mean_reversion(mut self, value: f64) -> Self {
        self.config.kappa = value;
        self
    }

    pub fn long_run_variance(mut self, value: f64) -> Self {
        self.config.theta = value;
        self
    }

    pub fn vol_of_vol(mut self, value: f64) -> Self {
        self.config.vol_of_vol = value;
        self
    }

    pub fn correlation(mut self, value: f64) -> Self {
        self.config.rho = value;
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

    pub fn config(&self) -> &HestonEuropeanCallConfig {
        &self.config
    }

    pub fn price(&self) -> HestonEuropeanCallResult {
        heston_european_call_price_mc_cpu(&self.config)
    }

    pub fn greeks(&self, estimator: GreekEstimator) -> GreekReport {
        heston_european_call_greeks_cpu(&self.config, estimator)
    }

    pub fn black_scholes_limit_check(&self) -> HestonReferenceComparison {
        compare_heston_black_scholes_limit_cpu(&self.config)
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

pub fn price_european_call_parameter_sweep_cpu(
    cfg: &EuropeanCallParameterSweepConfig,
) -> EuropeanCallParameterSweepResult {
    validate_european_call_parameter_sweep_config(cfg);

    let mut rows = Vec::with_capacity(cfg.scenarios.len());
    let mut total_paths = 0usize;
    let mut warnings = Vec::new();

    for (scenario_index, scenario) in cfg.scenarios.iter().enumerate() {
        let scenario_cfg = apply_european_call_sweep_scenario(
            cfg.base_config,
            scenario,
            scenario_index,
            cfg.seed_stride,
        );
        validate_european_call_config(&scenario_cfg);

        let method = scenario.method.unwrap_or(cfg.method);
        let result = european_call_price_mc_cpu_with_method(&scenario_cfg, method);
        let analytic_price = black_scholes_european_call_price(
            scenario_cfg.s0,
            scenario_cfg.k,
            scenario_cfg.r,
            scenario_cfg.sigma,
            scenario_cfg.t,
        );
        let signed_error = result.price - analytic_price;
        let abs_error = signed_error.abs();
        let abs_error_stderr_units = if result.stderr > 0.0 {
            abs_error / result.stderr
        } else if abs_error == 0.0 {
            0.0
        } else {
            f64::INFINITY
        };
        if abs_error_stderr_units > 4.0 {
            warnings.push(format!(
                "scenario '{}' is {:.3} standard errors from Black-Scholes reference",
                scenario.scenario_id, abs_error_stderr_units
            ));
        }

        total_paths += scenario_cfg.n_paths;
        rows.push(EuropeanCallParameterSweepRow {
            scenario_index,
            scenario_id: scenario.scenario_id.clone(),
            method,
            config: scenario_cfg,
            result,
            analytic_price,
            signed_error_vs_black_scholes: signed_error,
            abs_error_vs_black_scholes: abs_error,
            abs_error_stderr_units,
        });
    }

    EuropeanCallParameterSweepResult {
        schema_version: "european-call-sweep.v1".to_string(),
        workload: PricingWorkloadFamily::EuropeanCall,
        method: cfg.method,
        scenario_count: rows.len(),
        total_paths,
        base_seed: cfg.base_config.seed,
        seed_stride: cfg.seed_stride,
        rows,
        warnings,
    }
}

pub fn heston_european_call_price_mc_cpu(
    cfg: &HestonEuropeanCallConfig,
) -> HestonEuropeanCallResult {
    validate_heston_european_call_config(cfg);

    match cfg.technique {
        MonteCarloTechnique::Standard => heston_european_call_price_mc_standard(cfg),
        MonteCarloTechnique::Antithetic => heston_european_call_price_mc_antithetic(cfg),
        MonteCarloTechnique::ControlVariate => heston_european_call_price_mc_control_variate(cfg),
    }
}

pub fn merton_jump_diffusion_call_reference_price(
    cfg: &MertonJumpDiffusionCallConfig,
    max_terms: usize,
    tail_tolerance: f64,
) -> f64 {
    validate_merton_jump_diffusion_call_config(cfg);
    assert!(max_terms > 0, "max_terms must be > 0");
    assert!(tail_tolerance >= 0.0, "tail_tolerance must be >= 0");

    let lambda_t = cfg.jump_intensity * cfg.t;
    if lambda_t == 0.0 {
        return black_scholes_european_call_price(cfg.s0, cfg.k, cfg.r, cfg.sigma, cfg.t);
    }

    let jump_compensator =
        cfg.jump_intensity * ((cfg.jump_mean + 0.5 * cfg.jump_volatility.powi(2)).exp() - 1.0);
    let base_log_mean =
        cfg.s0.ln() + (cfg.r - jump_compensator - 0.5 * cfg.sigma * cfg.sigma) * cfg.t;
    let mut poisson_weight = (-lambda_t).exp();
    let mut cumulative_weight = 0.0;
    let mut price = 0.0;

    for n_jumps in 0..max_terms {
        let log_mean = base_log_mean + n_jumps as f64 * cfg.jump_mean;
        let log_variance = cfg.sigma * cfg.sigma * cfg.t
            + n_jumps as f64 * cfg.jump_volatility * cfg.jump_volatility;
        price += poisson_weight
            * discounted_lognormal_call_price(log_mean, log_variance, cfg.k, cfg.r, cfg.t);
        cumulative_weight += poisson_weight;

        if (1.0 - cumulative_weight).max(0.0) <= tail_tolerance {
            break;
        }

        let next = n_jumps + 1;
        poisson_weight *= lambda_t / next as f64;
    }

    price
}

pub fn merton_jump_diffusion_call_price_mc_cpu(
    cfg: &MertonJumpDiffusionCallConfig,
) -> MertonJumpDiffusionCallResult {
    validate_merton_jump_diffusion_call_config(cfg);

    let jump_compensator =
        cfg.jump_intensity * ((cfg.jump_mean + 0.5 * cfg.jump_volatility.powi(2)).exp() - 1.0);
    let drift_t = (cfg.r - jump_compensator - 0.5 * cfg.sigma * cfg.sigma) * cfg.t;
    let diffusion_vol_t = cfg.sigma * cfg.t.sqrt();
    let jump_mean_t = cfg.jump_intensity * cfg.t;
    let discount = (-cfg.r * cfg.t).exp();

    let thread_count = resolved_thread_count(cfg.n_threads);
    let (payoff_sum, payoff_sq_sum) = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_merton_jump_diffusion_chunk(
            cfg.seed,
            cfg.n_paths,
            cfg.s0,
            cfg.k,
            drift_t,
            diffusion_vol_t,
            jump_mean_t,
            cfg.jump_mean,
            cfg.jump_volatility,
            discount,
        )
    } else {
        simulate_merton_jump_diffusion_parallel(
            cfg,
            thread_count,
            drift_t,
            diffusion_vol_t,
            jump_mean_t,
            discount,
        )
    };

    summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
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

pub fn compare_arithmetic_asian_mlmc_reference_cpu(
    cfg: &ArithmeticAsianMlmcConfig,
    reference_cfg: &ArithmeticAsianCallConfig,
) -> MlmcReferenceComparison {
    validate_arithmetic_asian_mlmc_config(cfg);
    assert!(reference_cfg.n_paths > 0, "reference n_paths must be > 0");
    assert!(reference_cfg.n_steps > 0, "reference n_steps must be > 0");

    let estimate = arithmetic_asian_call_price_mlmc_cpu(cfg);
    let reference = arithmetic_asian_call_price_mc_cpu(reference_cfg);
    let error = estimate.price - reference.price;
    let abs_error = error.abs();
    let combined_stderr =
        (estimate.stderr * estimate.stderr + reference.stderr * reference.stderr).sqrt();
    let error_combined_stderr_units = if combined_stderr > 0.0 {
        error / combined_stderr
    } else if error == 0.0 {
        0.0
    } else {
        f64::INFINITY.copysign(error)
    };
    let mut warnings = Vec::new();

    if (cfg.s0 - reference_cfg.s0).abs() > f64::EPSILON
        || (cfg.k - reference_cfg.k).abs() > f64::EPSILON
        || (cfg.r - reference_cfg.r).abs() > f64::EPSILON
        || (cfg.sigma - reference_cfg.sigma).abs() > f64::EPSILON
        || (cfg.t - reference_cfg.t).abs() > f64::EPSILON
    {
        warnings.push(
            "MLMC config and reference config do not share the same market parameters".to_string(),
        );
    }
    if reference_cfg.n_steps
        < mlmc_fine_steps(cfg.base_steps, cfg.refinement_factor, cfg.levels - 1)
    {
        warnings.push(
            "reference step count is below the finest MLMC level; comparison includes time-discretization mismatch"
                .to_string(),
        );
    }
    if error_combined_stderr_units.abs() > 4.0 {
        warnings.push(format!(
            "MLMC estimate is {:.3} combined standard errors from the reference run",
            error_combined_stderr_units.abs()
        ));
    }

    MlmcReferenceComparison {
        workload: PricingWorkloadFamily::ArithmeticAsianCall,
        reference_name: "high_budget_arithmetic_asian_standard_mc".to_string(),
        sampling: cfg.sampling,
        levels: cfg.levels,
        estimate_price: estimate.price,
        estimate_stderr: estimate.stderr,
        estimate_step_updates: estimate.total_step_updates,
        reference_price: reference.price,
        reference_stderr: reference.stderr,
        reference_paths: reference_cfg.n_paths,
        reference_steps: reference_cfg.n_steps,
        error,
        abs_error,
        combined_stderr,
        error_combined_stderr_units,
        warnings,
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

pub fn lookback_call_price_mc_cpu(cfg: &LookbackCallConfig) -> LookbackCallResult {
    validate_lookback_call_config(cfg);

    if cfg.sampling != SamplingMethod::Pseudorandom {
        return simulate_lookback_stepwise_qmc(cfg);
    }

    match cfg.technique {
        MonteCarloTechnique::Standard => lookback_call_price_mc_stepwise_standard(cfg),
        MonteCarloTechnique::Antithetic => lookback_call_price_mc_stepwise_antithetic(cfg),
        MonteCarloTechnique::ControlVariate => lookback_call_price_mc_stepwise_control_variate(cfg),
    }
}

struct PutLsmInput {
    s0: f64,
    k: f64,
    r: f64,
    sigma: f64,
    t: f64,
    n_paths: usize,
    n_steps: usize,
    seed: u64,
    n_threads: usize,
    basis_degree: usize,
    exercise_schedule: Vec<usize>,
    allow_intrinsic_now: bool,
    support_warning: &'static str,
    schedule_warning: Option<String>,
}

struct PutLsmCoreResult {
    price: f64,
    stderr: f64,
    exercise_schedule: Vec<usize>,
    early_exercise_count: usize,
    maturity_exercise_count: usize,
    regression_steps: usize,
    regression_basis: String,
    warnings: Vec<String>,
}

fn price_put_lsm_cpu(input: PutLsmInput) -> PutLsmCoreResult {
    let degree = input.basis_degree.clamp(1, 3);
    let coeff_count = degree + 1;
    let dt = input.t / input.n_steps as f64;
    let drift = (input.r - 0.5 * input.sigma * input.sigma) * dt;
    let vol_dt = input.sigma * dt.sqrt();
    let discount_step = (-input.r * dt).exp();
    let path_len = input.n_steps + 1;
    let mut paths = vec![0.0; input.n_paths.saturating_mul(path_len)];
    let mut rng = MonteCarloRng::new(input.seed);

    for path_idx in 0..input.n_paths {
        let offset = path_idx * path_len;
        let mut s_t = input.s0;
        paths[offset] = s_t;
        for step in 1..=input.n_steps {
            let z = rng.standard_normal();
            s_t *= (drift + vol_dt * z).exp();
            paths[offset + step] = s_t;
        }
    }

    let mut cashflows = vec![0.0; input.n_paths];
    let mut exercise_steps = vec![input.n_steps; input.n_paths];
    for path_idx in 0..input.n_paths {
        let terminal = paths[path_idx * path_len + input.n_steps];
        cashflows[path_idx] = (input.k - terminal).max(0.0);
    }

    let mut regression_steps = 0usize;
    for &step in input.exercise_schedule.iter().rev().skip(1) {
        let mut ata = [[0.0; 4]; 4];
        let mut atb = [0.0; 4];
        let mut itm_count = 0usize;

        for path_idx in 0..input.n_paths {
            let s_t = paths[path_idx * path_len + step];
            let immediate = (input.k - s_t).max(0.0);
            if immediate <= 0.0 {
                continue;
            }

            let continuation =
                cashflows[path_idx] * discount_step.powi((exercise_steps[path_idx] - step) as i32);
            let basis = laguerre_lsm_basis(s_t / input.k.max(f64::MIN_POSITIVE), degree);
            for row in 0..coeff_count {
                atb[row] += basis[row] * continuation;
                for col in 0..coeff_count {
                    ata[row][col] += basis[row] * basis[col];
                }
            }
            itm_count += 1;
        }

        if itm_count < coeff_count {
            continue;
        }

        let Some(coefficients) = solve_lsm_normal_equations(ata, atb, coeff_count) else {
            continue;
        };
        regression_steps += 1;

        for path_idx in 0..input.n_paths {
            let s_t = paths[path_idx * path_len + step];
            let immediate = (input.k - s_t).max(0.0);
            if immediate <= 0.0 {
                continue;
            }

            let basis = laguerre_lsm_basis(s_t / input.k.max(f64::MIN_POSITIVE), degree);
            let continuation = (0..coeff_count)
                .map(|idx| coefficients[idx] * basis[idx])
                .sum::<f64>();
            if immediate > continuation.max(0.0) {
                cashflows[path_idx] = immediate;
                exercise_steps[path_idx] = step;
            }
        }
    }

    let mut sum = 0.0;
    let mut sq_sum = 0.0;
    let mut early_exercise_count = 0usize;
    let mut maturity_exercise_count = 0usize;

    for path_idx in 0..input.n_paths {
        let discounted = cashflows[path_idx] * discount_step.powi(exercise_steps[path_idx] as i32);
        sum += discounted;
        sq_sum += discounted * discounted;
        if exercise_steps[path_idx] < input.n_steps {
            early_exercise_count += 1;
        } else if cashflows[path_idx] > 0.0 {
            maturity_exercise_count += 1;
        }
    }

    let mut summary = summarize_payoffs(input.n_paths, sum, sq_sum);
    if input.allow_intrinsic_now {
        let intrinsic_now = (input.k - input.s0).max(0.0);
        if intrinsic_now > summary.price {
            summary.price = intrinsic_now;
            summary.stderr = 0.0;
        }
    }

    let mut warnings = vec![
        "Longstaff-Schwartz estimate uses Laguerre basis regression on simulated GBM paths; it is a lower-biased Monte Carlo estimator and should be benchmarked against external references before broad product claims.".to_string(),
        input.support_warning.to_string(),
    ];
    if input.basis_degree != degree {
        warnings.push("basis_degree was clamped to the supported range [1, 3].".to_string());
    }
    if input.n_threads > 1 {
        warnings.push(
            "n_threads is accepted for API symmetry, but the v1 LSM regression path is single-thread deterministic.".to_string(),
        );
    }
    if let Some(warning) = input.schedule_warning {
        warnings.push(warning);
    }

    PutLsmCoreResult {
        price: summary.price,
        stderr: summary.stderr,
        exercise_schedule: input.exercise_schedule,
        early_exercise_count,
        maturity_exercise_count,
        regression_steps,
        regression_basis: format!("laguerre_lsm_degree_{degree}"),
        warnings,
    }
}

pub fn american_put_price_lsm_cpu(cfg: &AmericanPutConfig) -> AmericanPutResult {
    validate_american_put_config(cfg);

    let exercise_schedule = (1..=cfg.n_steps).collect::<Vec<_>>();
    let core = price_put_lsm_cpu(PutLsmInput {
        s0: cfg.s0,
        k: cfg.k,
        r: cfg.r,
        sigma: cfg.sigma,
        t: cfg.t,
        n_paths: cfg.n_paths,
        n_steps: cfg.n_steps,
        seed: cfg.seed,
        n_threads: cfg.n_threads,
        basis_degree: cfg.basis_degree,
        exercise_schedule,
        allow_intrinsic_now: true,
        support_warning: "American-put support is CPU reference only; native GPU, Greeks, dividends, stochastic rates, and multi-asset exercise policies are not implemented yet.",
        schedule_warning: None,
    });

    AmericanPutResult {
        workload: PricingWorkloadFamily::AmericanPut,
        price: core.price,
        stderr: core.stderr,
        paths: cfg.n_paths,
        steps: cfg.n_steps,
        seed: cfg.seed,
        early_exercise_count: core.early_exercise_count,
        maturity_exercise_count: core.maturity_exercise_count,
        regression_steps: core.regression_steps,
        regression_basis: core.regression_basis,
        warnings: core.warnings,
    }
}

pub fn bermudan_put_price_lsm_cpu(cfg: &BermudanPutConfig) -> BermudanPutResult {
    validate_bermudan_put_config(cfg);

    let (exercise_schedule, schedule_warning) =
        normalize_bermudan_exercise_schedule(&cfg.exercise_steps, cfg.n_steps);
    let core = price_put_lsm_cpu(PutLsmInput {
        s0: cfg.s0,
        k: cfg.k,
        r: cfg.r,
        sigma: cfg.sigma,
        t: cfg.t,
        n_paths: cfg.n_paths,
        n_steps: cfg.n_steps,
        seed: cfg.seed,
        n_threads: cfg.n_threads,
        basis_degree: cfg.basis_degree,
        exercise_schedule,
        allow_intrinsic_now: false,
        support_warning: "Bermudan-put support is CPU reference only; native GPU, Greeks, dividends, stochastic rates, and multi-asset exercise policies are not implemented yet.",
        schedule_warning,
    });

    BermudanPutResult {
        workload: PricingWorkloadFamily::BermudanPut,
        price: core.price,
        stderr: core.stderr,
        paths: cfg.n_paths,
        steps: cfg.n_steps,
        seed: cfg.seed,
        exercise_schedule: core.exercise_schedule.clone(),
        exercise_date_count: core.exercise_schedule.len(),
        early_exercise_count: core.early_exercise_count,
        maturity_exercise_count: core.maturity_exercise_count,
        regression_steps: core.regression_steps,
        regression_basis: core.regression_basis,
        warnings: core.warnings,
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

pub fn european_call_greeks_cpu(
    cfg: &EuropeanCallConfig,
    estimator: GreekEstimator,
) -> GreekReport {
    match estimator {
        GreekEstimator::BumpAndRevalue => european_call_bump_greeks_cpu(cfg),
        GreekEstimator::Pathwise => european_call_pathwise_greeks_cpu(cfg),
        GreekEstimator::LikelihoodRatio => european_call_likelihood_ratio_greeks_cpu(cfg),
    }
}

pub fn arithmetic_asian_call_greeks_cpu(
    cfg: &ArithmeticAsianCallConfig,
    estimator: GreekEstimator,
) -> GreekReport {
    match estimator {
        GreekEstimator::BumpAndRevalue => {
            let base = arithmetic_asian_call_price_mc_cpu(cfg);
            let mut estimates = Vec::new();
            push_bump_estimate(
                &mut estimates,
                Greek::Delta,
                base,
                cfg.s0,
                bump_spot(cfg.s0),
                |mut c: ArithmeticAsianCallConfig, value| {
                    c.s0 = value;
                    c
                },
                *cfg,
                arithmetic_asian_call_price_mc_cpu,
                false,
                "central common-random-number finite difference on spot",
            );
            push_bump_estimate(
                &mut estimates,
                Greek::Vega,
                base,
                cfg.sigma,
                bump_volatility(cfg.sigma),
                |mut c: ArithmeticAsianCallConfig, value| {
                    c.sigma = value;
                    c
                },
                *cfg,
                arithmetic_asian_call_price_mc_cpu,
                false,
                "central common-random-number finite difference on volatility",
            );
            push_bump_estimate(
                &mut estimates,
                Greek::Rho,
                base,
                cfg.r,
                bump_rate(),
                |mut c: ArithmeticAsianCallConfig, value| {
                    c.r = value;
                    c
                },
                *cfg,
                arithmetic_asian_call_price_mc_cpu,
                false,
                "central common-random-number finite difference on rate",
            );
            push_bump_estimate(
                &mut estimates,
                Greek::Theta,
                base,
                cfg.t,
                bump_maturity(cfg.t),
                |mut c: ArithmeticAsianCallConfig, value| {
                    c.t = value;
                    c
                },
                *cfg,
                arithmetic_asian_call_price_mc_cpu,
                true,
                "conventional calendar theta from central maturity finite difference",
            );
            greek_report(
                PricingWorkloadFamily::ArithmeticAsianCall,
                estimator,
                cfg.n_paths,
                cfg.n_steps,
                cfg.seed,
                base,
                estimates,
                vec![
                    "Bump-and-revalue uses common seeds to reduce finite-difference noise"
                        .to_string(),
                ],
            )
        }
        GreekEstimator::Pathwise | GreekEstimator::LikelihoodRatio => unsupported_greek_report(
            PricingWorkloadFamily::ArithmeticAsianCall,
            estimator,
            cfg.n_paths,
            cfg.n_steps,
            cfg.seed,
            arithmetic_asian_call_price_mc_cpu(cfg),
            "Pathwise and likelihood-ratio estimators are not exposed for arithmetic Asian calls yet; use bump-and-revalue.",
        ),
    }
}

pub fn down_and_out_call_greeks_cpu(
    cfg: &DownAndOutCallConfig,
    estimator: GreekEstimator,
) -> GreekReport {
    match estimator {
        GreekEstimator::BumpAndRevalue => {
            let base = down_and_out_call_price_mc_cpu(cfg);
            let mut estimates = Vec::new();
            push_bump_estimate(
                &mut estimates,
                Greek::Delta,
                base,
                cfg.s0,
                bump_spot(cfg.s0),
                |mut c: DownAndOutCallConfig, value| {
                    c.s0 = value;
                    c
                },
                *cfg,
                down_and_out_call_price_mc_cpu,
                false,
                "central common-random-number finite difference on spot",
            );
            push_bump_estimate(
                &mut estimates,
                Greek::Vega,
                base,
                cfg.sigma,
                bump_volatility(cfg.sigma),
                |mut c: DownAndOutCallConfig, value| {
                    c.sigma = value;
                    c
                },
                *cfg,
                down_and_out_call_price_mc_cpu,
                false,
                "central common-random-number finite difference on volatility",
            );
            push_bump_estimate(
                &mut estimates,
                Greek::Rho,
                base,
                cfg.r,
                bump_rate(),
                |mut c: DownAndOutCallConfig, value| {
                    c.r = value;
                    c
                },
                *cfg,
                down_and_out_call_price_mc_cpu,
                false,
                "central common-random-number finite difference on rate",
            );
            push_bump_estimate(
                &mut estimates,
                Greek::Theta,
                base,
                cfg.t,
                bump_maturity(cfg.t),
                |mut c: DownAndOutCallConfig, value| {
                    c.t = value;
                    c
                },
                *cfg,
                down_and_out_call_price_mc_cpu,
                true,
                "conventional calendar theta from central maturity finite difference",
            );
            greek_report(
                PricingWorkloadFamily::DownAndOutCall,
                estimator,
                cfg.n_paths,
                cfg.n_steps,
                cfg.seed,
                base,
                estimates,
                vec!["Barrier discontinuities can make pathwise Greeks unstable; bump-and-revalue is the current supported estimator.".to_string()],
            )
        }
        GreekEstimator::Pathwise | GreekEstimator::LikelihoodRatio => unsupported_greek_report(
            PricingWorkloadFamily::DownAndOutCall,
            estimator,
            cfg.n_paths,
            cfg.n_steps,
            cfg.seed,
            down_and_out_call_price_mc_cpu(cfg),
            "Pathwise and likelihood-ratio estimators are not exposed for barrier calls because knock-out discontinuities need separate treatment.",
        ),
    }
}

pub fn lookback_call_greeks_cpu(
    cfg: &LookbackCallConfig,
    estimator: GreekEstimator,
) -> GreekReport {
    match estimator {
        GreekEstimator::BumpAndRevalue => {
            let base = lookback_call_price_mc_cpu(cfg);
            let mut estimates = Vec::new();
            push_bump_estimate(
                &mut estimates,
                Greek::Delta,
                base,
                cfg.s0,
                bump_spot(cfg.s0),
                |mut c: LookbackCallConfig, value| {
                    c.s0 = value;
                    c
                },
                *cfg,
                lookback_call_price_mc_cpu,
                false,
                "central common-random-number finite difference on spot",
            );
            push_bump_estimate(
                &mut estimates,
                Greek::Vega,
                base,
                cfg.sigma,
                bump_volatility(cfg.sigma),
                |mut c: LookbackCallConfig, value| {
                    c.sigma = value;
                    c
                },
                *cfg,
                lookback_call_price_mc_cpu,
                false,
                "central common-random-number finite difference on volatility",
            );
            push_bump_estimate(
                &mut estimates,
                Greek::Rho,
                base,
                cfg.r,
                bump_rate(),
                |mut c: LookbackCallConfig, value| {
                    c.r = value;
                    c
                },
                *cfg,
                lookback_call_price_mc_cpu,
                false,
                "central common-random-number finite difference on rate",
            );
            push_bump_estimate(
                &mut estimates,
                Greek::Theta,
                base,
                cfg.t,
                bump_maturity(cfg.t),
                |mut c: LookbackCallConfig, value| {
                    c.t = value;
                    c
                },
                *cfg,
                lookback_call_price_mc_cpu,
                true,
                "conventional calendar theta from central maturity finite difference",
            );
            greek_report(
                PricingWorkloadFamily::LookbackCall,
                estimator,
                cfg.n_paths,
                cfg.n_steps,
                cfg.seed,
                base,
                estimates,
                vec![
                    "Lookback Greeks are finite-difference estimates for the discretely monitored payoff.".to_string(),
                ],
            )
        }
        GreekEstimator::Pathwise | GreekEstimator::LikelihoodRatio => unsupported_greek_report(
            PricingWorkloadFamily::LookbackCall,
            estimator,
            cfg.n_paths,
            cfg.n_steps,
            cfg.seed,
            lookback_call_price_mc_cpu(cfg),
            "Pathwise and likelihood-ratio estimators are not exposed for lookback calls yet; use bump-and-revalue.",
        ),
    }
}

pub fn basket_call_greeks_cpu(cfg: &BasketCallConfig, estimator: GreekEstimator) -> GreekReport {
    match estimator {
        GreekEstimator::BumpAndRevalue => basket_call_bump_greeks_cpu(cfg),
        GreekEstimator::Pathwise | GreekEstimator::LikelihoodRatio => unsupported_greek_report(
            PricingWorkloadFamily::BasketCall,
            estimator,
            cfg.n_paths,
            2,
            cfg.seed,
            basket_call_price_mc_cpu(cfg),
            "Pathwise basket Greeks are mathematically valid for terminal baskets but not exposed yet; bump-and-revalue is the current supported basket estimator.",
        ),
    }
}

pub fn heston_european_call_greeks_cpu(
    cfg: &HestonEuropeanCallConfig,
    estimator: GreekEstimator,
) -> GreekReport {
    match estimator {
        GreekEstimator::BumpAndRevalue => heston_call_bump_greeks_cpu(cfg),
        GreekEstimator::Pathwise | GreekEstimator::LikelihoodRatio => unsupported_greek_report(
            PricingWorkloadFamily::HestonEuropeanCall,
            estimator,
            cfg.n_paths,
            cfg.n_steps,
            cfg.seed,
            heston_european_call_price_mc_cpu(cfg),
            "Pathwise and likelihood-ratio Heston Greeks require variance-path adjoints or score tracking and are not exposed yet; use bump-and-revalue.",
        ),
    }
}

pub fn price_all_current_greeks_bump_and_revalue_cpu(
    n_paths: usize,
    n_steps: usize,
    seed: u64,
) -> Vec<GreekReport> {
    let european = EuropeanCallConfig {
        n_paths,
        n_steps,
        seed,
        ..EuropeanCallConfig::default()
    };
    let asian = ArithmeticAsianCallConfig {
        n_paths,
        n_steps,
        seed: seed.wrapping_add(1),
        ..ArithmeticAsianCallConfig::default()
    };
    let barrier = DownAndOutCallConfig {
        n_paths,
        n_steps,
        seed: seed.wrapping_add(2),
        ..DownAndOutCallConfig::default()
    };
    let lookback = LookbackCallConfig {
        n_paths,
        n_steps,
        seed: seed.wrapping_add(3),
        ..LookbackCallConfig::default()
    };
    let basket = BasketCallConfig {
        n_paths,
        seed: seed.wrapping_add(4),
        ..BasketCallConfig::default()
    };
    let heston = HestonEuropeanCallConfig {
        n_paths,
        n_steps,
        seed: seed.wrapping_add(5),
        ..HestonEuropeanCallConfig::default()
    };

    vec![
        european_call_greeks_cpu(&european, GreekEstimator::BumpAndRevalue),
        arithmetic_asian_call_greeks_cpu(&asian, GreekEstimator::BumpAndRevalue),
        down_and_out_call_greeks_cpu(&barrier, GreekEstimator::BumpAndRevalue),
        lookback_call_greeks_cpu(&lookback, GreekEstimator::BumpAndRevalue),
        basket_call_greeks_cpu(&basket, GreekEstimator::BumpAndRevalue),
        heston_european_call_greeks_cpu(&heston, GreekEstimator::BumpAndRevalue),
    ]
}

fn european_call_bump_greeks_cpu(cfg: &EuropeanCallConfig) -> GreekReport {
    let base = european_call_price_mc_cpu(cfg);
    let mut estimates = Vec::new();
    push_bump_estimate(
        &mut estimates,
        Greek::Delta,
        base,
        cfg.s0,
        bump_spot(cfg.s0),
        |mut c: EuropeanCallConfig, value| {
            c.s0 = value;
            c
        },
        *cfg,
        european_call_price_mc_cpu,
        false,
        "central common-random-number finite difference on spot",
    );
    push_bump_estimate(
        &mut estimates,
        Greek::Vega,
        base,
        cfg.sigma,
        bump_volatility(cfg.sigma),
        |mut c: EuropeanCallConfig, value| {
            c.sigma = value;
            c
        },
        *cfg,
        european_call_price_mc_cpu,
        false,
        "central common-random-number finite difference on volatility",
    );
    push_bump_estimate(
        &mut estimates,
        Greek::Rho,
        base,
        cfg.r,
        bump_rate(),
        |mut c: EuropeanCallConfig, value| {
            c.r = value;
            c
        },
        *cfg,
        european_call_price_mc_cpu,
        false,
        "central common-random-number finite difference on rate",
    );
    push_bump_estimate(
        &mut estimates,
        Greek::Theta,
        base,
        cfg.t,
        bump_maturity(cfg.t),
        |mut c: EuropeanCallConfig, value| {
            c.t = value;
            c
        },
        *cfg,
        european_call_price_mc_cpu,
        true,
        "conventional calendar theta from central maturity finite difference",
    );
    greek_report(
        PricingWorkloadFamily::EuropeanCall,
        GreekEstimator::BumpAndRevalue,
        cfg.n_paths,
        cfg.n_steps,
        cfg.seed,
        base,
        estimates,
        vec!["Black-Scholes analytic Greeks are available for validation".to_string()],
    )
}

fn basket_call_bump_greeks_cpu(cfg: &BasketCallConfig) -> GreekReport {
    let base = basket_call_price_mc_cpu(cfg);
    let mut estimates = Vec::new();
    push_bump_estimate(
        &mut estimates,
        Greek::Delta,
        base,
        cfg.s01,
        bump_spot(cfg.s01),
        |mut c: BasketCallConfig, value| {
            c.s01 = value;
            c
        },
        *cfg,
        basket_call_price_mc_cpu,
        false,
        "central common-random-number finite difference on first spot",
    );
    push_bump_estimate(
        &mut estimates,
        Greek::Delta2,
        base,
        cfg.s02,
        bump_spot(cfg.s02),
        |mut c: BasketCallConfig, value| {
            c.s02 = value;
            c
        },
        *cfg,
        basket_call_price_mc_cpu,
        false,
        "central common-random-number finite difference on second spot",
    );
    push_bump_estimate(
        &mut estimates,
        Greek::Vega,
        base,
        cfg.sigma1,
        bump_volatility(cfg.sigma1),
        |mut c: BasketCallConfig, value| {
            c.sigma1 = value;
            c
        },
        *cfg,
        basket_call_price_mc_cpu,
        false,
        "central common-random-number finite difference on first volatility",
    );
    push_bump_estimate(
        &mut estimates,
        Greek::Vega2,
        base,
        cfg.sigma2,
        bump_volatility(cfg.sigma2),
        |mut c: BasketCallConfig, value| {
            c.sigma2 = value;
            c
        },
        *cfg,
        basket_call_price_mc_cpu,
        false,
        "central common-random-number finite difference on second volatility",
    );
    push_bump_estimate(
        &mut estimates,
        Greek::Rho,
        base,
        cfg.r,
        bump_rate(),
        |mut c: BasketCallConfig, value| {
            c.r = value;
            c
        },
        *cfg,
        basket_call_price_mc_cpu,
        false,
        "central common-random-number finite difference on rate",
    );
    push_bump_estimate(
        &mut estimates,
        Greek::Theta,
        base,
        cfg.t,
        bump_maturity(cfg.t),
        |mut c: BasketCallConfig, value| {
            c.t = value;
            c
        },
        *cfg,
        basket_call_price_mc_cpu,
        true,
        "conventional calendar theta from central maturity finite difference",
    );
    greek_report(
        PricingWorkloadFamily::BasketCall,
        GreekEstimator::BumpAndRevalue,
        cfg.n_paths,
        2,
        cfg.seed,
        base,
        estimates,
        vec![
            "Basket Greeks use Delta/Delta2 and Vega/Vega2 for the first and second assets."
                .to_string(),
        ],
    )
}

fn heston_call_bump_greeks_cpu(cfg: &HestonEuropeanCallConfig) -> GreekReport {
    let base = heston_european_call_price_mc_cpu(cfg);
    let mut estimates = Vec::new();
    push_bump_estimate(
        &mut estimates,
        Greek::Delta,
        base,
        cfg.s0,
        bump_spot(cfg.s0),
        |mut c: HestonEuropeanCallConfig, value| {
            c.s0 = value;
            c
        },
        *cfg,
        heston_european_call_price_mc_cpu,
        false,
        "central common-random-number finite difference on spot",
    );
    let vol_bump = bump_volatility(cfg.v0.max(0.0).sqrt());
    push_bump_estimate(
        &mut estimates,
        Greek::Vega,
        base,
        cfg.v0.max(0.0).sqrt(),
        vol_bump,
        |mut c: HestonEuropeanCallConfig, value| {
            c.v0 = value.max(0.0) * value.max(0.0);
            c
        },
        *cfg,
        heston_european_call_price_mc_cpu,
        false,
        "central finite difference on sqrt(v0); theta is held fixed",
    );
    push_bump_estimate(
        &mut estimates,
        Greek::Rho,
        base,
        cfg.r,
        bump_rate(),
        |mut c: HestonEuropeanCallConfig, value| {
            c.r = value;
            c
        },
        *cfg,
        heston_european_call_price_mc_cpu,
        false,
        "central common-random-number finite difference on rate",
    );
    push_bump_estimate(
        &mut estimates,
        Greek::Theta,
        base,
        cfg.t,
        bump_maturity(cfg.t),
        |mut c: HestonEuropeanCallConfig, value| {
            c.t = value;
            c
        },
        *cfg,
        heston_european_call_price_mc_cpu,
        true,
        "conventional calendar theta from central maturity finite difference",
    );
    greek_report(
        PricingWorkloadFamily::HestonEuropeanCall,
        GreekEstimator::BumpAndRevalue,
        cfg.n_paths,
        cfg.n_steps,
        cfg.seed,
        base,
        estimates,
        vec![
            "Heston Vega is reported as sensitivity to sqrt(v0) with theta held fixed.".to_string(),
        ],
    )
}

fn european_call_pathwise_greeks_cpu(cfg: &EuropeanCallConfig) -> GreekReport {
    assert!(cfg.n_paths > 0, "n_paths must be > 0");
    assert!(cfg.s0 > 0.0, "s0 must be > 0");
    assert!(cfg.sigma > 0.0, "sigma must be > 0 for pathwise Greeks");
    assert!(cfg.t > 0.0, "t must be > 0");

    let base = european_call_price_mc_cpu(cfg);
    let mut rng = MonteCarloRng::new(cfg.seed);
    let sqrt_t = cfg.t.sqrt();
    let drift_t = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * cfg.t;
    let vol_t = cfg.sigma * sqrt_t;
    let discount = (-cfg.r * cfg.t).exp();
    let mut delta = MomentAccumulator::default();
    let mut vega = MomentAccumulator::default();
    let mut rho = MomentAccumulator::default();
    let mut theta = MomentAccumulator::default();

    for _ in 0..cfg.n_paths {
        let z = rng.standard_normal();
        let st = cfg.s0 * (drift_t + vol_t * z).exp();
        let intrinsic = st - cfg.k;
        let active = intrinsic > 0.0;
        let payoff = intrinsic.max(0.0);
        let discounted_payoff = discount * payoff;

        delta.record(if active { discount * st / cfg.s0 } else { 0.0 });
        vega.record(if active {
            discount * st * (-cfg.sigma * cfg.t + sqrt_t * z)
        } else {
            0.0
        });
        rho.record(if active {
            discount * (cfg.t * st - cfg.t * payoff)
        } else {
            -cfg.t * discounted_payoff
        });
        let dst_dt = st * (cfg.r - 0.5 * cfg.sigma * cfg.sigma + cfg.sigma * z / (2.0 * sqrt_t));
        let d_price_d_maturity = if active {
            discount * (dst_dt - cfg.r * payoff)
        } else {
            -cfg.r * discounted_payoff
        };
        theta.record(-d_price_d_maturity);
    }

    let estimates = vec![
        single_pass_estimate(
            Greek::Delta,
            GreekEstimator::Pathwise,
            base,
            delta,
            "pathwise terminal-GBM Delta",
        ),
        single_pass_estimate(
            Greek::Vega,
            GreekEstimator::Pathwise,
            base,
            vega,
            "pathwise terminal-GBM Vega",
        ),
        single_pass_estimate(
            Greek::Rho,
            GreekEstimator::Pathwise,
            base,
            rho,
            "pathwise terminal-GBM Rho",
        ),
        single_pass_estimate(
            Greek::Theta,
            GreekEstimator::Pathwise,
            base,
            theta,
            "pathwise terminal-GBM conventional calendar Theta",
        ),
    ];
    let mut warnings = vec![
        "Pathwise Greeks are exposed only for the smooth terminal GBM European-call path; the payoff kink is handled almost surely.".to_string(),
    ];
    if cfg.sampling != SamplingMethod::Pseudorandom {
        warnings.push("Pathwise Greek execution currently uses pseudorandom terminal shocks; structured-sampling Greek estimators are not exposed yet.".to_string());
    }
    if cfg.technique != MonteCarloTechnique::Standard {
        warnings.push("Pathwise Greek execution estimates raw single-pass Greeks and does not apply the configured variance-reduction technique.".to_string());
    }

    greek_report(
        PricingWorkloadFamily::EuropeanCall,
        GreekEstimator::Pathwise,
        cfg.n_paths,
        cfg.n_steps,
        cfg.seed,
        base,
        estimates,
        warnings,
    )
}

fn european_call_likelihood_ratio_greeks_cpu(cfg: &EuropeanCallConfig) -> GreekReport {
    assert!(cfg.n_paths > 0, "n_paths must be > 0");
    assert!(cfg.s0 > 0.0, "s0 must be > 0");
    assert!(
        cfg.sigma > 0.0,
        "sigma must be > 0 for likelihood-ratio Greeks"
    );
    assert!(cfg.t > 0.0, "t must be > 0");

    let base = european_call_price_mc_cpu(cfg);
    let mut rng = MonteCarloRng::new(cfg.seed);
    let sqrt_t = cfg.t.sqrt();
    let drift_t = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * cfg.t;
    let vol_t = cfg.sigma * sqrt_t;
    let discount = (-cfg.r * cfg.t).exp();
    let mut delta = MomentAccumulator::default();
    let mut vega = MomentAccumulator::default();
    let mut rho = MomentAccumulator::default();

    for _ in 0..cfg.n_paths {
        let z = rng.standard_normal();
        let st = cfg.s0 * (drift_t + vol_t * z).exp();
        let discounted_payoff = discount * (st - cfg.k).max(0.0);
        delta.record(discounted_payoff * z / (cfg.s0 * cfg.sigma * sqrt_t));
        let vega_score = (z * z - 1.0) / cfg.sigma - sqrt_t * z;
        vega.record(discounted_payoff * vega_score);
        rho.record(discounted_payoff * (z / (cfg.sigma * sqrt_t) - cfg.t));
    }

    let estimates = vec![
        single_pass_estimate(
            Greek::Delta,
            GreekEstimator::LikelihoodRatio,
            base,
            delta,
            "likelihood-ratio terminal-GBM Delta",
        ),
        single_pass_estimate(
            Greek::Vega,
            GreekEstimator::LikelihoodRatio,
            base,
            vega,
            "likelihood-ratio terminal-GBM Vega",
        ),
        single_pass_estimate(
            Greek::Rho,
            GreekEstimator::LikelihoodRatio,
            base,
            rho,
            "likelihood-ratio terminal-GBM Rho",
        ),
    ];
    let mut warnings = vec![
        "Likelihood-ratio estimator currently exposes Delta, Vega, and Rho for terminal GBM European calls.".to_string(),
        "Delta is generally the most useful LR Greek here; pathwise Delta usually has lower variance for vanilla calls.".to_string(),
    ];
    if cfg.sampling != SamplingMethod::Pseudorandom {
        warnings.push("Likelihood-ratio Greek execution currently uses pseudorandom terminal shocks; structured-sampling LR estimators are not exposed yet.".to_string());
    }
    if cfg.technique != MonteCarloTechnique::Standard {
        warnings.push("Likelihood-ratio Greek execution estimates raw single-pass Greeks and does not apply the configured variance-reduction technique.".to_string());
    }

    greek_report(
        PricingWorkloadFamily::EuropeanCall,
        GreekEstimator::LikelihoodRatio,
        cfg.n_paths,
        cfg.n_steps,
        cfg.seed,
        base,
        estimates,
        warnings,
    )
}

#[derive(Debug, Clone, Copy, Default)]
struct MomentAccumulator {
    sum: f64,
    sq_sum: f64,
    count: usize,
}

impl MomentAccumulator {
    fn record(&mut self, value: f64) {
        self.sum += value;
        self.sq_sum += value * value;
        self.count += 1;
    }

    fn mean_stderr(self) -> (f64, f64) {
        if self.count == 0 {
            return (0.0, 0.0);
        }
        let n = self.count as f64;
        let mean = self.sum / n;
        let variance = if self.count > 1 {
            ((self.sq_sum - self.sum * self.sum / n) / (n - 1.0)).max(0.0)
        } else {
            0.0
        };
        (mean, (variance / n).sqrt())
    }
}

fn single_pass_estimate(
    greek: Greek,
    estimator: GreekEstimator,
    base: EuropeanCallResult,
    moments: MomentAccumulator,
    note: &str,
) -> GreekEstimate {
    let (value, stderr) = moments.mean_stderr();
    GreekEstimate {
        greek,
        estimator,
        value,
        stderr: Some(stderr),
        bump_size: None,
        base_price: base.price,
        bumped_up_price: None,
        bumped_down_price: None,
        stderr_estimate_kind: "sample standard error of single-pass Greek estimator".to_string(),
        notes: vec![note.to_string()],
    }
}

fn greek_report(
    workload: PricingWorkloadFamily,
    estimator: GreekEstimator,
    paths: usize,
    steps: usize,
    seed: u64,
    base: EuropeanCallResult,
    estimates: Vec<GreekEstimate>,
    warnings: Vec<String>,
) -> GreekReport {
    GreekReport {
        workload,
        estimator,
        paths,
        steps,
        seed,
        base_price: base.price,
        base_stderr: base.stderr,
        estimated_runtime_ms: None,
        estimates,
        warnings,
    }
}

fn unsupported_greek_report(
    workload: PricingWorkloadFamily,
    estimator: GreekEstimator,
    paths: usize,
    steps: usize,
    seed: u64,
    base: EuropeanCallResult,
    warning: &str,
) -> GreekReport {
    greek_report(
        workload,
        estimator,
        paths,
        steps,
        seed,
        base,
        Vec::new(),
        vec![warning.to_string()],
    )
}

fn push_bump_estimate<C, F, P>(
    estimates: &mut Vec<GreekEstimate>,
    greek: Greek,
    base: EuropeanCallResult,
    center: f64,
    bump: f64,
    set_parameter: F,
    cfg: C,
    price: P,
    reverse_sign: bool,
    note: &str,
) where
    C: Copy,
    F: Fn(C, f64) -> C,
    P: Fn(&C) -> EuropeanCallResult,
{
    let up_cfg = set_parameter(cfg, center + bump);
    let down_cfg = set_parameter(cfg, (center - bump).max(parameter_floor(greek)));
    let up = price(&up_cfg);
    let down = price(&down_cfg);
    let effective_bump = ((center + bump) - (center - bump).max(parameter_floor(greek))) * 0.5;
    let raw_value = if effective_bump > 0.0 {
        (up.price - down.price) / (2.0 * effective_bump)
    } else {
        0.0
    };
    let value = if reverse_sign { -raw_value } else { raw_value };
    let stderr = if effective_bump > 0.0 {
        Some((up.stderr * up.stderr + down.stderr * down.stderr).sqrt() / (2.0 * effective_bump))
    } else {
        Some(0.0)
    };

    estimates.push(GreekEstimate {
        greek,
        estimator: GreekEstimator::BumpAndRevalue,
        value,
        stderr,
        bump_size: Some(effective_bump),
        base_price: base.price,
        bumped_up_price: Some(up.price),
        bumped_down_price: Some(down.price),
        stderr_estimate_kind: "conservative independent-leg standard-error propagation; common random numbers usually reduce actual noise".to_string(),
        notes: vec![note.to_string()],
    });
}

fn parameter_floor(greek: Greek) -> f64 {
    match greek {
        Greek::Delta | Greek::Delta2 | Greek::Vega | Greek::Vega2 | Greek::Theta => {
            f64::MIN_POSITIVE
        }
        Greek::Rho => -1.0,
    }
}

fn bump_spot(spot: f64) -> f64 {
    (spot.abs() * 0.01).max(0.01)
}

fn bump_volatility(volatility: f64) -> f64 {
    (volatility.abs() * 0.05).max(0.005)
}

fn bump_rate() -> f64 {
    0.001
}

fn bump_maturity(t: f64) -> f64 {
    (1.0_f64 / 365.0).min(t * 0.25)
}

fn validate_european_call_config(cfg: &EuropeanCallConfig) {
    assert!(cfg.n_paths > 0, "n_paths must be > 0");
    assert!(cfg.n_steps > 0, "n_steps must be > 0");
    assert!(cfg.s0 > 0.0, "s0 must be > 0");
    assert!(cfg.k > 0.0, "k must be > 0");
    assert!(cfg.sigma > 0.0, "sigma must be > 0");
    assert!(cfg.t > 0.0, "t must be > 0");
}

fn validate_european_call_parameter_sweep_config(cfg: &EuropeanCallParameterSweepConfig) {
    validate_european_call_config(&cfg.base_config);
    assert!(
        !cfg.scenarios.is_empty(),
        "parameter sweep must contain at least one scenario"
    );

    for (idx, scenario) in cfg.scenarios.iter().enumerate() {
        assert!(
            !scenario.scenario_id.trim().is_empty(),
            "scenario_id must not be empty"
        );
        assert!(
            !cfg.scenarios[..idx]
                .iter()
                .any(|seen| seen.scenario_id == scenario.scenario_id),
            "scenario_id values must be unique"
        );

        let scenario_cfg =
            apply_european_call_sweep_scenario(cfg.base_config, scenario, idx, cfg.seed_stride);
        validate_european_call_config(&scenario_cfg);
    }
}

fn apply_european_call_sweep_scenario(
    mut base: EuropeanCallConfig,
    scenario: &EuropeanCallSweepScenario,
    scenario_index: usize,
    seed_stride: u64,
) -> EuropeanCallConfig {
    if let Some(value) = scenario.s0 {
        base.s0 = value;
    }
    if let Some(value) = scenario.k {
        base.k = value;
    }
    if let Some(value) = scenario.r {
        base.r = value;
    }
    if let Some(value) = scenario.sigma {
        base.sigma = value;
    }
    if let Some(value) = scenario.t {
        base.t = value;
    }
    if let Some(value) = scenario.n_paths {
        base.n_paths = value;
    }
    if let Some(value) = scenario.n_steps {
        base.n_steps = value;
    }
    base.seed = scenario.seed.unwrap_or_else(|| {
        base.seed
            .wrapping_add(seed_stride.wrapping_mul(scenario_index as u64))
    });
    if let Some(value) = scenario.sampling {
        base.sampling = value;
    }
    if let Some(value) = scenario.technique {
        base.technique = value;
    }
    base
}

fn validate_lookback_call_config(cfg: &LookbackCallConfig) {
    assert!(cfg.n_paths > 0, "n_paths must be > 0");
    assert!(cfg.n_steps > 0, "n_steps must be > 0");
    assert!(cfg.s0 > 0.0, "s0 must be > 0");
    assert!(cfg.k >= 0.0, "k must be >= 0");
    assert!(cfg.sigma >= 0.0, "sigma must be >= 0");
    assert!(cfg.t > 0.0, "t must be > 0");
}

fn validate_heston_european_call_config(cfg: &HestonEuropeanCallConfig) {
    assert!(cfg.n_paths > 0, "n_paths must be > 0");
    assert!(cfg.n_steps > 0, "n_steps must be > 0");
    assert!(cfg.s0 > 0.0, "s0 must be > 0");
    assert!(cfg.k >= 0.0, "k must be >= 0");
    assert!(cfg.v0 >= 0.0, "v0 must be >= 0");
    assert!(cfg.kappa >= 0.0, "kappa must be >= 0");
    assert!(cfg.theta >= 0.0, "theta must be >= 0");
    assert!(cfg.vol_of_vol >= 0.0, "vol_of_vol must be >= 0");
    assert!((-1.0..=1.0).contains(&cfg.rho), "rho must be in [-1, 1]");
    assert!(cfg.t > 0.0, "t must be > 0");
}

fn validate_merton_jump_diffusion_call_config(cfg: &MertonJumpDiffusionCallConfig) {
    assert!(cfg.n_paths > 0, "n_paths must be > 0");
    assert!(cfg.s0 > 0.0, "s0 must be > 0");
    assert!(cfg.k > 0.0, "k must be > 0");
    assert!(cfg.sigma >= 0.0, "sigma must be >= 0");
    assert!(cfg.jump_intensity >= 0.0, "jump_intensity must be >= 0");
    assert!(cfg.jump_volatility >= 0.0, "jump_volatility must be >= 0");
    assert!(cfg.t > 0.0, "t must be > 0");
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

fn validate_american_put_config(cfg: &AmericanPutConfig) {
    assert!(cfg.n_paths > 0, "n_paths must be > 0");
    assert!(cfg.n_steps >= 2, "n_steps must be >= 2 for LSM regression");
    assert!(cfg.s0 > 0.0, "s0 must be > 0");
    assert!(cfg.k > 0.0, "k must be > 0");
    assert!(cfg.sigma >= 0.0, "sigma must be >= 0");
    assert!(cfg.t > 0.0, "t must be > 0");
    assert!(cfg.basis_degree > 0, "basis_degree must be > 0");
}

fn validate_bermudan_put_config(cfg: &BermudanPutConfig) {
    assert!(cfg.n_paths > 0, "n_paths must be > 0");
    assert!(cfg.n_steps >= 2, "n_steps must be >= 2 for LSM regression");
    assert!(cfg.s0 > 0.0, "s0 must be > 0");
    assert!(cfg.k > 0.0, "k must be > 0");
    assert!(cfg.sigma >= 0.0, "sigma must be >= 0");
    assert!(cfg.t > 0.0, "t must be > 0");
    assert!(cfg.basis_degree > 0, "basis_degree must be > 0");
    assert!(
        !cfg.exercise_steps.is_empty(),
        "exercise_steps must include at least one exercise date"
    );
    for &step in &cfg.exercise_steps {
        assert!(step > 0, "exercise_steps must be in 1..=n_steps");
        assert!(step <= cfg.n_steps, "exercise_steps must be in 1..=n_steps");
    }
}

fn normalize_bermudan_exercise_schedule(
    exercise_steps: &[usize],
    n_steps: usize,
) -> (Vec<usize>, Option<String>) {
    let mut schedule = exercise_steps.to_vec();
    schedule.sort_unstable();
    schedule.dedup();

    let mut warning = None;
    if schedule != exercise_steps {
        warning =
            Some("exercise_steps were sorted and deduplicated before LSM execution.".to_string());
    }

    if !schedule.contains(&n_steps) {
        schedule.push(n_steps);
        if let Some(existing) = &mut warning {
            existing
                .push_str(" Maturity was appended because terminal payoff is always evaluated.");
        } else {
            warning = Some(
                "Maturity was appended to exercise_steps because terminal payoff is always evaluated."
                    .to_string(),
            );
        }
    }

    (schedule, warning)
}

fn map_exercise_schedule_to_reference_steps(
    exercise_steps: &[usize],
    simulation_steps: usize,
    reference_steps: usize,
) -> Vec<usize> {
    let mut schedule = exercise_steps
        .iter()
        .map(|&step| {
            ((step as f64 / simulation_steps as f64) * reference_steps as f64).round() as usize
        })
        .map(|step| step.clamp(1, reference_steps))
        .collect::<Vec<_>>();
    schedule.push(reference_steps);
    schedule.sort_unstable();
    schedule.dedup();
    schedule
}

fn build_early_exercise_reference_comparison(
    workload: PricingWorkloadFamily,
    reference_name: &str,
    paths: usize,
    steps: usize,
    seed: u64,
    reference_steps: usize,
    exercise_schedule: Vec<usize>,
    reference_price: f64,
    simulated_price: f64,
    stderr: f64,
    warnings: Vec<String>,
) -> EarlyExerciseReferenceComparison {
    let error = simulated_price - reference_price;
    let abs_error = error.abs();
    let error_stderr_units = if stderr > 0.0 {
        error / stderr
    } else if error == 0.0 {
        0.0
    } else {
        f64::INFINITY.copysign(error)
    };

    EarlyExerciseReferenceComparison {
        workload,
        reference_name: reference_name.to_string(),
        paths,
        steps,
        seed,
        reference_steps,
        exercise_schedule,
        reference_price,
        simulated_price,
        stderr,
        error,
        abs_error,
        error_stderr_units,
        warnings,
    }
}

fn binomial_put_reference_price(
    s0: f64,
    k: f64,
    r: f64,
    sigma: f64,
    t: f64,
    reference_steps: usize,
    exercise_steps: &[usize],
) -> f64 {
    if sigma == 0.0 {
        return deterministic_put_exercise_reference_price(
            s0,
            k,
            r,
            t,
            reference_steps,
            exercise_steps,
        );
    }

    let dt = t / reference_steps as f64;
    let u = (sigma * dt.sqrt()).exp();
    let d = 1.0 / u;
    let growth = (r * dt).exp();
    let p = (growth - d) / (u - d);
    assert!(
        p.is_finite() && (0.0..=1.0).contains(&p),
        "CRR risk-neutral probability must be finite and within [0, 1]"
    );
    let discount = (-r * dt).exp();
    let mut values = vec![0.0; reference_steps + 1];
    let mut exercise_mask = vec![false; reference_steps + 1];

    for &step in exercise_steps {
        if step <= reference_steps {
            exercise_mask[step] = true;
        }
    }
    exercise_mask[reference_steps] = true;

    for (down_moves, value) in values.iter_mut().enumerate().take(reference_steps + 1) {
        let up_moves = reference_steps - down_moves;
        let s_t = s0 * u.powi(up_moves as i32) * d.powi(down_moves as i32);
        *value = (k - s_t).max(0.0);
    }

    for step in (0..reference_steps).rev() {
        for down_moves in 0..=step {
            let continuation =
                discount * (p * values[down_moves] + (1.0 - p) * values[down_moves + 1]);
            if exercise_mask[step] {
                let up_moves = step - down_moves;
                let s_t = s0 * u.powi(up_moves as i32) * d.powi(down_moves as i32);
                values[down_moves] = continuation.max(k - s_t);
            } else {
                values[down_moves] = continuation;
            }
        }
    }

    values[0]
}

fn deterministic_put_exercise_reference_price(
    s0: f64,
    k: f64,
    r: f64,
    t: f64,
    reference_steps: usize,
    exercise_steps: &[usize],
) -> f64 {
    exercise_steps
        .iter()
        .copied()
        .filter(|&step| step <= reference_steps)
        .map(|step| {
            let time = t * step as f64 / reference_steps as f64;
            let s_t = s0 * (r * time).exp();
            (-r * time).exp() * (k - s_t).max(0.0)
        })
        .fold(0.0, f64::max)
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
pub(crate) fn lookback_call_price_mc_stepwise_from_f32_normals(
    cfg: &LookbackCallConfig,
    normals: &[f32],
) -> LookbackCallResult {
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
        let mut max_s = cfg.s0 as f32;
        let base_offset = path_idx * cfg.n_steps;
        for step_idx in 0..cfg.n_steps {
            let z = normals[base_offset + step_idx];
            log_s_t += drift_dt + vol_dt * z;
            max_s = max_s.max(log_s_t.exp());
        }

        let payoff = ((max_s - strike).max(0.0) * discount) as f64;
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

fn simulate_merton_jump_diffusion_parallel(
    cfg: &MertonJumpDiffusionCallConfig,
    thread_count: usize,
    drift_t: f64,
    diffusion_vol_t: f64,
    jump_mean_t: f64,
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
        let jump_mean = cfg.jump_mean;
        let jump_volatility = cfg.jump_volatility;
        handles.push(thread::spawn(move || {
            simulate_merton_jump_diffusion_chunk(
                seed,
                n_paths_chunk,
                s0,
                k,
                drift_t,
                diffusion_vol_t,
                jump_mean_t,
                jump_mean,
                jump_volatility,
                discount,
            )
        }));
    }

    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;
    for handle in handles {
        let (chunk_sum, chunk_sq_sum) = handle
            .join()
            .expect("CPU Merton jump-diffusion worker thread panicked");
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

fn simulate_merton_jump_diffusion_chunk(
    seed: u64,
    n_paths: usize,
    s0: f64,
    k: f64,
    drift_t: f64,
    diffusion_vol_t: f64,
    jump_mean_t: f64,
    jump_mean: f64,
    jump_volatility: f64,
    discount: f64,
) -> (f64, f64) {
    let mut rng = MonteCarloRng::new(seed);
    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;
    let log_s0 = s0.ln();

    for _ in 0..n_paths {
        let diffusion_shock = diffusion_vol_t * rng.standard_normal();
        let jump_count = sample_poisson(&mut rng, jump_mean_t);
        let jump_shock = if jump_count == 0 {
            0.0
        } else {
            jump_count as f64 * jump_mean
                + jump_volatility * (jump_count as f64).sqrt() * rng.standard_normal()
        };
        let s_t = (log_s0 + drift_t + diffusion_shock + jump_shock).exp();
        let payoff = (s_t - k).max(0.0) * discount;
        payoff_sum += payoff;
        payoff_sq_sum += payoff * payoff;
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

fn simulate_lookback_stepwise_qmc(cfg: &LookbackCallConfig) -> LookbackCallResult {
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
                let mut max_s = cfg.s0;
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
                    max_s = max_s.max(log_s_t.exp());
                }
                let payoff = (max_s - cfg.k).max(0.0) * discount;
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
                let mut max_a = cfg.s0;
                let mut max_b = cfg.s0;
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
                    max_a = max_a.max(log_a.exp());
                    max_b = max_b.max(log_b.exp());
                }
                let payoff_a = (max_a - cfg.k).max(0.0) * discount;
                let payoff_b = (max_b - cfg.k).max(0.0) * discount;
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
                let mut max_s = cfg.s0;
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
                    max_s = max_s.max(log_s_t.exp());
                }
                moments.record(
                    (max_s - cfg.k).max(0.0) * discount,
                    discount * log_s_t.exp(),
                );
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

fn laguerre_lsm_basis(x: f64, degree: usize) -> [f64; 4] {
    let x2 = x * x;
    let x3 = x2 * x;
    let mut basis = [0.0; 4];
    basis[0] = 1.0;
    if degree >= 1 {
        basis[1] = 1.0 - x;
    }
    if degree >= 2 {
        basis[2] = 1.0 - 2.0 * x + 0.5 * x2;
    }
    if degree >= 3 {
        basis[3] = 1.0 - 3.0 * x + 1.5 * x2 - x3 / 6.0;
    }
    basis
}

fn solve_lsm_normal_equations(
    mut matrix: [[f64; 4]; 4],
    mut rhs: [f64; 4],
    dimension: usize,
) -> Option<[f64; 4]> {
    for pivot_idx in 0..dimension {
        let mut best_row = pivot_idx;
        let mut best_abs = matrix[pivot_idx][pivot_idx].abs();
        for (row_idx, row) in matrix
            .iter()
            .enumerate()
            .take(dimension)
            .skip(pivot_idx + 1)
        {
            let candidate = row[pivot_idx].abs();
            if candidate > best_abs {
                best_row = row_idx;
                best_abs = candidate;
            }
        }

        if best_abs <= 1e-12 {
            return None;
        }

        if best_row != pivot_idx {
            matrix.swap(pivot_idx, best_row);
            rhs.swap(pivot_idx, best_row);
        }

        let pivot = matrix[pivot_idx][pivot_idx];
        for col_idx in pivot_idx..dimension {
            matrix[pivot_idx][col_idx] /= pivot;
        }
        rhs[pivot_idx] /= pivot;

        for row_idx in 0..dimension {
            if row_idx == pivot_idx {
                continue;
            }
            let factor = matrix[row_idx][pivot_idx];
            if factor == 0.0 {
                continue;
            }
            for col_idx in pivot_idx..dimension {
                matrix[row_idx][col_idx] -= factor * matrix[pivot_idx][col_idx];
            }
            rhs[row_idx] -= factor * rhs[pivot_idx];
        }
    }

    Some(rhs)
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

fn simulate_lookback_stepwise_chunk(
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
        let mut max_s = s0;
        for _ in 0..n_steps {
            let z = rng.standard_normal();
            log_s_t += drift_dt + vol_dt * z;
            max_s = max_s.max(log_s_t.exp());
        }

        let payoff = (max_s - k).max(0.0) * discount;
        payoff_sum += payoff;
        payoff_sq_sum += payoff * payoff;
    }

    (payoff_sum, payoff_sq_sum)
}

fn simulate_lookback_stepwise_antithetic_chunk(
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
        let mut max_a = s0;
        let mut max_b = s0;
        for _ in 0..n_steps {
            let z = rng.standard_normal();
            log_a += drift_dt + vol_dt * z;
            log_b += drift_dt - vol_dt * z;
            max_a = max_a.max(log_a.exp());
            max_b = max_b.max(log_b.exp());
        }

        let payoff_a = (max_a - k).max(0.0) * discount;
        let payoff_b = (max_b - k).max(0.0) * discount;
        let block_estimate = 0.5 * (payoff_a + payoff_b);
        block_sum += block_estimate;
        block_sq_sum += block_estimate * block_estimate;
    }

    (block_sum, block_sq_sum)
}

fn simulate_lookback_stepwise_control_variate_chunk(
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
        let mut max_s = s0;
        for _ in 0..n_steps {
            let z = rng.standard_normal();
            log_s_t += drift_dt + vol_dt * z;
            max_s = max_s.max(log_s_t.exp());
        }

        moments.record((max_s - k).max(0.0) * discount, discount * log_s_t.exp());
    }

    moments
}

fn simulate_lookback_stepwise_parallel(
    cfg: &LookbackCallConfig,
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
            simulate_lookback_stepwise_chunk(
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

fn simulate_lookback_stepwise_antithetic_parallel(
    cfg: &LookbackCallConfig,
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
        let n_steps = cfg.n_steps;
        handles.push(thread::spawn(move || {
            simulate_lookback_stepwise_antithetic_chunk(
                seed, pair_chunk, n_steps, s0, k, drift_dt, vol_dt, discount,
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

fn simulate_lookback_stepwise_control_variate_parallel(
    cfg: &LookbackCallConfig,
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
            simulate_lookback_stepwise_control_variate_chunk(
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

fn lookback_call_price_mc_stepwise_standard(cfg: &LookbackCallConfig) -> LookbackCallResult {
    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let thread_count = resolved_thread_count(cfg.n_threads);

    let (payoff_sum, payoff_sq_sum) = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_lookback_stepwise_chunk(
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
        simulate_lookback_stepwise_parallel(cfg, thread_count, drift_dt, vol_dt, discount)
    };

    summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
}

fn lookback_call_price_mc_stepwise_antithetic(cfg: &LookbackCallConfig) -> LookbackCallResult {
    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let pair_count = cfg.n_paths.div_ceil(2);
    let thread_count = resolved_thread_count(cfg.n_threads);

    let (block_sum, block_sq_sum) = if thread_count <= 1 || pair_count < thread_count * 2_000 {
        simulate_lookback_stepwise_antithetic_chunk(
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
        simulate_lookback_stepwise_antithetic_parallel(
            cfg,
            thread_count,
            drift_dt,
            vol_dt,
            discount,
        )
    };

    summarize_block_estimates(pair_count, block_sum, block_sq_sum)
}

fn lookback_call_price_mc_stepwise_control_variate(cfg: &LookbackCallConfig) -> LookbackCallResult {
    let dt = cfg.t / cfg.n_steps as f64;
    let drift_dt = (cfg.r - 0.5 * cfg.sigma * cfg.sigma) * dt;
    let vol_dt = cfg.sigma * dt.sqrt();
    let discount = (-cfg.r * cfg.t).exp();
    let thread_count = resolved_thread_count(cfg.n_threads);

    let moments = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_lookback_stepwise_control_variate_chunk(
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
        simulate_lookback_stepwise_control_variate_parallel(
            cfg,
            thread_count,
            drift_dt,
            vol_dt,
            discount,
        )
    };

    summarize_control_variate(moments, cfg.s0)
}

#[derive(Debug, Clone, Copy)]
struct HestonStepParams {
    log_s0: f64,
    strike: f64,
    v0: f64,
    kappa: f64,
    theta: f64,
    vol_of_vol: f64,
    rho: f64,
    rho_perp: f64,
    dt: f64,
    sqrt_dt: f64,
    n_steps: usize,
    rate: f64,
    discount: f64,
}

fn heston_params(cfg: &HestonEuropeanCallConfig) -> HestonStepParams {
    HestonStepParams {
        log_s0: cfg.s0.ln(),
        strike: cfg.k,
        v0: cfg.v0,
        kappa: cfg.kappa,
        theta: cfg.theta,
        vol_of_vol: cfg.vol_of_vol,
        rho: cfg.rho,
        rho_perp: (1.0 - cfg.rho * cfg.rho).max(0.0).sqrt(),
        dt: cfg.t / cfg.n_steps as f64,
        sqrt_dt: (cfg.t / cfg.n_steps as f64).sqrt(),
        n_steps: cfg.n_steps,
        rate: cfg.r,
        discount: (-cfg.r * cfg.t).exp(),
    }
}

fn heston_path_terminal(params: HestonStepParams, rng: &mut MonteCarloRng, sign: f64) -> f64 {
    let mut log_s_t = params.log_s0;
    let mut variance = params.v0;

    for _ in 0..params.n_steps {
        let z_var = sign * rng.standard_normal();
        let z_independent = sign * rng.standard_normal();
        let variance_pos = variance.max(0.0);
        let z_price = params.rho * z_var + params.rho_perp * z_independent;

        log_s_t += (params.rate - 0.5 * variance_pos) * params.dt
            + variance_pos.sqrt() * params.sqrt_dt * z_price;
        variance = variance
            + params.kappa * (params.theta - variance_pos) * params.dt
            + params.vol_of_vol * variance_pos.sqrt() * params.sqrt_dt * z_var;
        variance = variance.max(0.0);
    }

    log_s_t.exp()
}

fn heston_discounted_payoff(
    params: HestonStepParams,
    rng: &mut MonteCarloRng,
    sign: f64,
) -> (f64, f64) {
    let s_t = heston_path_terminal(params, rng, sign);
    (
        (s_t - params.strike).max(0.0) * params.discount,
        s_t * params.discount,
    )
}

fn simulate_heston_chunk(seed: u64, n_paths: usize, params: HestonStepParams) -> (f64, f64) {
    let mut rng = MonteCarloRng::new(seed);
    let mut payoff_sum = 0.0;
    let mut payoff_sq_sum = 0.0;

    for _ in 0..n_paths {
        let (payoff, _) = heston_discounted_payoff(params, &mut rng, 1.0);
        payoff_sum += payoff;
        payoff_sq_sum += payoff * payoff;
    }

    (payoff_sum, payoff_sq_sum)
}

fn simulate_heston_antithetic_chunk(
    seed: u64,
    pair_count: usize,
    params: HestonStepParams,
) -> (f64, f64) {
    let mut rng = MonteCarloRng::new(seed);
    let mut block_sum = 0.0;
    let mut block_sq_sum = 0.0;

    for _ in 0..pair_count {
        let mut log_a = params.log_s0;
        let mut log_b = params.log_s0;
        let mut var_a = params.v0;
        let mut var_b = params.v0;

        for _ in 0..params.n_steps {
            let z_var = rng.standard_normal();
            let z_independent = rng.standard_normal();

            let var_a_pos = var_a.max(0.0);
            let z_price_a = params.rho * z_var + params.rho_perp * z_independent;
            log_a += (params.rate - 0.5 * var_a_pos) * params.dt
                + var_a_pos.sqrt() * params.sqrt_dt * z_price_a;
            var_a = var_a
                + params.kappa * (params.theta - var_a_pos) * params.dt
                + params.vol_of_vol * var_a_pos.sqrt() * params.sqrt_dt * z_var;
            var_a = var_a.max(0.0);

            let z_var_b = -z_var;
            let z_independent_b = -z_independent;
            let var_b_pos = var_b.max(0.0);
            let z_price_b = params.rho * z_var_b + params.rho_perp * z_independent_b;
            log_b += (params.rate - 0.5 * var_b_pos) * params.dt
                + var_b_pos.sqrt() * params.sqrt_dt * z_price_b;
            var_b = var_b
                + params.kappa * (params.theta - var_b_pos) * params.dt
                + params.vol_of_vol * var_b_pos.sqrt() * params.sqrt_dt * z_var_b;
            var_b = var_b.max(0.0);
        }

        let payoff_a = (log_a.exp() - params.strike).max(0.0) * params.discount;
        let payoff_b = (log_b.exp() - params.strike).max(0.0) * params.discount;
        let block_estimate = 0.5 * (payoff_a + payoff_b);
        block_sum += block_estimate;
        block_sq_sum += block_estimate * block_estimate;
    }

    (block_sum, block_sq_sum)
}

fn simulate_heston_control_variate_chunk(
    seed: u64,
    n_paths: usize,
    params: HestonStepParams,
) -> ControlVariateMoments {
    let mut rng = MonteCarloRng::new(seed);
    let mut moments = ControlVariateMoments::default();

    for _ in 0..n_paths {
        let (payoff, discounted_terminal) = heston_discounted_payoff(params, &mut rng, 1.0);
        moments.record(payoff, discounted_terminal);
    }

    moments
}

fn simulate_heston_parallel(
    cfg: &HestonEuropeanCallConfig,
    thread_count: usize,
    params: HestonStepParams,
) -> (f64, f64) {
    let base_chunk = cfg.n_paths / thread_count;
    let remainder = cfg.n_paths % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let n_paths_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        handles.push(thread::spawn(move || {
            simulate_heston_chunk(seed, n_paths_chunk, params)
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

fn simulate_heston_antithetic_parallel(
    cfg: &HestonEuropeanCallConfig,
    thread_count: usize,
    params: HestonStepParams,
) -> (f64, f64) {
    let pair_count = cfg.n_paths.div_ceil(2);
    let base_chunk = pair_count / thread_count;
    let remainder = pair_count % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let pair_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        handles.push(thread::spawn(move || {
            simulate_heston_antithetic_chunk(seed, pair_chunk, params)
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

fn simulate_heston_control_variate_parallel(
    cfg: &HestonEuropeanCallConfig,
    thread_count: usize,
    params: HestonStepParams,
) -> ControlVariateMoments {
    let base_chunk = cfg.n_paths / thread_count;
    let remainder = cfg.n_paths % thread_count;

    let mut handles = Vec::with_capacity(thread_count);
    for idx in 0..thread_count {
        let n_paths_chunk = base_chunk + usize::from(idx < remainder);
        let seed = derive_chunk_seed(cfg.seed, idx as u64);
        handles.push(thread::spawn(move || {
            simulate_heston_control_variate_chunk(seed, n_paths_chunk, params)
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

fn heston_european_call_price_mc_standard(
    cfg: &HestonEuropeanCallConfig,
) -> HestonEuropeanCallResult {
    let params = heston_params(cfg);
    let thread_count = resolved_thread_count(cfg.n_threads);
    let (payoff_sum, payoff_sq_sum) = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_heston_chunk(cfg.seed, cfg.n_paths, params)
    } else {
        simulate_heston_parallel(cfg, thread_count, params)
    };

    summarize_payoffs(cfg.n_paths, payoff_sum, payoff_sq_sum)
}

fn heston_european_call_price_mc_antithetic(
    cfg: &HestonEuropeanCallConfig,
) -> HestonEuropeanCallResult {
    let params = heston_params(cfg);
    let pair_count = cfg.n_paths.div_ceil(2);
    let thread_count = resolved_thread_count(cfg.n_threads);
    let (block_sum, block_sq_sum) = if thread_count <= 1 || pair_count < thread_count * 2_000 {
        simulate_heston_antithetic_chunk(cfg.seed, pair_count, params)
    } else {
        simulate_heston_antithetic_parallel(cfg, thread_count, params)
    };

    summarize_block_estimates(pair_count, block_sum, block_sq_sum)
}

fn heston_european_call_price_mc_control_variate(
    cfg: &HestonEuropeanCallConfig,
) -> HestonEuropeanCallResult {
    let params = heston_params(cfg);
    let thread_count = resolved_thread_count(cfg.n_threads);
    let moments = if thread_count <= 1 || cfg.n_paths < thread_count * 2_000 {
        simulate_heston_control_variate_chunk(cfg.seed, cfg.n_paths, params)
    } else {
        simulate_heston_control_variate_parallel(cfg, thread_count, params)
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
