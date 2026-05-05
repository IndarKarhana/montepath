use mc_bench::{build_competitiveness_plan, run_compact_benchmarks};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("mc-bench should live under crates/")
        .to_path_buf()
}

#[test]
fn benchmark_harness_produces_non_empty_results() {
    let report = run_compact_benchmarks();
    assert!(
        !report.results.is_empty(),
        "expected benchmark results to be non-empty"
    );
}

#[test]
fn benchmark_metrics_are_non_negative() {
    let report = run_compact_benchmarks();
    for result in &report.results {
        assert!(result.total_runtime_ms >= 0.0);
        assert!(result.per_iteration_us >= 0.0);
        assert!(result.throughput_per_sec >= 0.0);
    }
}

#[test]
fn planner_accuracy_benchmark_has_accuracy_metric() {
    let report = run_compact_benchmarks();
    let planner_accuracy = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "planner_choice_accuracy")
        .expect("planner choice accuracy benchmark should be present");

    assert_eq!(
        planner_accuracy.metric_name.as_deref(),
        Some("accuracy_pct")
    );
    let accuracy = planner_accuracy
        .metric_value
        .expect("planner accuracy metric should contain a value");
    assert!(accuracy >= 0.0);
    assert!(accuracy <= 100.0);
}

#[test]
fn competitiveness_plan_is_generated() {
    let report = run_compact_benchmarks();
    let plan = build_competitiveness_plan(&report);
    assert!(plan.contains("Competitiveness Plan"));
    assert!(plan.contains("Action plan") || plan.contains("Maintain lead plan"));
}

