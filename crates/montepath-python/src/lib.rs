#![allow(clippy::useless_conversion)]

use mc_core::{
    american_put_price_lsm_cpu, arithmetic_asian_call_price_mc_cpu,
    arithmetic_asian_call_price_mlmc_cpu, basket_call_price_mc_cpu, bermudan_put_price_lsm_cpu,
    down_and_out_call_price_mc_cpu, european_call_price_mc_cpu, gaussian_uncertainty_moments_cpu,
    heston_european_call_price_mc_cpu, lookback_call_price_mc_cpu,
    merton_jump_diffusion_call_price_mc_cpu, price_european_call_parameter_sweep_cpu,
    simulate_inventory_policy_cpu, validate_inventory_config as validate_inventory_config_rust,
    AmericanPutConfig, ArithmeticAsianCallConfig, ArithmeticAsianMlmcConfig, BasketCallConfig,
    BermudanPutConfig, DownAndOutCallConfig, EuropeanCallConfig, EuropeanCallParameterSweepConfig,
    EuropeanCallSweepScenario, GaussianUncertaintyConfig, HestonEuropeanCallConfig,
    InventoryConstraints, InventoryCostConfig, InventoryDemandDistribution, InventoryPolicy,
    InventoryShortagePolicy, InventorySimulationConfig, InventoryTraceConfig, LookbackCallConfig,
    MertonJumpDiffusionCallConfig, MonteCarloTechnique, SamplingMethod,
};
#[cfg(feature = "metal-native")]
use mc_core::{execute_apple_metal_native, BackendExecutionInput};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use serde::Serialize;
use serde_json::{json, Map, Value};

#[pyfunction]
fn price_european_call(py: Python<'_>, config: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let cfg = european_config(config)?;
    let result = european_call_price_mc_cpu(&cfg);
    pricing_response(
        py,
        "european_call",
        "price_european_call",
        &cfg,
        result,
        Vec::new(),
    )
}

#[pyfunction]
fn price_arithmetic_asian_call(py: Python<'_>, config: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let cfg = asian_config(config)?;
    let result = arithmetic_asian_call_price_mc_cpu(&cfg);
    pricing_response(
        py,
        "arithmetic_asian_call",
        "price_arithmetic_asian_call",
        &cfg,
        result,
        Vec::new(),
    )
}

#[pyfunction]
fn price_down_and_out_call(py: Python<'_>, config: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let cfg = down_and_out_config(config)?;
    let result = down_and_out_call_price_mc_cpu(&cfg);
    pricing_response(
        py,
        "down_and_out_call",
        "price_down_and_out_call",
        &cfg,
        result,
        Vec::new(),
    )
}

#[cfg(feature = "metal-native")]
#[pyfunction]
fn price_european_call_metal(py: Python<'_>, config: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let cfg = european_config(config)?;
    let output = execute_apple_metal_native(&BackendExecutionInput::EuropeanCall(cfg.clone()))
        .map_err(|err| PyValueError::new_err(format!("native Metal execution failed: {err}")))?;
    metal_pricing_response(
        py,
        "european_call",
        "price_european_call_metal",
        &cfg,
        output,
    )
}

#[cfg(feature = "metal-native")]
#[pyfunction]
fn price_arithmetic_asian_call_metal(
    py: Python<'_>,
    config: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let cfg = asian_config(config)?;
    let output =
        execute_apple_metal_native(&BackendExecutionInput::ArithmeticAsianCall(cfg.clone()))
            .map_err(|err| {
                PyValueError::new_err(format!("native Metal execution failed: {err}"))
            })?;
    metal_pricing_response(
        py,
        "arithmetic_asian_call",
        "price_arithmetic_asian_call_metal",
        &cfg,
        output,
    )
}

#[cfg(feature = "metal-native")]
#[pyfunction]
fn price_down_and_out_call_metal(py: Python<'_>, config: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let cfg = down_and_out_config(config)?;
    let output = execute_apple_metal_native(&BackendExecutionInput::DownAndOutCall(cfg.clone()))
        .map_err(|err| PyValueError::new_err(format!("native Metal execution failed: {err}")))?;
    metal_pricing_response(
        py,
        "down_and_out_call",
        "price_down_and_out_call_metal",
        &cfg,
        output,
    )
}

#[pyfunction]
fn price_lookback_call(py: Python<'_>, config: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let cfg = lookback_config(config)?;
    let result = lookback_call_price_mc_cpu(&cfg);
    pricing_response(
        py,
        "lookback_call",
        "price_lookback_call",
        &cfg,
        result,
        Vec::new(),
    )
}

#[pyfunction]
fn price_basket_call(py: Python<'_>, config: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let cfg = basket_config(config)?;
    let result = basket_call_price_mc_cpu(&cfg);
    pricing_response(
        py,
        "basket_call",
        "price_basket_call",
        &cfg,
        result,
        Vec::new(),
    )
}

