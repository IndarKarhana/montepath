use mc_core::{
    american_put_price_lsm_cpu, arithmetic_asian_call_price_mc_cpu,
    arithmetic_asian_call_price_mlmc_cpu, basket_call_price_mc_cpu,
    black_scholes_european_call_greeks, black_scholes_european_call_price,
    black_scholes_european_put_price, compare_arithmetic_asian_sampling_quality_cpu,
    compare_basket_call_sampling_quality_cpu, compare_down_and_out_sampling_quality_cpu,
    compare_european_call_realized_error_cpu, compare_european_call_sampling_quality_cpu,
    compare_heston_black_scholes_limit_cpu, diagnose_standard_normals_cpu,
    down_and_out_call_price_mc_cpu, european_call_greeks_cpu, european_call_price_mc_cpu,
    european_call_price_mc_cpu_stepwise, european_call_price_mc_cpu_terminal,
    gaussian_uncertainty_mean_cpu, generate_standard_normals_cpu, heston_european_call_greeks_cpu,
    heston_european_call_price_mc_cpu, lookback_call_price_mc_cpu, monte_carlo_method_capabilities,
    price_all_current_greeks_bump_and_revalue_cpu, solve_arithmetic_asian_mlmc_tolerance_cpu,
    structured_sampling_guidance_cpu, tune_arithmetic_asian_mlmc_allocation_cpu, AmericanPutConfig,
    AmericanPutPricer, ArithmeticAsianCallConfig, ArithmeticAsianCallPricer,
    ArithmeticAsianMlmcConfig, ArithmeticAsianMlmcPricer, ArithmeticAsianMlmcToleranceConfig,
    BackendMethodSupport, BasketCallConfig, BasketCallPricer, DownAndOutCallConfig,
    DownAndOutCallPricer, EuropeanCallConfig, EuropeanCallMethod, EuropeanCallPricer,
    GaussianUncertaintyConfig, Greek, GreekEstimator, HestonEuropeanCallConfig,
    HestonEuropeanCallPricer, LookbackCallConfig, LookbackCallPricer, MonteCarloRng,
    MonteCarloTechnique, PricingWorkloadFamily, SamplingMethod,
};

#[test]
fn rng_is_deterministic_for_same_seed() {
    let mut rng_a = MonteCarloRng::new(12345);
    let mut rng_b = MonteCarloRng::new(12345);

    for _ in 0..100 {
        let a = rng_a.standard_normal();
        let b = rng_b.standard_normal();
        assert_eq!(a, b);
    }
}

