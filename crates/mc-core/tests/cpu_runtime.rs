use mc_core::{european_call_price_mc_cpu, EuropeanCallConfig, MonteCarloRng};

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