#[pyfunction]
fn price_american_put(py: Python<'_>, config: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let cfg = american_put_config(config)?;
    let result = american_put_price_lsm_cpu(&cfg);
    pricing_response(
        py,
        "american_put",
        "price_american_put",
        &cfg,
        json!({"price": result.price, "stderr": result.stderr, "details": result}),
        result.warnings,
    )
}

#[pyfunction]
fn price_bermudan_put(py: Python<'_>, config: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let cfg = bermudan_put_config(config)?;
    let result = bermudan_put_price_lsm_cpu(&cfg);
    pricing_response(
        py,
        "bermudan_put",
        "price_bermudan_put",
        &cfg,
        json!({"price": result.price, "stderr": result.stderr, "details": result}),
        result.warnings,
    )
}

#[pyfunction]
fn price_heston_european_call(py: Python<'_>, config: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let cfg = heston_config(config)?;
    let result = heston_european_call_price_mc_cpu(&cfg);
    pricing_response(
        py,
        "heston_european_call",
        "price_heston_european_call",
        &cfg,
        result,
        Vec::new(),
    )
}

#[pyfunction]
fn price_merton_jump_diffusion_call(
    py: Python<'_>,
    config: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let cfg = merton_config(config)?;
    let result = merton_jump_diffusion_call_price_mc_cpu(&cfg);
    pricing_response(
        py,
        "merton_jump_diffusion_call",
        "price_merton_jump_diffusion_call",
        &cfg,
        result,
        Vec::new(),
    )
}

#[pyfunction]
fn gaussian_uncertainty_moments(py: Python<'_>, config: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let cfg = gaussian_config(config)?;
    let result = gaussian_uncertainty_moments_cpu(&cfg);
    workload_response(
        py,
        "gaussian_uncertainty_moments",
        "gaussian_uncertainty_moments",
        &cfg,
        result,
        None,
    )
}

#[pyfunction]
fn arithmetic_asian_mlmc(py: Python<'_>, config: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let cfg = asian_mlmc_config(config)?;
    let result = arithmetic_asian_call_price_mlmc_cpu(&cfg);
    let stderr = result.stderr;
    workload_response(
        py,
        "arithmetic_asian_mlmc",
        "arithmetic_asian_mlmc",
        &cfg,
        result,
        Some(stderr),
    )
}

#[pyfunction]
fn price_european_call_parameter_sweep(
    py: Python<'_>,
    config: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let cfg = european_parameter_sweep_config(config)?;
    let result = price_european_call_parameter_sweep_cpu(&cfg);
    workload_response(
        py,
        "european_call_parameter_sweep",
        "price_european_call_parameter_sweep",
        &cfg,
        result,
        None,
    )
}

#[pyfunction(name = "validate_inventory_config")]
fn validate_inventory_config_py(py: Python<'_>, config: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let cfg = inventory_config(config)?;
    let diagnostics = validate_inventory_config_rust(&cfg);
    json_to_py(
        py,
        json!({
            "ok": diagnostics.is_empty(),
            "schema_version": "inventory-validation.v1",
            "diagnostics": diagnostics,
            "manifest": manifest("inventory_policy", "validate_inventory_config", &cfg)?,
            "warnings": [],
        }),
    )
}

#[pyfunction(name = "simulate_inventory_policy")]
fn simulate_inventory_policy_py(py: Python<'_>, config: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let cfg = inventory_config(config)?;
    let result = simulate_inventory_policy_cpu(&cfg).map_err(|error| {
        PyValueError::new_err(
            serde_json::to_string(&error)
                .unwrap_or_else(|_| "inventory simulation validation failed".to_string()),
        )
    })?;
    workload_response(
        py,
        "inventory_policy",
        "simulate_inventory_policy",
        &cfg,
        result,
        None,
    )
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(price_european_call, m)?)?;
    m.add_function(wrap_pyfunction!(price_arithmetic_asian_call, m)?)?;
    m.add_function(wrap_pyfunction!(price_down_and_out_call, m)?)?;
    #[cfg(feature = "metal-native")]
    {
        m.add_function(wrap_pyfunction!(price_european_call_metal, m)?)?;
        m.add_function(wrap_pyfunction!(price_arithmetic_asian_call_metal, m)?)?;
        m.add_function(wrap_pyfunction!(price_down_and_out_call_metal, m)?)?;
    }
    m.add_function(wrap_pyfunction!(price_lookback_call, m)?)?;
    m.add_function(wrap_pyfunction!(price_basket_call, m)?)?;
    m.add_function(wrap_pyfunction!(price_american_put, m)?)?;
    m.add_function(wrap_pyfunction!(price_bermudan_put, m)?)?;
    m.add_function(wrap_pyfunction!(price_heston_european_call, m)?)?;
    m.add_function(wrap_pyfunction!(price_merton_jump_diffusion_call, m)?)?;
    m.add_function(wrap_pyfunction!(price_european_call_parameter_sweep, m)?)?;
    m.add_function(wrap_pyfunction!(gaussian_uncertainty_moments, m)?)?;
    m.add_function(wrap_pyfunction!(arithmetic_asian_mlmc, m)?)?;
    m.add_function(wrap_pyfunction!(validate_inventory_config_py, m)?)?;
    m.add_function(wrap_pyfunction!(simulate_inventory_policy_py, m)?)?;
    Ok(())
}