#[test]
fn european_call_mc_is_deterministic_for_same_seed() {
    let cfg = EuropeanCallConfig {
        n_paths: 50_000,
        n_steps: 64,
        seed: 7,
        ..EuropeanCallConfig::default()
    };

    let r1 = european_call_price_mc_cpu(&cfg);
    let r2 = european_call_price_mc_cpu(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn european_call_mc_is_deterministic_for_parallel_seed() {
    let cfg = EuropeanCallConfig {
        n_paths: 50_000,
        n_steps: 64,
        seed: 11,
        n_threads: 4,
        ..EuropeanCallConfig::default()
    };

    let r1 = european_call_price_mc_cpu(&cfg);
    let r2 = european_call_price_mc_cpu(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn european_call_terminal_method_is_deterministic_for_same_seed() {
    let cfg = EuropeanCallConfig {
        n_paths: 50_000,
        n_steps: 64,
        seed: 33,
        ..EuropeanCallConfig::default()
    };

    let r1 = european_call_price_mc_cpu_terminal(&cfg);
    let r2 = european_call_price_mc_cpu_terminal(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn european_call_stepwise_method_is_deterministic_for_same_seed() {
    let cfg = EuropeanCallConfig {
        n_paths: 50_000,
        n_steps: 64,
        seed: 44,
        n_threads: 4,
        ..EuropeanCallConfig::default()
    };

    let r1 = european_call_price_mc_cpu_stepwise(&cfg);
    let r2 = european_call_price_mc_cpu_stepwise(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn european_call_antithetic_terminal_is_deterministic_for_same_seed() {
    let cfg = EuropeanCallConfig {
        n_paths: 50_000,
        n_steps: 64,
        seed: 55,
        technique: MonteCarloTechnique::Antithetic,
        ..EuropeanCallConfig::default()
    };

    let r1 = european_call_price_mc_cpu_terminal(&cfg);
    let r2 = european_call_price_mc_cpu_terminal(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn european_call_antithetic_stepwise_is_deterministic_for_same_seed() {
    let cfg = EuropeanCallConfig {
        n_paths: 50_000,
        n_steps: 64,
        seed: 66,
        n_threads: 4,
        technique: MonteCarloTechnique::Antithetic,
        ..EuropeanCallConfig::default()
    };

    let r1 = european_call_price_mc_cpu_stepwise(&cfg);
    let r2 = european_call_price_mc_cpu_stepwise(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn european_call_control_variate_terminal_is_deterministic_for_same_seed() {
    let cfg = EuropeanCallConfig {
        n_paths: 50_000,
        n_steps: 64,
        seed: 77,
        technique: MonteCarloTechnique::ControlVariate,
        ..EuropeanCallConfig::default()
    };

    let r1 = european_call_price_mc_cpu_terminal(&cfg);
    let r2 = european_call_price_mc_cpu_terminal(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn european_call_control_variate_stepwise_is_deterministic_for_same_seed() {
    let cfg = EuropeanCallConfig {
        n_paths: 50_000,
        n_steps: 64,
        seed: 78,
        n_threads: 4,
        technique: MonteCarloTechnique::ControlVariate,
        ..EuropeanCallConfig::default()
    };

    let r1 = european_call_price_mc_cpu_stepwise(&cfg);
    let r2 = european_call_price_mc_cpu_stepwise(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn structured_normal_generation_is_deterministic_and_sane() {
    let samples_a = generate_standard_normals_cpu(SamplingMethod::ScrambledSobol, 512, 4, 99);
    let samples_b = generate_standard_normals_cpu(SamplingMethod::ScrambledSobol, 512, 4, 99);
    let odd_dimensions = generate_standard_normals_cpu(SamplingMethod::ScrambledSobol, 17, 5, 99);
    let halton = generate_standard_normals_cpu(SamplingMethod::RandomizedHalton, 512, 4, 99);
    let lhs = generate_standard_normals_cpu(SamplingMethod::LatinHypercube, 512, 4, 99);

    assert_eq!(samples_a, samples_b);
    assert_eq!(samples_a.len(), 2_048);
    assert_eq!(odd_dimensions.len(), 85);
    assert_eq!(halton.len(), 2_048);
    assert_eq!(lhs.len(), 2_048);
    assert!(samples_a.iter().all(|value| value.is_finite()));
    assert!(odd_dimensions.iter().all(|value| value.is_finite()));
    assert!(halton.iter().all(|value| value.is_finite()));
    assert!(lhs.iter().all(|value| value.is_finite()));

    let mean = samples_a.iter().sum::<f64>() / samples_a.len() as f64;
    assert!(mean.abs() < 0.2);
}

#[test]
fn structured_sampling_guidance_flags_sobol_balance() {
    let guidance = structured_sampling_guidance_cpu(SamplingMethod::ScrambledSobol, 1_000, 8);

    assert_eq!(guidance.recommended_points, 1_024);
    assert!(!guidance.is_power_of_two);
    assert!(guidance
        .warnings
        .iter()
        .any(|warning| warning.contains("powers of two")));
}

#[test]
fn standard_normal_diagnostics_report_distribution_quality() {
    let diagnostics = diagnose_standard_normals_cpu(SamplingMethod::LatinHypercube, 1_024, 4, 7);

    assert_eq!(diagnostics.sample_count, 4_096);
    assert_eq!(diagnostics.dimensions, 4);
    assert!(diagnostics.finite);
    assert!(diagnostics.mean_abs < 0.05);
    assert!(diagnostics.variance_abs_error < 0.1);
    assert!(diagnostics.max_axis_mean_abs < 0.05);
    assert!(diagnostics.tail_2sigma_abs_error < 0.02);
}

#[test]
fn pricing_quality_comparison_reports_structured_sampling_health() {
    let cfg = EuropeanCallConfig {
        n_paths: 8_192,
        n_steps: 32,
        seed: 91,
        ..EuropeanCallConfig::default()
    };

    let comparison =
        compare_european_call_sampling_quality_cpu(&cfg, SamplingMethod::ScrambledSobol);

    assert_eq!(comparison.workload, PricingWorkloadFamily::EuropeanCall);
    assert_eq!(comparison.sampling, SamplingMethod::ScrambledSobol);
    assert_eq!(comparison.paths, cfg.n_paths);
    assert_eq!(comparison.steps, cfg.n_steps);
    assert!(comparison.pseudorandom_stderr > 0.0);
    assert!(comparison.structured_stderr > 0.0);
    assert!(comparison.stderr_ratio_vs_pseudorandom > 0.0);
    assert!(comparison.price_delta_stderr_units.is_finite());
    assert!(comparison.normal_diagnostics.finite);
    assert!(comparison.guidance.is_power_of_two);
}

#[test]
fn european_realized_error_comparison_uses_black_scholes_reference() {
    let cfg = EuropeanCallConfig {
        n_paths: 65_536,
        n_steps: 64,
        seed: 8_801,
        ..EuropeanCallConfig::default()
    };

    let comparison = compare_european_call_realized_error_cpu(&cfg, SamplingMethod::LatinHypercube);
    let analytic = black_scholes_european_call_price(cfg.s0, cfg.k, cfg.r, cfg.sigma, cfg.t);

    assert_eq!(comparison.workload, PricingWorkloadFamily::EuropeanCall);
    assert_eq!(comparison.sampling, SamplingMethod::LatinHypercube);
    assert!((comparison.analytic_price - analytic).abs() < 1e-12);
    assert!(comparison.pseudorandom_abs_error >= 0.0);
    assert!(comparison.structured_abs_error >= 0.0);
    assert!(comparison.abs_error_ratio_vs_pseudorandom.is_finite());
    assert!(comparison.abs_error_ratio_vs_pseudorandom >= 0.0);
    assert_eq!(comparison.guidance.dimensions, cfg.n_steps);
}

#[test]
fn pricing_quality_comparison_covers_path_dependent_workloads() {
    let asian_cfg = ArithmeticAsianCallConfig {
        n_paths: 8_192,
        n_steps: 32,
        seed: 92,
        ..ArithmeticAsianCallConfig::default()
    };
    let barrier_cfg = DownAndOutCallConfig {
        n_paths: 8_192,
        n_steps: 32,
        seed: 93,
        ..DownAndOutCallConfig::default()
    };

    let asian =
        compare_arithmetic_asian_sampling_quality_cpu(&asian_cfg, SamplingMethod::LatinHypercube);
    let barrier =
        compare_down_and_out_sampling_quality_cpu(&barrier_cfg, SamplingMethod::RandomizedHalton);

    assert_eq!(asian.workload, PricingWorkloadFamily::ArithmeticAsianCall);
    assert_eq!(barrier.workload, PricingWorkloadFamily::DownAndOutCall);
    assert!(asian.stderr_ratio_vs_pseudorandom > 0.0);
    assert!(barrier.stderr_ratio_vs_pseudorandom > 0.0);
    assert!(asian.price_delta_abs.is_finite());
    assert!(barrier.price_delta_abs.is_finite());
}

#[test]
fn basket_call_mc_is_deterministic_for_same_seed() {
    let cfg = BasketCallConfig {
        n_paths: 50_000,
        seed: 95,
        n_threads: 4,
        ..BasketCallConfig::default()
    };

    let r1 = basket_call_price_mc_cpu(&cfg);
    let r2 = basket_call_price_mc_cpu(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn basket_call_structured_sampling_is_deterministic() {
    let cfg = BasketCallConfig {
        n_paths: 32_768,
        seed: 96,
        sampling: SamplingMethod::ScrambledSobol,
        ..BasketCallConfig::default()
    };

    let r1 = basket_call_price_mc_cpu(&cfg);
    let r2 = basket_call_price_mc_cpu(&cfg);
    assert_eq!(r1, r2);
    assert!(r1.price >= 0.0);
    assert!(r1.stderr >= 0.0);
}

#[test]
fn basket_call_pricer_builder_supports_expressive_configuration() {
    let result = BasketCallPricer::new()
        .spot1(105.0)
        .spot2(90.0)
        .strike(100.0)
        .rate(0.02)
        .volatility1(0.18)
        .volatility2(0.28)
        .correlation(0.4)
        .weights(0.6, 0.4)
        .maturity(1.5)
        .paths(20_000)
        .seed(97)
        .latin_hypercube()
        .control_variate()
        .price();

    assert!(result.price >= 0.0);
    assert!(result.stderr >= 0.0);
}

#[test]
fn basket_quality_comparison_reports_multi_asset_qmc_health() {
    let cfg = BasketCallConfig {
        n_paths: 16_384,
        seed: 98,
        ..BasketCallConfig::default()
    };

    let comparison = compare_basket_call_sampling_quality_cpu(&cfg, SamplingMethod::LatinHypercube);

    assert_eq!(comparison.workload, PricingWorkloadFamily::BasketCall);
    assert_eq!(comparison.steps, 2);
    assert!(comparison.stderr_ratio_vs_pseudorandom > 0.0);
    assert!(comparison.normal_diagnostics.finite);
    assert!(comparison.price_delta_stderr_units.is_finite());
}

#[test]
fn gaussian_uncertainty_mean_matches_analytic_reference() {
    let cfg = GaussianUncertaintyConfig {
        n_samples: 65_536,
        dimensions: 3,
        seed: 94,
        sampling: SamplingMethod::ScrambledSobol,
    };

    let result = gaussian_uncertainty_mean_cpu(&cfg);

    assert!(result.stderr >= 0.0);
    assert!(result.abs_error < 0.02);
    assert!((result.analytic_mean - (1.0 + 0.005f64.exp())).abs() < 1e-12);
}

#[test]
fn european_call_mc_outputs_sane_values() {
    let cfg = EuropeanCallConfig {
        n_paths: 30_000,
        n_steps: 64,
        seed: 101,
        ..EuropeanCallConfig::default()
    };

    let result = european_call_price_mc_cpu(&cfg);
    assert!(result.price >= 0.0);
    assert!(result.stderr >= 0.0);
    assert!(result.price < cfg.s0 * 2.0);
}

#[test]
fn arithmetic_asian_call_mc_is_deterministic_for_same_seed() {
    let cfg = ArithmeticAsianCallConfig {
        n_paths: 50_000,
        n_steps: 64,
        seed: 79,
        n_threads: 4,
        ..ArithmeticAsianCallConfig::default()
    };

    let r1 = arithmetic_asian_call_price_mc_cpu(&cfg);
    let r2 = arithmetic_asian_call_price_mc_cpu(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn arithmetic_asian_call_control_variate_is_deterministic_for_same_seed() {
    let cfg = ArithmeticAsianCallConfig {
        n_paths: 50_000,
        n_steps: 64,
        seed: 80,
        n_threads: 4,
        technique: MonteCarloTechnique::ControlVariate,
        ..ArithmeticAsianCallConfig::default()
    };

    let r1 = arithmetic_asian_call_price_mc_cpu(&cfg);
    let r2 = arithmetic_asian_call_price_mc_cpu(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn arithmetic_asian_call_outputs_sane_values() {
    let cfg = ArithmeticAsianCallConfig {
        n_paths: 30_000,
        n_steps: 64,
        seed: 101,
        ..ArithmeticAsianCallConfig::default()
    };

    let result = arithmetic_asian_call_price_mc_cpu(&cfg);
    assert!(result.price >= 0.0);
    assert!(result.stderr >= 0.0);
    assert!(result.price < cfg.s0 * 2.0);
}

#[test]
fn arithmetic_asian_mlmc_is_deterministic_for_same_seed() {
    let cfg = ArithmeticAsianMlmcConfig {
        base_steps: 8,
        levels: 3,
        paths_per_level: vec![8_000, 4_000, 2_000],
        seed: 909,
        ..ArithmeticAsianMlmcConfig::default()
    };

    let r1 = arithmetic_asian_call_price_mlmc_cpu(&cfg);
    let r2 = arithmetic_asian_call_price_mlmc_cpu(&cfg);

    assert_eq!(r1, r2);
    assert_eq!(r1.levels.len(), 3);
    assert_eq!(r1.total_paths, 14_000);
    assert!(r1.total_step_updates > 0);
}

#[test]
fn arithmetic_asian_mlmc_outputs_sane_level_metadata() {
    let cfg = ArithmeticAsianMlmcConfig {
        base_steps: 8,
        levels: 4,
        refinement_factor: 2,
        paths_per_level: vec![16_000, 8_000, 4_000, 2_000],
        seed: 910,
        ..ArithmeticAsianMlmcConfig::default()
    };

    let result = arithmetic_asian_call_price_mlmc_cpu(&cfg);

    assert!(result.price >= 0.0);
    assert!(result.stderr >= 0.0);
    assert!(result.price < cfg.s0 * 2.0);
    assert_eq!(result.levels.len(), cfg.levels);
    assert_eq!(result.levels[0].coarse_steps, None);
    assert_eq!(result.levels[0].fine_steps, cfg.base_steps);
    assert_eq!(result.levels[1].coarse_steps, Some(cfg.base_steps));
    assert_eq!(
        result.levels[1].fine_steps,
        cfg.base_steps * cfg.refinement_factor
    );
    assert!(result.levels.iter().all(|level| level.paths > 0));
    assert!(result.levels.iter().all(|level| level.variance >= 0.0));
}

#[test]
fn arithmetic_asian_mlmc_pricer_builder_supports_expressive_configuration() {
    let result = ArithmeticAsianMlmcPricer::new()
        .s0(100.0)
        .strike(100.0)
        .rate(0.03)
        .volatility(0.2)
        .maturity(1.0)
        .base_steps(8)
        .levels(3)
        .refinement_factor(2)
        .paths_per_level(vec![8_000, 4_000, 2_000])
        .seed(911)
        .price();

    assert!(result.price >= 0.0);
    assert!(result.stderr >= 0.0);
    assert_eq!(result.levels.len(), 3);
}

#[test]
fn arithmetic_asian_mlqmc_scrambled_sobol_is_deterministic_for_same_seed() {
    let cfg = ArithmeticAsianMlmcConfig {
        base_steps: 8,
        levels: 3,
        paths_per_level: vec![8_192, 4_096, 2_048],
        seed: 912,
        sampling: SamplingMethod::ScrambledSobol,
        scramble_replicates: 4,
        ..ArithmeticAsianMlmcConfig::default()
    };

    let r1 = arithmetic_asian_call_price_mlmc_cpu(&cfg);
    let r2 = arithmetic_asian_call_price_mlmc_cpu(&cfg);

    assert_eq!(r1, r2);
    assert_eq!(r1.scramble_replicates, 4);
    assert_eq!(r1.replicate_estimates.len(), 4);
    assert_eq!(
        r1.total_paths,
        cfg.paths_per_level.iter().sum::<usize>() * cfg.scramble_replicates
    );
    assert!(r1.price >= 0.0);
    assert!(r1.stderr >= 0.0);
}

#[test]
fn arithmetic_asian_mlmc_allocation_tuner_returns_positive_paths() {
    let cfg = ArithmeticAsianMlmcConfig {
        base_steps: 8,
        levels: 4,
        paths_per_level: vec![4_000, 2_000, 1_000, 500],
        seed: 913,
        ..ArithmeticAsianMlmcConfig::default()
    };

    let plan = tune_arithmetic_asian_mlmc_allocation_cpu(&cfg, 500_000, 512);

    assert_eq!(plan.paths_per_level.len(), cfg.levels);
    assert_eq!(plan.levels.len(), cfg.levels);
    assert!(plan.estimated_step_updates > 0);
    assert!(plan.paths_per_level.iter().all(|paths| *paths >= 2));
    assert!(plan
        .levels
        .iter()
        .all(|level| level.pilot_variance >= 0.0 && level.cost_per_path > 0));
}

#[test]
fn arithmetic_asian_mlmc_tolerance_solver_returns_executable_plan() {
    let cfg = ArithmeticAsianMlmcConfig {
        base_steps: 8,
        levels: 4,
        paths_per_level: vec![2_000, 1_000, 500, 250],
        seed: 914,
        ..ArithmeticAsianMlmcConfig::default()
    };
    let tolerance = ArithmeticAsianMlmcToleranceConfig {
        target_stderr: 0.08,
        pilot_paths_per_level: 512,
        min_step_updates: 50_000,
        max_step_updates: 2_000_000,
    };

    let plan = solve_arithmetic_asian_mlmc_tolerance_cpu(&cfg, &tolerance);
    let result = arithmetic_asian_call_price_mlmc_cpu(&plan.recommended_config);

    assert_eq!(
        plan.paths_per_level,
        plan.recommended_config.paths_per_level
    );
    assert_eq!(plan.allocation.paths_per_level, plan.paths_per_level);
    assert_eq!(
        plan.recommended_config.scramble_replicates,
        cfg.scramble_replicates
    );
    assert!(plan.estimated_stderr > 0.0);
    assert!(plan.estimated_step_updates >= tolerance.min_step_updates);
    assert!(plan.estimated_step_updates <= tolerance.max_step_updates);
    assert!(plan.target_met);
    assert!(result.price >= 0.0);
    assert!(result.stderr >= 0.0);
}

#[test]
fn arithmetic_asian_mlqmc_tolerance_solver_accounts_for_replicates() {
    let cfg = ArithmeticAsianMlmcConfig {
        base_steps: 8,
        levels: 3,
        paths_per_level: vec![1_024, 512, 256],
        seed: 915,
        sampling: SamplingMethod::ScrambledSobol,
        scramble_replicates: 4,
        ..ArithmeticAsianMlmcConfig::default()
    };
    let tolerance = ArithmeticAsianMlmcToleranceConfig {
        target_stderr: 0.05,
        pilot_paths_per_level: 256,
        min_step_updates: 40_000,
        max_step_updates: 1_500_000,
    };

    let plan = solve_arithmetic_asian_mlmc_tolerance_cpu(&cfg, &tolerance);
    let result = arithmetic_asian_call_price_mlmc_cpu(&plan.recommended_config);

    assert_eq!(plan.scramble_replicates, 4);
    assert_eq!(plan.recommended_config.scramble_replicates, 4);
    assert_eq!(result.replicate_estimates.len(), 4);
    assert_eq!(result.total_step_updates, plan.estimated_step_updates);
    assert!(plan.estimated_stderr <= tolerance.target_stderr);
    assert!(plan.target_met);
}

#[test]
fn european_call_stepwise_is_close_to_black_scholes_price() {
    let cfg = EuropeanCallConfig {
        s0: 100.0,
        k: 100.0,
        r: 0.03,
        sigma: 0.2,
        t: 1.0,
        n_paths: 200_000,
        n_steps: 64,
        seed: 2027,
        n_threads: 4,
        technique: MonteCarloTechnique::Standard,
        sampling: SamplingMethod::Pseudorandom,
    };

    let mc = european_call_price_mc_cpu_stepwise(&cfg);
    let analytic = black_scholes_call(cfg.s0, cfg.k, cfg.r, cfg.sigma, cfg.t);
    let error = (mc.price - analytic).abs();

    assert!(
        error <= (6.0 * mc.stderr + 1e-9),
        "stepwise mc price deviates too much from analytic price: mc={} analytic={} stderr={}",
        mc.price,
        analytic,
        mc.stderr
    );
}

#[test]
fn pricer_builder_supports_expressive_configuration() {
    let result = EuropeanCallPricer::new()
        .s0(100.0)
        .strike(100.0)
        .rate(0.03)
        .volatility(0.2)
        .maturity(1.0)
        .paths(50_000)
        .steps(64)
        .seed(123)
        .threads(2)
        .method(EuropeanCallMethod::StepwisePaths)
        .price();

    assert!(result.price >= 0.0);
    assert!(result.stderr >= 0.0);
}

#[test]
fn pricer_builder_supports_antithetic_configuration() {
    let result = EuropeanCallPricer::new()
        .paths(50_000)
        .steps(64)
        .seed(888)
        .stepwise()
        .antithetic()
        .price();

    assert!(result.price >= 0.0);
    assert!(result.stderr >= 0.0);
}

#[test]
fn pricer_builder_supports_control_variate_configuration() {
    let result = EuropeanCallPricer::new()
        .paths(50_000)
        .steps(64)
        .seed(889)
        .stepwise()
        .control_variate()
        .price();

    assert!(result.price >= 0.0);
    assert!(result.stderr >= 0.0);
}

#[test]
fn arithmetic_asian_pricer_builder_supports_expressive_configuration() {
    let result = ArithmeticAsianCallPricer::new()
        .s0(100.0)
        .strike(95.0)
        .rate(0.03)
        .volatility(0.2)
        .maturity(1.0)
        .paths(50_000)
        .steps(64)
        .seed(901)
        .threads(2)
        .control_variate()
        .price();

    assert!(result.price >= 0.0);
    assert!(result.stderr >= 0.0);
}

#[test]
fn auto_and_terminal_pricer_methods_match_for_european_call() {
    let pricer = EuropeanCallPricer::new().paths(80_000).steps(64).seed(321);

    let auto = pricer.clone().price();
    let terminal = pricer.terminal().price();
    assert_eq!(auto, terminal);
}

#[test]
fn european_call_mc_is_close_to_black_scholes_price() {
    let cfg = EuropeanCallConfig {
        s0: 100.0,
        k: 100.0,
        r: 0.03,
        sigma: 0.2,
        t: 1.0,
        n_paths: 200_000,
        n_steps: 64,
        seed: 2026,
        n_threads: 4,
        technique: MonteCarloTechnique::Standard,
        sampling: SamplingMethod::Pseudorandom,
    };

    let mc = european_call_price_mc_cpu(&cfg);
    let analytic = black_scholes_call(cfg.s0, cfg.k, cfg.r, cfg.sigma, cfg.t);
    let error = (mc.price - analytic).abs();

    // 5-sigma band keeps this test stable while still catching large numeric regressions.
    assert!(
        error <= (5.0 * mc.stderr + 1e-9),
        "mc price deviates too much from analytic price: mc={} analytic={} stderr={}",
        mc.price,
        analytic,
        mc.stderr
    );
}

#[test]
fn antithetic_terminal_reduces_standard_error() {
    let standard_cfg = EuropeanCallConfig {
        n_paths: 200_000,
        n_steps: 64,
        seed: 900,
        ..EuropeanCallConfig::default()
    };
    let antithetic_cfg = EuropeanCallConfig {
        technique: MonteCarloTechnique::Antithetic,
        ..standard_cfg
    };

    let standard = european_call_price_mc_cpu_terminal(&standard_cfg);
    let antithetic = european_call_price_mc_cpu_terminal(&antithetic_cfg);

    assert!(antithetic.stderr < standard.stderr);
}

#[test]
fn antithetic_stepwise_reduces_standard_error() {
    let standard_cfg = EuropeanCallConfig {
        n_paths: 200_000,
        n_steps: 64,
        seed: 901,
        n_threads: 4,
        ..EuropeanCallConfig::default()
    };
    let antithetic_cfg = EuropeanCallConfig {
        technique: MonteCarloTechnique::Antithetic,
        ..standard_cfg
    };

    let standard = european_call_price_mc_cpu_stepwise(&standard_cfg);
    let antithetic = european_call_price_mc_cpu_stepwise(&antithetic_cfg);

    assert!(antithetic.stderr < standard.stderr);
}

#[test]
fn control_variate_terminal_reduces_standard_error() {
    let standard_cfg = EuropeanCallConfig {
        n_paths: 200_000,
        n_steps: 64,
        seed: 902,
        ..EuropeanCallConfig::default()
    };
    let control_cfg = EuropeanCallConfig {
        technique: MonteCarloTechnique::ControlVariate,
        ..standard_cfg
    };

    let standard = european_call_price_mc_cpu_terminal(&standard_cfg);
    let control = european_call_price_mc_cpu_terminal(&control_cfg);

    assert!(control.stderr < standard.stderr);
}

#[test]
fn control_variate_stepwise_reduces_standard_error() {
    let standard_cfg = EuropeanCallConfig {
        n_paths: 200_000,
        n_steps: 64,
        seed: 903,
        n_threads: 4,
        ..EuropeanCallConfig::default()
    };
    let control_cfg = EuropeanCallConfig {
        technique: MonteCarloTechnique::ControlVariate,
        ..standard_cfg
    };

    let standard = european_call_price_mc_cpu_stepwise(&standard_cfg);
    let control = european_call_price_mc_cpu_stepwise(&control_cfg);

    assert!(control.stderr < standard.stderr);
}

#[test]
fn randomized_halton_stepwise_is_deterministic_for_same_seed() {
    let cfg = EuropeanCallConfig {
        n_paths: 40_000,
        n_steps: 32,
        seed: 904,
        sampling: SamplingMethod::RandomizedHalton,
        ..EuropeanCallConfig::default()
    };

    let r1 = european_call_price_mc_cpu_stepwise(&cfg);
    let r2 = european_call_price_mc_cpu_stepwise(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn latin_hypercube_stepwise_is_deterministic_for_same_seed() {
    let cfg = EuropeanCallConfig {
        n_paths: 40_000,
        n_steps: 32,
        seed: 914,
        sampling: SamplingMethod::LatinHypercube,
        ..EuropeanCallConfig::default()
    };

    let r1 = european_call_price_mc_cpu_stepwise(&cfg);
    let r2 = european_call_price_mc_cpu_stepwise(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn scrambled_sobol_stepwise_is_deterministic_for_same_seed() {
    let cfg = EuropeanCallConfig {
        n_paths: 40_000,
        n_steps: 32,
        seed: 924,
        sampling: SamplingMethod::ScrambledSobol,
        ..EuropeanCallConfig::default()
    };

    let r1 = european_call_price_mc_cpu_stepwise(&cfg);
    let r2 = european_call_price_mc_cpu_stepwise(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn scrambled_sobol_brownian_bridge_is_deterministic_for_same_seed() {
    let cfg = EuropeanCallConfig {
        n_paths: 40_000,
        n_steps: 32,
        seed: 925,
        sampling: SamplingMethod::ScrambledSobolBrownianBridge,
        ..EuropeanCallConfig::default()
    };

    let r1 = european_call_price_mc_cpu_stepwise(&cfg);
    let r2 = european_call_price_mc_cpu_stepwise(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn randomized_halton_control_variate_reduces_standard_error_for_european_call() {
    let standard_cfg = EuropeanCallConfig {
        n_paths: 65_536,
        n_steps: 32,
        seed: 905,
        sampling: SamplingMethod::RandomizedHalton,
        ..EuropeanCallConfig::default()
    };
    let control_cfg = EuropeanCallConfig {
        technique: MonteCarloTechnique::ControlVariate,
        ..standard_cfg
    };

    let standard = european_call_price_mc_cpu_stepwise(&standard_cfg);
    let control = european_call_price_mc_cpu_stepwise(&control_cfg);

    assert!(control.stderr < standard.stderr);
}

#[test]
fn latin_hypercube_control_variate_reduces_standard_error_for_european_call() {
    let standard_cfg = EuropeanCallConfig {
        n_paths: 65_536,
        n_steps: 32,
        seed: 915,
        sampling: SamplingMethod::LatinHypercube,
        ..EuropeanCallConfig::default()
    };
    let control_cfg = EuropeanCallConfig {
        technique: MonteCarloTechnique::ControlVariate,
        ..standard_cfg
    };

    let standard = european_call_price_mc_cpu_stepwise(&standard_cfg);
    let control = european_call_price_mc_cpu_stepwise(&control_cfg);

    assert!(control.stderr < standard.stderr);
}

#[test]
fn scrambled_sobol_control_variate_reduces_standard_error_for_european_call() {
    let standard_cfg = EuropeanCallConfig {
        n_paths: 65_536,
        n_steps: 32,
        seed: 926,
        sampling: SamplingMethod::ScrambledSobol,
        ..EuropeanCallConfig::default()
    };
    let control_cfg = EuropeanCallConfig {
        technique: MonteCarloTechnique::ControlVariate,
        ..standard_cfg
    };

    let standard = european_call_price_mc_cpu_stepwise(&standard_cfg);
    let control = european_call_price_mc_cpu_stepwise(&control_cfg);

    assert!(control.stderr < standard.stderr);
}

#[test]
fn scrambled_sobol_brownian_bridge_control_variate_reduces_standard_error_for_european_call() {
    let standard_cfg = EuropeanCallConfig {
        n_paths: 65_536,
        n_steps: 32,
        seed: 927,
        sampling: SamplingMethod::ScrambledSobolBrownianBridge,
        ..EuropeanCallConfig::default()
    };
    let control_cfg = EuropeanCallConfig {
        technique: MonteCarloTechnique::ControlVariate,
        ..standard_cfg
    };

    let standard = european_call_price_mc_cpu_stepwise(&standard_cfg);
    let control = european_call_price_mc_cpu_stepwise(&control_cfg);

    assert!(control.stderr < standard.stderr);
}

#[test]
fn arithmetic_asian_randomized_halton_is_deterministic_for_same_seed() {
    let cfg = ArithmeticAsianCallConfig {
        n_paths: 40_000,
        n_steps: 32,
        seed: 906,
        sampling: SamplingMethod::RandomizedHalton,
        ..ArithmeticAsianCallConfig::default()
    };

    let r1 = arithmetic_asian_call_price_mc_cpu(&cfg);
    let r2 = arithmetic_asian_call_price_mc_cpu(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn arithmetic_asian_latin_hypercube_is_deterministic_for_same_seed() {
    let cfg = ArithmeticAsianCallConfig {
        n_paths: 40_000,
        n_steps: 32,
        seed: 916,
        sampling: SamplingMethod::LatinHypercube,
        ..ArithmeticAsianCallConfig::default()
    };

    let r1 = arithmetic_asian_call_price_mc_cpu(&cfg);
    let r2 = arithmetic_asian_call_price_mc_cpu(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn arithmetic_asian_scrambled_sobol_brownian_bridge_is_deterministic_for_same_seed() {
    let cfg = ArithmeticAsianCallConfig {
        n_paths: 40_000,
        n_steps: 32,
        seed: 928,
        sampling: SamplingMethod::ScrambledSobolBrownianBridge,
        ..ArithmeticAsianCallConfig::default()
    };

    let r1 = arithmetic_asian_call_price_mc_cpu(&cfg);
    let r2 = arithmetic_asian_call_price_mc_cpu(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn down_and_out_call_is_deterministic_for_same_seed() {
    let cfg = DownAndOutCallConfig {
        n_paths: 50_000,
        n_steps: 64,
        seed: 907,
        n_threads: 4,
        ..DownAndOutCallConfig::default()
    };

    let r1 = down_and_out_call_price_mc_cpu(&cfg);
    let r2 = down_and_out_call_price_mc_cpu(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn down_and_out_call_randomized_halton_is_deterministic_for_same_seed() {
    let cfg = DownAndOutCallConfig {
        n_paths: 40_000,
        n_steps: 32,
        seed: 908,
        sampling: SamplingMethod::RandomizedHalton,
        ..DownAndOutCallConfig::default()
    };

    let r1 = down_and_out_call_price_mc_cpu(&cfg);
    let r2 = down_and_out_call_price_mc_cpu(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn down_and_out_call_latin_hypercube_is_deterministic_for_same_seed() {
    let cfg = DownAndOutCallConfig {
        n_paths: 40_000,
        n_steps: 32,
        seed: 918,
        sampling: SamplingMethod::LatinHypercube,
        ..DownAndOutCallConfig::default()
    };

    let r1 = down_and_out_call_price_mc_cpu(&cfg);
    let r2 = down_and_out_call_price_mc_cpu(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn down_and_out_call_scrambled_sobol_brownian_bridge_is_deterministic_for_same_seed() {
    let cfg = DownAndOutCallConfig {
        n_paths: 40_000,
        n_steps: 32,
        seed: 929,
        sampling: SamplingMethod::ScrambledSobolBrownianBridge,
        ..DownAndOutCallConfig::default()
    };

    let r1 = down_and_out_call_price_mc_cpu(&cfg);
    let r2 = down_and_out_call_price_mc_cpu(&cfg);
    assert_eq!(r1, r2);
}

#[test]
fn down_and_out_call_outputs_sane_values() {
    let cfg = DownAndOutCallConfig {
        n_paths: 30_000,
        n_steps: 64,
        seed: 909,
        ..DownAndOutCallConfig::default()
    };

    let result = down_and_out_call_price_mc_cpu(&cfg);
    assert!(result.price >= 0.0);
    assert!(result.stderr >= 0.0);
    assert!(result.price < cfg.s0 * 2.0);
}

#[test]
fn down_and_out_pricer_builder_supports_expressive_configuration() {
    let result = DownAndOutCallPricer::new()
        .s0(100.0)
        .strike(100.0)
        .barrier(80.0)
        .rate(0.03)
        .volatility(0.2)
        .maturity(1.0)
        .paths(50_000)
        .steps(64)
        .seed(910)
        .control_variate()
        .latin_hypercube()
        .price();

    assert!(result.price >= 0.0);
    assert!(result.stderr >= 0.0);
}

#[test]
fn lookback_call_is_deterministic_for_same_seed() {
    let cfg = LookbackCallConfig {
        n_paths: 50_000,
        n_steps: 64,
        seed: 12_345,
        ..LookbackCallConfig::default()
    };

    let r1 = lookback_call_price_mc_cpu(&cfg);
    let r2 = lookback_call_price_mc_cpu(&cfg);

    assert_eq!(r1, r2);
}

#[test]
fn lookback_call_structured_sampling_is_deterministic() {
    let cfg = LookbackCallConfig {
        n_paths: 4_096,
        n_steps: 32,
        seed: 12_346,
        sampling: SamplingMethod::ScrambledSobolBrownianBridge,
        ..LookbackCallConfig::default()
    };

    let r1 = lookback_call_price_mc_cpu(&cfg);
    let r2 = lookback_call_price_mc_cpu(&cfg);

    assert_eq!(r1, r2);
}

#[test]
fn lookback_call_outputs_sane_values() {
    let cfg = LookbackCallConfig {
        n_paths: 80_000,
        n_steps: 64,
        seed: 12_347,
        ..LookbackCallConfig::default()
    };

    let lookback = lookback_call_price_mc_cpu(&cfg);
    let european = european_call_price_mc_cpu_stepwise(&EuropeanCallConfig {
        n_paths: cfg.n_paths,
        n_steps: cfg.n_steps,
        seed: cfg.seed,
        ..EuropeanCallConfig::default()
    });

    assert!(lookback.price >= european.price);
    assert!(lookback.stderr >= 0.0);
    assert!(lookback.price.is_finite());
}

#[test]
fn lookback_pricer_builder_supports_expressive_configuration() {
    let result = LookbackCallPricer::new()
        .s0(100.0)
        .strike(100.0)
        .rate(0.03)
        .volatility(0.2)
        .maturity(1.0)
        .paths(20_000)
        .steps(32)
        .seed(12_348)
        .control_variate()
        .latin_hypercube()
        .price();

    assert!(result.price >= 0.0);
    assert!(result.stderr >= 0.0);
}

#[test]
fn american_put_lsm_is_deterministic_for_same_seed() {
    let cfg = AmericanPutConfig {
        n_paths: 30_000,
        n_steps: 48,
        seed: 14_001,
        ..AmericanPutConfig::default()
    };

    let r1 = american_put_price_lsm_cpu(&cfg);
    let r2 = american_put_price_lsm_cpu(&cfg);

    assert_eq!(r1, r2);
    assert_eq!(r1.workload, PricingWorkloadFamily::AmericanPut);
    assert_eq!(r1.regression_basis, "laguerre_lsm_degree_2");
}

#[test]
fn american_put_lsm_has_early_exercise_premium_over_european_put() {
    let cfg = AmericanPutConfig {
        n_paths: 80_000,
        n_steps: 64,
        seed: 14_002,
        ..AmericanPutConfig::default()
    };

    let american = american_put_price_lsm_cpu(&cfg);
    let european = black_scholes_european_put_price(cfg.s0, cfg.k, cfg.r, cfg.sigma, cfg.t);

    assert!(american.price.is_finite());
    assert!(american.stderr >= 0.0);
    assert!(american.price + 3.0 * american.stderr >= european);
    assert!(american.price <= cfg.k);
    assert!(american.early_exercise_count > 0);
    assert_eq!(american.regression_steps, cfg.n_steps - 1);
    assert!(american
        .warnings
        .iter()
        .any(|warning| warning.contains("Longstaff-Schwartz")));
}

#[test]
fn american_put_pricer_builder_supports_expressive_configuration() {
    let result = AmericanPutPricer::new()
        .s0(100.0)
        .strike(100.0)
        .rate(0.03)
        .volatility(0.2)
        .maturity(1.0)
        .paths(20_000)
        .steps(32)
        .seed(14_003)
        .basis_degree(2)
        .price();

    assert!(result.price > 0.0);
    assert!(result.stderr >= 0.0);
    assert_eq!(result.regression_basis, "laguerre_lsm_degree_2");
}

#[test]
fn heston_european_call_is_deterministic_for_same_seed() {
    let cfg = HestonEuropeanCallConfig {
        n_paths: 50_000,
        n_steps: 64,
        seed: 13_001,
        ..HestonEuropeanCallConfig::default()
    };

    let r1 = heston_european_call_price_mc_cpu(&cfg);
    let r2 = heston_european_call_price_mc_cpu(&cfg);

    assert_eq!(r1, r2);
}

#[test]
fn heston_black_scholes_limit_matches_reference_price() {
    let sigma = 0.2;
    let cfg = HestonEuropeanCallConfig {
        v0: sigma * sigma,
        theta: sigma * sigma,
        vol_of_vol: 0.0,
        rho: 0.0,
        n_paths: 150_000,
        n_steps: 64,
        seed: 13_002,
        n_threads: 4,
        ..HestonEuropeanCallConfig::default()
    };

    let comparison = compare_heston_black_scholes_limit_cpu(&cfg);

    assert_eq!(
        comparison.reference_name,
        "black_scholes_vol_of_vol_zero_limit"
    );
    assert!(comparison.abs_error <= comparison.stderr * 4.0 + 0.05);
    assert!(comparison.error_stderr_units.abs() <= 4.0 || comparison.abs_error <= 0.05);
}

#[test]
fn heston_pricer_builder_supports_expressive_configuration() {
    let result = HestonEuropeanCallPricer::new()
        .s0(100.0)
        .strike(100.0)
        .rate(0.03)
        .initial_variance(0.04)
        .long_run_variance(0.04)
        .mean_reversion(1.5)
        .vol_of_vol(0.3)
        .correlation(-0.6)
        .maturity(1.0)
        .paths(20_000)
        .steps(32)
        .seed(13_003)
        .control_variate()
        .price();

    assert!(result.price >= 0.0);
    assert!(result.stderr >= 0.0);
    assert!(result.price.is_finite());
}

#[test]
fn black_scholes_greeks_are_sane_and_structured() {
    let greeks = black_scholes_european_call_greeks(100.0, 100.0, 0.03, 0.2, 1.0);

    assert!(greeks.delta > 0.55 && greeks.delta < 0.65);
    assert!(greeks.vega > 35.0 && greeks.vega < 45.0);
    assert!(greeks.rho > 45.0 && greeks.rho < 55.0);
    assert!(greeks.theta < 0.0);
}

#[test]
fn european_bump_greeks_are_close_to_black_scholes_references() {
    let cfg = EuropeanCallConfig {
        n_paths: 200_000,
        n_steps: 64,
        seed: 14_001,
        ..EuropeanCallConfig::default()
    };

    let report = european_call_greeks_cpu(&cfg, GreekEstimator::BumpAndRevalue);
    let analytic = black_scholes_european_call_greeks(cfg.s0, cfg.k, cfg.r, cfg.sigma, cfg.t);

    assert_eq!(report.workload, PricingWorkloadFamily::EuropeanCall);
    assert_eq!(report.estimator, GreekEstimator::BumpAndRevalue);
    assert!(report.estimated_runtime_ms.is_none());

    let delta = report
        .estimate(Greek::Delta)
        .expect("Delta estimate should be present");
    let vega = report
        .estimate(Greek::Vega)
        .expect("Vega estimate should be present");
    let rho = report
        .estimate(Greek::Rho)
        .expect("Rho estimate should be present");
    let theta = report
        .estimate(Greek::Theta)
        .expect("Theta estimate should be present");

    assert!((delta.value - analytic.delta).abs() < 0.03);
    assert!((vega.value - analytic.vega).abs() < 2.0);
    assert!((rho.value - analytic.rho).abs() < 3.0);
    assert!((theta.value - analytic.theta).abs() < 2.0);
    assert!(delta.stderr.unwrap() >= 0.0);
}

#[test]
fn european_pathwise_and_likelihood_ratio_greeks_report_supported_estimators() {
    let cfg = EuropeanCallConfig {
        n_paths: 150_000,
        n_steps: 64,
        seed: 14_002,
        ..EuropeanCallConfig::default()
    };

    let pathwise = european_call_greeks_cpu(&cfg, GreekEstimator::Pathwise);
    let likelihood_ratio = european_call_greeks_cpu(&cfg, GreekEstimator::LikelihoodRatio);
    let analytic = black_scholes_european_call_greeks(cfg.s0, cfg.k, cfg.r, cfg.sigma, cfg.t);

    let pathwise_delta = pathwise
        .estimate(Greek::Delta)
        .expect("pathwise Delta should be present");
    let pathwise_vega = pathwise
        .estimate(Greek::Vega)
        .expect("pathwise Vega should be present");
    let lr_delta = likelihood_ratio
        .estimate(Greek::Delta)
        .expect("likelihood-ratio Delta should be present");

    assert!((pathwise_delta.value - analytic.delta).abs() < 0.03);
    assert!((pathwise_vega.value - analytic.vega).abs() < 2.5);
    assert!((lr_delta.value - analytic.delta).abs() < 0.04);
    assert!(pathwise_delta.stderr.unwrap() > 0.0);
    assert!(lr_delta.stderr.unwrap() > 0.0);
    assert!(likelihood_ratio
        .warnings
        .iter()
        .any(|warning| warning.contains("Delta")));
}

#[test]
fn bump_and_revalue_greeks_cover_current_cpu_workloads() {
    let report = price_all_current_greeks_bump_and_revalue_cpu(20_000, 32, 14_003);

    assert_eq!(report.len(), 6);
    for workload_report in &report {
        assert_eq!(workload_report.estimator, GreekEstimator::BumpAndRevalue);
        assert!(
            !workload_report.estimates.is_empty(),
            "{:?} should contain at least one Greek estimate",
            workload_report.workload
        );
        assert!(
            workload_report
                .estimates
                .iter()
                .all(|estimate| estimate.value.is_finite()),
            "{:?} should contain finite Greek values",
            workload_report.workload
        );
    }
}

#[test]
fn heston_black_scholes_limit_greeks_match_reference() {
    let sigma = 0.2;
    let cfg = HestonEuropeanCallConfig {
        v0: sigma * sigma,
        theta: sigma * sigma,
        vol_of_vol: 0.0,
        rho: 0.0,
        n_paths: 160_000,
        n_steps: 64,
        seed: 14_004,
        ..HestonEuropeanCallConfig::default()
    };

    let report = heston_european_call_greeks_cpu(&cfg, GreekEstimator::BumpAndRevalue);
    let analytic = black_scholes_european_call_greeks(cfg.s0, cfg.k, cfg.r, sigma, cfg.t);
    let delta = report
        .estimate(Greek::Delta)
        .expect("Heston Black-Scholes-limit Delta should be present");
    let rho = report
        .estimate(Greek::Rho)
        .expect("Heston Black-Scholes-limit Rho should be present");

    assert!((delta.value - analytic.delta).abs() < 0.04);
    assert!((rho.value - analytic.rho).abs() < 4.0);
}

#[test]
fn method_capability_catalog_exposes_supported_and_planned_methods() {
    let capabilities = monte_carlo_method_capabilities();

    let latin = capabilities
        .iter()
        .find(|capability| capability.method_id == "latin_hypercube")
        .expect("latin hypercube should be listed");
    assert_eq!(latin.cpu_native, BackendMethodSupport::CpuReference);
    assert_eq!(
        latin.apple_metal,
        BackendMethodSupport::DelegatedCpuFallback
    );

    let sobol = capabilities
        .iter()
        .find(|capability| capability.method_id == "scrambled_sobol")
        .expect("scrambled Sobol should be listed");
    assert_eq!(sobol.cpu_native, BackendMethodSupport::CpuReference);
    assert_eq!(
        sobol.apple_metal,
        BackendMethodSupport::DelegatedCpuFallback
    );
    assert_eq!(
        sobol.nvidia_cuda,
        BackendMethodSupport::DelegatedCpuFallback
    );

    let bridge = capabilities
        .iter()
        .find(|capability| capability.method_id == "scrambled_sobol_brownian_bridge")
        .expect("scrambled Sobol Brownian bridge should be listed");
    assert_eq!(bridge.cpu_native, BackendMethodSupport::CpuReference);

    let mlmc = capabilities
        .iter()
        .find(|capability| capability.method_id == "multilevel_monte_carlo")
        .expect("MLMC should be listed");
    assert_eq!(mlmc.cpu_native, BackendMethodSupport::CpuReference);
    assert!(mlmc
        .notes
        .iter()
        .any(|note| note.contains("arithmetic Asian")));

    let mlqmc = capabilities
        .iter()
        .find(|capability| capability.method_id == "multilevel_randomized_qmc")
        .expect("MLQMC should be listed");
    assert_eq!(mlqmc.cpu_native, BackendMethodSupport::CpuReference);

    let lsm = capabilities
        .iter()
        .find(|capability| capability.method_id == "longstaff_schwartz_lsm")
        .expect("Longstaff-Schwartz should be listed");
    assert_eq!(lsm.cpu_native, BackendMethodSupport::CpuReference);
    assert!(lsm.notes.iter().any(|note| note.contains("American puts")));

    let greeks = capabilities
        .iter()
        .find(|capability| capability.method_id == "bump_and_revalue_greeks")
        .expect("bump-and-revalue Greeks should be listed");
    assert_eq!(greeks.cpu_native, BackendMethodSupport::CpuReference);
    assert!(greeks.notes.iter().any(|note| note.contains("Heston")));
}

fn black_scholes_call(s0: f64, k: f64, r: f64, sigma: f64, t: f64) -> f64 {
    let sqrt_t = t.sqrt();
    let d1 = ((s0 / k).ln() + (r + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    s0 * normal_cdf(d1) - k * (-r * t).exp() * normal_cdf(d2)
}

fn normal_cdf(x: f64) -> f64 {
    // Abramowitz-Stegun style erf approximation.
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let z = x.abs() / 2f64.sqrt();
    let t = 1.0 / (1.0 + 0.3275911 * z);
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let poly = (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t;
    let erf_approx = 1.0 - poly * (-z * z).exp();
    0.5 * (1.0 + sign * erf_approx)
}
