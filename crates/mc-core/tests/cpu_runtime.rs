use mc_core::{
    arithmetic_asian_call_price_mc_cpu, european_call_price_mc_cpu,
    european_call_price_mc_cpu_stepwise, european_call_price_mc_cpu_terminal,
    ArithmeticAsianCallConfig, ArithmeticAsianCallPricer, EuropeanCallConfig, EuropeanCallMethod,
    EuropeanCallPricer, MonteCarloRng, MonteCarloTechnique,
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