fn european_config(config: &Bound<'_, PyAny>) -> PyResult<EuropeanCallConfig> {
    let mut cfg = EuropeanCallConfig::default();
    fill_gbm_fields(
        config,
        &mut cfg.s0,
        &mut cfg.k,
        &mut cfg.r,
        &mut cfg.sigma,
        &mut cfg.t,
    )?;
    fill_path_fields(config, &mut cfg.n_paths, &mut cfg.n_steps, &mut cfg.seed)?;
    cfg.n_threads = get_usize(config, &["n_threads"], cfg.n_threads)?;
    cfg.technique = get_technique(config, cfg.technique)?;
    cfg.sampling = get_sampling(config, cfg.sampling)?;
    Ok(cfg)
}

fn asian_config(config: &Bound<'_, PyAny>) -> PyResult<ArithmeticAsianCallConfig> {
    let mut cfg = ArithmeticAsianCallConfig::default();
    fill_gbm_fields(
        config,
        &mut cfg.s0,
        &mut cfg.k,
        &mut cfg.r,
        &mut cfg.sigma,
        &mut cfg.t,
    )?;
    fill_path_fields(config, &mut cfg.n_paths, &mut cfg.n_steps, &mut cfg.seed)?;
    cfg.n_threads = get_usize(config, &["n_threads"], cfg.n_threads)?;
    cfg.technique = get_technique(config, cfg.technique)?;
    cfg.sampling = get_sampling(config, cfg.sampling)?;
    Ok(cfg)
}

fn down_and_out_config(config: &Bound<'_, PyAny>) -> PyResult<DownAndOutCallConfig> {
    let mut cfg = DownAndOutCallConfig::default();
    fill_gbm_fields(
        config,
        &mut cfg.s0,
        &mut cfg.k,
        &mut cfg.r,
        &mut cfg.sigma,
        &mut cfg.t,
    )?;
    fill_path_fields(config, &mut cfg.n_paths, &mut cfg.n_steps, &mut cfg.seed)?;
    cfg.barrier = get_f64(config, &["barrier"], cfg.barrier)?;
    cfg.n_threads = get_usize(config, &["n_threads"], cfg.n_threads)?;
    cfg.technique = get_technique(config, cfg.technique)?;
    cfg.sampling = get_sampling(config, cfg.sampling)?;
    Ok(cfg)
}

fn lookback_config(config: &Bound<'_, PyAny>) -> PyResult<LookbackCallConfig> {
    let mut cfg = LookbackCallConfig::default();
    fill_gbm_fields(
        config,
        &mut cfg.s0,
        &mut cfg.k,
        &mut cfg.r,
        &mut cfg.sigma,
        &mut cfg.t,
    )?;
    fill_path_fields(config, &mut cfg.n_paths, &mut cfg.n_steps, &mut cfg.seed)?;
    cfg.n_threads = get_usize(config, &["n_threads"], cfg.n_threads)?;
    cfg.technique = get_technique(config, cfg.technique)?;
    cfg.sampling = get_sampling(config, cfg.sampling)?;
    Ok(cfg)
}

fn basket_config(config: &Bound<'_, PyAny>) -> PyResult<BasketCallConfig> {
    let mut cfg = BasketCallConfig::default();
    cfg.s01 = get_f64(config, &["s01", "spot_1", "spot1"], cfg.s01)?;
    cfg.s02 = get_f64(config, &["s02", "spot_2", "spot2"], cfg.s02)?;
    cfg.k = get_f64(config, &["k", "strike"], cfg.k)?;
    cfg.r = get_f64(config, &["r", "rate"], cfg.r)?;
    cfg.sigma1 = get_f64(
        config,
        &["sigma1", "volatility_1", "volatility1"],
        cfg.sigma1,
    )?;
    cfg.sigma2 = get_f64(
        config,
        &["sigma2", "volatility_2", "volatility2"],
        cfg.sigma2,
    )?;
    cfg.rho = get_f64(config, &["rho", "correlation"], cfg.rho)?;
    cfg.weight1 = get_f64(config, &["weight1", "weight_1"], cfg.weight1)?;
    cfg.weight2 = get_f64(config, &["weight2", "weight_2"], cfg.weight2)?;
    cfg.t = get_f64(config, &["t", "maturity"], cfg.t)?;
    cfg.n_paths = get_usize(config, &["n_paths", "paths"], cfg.n_paths)?;
    cfg.seed = get_u64(config, &["seed"], cfg.seed)?;
    cfg.n_threads = get_usize(config, &["n_threads"], cfg.n_threads)?;
    cfg.technique = get_technique(config, cfg.technique)?;
    cfg.sampling = get_sampling(config, cfg.sampling)?;
    Ok(cfg)
}

