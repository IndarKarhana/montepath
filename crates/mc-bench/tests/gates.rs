use mc_bench::run_compact_benchmarks;

fn find_metric<'a>(
    name: &str,
    report: &'a mc_bench::BenchmarkReport,
) -> &'a mc_bench::BenchmarkResult {
    report
        .results
        .iter()
        .find(|r| r.benchmark_name == name)
        .unwrap_or_else(|| panic!("missing benchmark result '{name}'"))
}

#[test]
fn benchmark_gates_hold_for_current_internal_suite() {
    let report = run_compact_benchmarks();

    let schema_validation = find_metric("schema_validation", &report);
    assert!(
        schema_validation.per_iteration_us < 100.0,
        "schema_validation gate failed: per_iteration_us={} expected<100",
        schema_validation.per_iteration_us
    );

    let planner_overhead = find_metric("planner_overhead_auto", &report);
    assert!(
        planner_overhead.per_iteration_us < 10.0,
        "planner_overhead_auto gate failed: per_iteration_us={} expected<10",
        planner_overhead.per_iteration_us
    );

    let planner_accuracy = find_metric("planner_choice_accuracy", &report);
    let accuracy = planner_accuracy
        .metric_value
        .expect("planner choice accuracy benchmark must contain metric_value");
    assert!(
        accuracy >= 75.0,
        "planner_choice_accuracy gate failed: accuracy_pct={} expected>=75",
        accuracy
    );

    let measured_planner_accuracy = find_metric("planner_choice_accuracy_measured", &report);
    let measured_accuracy = measured_planner_accuracy
        .metric_value
        .expect("measured planner choice accuracy benchmark must contain metric_value");
    assert!(
        measured_accuracy >= 75.0,
        "planner_choice_accuracy_measured gate failed: accuracy_pct={} expected>=75",
        measured_accuracy
    );

    let rust_mc = find_metric("mc_cpu_european_call_rust", &report);
    assert!(
        rust_mc.total_runtime_ms > 0.0,
        "mc_cpu_european_call_rust gate failed: expected benchmark presence and positive runtime"
    );
    assert_eq!(rust_mc.methodology.as_deref(), Some("stepwise_paths"));

    if let Some(numpy) = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_european_call_numpy")
    {
        assert!(
            rust_mc.per_iteration_us < numpy.per_iteration_us,
            "competitiveness gate failed: rust per_iteration_us={} numpy per_iteration_us={}",
            rust_mc.per_iteration_us,
            numpy.per_iteration_us
        );
    }

    if let Some(numba) = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_european_call_numba")
    {
        assert!(
            rust_mc.per_iteration_us < numba.per_iteration_us,
            "competitiveness gate failed: rust per_iteration_us={} numba per_iteration_us={}",
            rust_mc.per_iteration_us,
            numba.per_iteration_us
        );
    }

    let rust_terminal = find_metric("mc_cpu_european_call_rust_terminal", &report);
    assert!(
        rust_terminal.total_runtime_ms > 0.0,
        "mc_cpu_european_call_rust_terminal gate failed: expected benchmark presence and positive runtime"
    );

    let barrier = find_metric("mc_cpu_down_and_out_call_rust", &report);
    assert!(
        barrier.total_runtime_ms > 0.0,
        "mc_cpu_down_and_out_call_rust gate failed: expected benchmark presence and positive runtime"
    );

    let lookback = find_metric("mc_cpu_lookback_call_rust", &report);
    assert!(
        lookback.total_runtime_ms > 0.0,
        "mc_cpu_lookback_call_rust gate failed: expected benchmark presence and positive runtime"
    );

    let american_put = find_metric("mc_cpu_american_put_lsm_rust", &report);
    assert!(
        american_put.total_runtime_ms > 0.0,
        "mc_cpu_american_put_lsm_rust gate failed: expected benchmark presence and positive runtime"
    );
    assert_eq!(
        american_put.methodology.as_deref(),
        Some("american_put_longstaff_schwartz_laguerre")
    );

    let heston = find_metric("mc_cpu_heston_european_call_rust", &report);
    assert!(
        heston.total_runtime_ms > 0.0,
        "mc_cpu_heston_european_call_rust gate failed: expected benchmark presence and positive runtime"
    );
    assert_eq!(heston.metric_name.as_deref(), Some("price_estimate"));

    let heston_quality = find_metric("mc_cpu_heston_black_scholes_limit_quality", &report);
    assert_eq!(
        heston_quality.metric_name.as_deref(),
        Some("abs_error_vs_black_scholes")
    );
    let heston_abs_error = heston_quality
        .metric_value
        .expect("Heston Black-Scholes-limit benchmark must contain metric_value");
    assert!(
        heston_abs_error.is_finite() && heston_abs_error >= 0.0 && heston_abs_error < 0.5,
        "Heston Black-Scholes-limit gate failed: abs_error_vs_black_scholes={} expected finite in [0, 0.5)",
        heston_abs_error
    );

    for name in [
        "mc_cpu_european_call_greeks_bump_rust",
        "mc_cpu_european_call_greeks_pathwise_rust",
        "mc_cpu_european_call_greeks_likelihood_ratio_rust",
    ] {
        let greek = find_metric(name, &report);
        assert_eq!(
            greek.metric_name.as_deref(),
            Some("abs_delta_error_vs_black_scholes")
        );
        let abs_error = greek
            .metric_value
            .unwrap_or_else(|| panic!("{name} must contain metric_value"));
        assert!(
            abs_error.is_finite() && abs_error < 0.08,
            "{name} gate failed: abs_delta_error_vs_black_scholes={} expected<0.08",
            abs_error
        );
    }

    let all_greeks = find_metric("mc_cpu_all_workload_greeks_bump_rust", &report);
    assert_eq!(
        all_greeks.metric_name.as_deref(),
        Some("greek_estimate_count")
    );
    let greek_count = all_greeks
        .metric_value
        .expect("all-workload Greek benchmark must contain metric_value");
    assert!(
        greek_count >= 24.0,
        "all-workload Greek benchmark should cover at least 24 estimates, got {}",
        greek_count
    );

    let qmc_quality = find_metric(
        "mc_cpu_european_call_rust_randomized_halton_control_variate_quality",
        &report,
    );
    let qmc_stderr_ratio = qmc_quality
        .metric_value
        .expect("randomized halton quality benchmark must contain metric_value");
    assert!(
        qmc_stderr_ratio < 1.0,
        "randomized halton control-variate quality gate failed: stderr_ratio_vs_standard={} expected<1",
        qmc_stderr_ratio
    );

    let lhs = find_metric("mc_cpu_european_call_rust_latin_hypercube", &report);
    assert!(
        lhs.total_runtime_ms > 0.0,
        "mc_cpu_european_call_rust_latin_hypercube gate failed: expected benchmark presence and positive runtime"
    );

    let lhs_quality = find_metric(
        "mc_cpu_european_call_rust_latin_hypercube_control_variate_quality",
        &report,
    );
    let lhs_stderr_ratio = lhs_quality
        .metric_value
        .expect("latin hypercube quality benchmark must contain metric_value");
    assert!(
        lhs_stderr_ratio < 1.0,
        "latin hypercube control-variate quality gate failed: stderr_ratio_vs_standard={} expected<1",
        lhs_stderr_ratio
    );

    for name in [
        "mc_cpu_qmc_quality_european_scrambled_sobol",
        "mc_cpu_qmc_quality_arithmetic_asian_latin_hypercube",
        "mc_cpu_qmc_quality_down_and_out_randomized_halton",
        "mc_cpu_qmc_quality_lookback_latin_hypercube",
        "mc_cpu_qmc_quality_basket_latin_hypercube",
    ] {
        let quality = find_metric(name, &report);
        assert_eq!(
            quality.metric_name.as_deref(),
            Some("stderr_ratio_vs_pseudorandom")
        );
        let ratio = quality
            .metric_value
            .unwrap_or_else(|| panic!("{name} must contain metric_value"));
        assert!(
            ratio.is_finite() && ratio > 0.0,
            "{name} gate failed: stderr_ratio_vs_pseudorandom={} expected positive finite value",
            ratio
        );
    }

    let realized_error = find_metric(
        "mc_cpu_qmc_realized_error_european_latin_hypercube",
        &report,
    );
    assert_eq!(
        realized_error.metric_name.as_deref(),
        Some("abs_error_ratio_vs_pseudorandom")
    );
    let realized_error_ratio = realized_error
        .metric_value
        .expect("European realized-error benchmark must contain metric_value");
    assert!(
        realized_error_ratio.is_finite() && realized_error_ratio >= 0.0,
        "European realized-error gate failed: abs_error_ratio_vs_pseudorandom={} expected finite non-negative value",
        realized_error_ratio
    );

    let basket = find_metric("mc_cpu_basket_call_rust_scrambled_sobol", &report);
    assert!(
        basket.total_runtime_ms > 0.0,
        "mc_cpu_basket_call_rust_scrambled_sobol gate failed: expected benchmark presence and positive runtime"
    );
    assert_eq!(basket.metric_name.as_deref(), Some("price_estimate"));

    let uq = find_metric("mc_cpu_gaussian_uncertainty_rust_scrambled_sobol", &report);
    assert_eq!(
        uq.metric_name.as_deref(),
        Some("abs_error_vs_analytic_mean")
    );
    let uq_abs_error = uq
        .metric_value
        .expect("Gaussian uncertainty benchmark must contain metric_value");
    assert!(
        uq_abs_error < 0.05,
        "Gaussian uncertainty gate failed: abs_error_vs_analytic_mean={} expected<0.05",
        uq_abs_error
    );

    let antithetic_quality = find_metric("mc_cpu_european_call_rust_antithetic_quality", &report);
    let stderr_ratio = antithetic_quality
        .metric_value
        .expect("antithetic quality benchmark must contain metric_value");
    assert!(
        stderr_ratio < 1.0,
        "antithetic quality gate failed: stderr_ratio_vs_standard={} expected<1",
        stderr_ratio
    );
}