#[test]
fn python_competitor_script_reports_quantlib_lane() {
    let repo_root = repo_root();

    let output = Command::new("python3")
        .arg("benchmarks/competitors/python_cpu_baselines.py")
        .arg("--paths")
        .arg("16")
        .arg("--steps")
        .arg("2")
        .arg("--repeats")
        .arg("1")
        .arg("--seed")
        .arg("7")
        .current_dir(repo_root)
        .output()
        .expect("python competitor baseline script should run");

    assert!(
        output.status.success(),
        "python competitor baseline script failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value =
        serde_json::from_slice(&output.stdout).expect("script should emit JSON payload");
    let results = payload["results"]
        .as_array()
        .expect("payload should contain results array");

    let quantlib = results
        .iter()
        .find(|entry| entry["library"].as_str() == Some("quantlib"))
        .expect("QuantLib lane should be reported as available or unavailable");

    assert_eq!(
        quantlib["methodology"].as_str(),
        Some("stepwise_paths_quantlib_mceuropean")
    );

    let quantlib_lookback = results
        .iter()
        .find(|entry| {
            entry["library"].as_str() == Some("quantlib")
                && entry["methodology"].as_str()
                    == Some("lookback_fixed_strike_stepwise_quantlib_mc")
        })
        .expect("QuantLib lookback lane should be reported as available or unavailable");

    assert!(quantlib_lookback["available"].is_boolean());

    let quantlib_heston = results
        .iter()
        .find(|entry| {
            entry["library"].as_str() == Some("quantlib")
                && entry["methodology"].as_str() == Some("heston_analytic_reference_quantlib")
        })
        .expect("QuantLib Heston lane should be reported as available or unavailable");

    assert!(quantlib_heston["available"].is_boolean());
}

#[test]
fn phase2_capability_catalog_is_machine_readable_and_complete() {
    let catalog_path = repo_root().join("docs/product-model-capability-catalog.json");
    let catalog: Value = serde_json::from_slice(
        &fs::read(&catalog_path).expect("phase 2 capability catalog should exist"),
    )
    .expect("phase 2 capability catalog should be valid JSON");

    assert_eq!(catalog["schema_version"].as_str(), Some("phase2.v1"));
    let entries = catalog["workloads"]
        .as_array()
        .expect("catalog should contain workloads array");

    for workload_id in [
        "european_call_gbm",
        "arithmetic_asian_call_gbm",
        "down_and_out_call_gbm",
        "fixed_strike_lookback_call_gbm",
        "two_asset_basket_call_gbm",
        "heston_european_call",
        "gaussian_uncertainty_mean",
    ] {
        let entry = entries
            .iter()
            .find(|entry| entry["workload_id"].as_str() == Some(workload_id))
            .unwrap_or_else(|| panic!("missing workload capability entry for {workload_id}"));

        assert!(
            entry["assumptions"]
                .as_array()
                .is_some_and(|items| !items.is_empty()),
            "{workload_id} should document assumptions"
        );
        assert!(
            entry["unsupported_states"]
                .as_array()
                .is_some_and(|items| !items.is_empty()),
            "{workload_id} should document unsupported states"
        );
        assert!(
            entry["reference_status"].as_str().is_some(),
            "{workload_id} should document reference status"
        );
    }

    let european = entries
        .iter()
        .find(|entry| entry["workload_id"].as_str() == Some("european_call_gbm"))
        .expect("European capability entry should exist");
    assert_eq!(
        european["greek_estimators"]["pathwise"]["status"].as_str(),
        Some("supported")
    );
    assert_eq!(
        european["greek_estimators"]["likelihood_ratio"]["status"].as_str(),
        Some("supported")
    );

    for workload_id in [
        "arithmetic_asian_call_gbm",
        "down_and_out_call_gbm",
        "fixed_strike_lookback_call_gbm",
        "two_asset_basket_call_gbm",
        "heston_european_call",
    ] {
        let entry = entries
            .iter()
            .find(|entry| entry["workload_id"].as_str() == Some(workload_id))
            .expect("Greek workload entry should exist");
        assert_eq!(
            entry["greek_estimators"]["bump_and_revalue"]["status"].as_str(),
            Some("supported"),
            "{workload_id} should support bump-and-revalue Greeks"
        );
        assert_eq!(
            entry["greek_estimators"]["pathwise"]["status"].as_str(),
            Some("unsupported"),
            "{workload_id} should explicitly reject pathwise Greeks for now"
        );
        assert_eq!(
            entry["greek_estimators"]["likelihood_ratio"]["status"].as_str(),
            Some("unsupported"),
            "{workload_id} should explicitly reject likelihood-ratio Greeks for now"
        );
    }
}

#[test]
fn phase2_reference_fixtures_are_registered() {
    let fixture_path = repo_root().join("benchmarks/reference-fixtures.json");
    let fixtures: Value = serde_json::from_slice(
        &fs::read(&fixture_path).expect("phase 2 reference fixture registry should exist"),
    )
    .expect("phase 2 reference fixture registry should be valid JSON");

    assert_eq!(fixtures["schema_version"].as_str(), Some("phase2.v1"));
    let entries = fixtures["fixtures"]
        .as_array()
        .expect("fixture registry should contain fixtures array");

    for fixture_id in [
        "black_scholes_european_call_atm_1y",
        "black_scholes_european_call_greeks_atm_1y",
        "heston_black_scholes_limit_atm_1y",
        "gaussian_uncertainty_mean_reference",
    ] {
        let entry = entries
            .iter()
            .find(|entry| entry["fixture_id"].as_str() == Some(fixture_id))
            .unwrap_or_else(|| panic!("missing reference fixture {fixture_id}"));

        assert!(
            entry["reference_source"].as_str().is_some(),
            "{fixture_id} should document its reference source"
        );
        assert!(
            entry["comparison_metric"].as_str().is_some(),
            "{fixture_id} should document its comparison metric"
        );
    }
}

#[test]
fn quantlib_benchmark_environment_is_declared_for_ci() {
    let root = repo_root();
    let requirements =
        fs::read_to_string(root.join("benchmarks/competitors/requirements-quantlib.txt"))
            .expect("QuantLib competitor requirements should exist");
    assert!(
        requirements
            .lines()
            .any(|line| line.starts_with("QuantLib")),
        "QuantLib benchmark requirements should install QuantLib-Python"
    );

    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml"))
        .expect("CI workflow should exist");
    assert!(
        ci.contains("quantlib-benchmark"),
        "CI should declare a QuantLib-enabled benchmark artifact job"
    );
    assert!(
        ci.contains("benchmarks/quantlib-ci-results.json"),
        "CI should produce a QuantLib-populated benchmark artifact path"
    );
    assert!(
        ci.contains("QuantLib environment did not produce any available competitor rows"),
        "CI should gate the QuantLib environment without failing on explicit per-lane unavailable rows"
    );
    assert!(
        !ci.contains("QuantLib rows unavailable"),
        "CI should not fail merely because one QuantLib instrument API is unavailable"
    );
}

#[test]
fn accelerator_competitor_lanes_report_telemetry_or_explicit_unavailability() {
    let repo_root = repo_root();

    let output = Command::new("python3")
        .arg("benchmarks/competitors/python_cpu_baselines.py")
        .arg("--paths")
        .arg("16")
        .arg("--steps")
        .arg("2")
        .arg("--repeats")
        .arg("1")
        .arg("--seed")
        .arg("7")
        .current_dir(repo_root)
        .output()
        .expect("python competitor baseline script should run");

    assert!(
        output.status.success(),
        "python competitor baseline script failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value =
        serde_json::from_slice(&output.stdout).expect("script should emit JSON payload");
    let results = payload["results"]
        .as_array()
        .expect("payload should contain results array");

    for library in ["jax", "cupy", "torch"] {
        let row = results
            .iter()
            .find(|entry| entry["library"].as_str() == Some(library))
            .unwrap_or_else(|| panic!("{library} accelerator lane should be reported"));

        assert!(
            row["available"].is_boolean(),
            "{library} should report availability explicitly"
        );
        assert!(
            row["methodology"].as_str().is_some(),
            "{library} should report methodology explicitly"
        );
        assert!(
            row["telemetry"].is_object(),
            "{library} should report telemetry object even when unavailable"
        );
        assert!(
            row["reproducibility"].as_str().is_some(),
            "{library} should report reproducibility notes"
        );
    }
}

#[test]
fn competitor_environment_manifests_cover_all_phase5_libraries() {
    let root = repo_root();
    let manifest_dir = root.join("benchmarks/competitors/environments");
    for name in [
        "numpy",
        "numba",
        "scipy-qmc",
        "quantlib",
        "jax",
        "cupy",
        "pytorch",
    ] {
        let path = manifest_dir.join(format!("{name}.json"));
        let payload: Value =
            serde_json::from_slice(&fs::read(&path).unwrap_or_else(|_| panic!("missing {path:?}")))
                .unwrap_or_else(|_| panic!("{path:?} should contain valid JSON"));

        assert_eq!(
            payload["schema_version"].as_str(),
            Some("competitor-env.v1"),
            "{name} manifest should use the competitor environment schema"
        );
        assert!(
            payload["install"]
                .as_array()
                .is_some_and(|items| !items.is_empty()),
            "{name} manifest should document install commands"
        );
        assert!(
            payload["hardware"].as_str().is_some(),
            "{name} manifest should document hardware expectations"
        );
    }
}

#[test]
fn rust_mc_benchmark_is_present() {
    let report = run_compact_benchmarks();
    let rust_mc = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_european_call_rust")
        .expect("rust MC benchmark should be present");
    assert!(rust_mc.total_runtime_ms > 0.0);
    assert_eq!(rust_mc.methodology.as_deref(), Some("stepwise_paths"));

    let rust_terminal = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_european_call_rust_terminal")
        .expect("terminal MC benchmark should be present");
    assert!(rust_terminal.total_runtime_ms > 0.0);
    assert_eq!(
        rust_terminal.methodology.as_deref(),
        Some("terminal_distribution")
    );

    let antithetic_quality = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_european_call_rust_antithetic_quality")
        .expect("antithetic quality benchmark should be present");
    assert_eq!(
        antithetic_quality.metric_name.as_deref(),
        Some("stderr_ratio_vs_standard")
    );

    let qmc_pricing_quality = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_qmc_quality_european_scrambled_sobol")
        .expect("QMC pricing quality benchmark should be present");
    assert_eq!(
        qmc_pricing_quality.metric_name.as_deref(),
        Some("stderr_ratio_vs_pseudorandom")
    );

    let realized_error = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_qmc_realized_error_european_latin_hypercube")
        .expect("European realized-error benchmark should be present");
    assert_eq!(
        realized_error.metric_name.as_deref(),
        Some("abs_error_ratio_vs_pseudorandom")
    );

    let basket = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_basket_call_rust_scrambled_sobol")
        .expect("basket benchmark should be present");
    assert!(basket.total_runtime_ms > 0.0);
    assert_eq!(basket.metric_name.as_deref(), Some("price_estimate"));

    let basket_quality = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_qmc_quality_basket_latin_hypercube")
        .expect("basket QMC pricing quality benchmark should be present");
    assert_eq!(
        basket_quality.metric_name.as_deref(),
        Some("stderr_ratio_vs_pseudorandom")
    );

    let lookback = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_lookback_call_rust")
        .expect("lookback benchmark should be present");
    assert!(lookback.total_runtime_ms > 0.0);
    assert_eq!(lookback.metric_name.as_deref(), Some("price_estimate"));

    let lookback_quality = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_qmc_quality_lookback_latin_hypercube")
        .expect("lookback QMC pricing quality benchmark should be present");
    assert_eq!(
        lookback_quality.metric_name.as_deref(),
        Some("stderr_ratio_vs_pseudorandom")
    );

    let heston = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_heston_european_call_rust")
        .expect("Heston benchmark should be present");
    assert!(heston.total_runtime_ms > 0.0);
    assert_eq!(heston.metric_name.as_deref(), Some("price_estimate"));

    let heston_quality = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_heston_black_scholes_limit_quality")
        .expect("Heston Black-Scholes-limit quality benchmark should be present");
    assert_eq!(
        heston_quality.metric_name.as_deref(),
        Some("abs_error_vs_black_scholes")
    );

    let greek_bump = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_european_call_greeks_bump_rust")
        .expect("European bump-and-revalue Greek benchmark should be present");
    assert_eq!(
        greek_bump.metric_name.as_deref(),
        Some("abs_delta_error_vs_black_scholes")
    );

    let greek_pathwise = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_european_call_greeks_pathwise_rust")
        .expect("European pathwise Greek benchmark should be present");
    assert_eq!(
        greek_pathwise.metric_name.as_deref(),
        Some("abs_delta_error_vs_black_scholes")
    );

    let greek_lr = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_european_call_greeks_likelihood_ratio_rust")
        .expect("European likelihood-ratio Greek benchmark should be present");
    assert_eq!(
        greek_lr.metric_name.as_deref(),
        Some("abs_delta_error_vs_black_scholes")
    );

    let all_greeks = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_all_workload_greeks_bump_rust")
        .expect("all-workload Greek breadth benchmark should be present");
    assert_eq!(
        all_greeks.metric_name.as_deref(),
        Some("greek_estimate_count")
    );

    let uq = report
        .results
        .iter()
        .find(|r| r.benchmark_name == "mc_cpu_gaussian_uncertainty_rust_scrambled_sobol")
        .expect("Gaussian uncertainty benchmark should be present");
    assert_eq!(
        uq.metric_name.as_deref(),
        Some("abs_error_vs_analytic_mean")
    );
}