fn american_put_config(config: &Bound<'_, PyAny>) -> PyResult<AmericanPutConfig> {
    let mut cfg = AmericanPutConfig::default();
    fill_gbm_fields(
        config,
        &mut cfg.s0,
        &mut cfg.k,
        &mut cfg.r,
        &mut cfg.sigma,
        &mut cfg.t,
    )?;
    fill_path_fields(config, &mut cfg.n_paths, &mut cfg.n_steps, &mut cfg.seed)?;
    cfg.n_threads = get_usize(config, &["n_threads"], cfg.n_threads)?;
    cfg.basis_degree = get_usize(config, &["basis_degree"], cfg.basis_degree)?;
    Ok(cfg)
}

fn bermudan_put_config(config: &Bound<'_, PyAny>) -> PyResult<BermudanPutConfig> {
    let mut cfg = BermudanPutConfig::default();
    fill_gbm_fields(
        config,
        &mut cfg.s0,
        &mut cfg.k,
        &mut cfg.r,
        &mut cfg.sigma,
        &mut cfg.t,
    )?;
    fill_path_fields(config, &mut cfg.n_paths, &mut cfg.n_steps, &mut cfg.seed)?;
    cfg.n_threads = get_usize(config, &["n_threads"], cfg.n_threads)?;
    cfg.basis_degree = get_usize(config, &["basis_degree"], cfg.basis_degree)?;
    if let Some(value) = lookup(config, &["exercise_steps"])? {
        cfg.exercise_steps = value.extract::<Vec<usize>>()?;
    }
    Ok(cfg)
}

fn heston_config(config: &Bound<'_, PyAny>) -> PyResult<HestonEuropeanCallConfig> {
    let mut cfg = HestonEuropeanCallConfig::default();
    cfg.s0 = get_f64(config, &["s0", "spot"], cfg.s0)?;
    cfg.k = get_f64(config, &["k", "strike"], cfg.k)?;
    cfg.r = get_f64(config, &["r", "rate"], cfg.r)?;
    cfg.v0 = get_f64(config, &["v0", "initial_variance"], cfg.v0)?;
    cfg.kappa = get_f64(config, &["kappa", "mean_reversion"], cfg.kappa)?;
    cfg.theta = get_f64(config, &["theta", "long_run_variance"], cfg.theta)?;
    cfg.vol_of_vol = get_f64(config, &["vol_of_vol"], cfg.vol_of_vol)?;
    cfg.rho = get_f64(config, &["rho", "correlation"], cfg.rho)?;
    cfg.t = get_f64(config, &["t", "maturity"], cfg.t)?;
    cfg.n_paths = get_usize(config, &["n_paths", "paths"], cfg.n_paths)?;
    cfg.n_steps = get_usize(config, &["n_steps", "steps"], cfg.n_steps)?;
    cfg.seed = get_u64(config, &["seed"], cfg.seed)?;
    cfg.n_threads = get_usize(config, &["n_threads"], cfg.n_threads)?;
    cfg.technique = get_technique(config, cfg.technique)?;
    Ok(cfg)
}

fn merton_config(config: &Bound<'_, PyAny>) -> PyResult<MertonJumpDiffusionCallConfig> {
    let mut cfg = MertonJumpDiffusionCallConfig::default();
    fill_gbm_fields(
        config,
        &mut cfg.s0,
        &mut cfg.k,
        &mut cfg.r,
        &mut cfg.sigma,
        &mut cfg.t,
    )?;
    cfg.n_paths = get_usize(config, &["n_paths", "paths"], cfg.n_paths)?;
    cfg.seed = get_u64(config, &["seed"], cfg.seed)?;
    cfg.n_threads = get_usize(config, &["n_threads"], cfg.n_threads)?;
    cfg.jump_intensity = get_f64(config, &["jump_intensity"], cfg.jump_intensity)?;
    cfg.jump_mean = get_f64(config, &["jump_mean"], cfg.jump_mean)?;
    cfg.jump_volatility = get_f64(config, &["jump_volatility"], cfg.jump_volatility)?;
    Ok(cfg)
}

