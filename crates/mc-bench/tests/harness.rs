use mc_bench::{build_competitiveness_plan, run_compact_benchmarks};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

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
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("mc-bench should live under crates/")
        .to_path_buf();

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