fn gaussian_config(config: &Bound<'_, PyAny>) -> PyResult<GaussianUncertaintyConfig> {
    let mut cfg = GaussianUncertaintyConfig::default();
    cfg.n_samples = get_usize(config, &["n_samples", "n_paths", "samples"], cfg.n_samples)?;
    cfg.dimensions = get_usize(config, &["dimensions"], cfg.dimensions)?;
    cfg.seed = get_u64(config, &["seed"], cfg.seed)?;
    cfg.sampling = get_sampling(config, cfg.sampling)?;
    Ok(cfg)
}

fn asian_mlmc_config(config: &Bound<'_, PyAny>) -> PyResult<ArithmeticAsianMlmcConfig> {
    let mut cfg = ArithmeticAsianMlmcConfig::default();
    fill_gbm_fields(
        config,
        &mut cfg.s0,
        &mut cfg.k,
        &mut cfg.r,
        &mut cfg.sigma,
        &mut cfg.t,
    )?;
    cfg.base_steps = get_usize(config, &["base_steps"], cfg.base_steps)?;
    cfg.levels = get_usize(config, &["levels"], cfg.levels)?;
    cfg.refinement_factor = get_usize(config, &["refinement_factor"], cfg.refinement_factor)?;
    if let Some(value) = lookup(config, &["paths_per_level"])? {
        cfg.paths_per_level = value.extract::<Vec<usize>>()?;
    }
    cfg.seed = get_u64(config, &["seed"], cfg.seed)?;
    cfg.sampling = get_sampling(config, cfg.sampling)?;
    cfg.scramble_replicates = get_usize(config, &["scramble_replicates"], cfg.scramble_replicates)?;
    Ok(cfg)
}

fn european_parameter_sweep_config(
    config: &Bound<'_, PyAny>,
) -> PyResult<EuropeanCallParameterSweepConfig> {
    let mut cfg = EuropeanCallParameterSweepConfig::default();
    if let Some(base) = lookup(config, &["base_config"])? {
        cfg.base_config = european_config(&base)?;
    }
    if let Some(value) = lookup(config, &["n_paths"])? {
        cfg.base_config.n_paths = value.extract::<usize>()?;
    }
    if let Some(value) = lookup(config, &["n_steps"])? {
        cfg.base_config.n_steps = value.extract::<usize>()?;
    }
    if let Some(value) = lookup(config, &["seed"])? {
        cfg.base_config.seed = value.extract::<u64>()?;
    }
    cfg.seed_stride = get_u64(config, &["seed_stride"], cfg.seed_stride)?;
    if let Some(value) = lookup(config, &["method"])? {
        cfg.method = parse_european_method(&value.extract::<String>()?)?;
    }
    if let Some(value) = lookup(config, &["scenarios"])? {
        let scenarios = value.extract::<Vec<PyObject>>()?;
        cfg.scenarios.clear();
        for scenario in scenarios {
            let bound = scenario.bind(config.py());
            cfg.scenarios.push(sweep_scenario(bound)?);
        }
    }
    Ok(cfg)
}

fn inventory_config(config: &Bound<'_, PyAny>) -> PyResult<InventorySimulationConfig> {
    let mut cfg = InventorySimulationConfig::default();
    cfg.n_paths = get_usize(config, &["n_paths", "paths"], cfg.n_paths)?;
    cfg.n_periods = get_usize(config, &["n_periods", "periods"], cfg.n_periods)?;
    cfg.warmup_periods = get_usize(config, &["warmup_periods"], cfg.warmup_periods)?;
    cfg.seed = get_u64(config, &["seed"], cfg.seed)?;
    cfg.n_threads = get_usize(config, &["n_threads"], cfg.n_threads)?;
    cfg.initial_on_hand = get_f64(config, &["initial_on_hand"], cfg.initial_on_hand)?;
    cfg.initial_backorder = get_f64(config, &["initial_backorder"], cfg.initial_backorder)?;
    cfg.lead_time_periods = get_usize(config, &["lead_time_periods"], cfg.lead_time_periods)?;
    if let Some(value) = lookup(config, &["shortage_policy"])? {
        cfg.shortage_policy = parse_inventory_shortage_policy(&value.extract::<String>()?)?;
    }
    if let Some(demand) = lookup(config, &["demand"])? {
        cfg.demand = inventory_demand(&demand)?;
    }
    if let Some(policy) = lookup(config, &["policy"])? {
        cfg.policy = InventoryPolicy {
            reorder_point: get_f64(&policy, &["reorder_point"], cfg.policy.reorder_point)?,
            order_up_to: get_f64(&policy, &["order_up_to"], cfg.policy.order_up_to)?,
        };
    }
    if let Some(constraints) = lookup(config, &["constraints"])? {
        cfg.constraints = InventoryConstraints {
            minimum_order_quantity: get_f64(
                &constraints,
                &["minimum_order_quantity"],
                cfg.constraints.minimum_order_quantity,
            )?,
            case_pack: get_f64(&constraints, &["case_pack"], cfg.constraints.case_pack)?,
            supplier_capacity_per_period: get_optional_f64(
                &constraints,
                &["supplier_capacity_per_period"],
            )?,
            warehouse_capacity: get_optional_f64(&constraints, &["warehouse_capacity"])?,
        };
    }
    if let Some(costs) = lookup(config, &["costs"])? {
        cfg.costs = InventoryCostConfig {
            holding_cost_per_unit_period: get_f64(
                &costs,
                &["holding_cost_per_unit_period"],
                cfg.costs.holding_cost_per_unit_period,
            )?,
            backorder_cost_per_unit_period: get_f64(
                &costs,
                &["backorder_cost_per_unit_period"],
                cfg.costs.backorder_cost_per_unit_period,
            )?,
            lost_sale_cost_per_unit: get_f64(
                &costs,
                &["lost_sale_cost_per_unit"],
                cfg.costs.lost_sale_cost_per_unit,
            )?,
            fixed_order_cost: get_f64(&costs, &["fixed_order_cost"], cfg.costs.fixed_order_cost)?,
            variable_order_cost_per_unit: get_f64(
                &costs,
                &["variable_order_cost_per_unit"],
                cfg.costs.variable_order_cost_per_unit,
            )?,
        };
    }
    if let Some(trace) = lookup(config, &["trace"])? {
        cfg.trace = InventoryTraceConfig {
            path_indices: match lookup(&trace, &["path_indices"])? {
                Some(value) => value.extract::<Vec<usize>>()?,
                None => Vec::new(),
            },
            max_periods: get_usize(&trace, &["max_periods"], 0)?,
        };
    }
    Ok(cfg)
}

fn inventory_demand(config: &Bound<'_, PyAny>) -> PyResult<InventoryDemandDistribution> {
    let distribution = get_string(config, &["distribution"], "normal".to_string())?;
    match normalize(&distribution).as_str() {
        "deterministic" => Ok(InventoryDemandDistribution::Deterministic {
            units: get_f64(config, &["units"], 0.0)?,
        }),
        "normal" => Ok(InventoryDemandDistribution::Normal {
            mean: get_f64(config, &["mean"], 10.0)?,
            std_dev: get_f64(config, &["std_dev"], 2.0)?,
        }),
        _ => Err(PyValueError::new_err(format!(
            "unsupported inventory demand distribution '{distribution}'"
        ))),
    }
}

fn sweep_scenario(config: &Bound<'_, PyAny>) -> PyResult<EuropeanCallSweepScenario> {
    let mut scenario = EuropeanCallSweepScenario::default();
    scenario.scenario_id = get_string(config, &["scenario_id"], scenario.scenario_id)?;
    scenario.s0 = get_optional_f64(config, &["s0", "spot"])?;
    scenario.k = get_optional_f64(config, &["k", "strike"])?;
    scenario.r = get_optional_f64(config, &["r", "rate"])?;
    scenario.sigma = get_optional_f64(config, &["sigma", "volatility"])?;
    scenario.t = get_optional_f64(config, &["t", "maturity"])?;
    scenario.n_paths = get_optional_usize(config, &["n_paths", "paths"])?;
    scenario.n_steps = get_optional_usize(config, &["n_steps", "steps"])?;
    scenario.seed = get_optional_u64(config, &["seed"])?;
    scenario.sampling = get_optional_sampling(config)?;
    scenario.technique = get_optional_technique(config)?;
    if let Some(value) = lookup(config, &["method"])? {
        scenario.method = Some(parse_european_method(&value.extract::<String>()?)?);
    }
    Ok(scenario)
}

fn fill_gbm_fields(
    config: &Bound<'_, PyAny>,
    s0: &mut f64,
    k: &mut f64,
    r: &mut f64,
    sigma: &mut f64,
    t: &mut f64,
) -> PyResult<()> {
    *s0 = get_f64(config, &["s0", "spot"], *s0)?;
    *k = get_f64(config, &["k", "strike"], *k)?;
    *r = get_f64(config, &["r", "rate"], *r)?;
    *sigma = get_f64(config, &["sigma", "volatility"], *sigma)?;
    *t = get_f64(config, &["t", "maturity"], *t)?;
    Ok(())
}

fn fill_path_fields(
    config: &Bound<'_, PyAny>,
    n_paths: &mut usize,
    n_steps: &mut usize,
    seed: &mut u64,
) -> PyResult<()> {
    *n_paths = get_usize(config, &["n_paths", "paths"], *n_paths)?;
    *n_steps = get_usize(config, &["n_steps", "steps"], *n_steps)?;
    *seed = get_u64(config, &["seed"], *seed)?;
    Ok(())
}

fn pricing_response<T: Serialize>(
    py: Python<'_>,
    workload: &str,
    function_name: &str,
    config: &impl Serialize,
    result: T,
    warnings: Vec<String>,
) -> PyResult<PyObject> {
    let mut value = to_value(result)?;
    let price = value
        .get("price")
        .and_then(Value::as_f64)
        .ok_or_else(|| PyValueError::new_err("native pricing result did not include price"))?;
    let stderr = value
        .get("stderr")
        .and_then(Value::as_f64)
        .ok_or_else(|| PyValueError::new_err("native pricing result did not include stderr"))?;
    let mut root = Map::new();
    root.insert("price".to_string(), json!(price));
    root.insert("stderr".to_string(), json!(stderr));
    root.insert("values".to_string(), value.take());
    root.insert(
        "manifest".to_string(),
        manifest(workload, function_name, config)?,
    );
    root.insert("warnings".to_string(), json!(warnings));
    json_to_py(py, Value::Object(root))
}

fn workload_response<T: Serialize>(
    py: Python<'_>,
    workload: &str,
    function_name: &str,
    config: &impl Serialize,
    result: T,
    stderr: Option<f64>,
) -> PyResult<PyObject> {
    let mut root = Map::new();
    root.insert("values".to_string(), to_value(result)?);
    root.insert(
        "manifest".to_string(),
        manifest(workload, function_name, config)?,
    );
    root.insert("warnings".to_string(), json!([] as [String; 0]));
    if let Some(stderr) = stderr {
        root.insert("stderr".to_string(), json!(stderr));
    }
    json_to_py(py, Value::Object(root))
}

#[cfg(feature = "metal-native")]
fn metal_pricing_response(
    py: Python<'_>,
    workload: &str,
    function_name: &str,
    config: &impl Serialize,
    output: mc_core::RunOutput,
) -> PyResult<PyObject> {
    let mut root = Map::new();
    root.insert("price".to_string(), json!(output.price));
    root.insert("stderr".to_string(), json!(output.stderr));
    root.insert(
        "values".to_string(),
        json!({
            "price": output.price,
            "stderr": output.stderr,
            "runtime_ms": output.runtime_ms,
        }),
    );
    root.insert(
        "manifest".to_string(),
        manifest_for_backend(
            workload,
            function_name,
            config,
            "apple_metal",
            "native_extension_metal",
            "statistically reproducible for identical config, seed, native module version, Metal runtime, and Apple device class",
        )?,
    );
    root.insert(
        "warnings".to_string(),
        json!([
            "native Metal bridge supports pseudorandom sampling for the current GBM option family; unsupported requests fail explicitly",
            "Metal execution uses float32 kernels and should be compared against CPU/reference tolerances before production claims"
        ]),
    );
    json_to_py(py, Value::Object(root))
}

fn manifest(workload: &str, function_name: &str, config: &impl Serialize) -> PyResult<Value> {
    manifest_for_backend(
        workload,
        function_name,
        config,
        "cpu_native",
        "native_extension",
        "deterministic for identical config, seed, native module version, and CPU runtime semantics",
    )
}

fn manifest_for_backend(
    workload: &str,
    function_name: &str,
    config: &impl Serialize,
    backend: &str,
    execution_mode: &str,
    reproducibility: &str,
) -> PyResult<Value> {
    Ok(json!({
        "schema_version": "python-native-runtime.v1",
        "workload": workload,
        "backend": backend,
        "execution_mode": execution_mode,
        "function": function_name,
        "native_module": "montepath._native",
        "config": to_value(config)?,
        "reproducibility": reproducibility,
        "performance_claim": "use benchmark artifacts for timing claims"
    }))
}

fn to_value(value: impl Serialize) -> PyResult<Value> {
    serde_json::to_value(value).map_err(|err| PyValueError::new_err(err.to_string()))
}

fn json_to_py(py: Python<'_>, value: Value) -> PyResult<PyObject> {
    let json = py.import_bound("json")?;
    let text =
        serde_json::to_string(&value).map_err(|err| PyValueError::new_err(err.to_string()))?;
    Ok(json.call_method1("loads", (text,))?.into_py(py))
}

fn lookup<'py>(config: &Bound<'py, PyAny>, keys: &[&str]) -> PyResult<Option<Bound<'py, PyAny>>> {
    for key in keys {
        let value = config.call_method1("get", (*key,))?;
        if !value.is_none() {
            return Ok(Some(value));
        }
    }
    Ok(None)
}

fn get_f64(config: &Bound<'_, PyAny>, keys: &[&str], default: f64) -> PyResult<f64> {
    match lookup(config, keys)? {
        Some(value) => value.extract::<f64>(),
        None => Ok(default),
    }
}

fn get_usize(config: &Bound<'_, PyAny>, keys: &[&str], default: usize) -> PyResult<usize> {
    match lookup(config, keys)? {
        Some(value) => value.extract::<usize>(),
        None => Ok(default),
    }
}

fn get_u64(config: &Bound<'_, PyAny>, keys: &[&str], default: u64) -> PyResult<u64> {
    match lookup(config, keys)? {
        Some(value) => value.extract::<u64>(),
        None => Ok(default),
    }
}

fn get_string(config: &Bound<'_, PyAny>, keys: &[&str], default: String) -> PyResult<String> {
    match lookup(config, keys)? {
        Some(value) => value.extract::<String>(),
        None => Ok(default),
    }
}

fn get_optional_f64(config: &Bound<'_, PyAny>, keys: &[&str]) -> PyResult<Option<f64>> {
    match lookup(config, keys)? {
        Some(value) => Ok(Some(value.extract::<f64>()?)),
        None => Ok(None),
    }
}

fn get_optional_usize(config: &Bound<'_, PyAny>, keys: &[&str]) -> PyResult<Option<usize>> {
    match lookup(config, keys)? {
        Some(value) => Ok(Some(value.extract::<usize>()?)),
        None => Ok(None),
    }
}

fn get_optional_u64(config: &Bound<'_, PyAny>, keys: &[&str]) -> PyResult<Option<u64>> {
    match lookup(config, keys)? {
        Some(value) => Ok(Some(value.extract::<u64>()?)),
        None => Ok(None),
    }
}

fn get_sampling(config: &Bound<'_, PyAny>, default: SamplingMethod) -> PyResult<SamplingMethod> {
    match lookup(config, &["sampling"])? {
        Some(value) => parse_sampling(&value.extract::<String>()?),
        None => Ok(default),
    }
}

fn get_optional_sampling(config: &Bound<'_, PyAny>) -> PyResult<Option<SamplingMethod>> {
    match lookup(config, &["sampling"])? {
        Some(value) => Ok(Some(parse_sampling(&value.extract::<String>()?)?)),
        None => Ok(None),
    }
}

fn get_technique(
    config: &Bound<'_, PyAny>,
    default: MonteCarloTechnique,
) -> PyResult<MonteCarloTechnique> {
    match lookup(config, &["technique"])? {
        Some(value) => parse_technique(&value.extract::<String>()?),
        None => Ok(default),
    }
}

fn get_optional_technique(config: &Bound<'_, PyAny>) -> PyResult<Option<MonteCarloTechnique>> {
    match lookup(config, &["technique"])? {
        Some(value) => Ok(Some(parse_technique(&value.extract::<String>()?)?)),
        None => Ok(None),
    }
}

fn parse_sampling(value: &str) -> PyResult<SamplingMethod> {
    match normalize(value).as_str() {
        "pseudorandom" => Ok(SamplingMethod::Pseudorandom),
        "randomized_halton" => Ok(SamplingMethod::RandomizedHalton),
        "latin_hypercube" => Ok(SamplingMethod::LatinHypercube),
        "scrambled_sobol" => Ok(SamplingMethod::ScrambledSobol),
        "scrambled_sobol_brownian_bridge" => Ok(SamplingMethod::ScrambledSobolBrownianBridge),
        _ => Err(PyValueError::new_err(format!(
            "unsupported sampling method '{value}'"
        ))),
    }
}

fn parse_technique(value: &str) -> PyResult<MonteCarloTechnique> {
    match normalize(value).as_str() {
        "standard" => Ok(MonteCarloTechnique::Standard),
        "antithetic" => Ok(MonteCarloTechnique::Antithetic),
        "control_variate" => Ok(MonteCarloTechnique::ControlVariate),
        _ => Err(PyValueError::new_err(format!(
            "unsupported Monte Carlo technique '{value}'"
        ))),
    }
}

fn parse_european_method(value: &str) -> PyResult<mc_core::EuropeanCallMethod> {
    match normalize(value).as_str() {
        "auto" => Ok(mc_core::EuropeanCallMethod::Auto),
        "terminal_distribution" => Ok(mc_core::EuropeanCallMethod::TerminalDistribution),
        "stepwise_paths" => Ok(mc_core::EuropeanCallMethod::StepwisePaths),
        _ => Err(PyValueError::new_err(format!(
            "unsupported European call method '{value}'"
        ))),
    }
}

fn parse_inventory_shortage_policy(value: &str) -> PyResult<InventoryShortagePolicy> {
    match normalize(value).as_str() {
        "backorder" => Ok(InventoryShortagePolicy::Backorder),
        "lost_sales" => Ok(InventoryShortagePolicy::LostSales),
        _ => Err(PyValueError::new_err(format!(
            "unsupported inventory shortage policy '{value}'"
        ))),
    }
}

fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('-', "_")
}
