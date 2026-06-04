use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::thread;

use serde::{Deserialize, Serialize};

use super::cpu::MonteCarloRng;

const INVENTORY_SCHEMA_VERSION: &str = "inventory-simulation.v1";
const INVENTORY_SEMANTICS_VERSION: &str = "periodic-review.v1";
const PERIOD_ORDER: &str = "receive-demand-fulfill-review-order-record";
const MAX_FIXED_LEAD_TIME_PERIODS: usize = 10_000;
const MAX_TRACE_PATHS: usize = 16;
const MAX_TRACE_PERIODS: usize = 10_000;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InventoryShortagePolicy {
    Backorder,
    #[default]
    LostSales,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(tag = "distribution", rename_all = "snake_case")]
pub enum InventoryDemandDistribution {
    Deterministic { units: f64 },
    Normal { mean: f64, std_dev: f64 },
}

impl Default for InventoryDemandDistribution {
    fn default() -> Self {
        Self::Normal {
            mean: 10.0,
            std_dev: 2.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct InventoryPolicy {
    pub reorder_point: f64,
    pub order_up_to: f64,
}

impl Default for InventoryPolicy {
    fn default() -> Self {
        Self {
            reorder_point: 50.0,
            order_up_to: 100.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct InventoryConstraints {
    pub minimum_order_quantity: f64,
    pub case_pack: f64,
    pub supplier_capacity_per_period: Option<f64>,
    pub warehouse_capacity: Option<f64>,
}

impl Default for InventoryConstraints {
    fn default() -> Self {
        Self {
            minimum_order_quantity: 0.0,
            case_pack: 1.0,
            supplier_capacity_per_period: None,
            warehouse_capacity: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct InventoryCostConfig {
    pub holding_cost_per_unit_period: f64,
    pub backorder_cost_per_unit_period: f64,
    pub lost_sale_cost_per_unit: f64,
    pub fixed_order_cost: f64,
    pub variable_order_cost_per_unit: f64,
}

impl Default for InventoryCostConfig {
    fn default() -> Self {
        Self {
            holding_cost_per_unit_period: 0.0,
            backorder_cost_per_unit_period: 0.0,
            lost_sale_cost_per_unit: 0.0,
            fixed_order_cost: 0.0,
            variable_order_cost_per_unit: 0.0,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct InventoryTraceConfig {
    pub path_indices: Vec<usize>,
    pub max_periods: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InventorySimulationConfig {
    pub n_paths: usize,
    pub n_periods: usize,
    pub warmup_periods: usize,
    pub seed: u64,
    pub n_threads: usize,
    pub initial_on_hand: f64,
    pub initial_backorder: f64,
    pub lead_time_periods: usize,
    pub demand: InventoryDemandDistribution,
    pub shortage_policy: InventoryShortagePolicy,
    pub policy: InventoryPolicy,
    pub constraints: InventoryConstraints,
    pub costs: InventoryCostConfig,
    pub trace: InventoryTraceConfig,
}

impl Default for InventorySimulationConfig {
    fn default() -> Self {
        Self {
            n_paths: 10_000,
            n_periods: 52,
            warmup_periods: 0,
            seed: 42,
            n_threads: 0,
            initial_on_hand: 100.0,
            initial_backorder: 0.0,
            lead_time_periods: 1,
            demand: InventoryDemandDistribution::default(),
            shortage_policy: InventoryShortagePolicy::default(),
            policy: InventoryPolicy::default(),
            constraints: InventoryConstraints::default(),
            costs: InventoryCostConfig::default(),
            trace: InventoryTraceConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InventoryValidationDiagnostic {
    pub code: String,
    pub field: String,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InventorySimulationError {
    pub diagnostics: Vec<InventoryValidationDiagnostic>,
}

impl fmt::Display for InventorySimulationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "inventory simulation configuration has {} validation error(s)",
            self.diagnostics.len()
        )
    }
}

impl Error for InventorySimulationError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InventoryRunManifest {
    pub schema_version: String,
    pub semantics_version: String,
    pub backend: String,
    pub period_order: String,
    pub n_paths: usize,
    pub n_periods: usize,
    pub warmup_periods: usize,
    pub observed_periods: usize,
    pub seed: u64,
    pub shortage_policy: InventoryShortagePolicy,
    pub lead_time_periods: usize,
    pub lead_time_semantics: String,
    pub demand: InventoryDemandDistribution,
    pub normal_demand_semantics: String,
    pub warehouse_capacity_semantics: String,
    pub rng_stream_mapping: String,
    pub quantile_method: String,
    pub trace_paths: Vec<usize>,
    pub trace_period_limit: usize,
    pub config: InventorySimulationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InventoryPathResult {
    pub path_index: usize,
    pub cycle_service_level: f64,
    pub fill_rate: f64,
    pub average_on_hand: f64,
    pub stockout_events: usize,
    pub total_demand: f64,
    pub fulfilled_demand: f64,
    pub unmet_demand: f64,
    pub received_units: f64,
    pub ordered_units: f64,
    pub orders_placed: usize,
    pub constraint_events: InventoryConstraintEvents,
    pub holding_cost: f64,
    pub shortage_cost: f64,
    pub ordering_cost: f64,
    pub total_cost: f64,
    pub ending_on_hand: f64,
    pub ending_on_order: f64,
    pub ending_backorder: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct InventoryConstraintEvents {
    pub limited_order_attempts: usize,
    pub minimum_order_quantity_adjustments: usize,
    pub case_pack_roundups: usize,
    pub supplier_capacity_clips: usize,
    pub warehouse_capacity_clips: usize,
    pub blocked_order_attempts: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct InventoryMetricSummary {
    pub mean: f64,
    pub min: f64,
    pub p05: f64,
    pub p50: f64,
    pub p95: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InventorySimulationSummary {
    pub cycle_service_level: InventoryMetricSummary,
    pub fill_rate: InventoryMetricSummary,
    pub average_on_hand: InventoryMetricSummary,
    pub stockout_events: InventoryMetricSummary,
    pub total_cost: InventoryMetricSummary,
    pub ending_on_hand: InventoryMetricSummary,
    pub ending_on_order: InventoryMetricSummary,
    pub ending_backorder: InventoryMetricSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InventoryPeriodTrace {
    pub period: usize,
    pub observed: bool,
    pub received_units: f64,
    pub demand: f64,
    pub fulfilled_demand: f64,
    pub unmet_demand: f64,
    pub inventory_position_before_order: f64,
    pub order_quantity: f64,
    pub constraint_events: InventoryConstraintEvents,
    pub holding_cost: f64,
    pub shortage_cost: f64,
    pub ordering_cost: f64,
    pub ending_on_hand: f64,
    pub ending_on_order: f64,
    pub ending_backorder: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InventoryPathTrace {
    pub path_index: usize,
    pub periods: Vec<InventoryPeriodTrace>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InventorySimulationResult {
    pub manifest: InventoryRunManifest,
    pub summary: InventorySimulationSummary,
    pub paths: Vec<InventoryPathResult>,
    pub traces: Vec<InventoryPathTrace>,
}

pub fn validate_inventory_config(
    cfg: &InventorySimulationConfig,
) -> Vec<InventoryValidationDiagnostic> {
    let mut diagnostics = Vec::new();

    if cfg.n_paths == 0 {
        diagnostics.push(diagnostic(
            "inventory.paths.invalid",
            "n_paths",
            "n_paths must be greater than zero",
            "Set n_paths to at least 1.",
        ));
    }
    if cfg.n_periods == 0 {
        diagnostics.push(diagnostic(
            "inventory.periods.invalid",
            "n_periods",
            "n_periods must be greater than zero",
            "Set n_periods to at least 1.",
        ));
    }
    if cfg.n_periods == 0 || cfg.warmup_periods >= cfg.n_periods {
        diagnostics.push(diagnostic(
            "inventory.warmup.invalid",
            "warmup_periods",
            "warmup_periods must be less than n_periods",
            "Leave at least one observed period after warm-up.",
        ));
    }
    validate_non_negative(
        &mut diagnostics,
        cfg.initial_on_hand,
        "inventory.initial_on_hand.invalid",
        "initial_on_hand",
    );
    validate_non_negative(
        &mut diagnostics,
        cfg.initial_backorder,
        "inventory.initial_backorder.invalid",
        "initial_backorder",
    );
    if cfg.shortage_policy == InventoryShortagePolicy::LostSales && cfg.initial_backorder > 0.0 {
        diagnostics.push(diagnostic(
            "inventory.initial_backorder.incompatible",
            "initial_backorder",
            "lost-sales simulations cannot start with an existing backorder",
            "Set initial_backorder to zero or select the backorder shortage policy.",
        ));
    }
    if cfg.lead_time_periods > MAX_FIXED_LEAD_TIME_PERIODS {
        diagnostics.push(diagnostic(
            "inventory.lead_time.unsupported",
            "lead_time_periods",
            "lead_time_periods exceeds the bounded CPU reference-kernel limit",
            "Use a lead time of 10,000 periods or fewer.",
        ));
    }
    validate_trace_config(cfg, &mut diagnostics);

    match cfg.demand {
        InventoryDemandDistribution::Deterministic { units } => validate_non_negative(
            &mut diagnostics,
            units,
            "inventory.demand.deterministic.invalid",
            "demand.units",
        ),
        InventoryDemandDistribution::Normal { mean, std_dev } => {
            validate_non_negative(
                &mut diagnostics,
                mean,
                "inventory.demand.normal_mean.invalid",
                "demand.mean",
            );
            validate_non_negative(
                &mut diagnostics,
                std_dev,
                "inventory.demand.normal_std_dev.invalid",
                "demand.std_dev",
            );
        }
    }

    if !cfg.policy.reorder_point.is_finite() {
        diagnostics.push(diagnostic(
            "inventory.policy.reorder_point.invalid",
            "policy.reorder_point",
            "reorder_point must be finite",
            "Provide a finite inventory-position threshold.",
        ));
    }
    if !cfg.policy.order_up_to.is_finite() || cfg.policy.order_up_to < cfg.policy.reorder_point {
        diagnostics.push(diagnostic(
            "inventory.policy.order_up_to.invalid",
            "policy.order_up_to",
            "order_up_to must be finite and greater than or equal to reorder_point",
            "Increase order_up_to or lower reorder_point.",
        ));
    }

    validate_non_negative(
        &mut diagnostics,
        cfg.constraints.minimum_order_quantity,
        "inventory.constraints.minimum_order_quantity.invalid",
        "constraints.minimum_order_quantity",
    );
    if !cfg.constraints.case_pack.is_finite() || cfg.constraints.case_pack <= 0.0 {
        diagnostics.push(diagnostic(
            "inventory.constraints.case_pack.invalid",
            "constraints.case_pack",
            "case_pack must be finite and greater than zero",
            "Use a positive case-pack quantity, such as 1.0.",
        ));
    }
    validate_optional_non_negative(
        &mut diagnostics,
        cfg.constraints.supplier_capacity_per_period,
        "inventory.constraints.supplier_capacity.invalid",
        "constraints.supplier_capacity_per_period",
    );
    validate_optional_non_negative(
        &mut diagnostics,
        cfg.constraints.warehouse_capacity,
        "inventory.constraints.warehouse_capacity.invalid",
        "constraints.warehouse_capacity",
    );
    if cfg
        .constraints
        .warehouse_capacity
        .is_some_and(|capacity| capacity < cfg.initial_on_hand)
    {
        diagnostics.push(diagnostic(
            "inventory.constraints.warehouse_capacity.initial_state",
            "constraints.warehouse_capacity",
            "warehouse_capacity cannot be below initial_on_hand",
            "Increase warehouse_capacity or lower initial_on_hand.",
        ));
    }

    for (value, code, field) in [
        (
            cfg.costs.holding_cost_per_unit_period,
            "inventory.costs.holding.invalid",
            "costs.holding_cost_per_unit_period",
        ),
        (
            cfg.costs.backorder_cost_per_unit_period,
            "inventory.costs.backorder.invalid",
            "costs.backorder_cost_per_unit_period",
        ),
        (
            cfg.costs.lost_sale_cost_per_unit,
            "inventory.costs.lost_sale.invalid",
            "costs.lost_sale_cost_per_unit",
        ),
        (
            cfg.costs.fixed_order_cost,
            "inventory.costs.fixed_order.invalid",
            "costs.fixed_order_cost",
        ),
        (
            cfg.costs.variable_order_cost_per_unit,
            "inventory.costs.variable_order.invalid",
            "costs.variable_order_cost_per_unit",
        ),
    ] {
        validate_non_negative(&mut diagnostics, value, code, field);
    }

    diagnostics
}

pub fn simulate_inventory_policy_cpu(
    cfg: &InventorySimulationConfig,
) -> Result<InventorySimulationResult, InventorySimulationError> {
    let diagnostics = validate_inventory_config(cfg);
    if !diagnostics.is_empty() {
        return Err(InventorySimulationError { diagnostics });
    }

    let mut paths = Vec::with_capacity(cfg.n_paths);
    let thread_count = inventory_thread_count(cfg);
    if thread_count == 1 {
        let mut arrival_schedule = vec![0.0; cfg.lead_time_periods.max(1)];
        for path_index in 0..cfg.n_paths {
            arrival_schedule.fill(0.0);
            paths.push(simulate_path(cfg, path_index, &mut arrival_schedule, None));
        }
    } else {
        let chunk_size = cfg.n_paths.div_ceil(thread_count);
        thread::scope(|scope| {
            let mut handles = Vec::with_capacity(thread_count);
            for chunk_index in 0..thread_count {
                let start = chunk_index * chunk_size;
                let end = (start + chunk_size).min(cfg.n_paths);
                if start >= end {
                    break;
                }
                handles.push(scope.spawn(move || {
                    let mut chunk = Vec::with_capacity(end - start);
                    let mut arrival_schedule = vec![0.0; cfg.lead_time_periods.max(1)];
                    for path_index in start..end {
                        arrival_schedule.fill(0.0);
                        chunk.push(simulate_path(cfg, path_index, &mut arrival_schedule, None));
                    }
                    chunk
                }));
            }
            for handle in handles {
                paths.extend(
                    handle
                        .join()
                        .expect("inventory worker thread should not panic"),
                );
            }
        });
    }

    let summary = summarize_paths(&paths);
    let traces = collect_traces(cfg);
    Ok(InventorySimulationResult {
        manifest: InventoryRunManifest {
            schema_version: INVENTORY_SCHEMA_VERSION.to_string(),
            semantics_version: INVENTORY_SEMANTICS_VERSION.to_string(),
            backend: "cpu_reference".to_string(),
            period_order: PERIOD_ORDER.to_string(),
            n_paths: cfg.n_paths,
            n_periods: cfg.n_periods,
            warmup_periods: cfg.warmup_periods,
            observed_periods: cfg.n_periods - cfg.warmup_periods,
            seed: cfg.seed,
            shortage_policy: cfg.shortage_policy,
            lead_time_periods: cfg.lead_time_periods,
            lead_time_semantics: "arrival_after_max(1,lead_time_periods)_period_boundaries"
                .to_string(),
            demand: cfg.demand,
            normal_demand_semantics: "max(0,mean+std_dev*standard_normal)".to_string(),
            warehouse_capacity_semantics: "order_commitment_clip_on_hand_plus_on_order".to_string(),
            rng_stream_mapping: "splitmix64(base_seed,path_index)".to_string(),
            quantile_method: "linear_interpolation_sorted_path_values".to_string(),
            trace_paths: cfg.trace.path_indices.clone(),
            trace_period_limit: cfg.trace.max_periods.min(cfg.n_periods),
            config: cfg.clone(),
        },
        summary,
        paths,
        traces,
    })
}

fn collect_traces(cfg: &InventorySimulationConfig) -> Vec<InventoryPathTrace> {
    if cfg.trace.path_indices.is_empty() || cfg.trace.max_periods == 0 {
        return Vec::new();
    }

    cfg.trace
        .path_indices
        .iter()
        .map(|&path_index| {
            let mut periods = Vec::with_capacity(cfg.trace.max_periods.min(cfg.n_periods));
            let mut arrival_schedule = vec![0.0; cfg.lead_time_periods.max(1)];
            simulate_path(cfg, path_index, &mut arrival_schedule, Some(&mut periods));
            InventoryPathTrace {
                path_index,
                periods,
            }
        })
        .collect()
}

fn inventory_thread_count(cfg: &InventorySimulationConfig) -> usize {
    let requested = if cfg.n_threads == 0 {
        thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1)
    } else {
        cfg.n_threads
    };
    requested.clamp(1, cfg.n_paths)
}

fn simulate_path(
    cfg: &InventorySimulationConfig,
    path_index: usize,
    arrival_schedule: &mut [f64],
    mut trace: Option<&mut Vec<InventoryPeriodTrace>>,
) -> InventoryPathResult {
    let arrival_offset = cfg.lead_time_periods.max(1);
    let mut rng = MonteCarloRng::new(path_seed(cfg.seed, path_index));
    let mut on_hand = cfg.initial_on_hand;
    let mut on_order = 0.0;
    let mut backorder = cfg.initial_backorder;

    let mut service_periods = 0usize;
    let mut stockout_events = 0usize;
    let mut total_demand = 0.0;
    let mut fulfilled_demand = 0.0;
    let mut unmet_demand = 0.0;
    let mut on_hand_sum = 0.0;
    let mut received_units = 0.0;
    let mut ordered_units = 0.0;
    let mut orders_placed = 0usize;
    let mut constraint_events = InventoryConstraintEvents::default();
    let mut holding_cost = 0.0;
    let mut shortage_cost = 0.0;
    let mut ordering_cost = 0.0;

    for period in 0..cfg.n_periods {
        let schedule_index = period % arrival_offset;
        let receipt = arrival_schedule[schedule_index];
        arrival_schedule[schedule_index] = 0.0;
        on_order = non_negative(on_order - receipt);

        let backorder_receipt = receipt.min(backorder);
        backorder -= backorder_receipt;
        on_hand += receipt - backorder_receipt;

        let demand = sample_demand(cfg.demand, &mut rng);
        let fulfilled = demand.min(on_hand);
        on_hand -= fulfilled;
        let unmet = demand - fulfilled;
        match cfg.shortage_policy {
            InventoryShortagePolicy::Backorder => backorder += unmet,
            InventoryShortagePolicy::LostSales => {}
        }

        let inventory_position = on_hand + on_order - backorder;
        let order_decision = calculate_order_quantity(cfg, inventory_position, on_hand, on_order);
        let order_quantity = order_decision.quantity;
        if order_quantity > 0.0 {
            let arrival_index = (period + arrival_offset) % arrival_offset;
            arrival_schedule[arrival_index] += order_quantity;
            on_order += order_quantity;
        }

        let observed = period >= cfg.warmup_periods;
        let period_holding_cost = if observed {
            on_hand * cfg.costs.holding_cost_per_unit_period
        } else {
            0.0
        };
        let period_shortage_cost = if observed {
            match cfg.shortage_policy {
                InventoryShortagePolicy::Backorder => {
                    backorder * cfg.costs.backorder_cost_per_unit_period
                }
                InventoryShortagePolicy::LostSales => unmet * cfg.costs.lost_sale_cost_per_unit,
            }
        } else {
            0.0
        };
        let period_ordering_cost = if observed && order_quantity > 0.0 {
            cfg.costs.fixed_order_cost + order_quantity * cfg.costs.variable_order_cost_per_unit
        } else {
            0.0
        };

        if observed {
            constraint_events.add_assign(order_decision.constraint_events);
            total_demand += demand;
            fulfilled_demand += fulfilled;
            unmet_demand += unmet;
            on_hand_sum += on_hand;
            received_units += receipt;
            if unmet == 0.0 {
                service_periods += 1;
            } else {
                stockout_events += 1;
            }
            if order_quantity > 0.0 {
                ordered_units += order_quantity;
                orders_placed += 1;
                ordering_cost += period_ordering_cost;
            }
            holding_cost += period_holding_cost;
            shortage_cost += period_shortage_cost;
        }
        if let Some(periods) = trace.as_deref_mut() {
            if period < cfg.trace.max_periods {
                periods.push(InventoryPeriodTrace {
                    period,
                    observed,
                    received_units: receipt,
                    demand,
                    fulfilled_demand: fulfilled,
                    unmet_demand: unmet,
                    inventory_position_before_order: inventory_position,
                    order_quantity,
                    constraint_events: order_decision.constraint_events,
                    holding_cost: period_holding_cost,
                    shortage_cost: period_shortage_cost,
                    ordering_cost: period_ordering_cost,
                    ending_on_hand: non_negative(on_hand),
                    ending_on_order: non_negative(on_order),
                    ending_backorder: non_negative(backorder),
                });
            }
        }
    }

    let observed_periods = (cfg.n_periods - cfg.warmup_periods) as f64;
    InventoryPathResult {
        path_index,
        cycle_service_level: service_periods as f64 / observed_periods,
        fill_rate: if total_demand > 0.0 {
            fulfilled_demand / total_demand
        } else {
            1.0
        },
        average_on_hand: on_hand_sum / observed_periods,
        stockout_events,
        total_demand,
        fulfilled_demand,
        unmet_demand,
        received_units,
        ordered_units,
        orders_placed,
        constraint_events,
        holding_cost,
        shortage_cost,
        ordering_cost,
        total_cost: holding_cost + shortage_cost + ordering_cost,
        ending_on_hand: non_negative(on_hand),
        ending_on_order: non_negative(on_order),
        ending_backorder: non_negative(backorder),
    }
}

fn validate_trace_config(
    cfg: &InventorySimulationConfig,
    diagnostics: &mut Vec<InventoryValidationDiagnostic>,
) {
    if cfg.trace.path_indices.len() > MAX_TRACE_PATHS {
        diagnostics.push(diagnostic(
            "inventory.trace.path_limit",
            "trace.path_indices",
            "trace path count exceeds the bounded diagnostic limit",
            "Request traces for at most 16 paths.",
        ));
    }
    if cfg.trace.max_periods > MAX_TRACE_PERIODS {
        diagnostics.push(diagnostic(
            "inventory.trace.period_limit",
            "trace.max_periods",
            "trace period count exceeds the bounded diagnostic limit",
            "Request at most 10,000 periods per traced path.",
        ));
    }
    if !cfg.trace.path_indices.is_empty() && cfg.trace.max_periods == 0 {
        diagnostics.push(diagnostic(
            "inventory.trace.periods_required",
            "trace.max_periods",
            "trace max_periods must be positive when trace paths are requested",
            "Set trace.max_periods to a positive bounded value.",
        ));
    }

    let mut unique = HashSet::with_capacity(cfg.trace.path_indices.len());
    for &path_index in &cfg.trace.path_indices {
        if !unique.insert(path_index) {
            diagnostics.push(diagnostic(
                "inventory.trace.path_duplicate",
                "trace.path_indices",
                "trace path indices must be unique",
                "Remove duplicate path indices.",
            ));
        }
        if path_index >= cfg.n_paths {
            diagnostics.push(diagnostic(
                "inventory.trace.path_out_of_range",
                "trace.path_indices",
                "trace path index must be less than n_paths",
                "Use only existing path indices.",
            ));
        }
    }
}

fn sample_demand(distribution: InventoryDemandDistribution, rng: &mut MonteCarloRng) -> f64 {
    match distribution {
        InventoryDemandDistribution::Deterministic { units } => units,
        InventoryDemandDistribution::Normal { mean, std_dev } => {
            non_negative(mean + std_dev * rng.standard_normal())
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct InventoryOrderDecision {
    quantity: f64,
    constraint_events: InventoryConstraintEvents,
}

fn calculate_order_quantity(
    cfg: &InventorySimulationConfig,
    inventory_position: f64,
    on_hand: f64,
    on_order: f64,
) -> InventoryOrderDecision {
    if inventory_position > cfg.policy.reorder_point {
        return InventoryOrderDecision::default();
    }

    let desired = non_negative(cfg.policy.order_up_to - inventory_position);
    if desired == 0.0 {
        return InventoryOrderDecision::default();
    }

    let mut constraint_events = InventoryConstraintEvents::default();
    let mut quantity = desired;
    let with_minimum = quantity.max(cfg.constraints.minimum_order_quantity);
    if materially_different(with_minimum, quantity) {
        constraint_events.minimum_order_quantity_adjustments += 1;
    }
    quantity = with_minimum;

    let packed = (quantity / cfg.constraints.case_pack).ceil() * cfg.constraints.case_pack;
    if materially_different(packed, quantity) {
        constraint_events.case_pack_roundups += 1;
    }
    quantity = packed;

    if let Some(capacity) = cfg.constraints.supplier_capacity_per_period {
        let clipped = quantity.min(capacity);
        if materially_different(clipped, quantity) {
            constraint_events.supplier_capacity_clips += 1;
        }
        quantity = clipped;
    }
    if let Some(capacity) = cfg.constraints.warehouse_capacity {
        let clipped = quantity.min(non_negative(capacity - on_hand - on_order));
        if materially_different(clipped, quantity) {
            constraint_events.warehouse_capacity_clips += 1;
        }
        quantity = clipped;
    }

    let quantity = non_negative(quantity);
    if constraint_events.has_adjustment() {
        constraint_events.limited_order_attempts += 1;
    }
    if quantity == 0.0 {
        constraint_events.blocked_order_attempts += 1;
    }
    InventoryOrderDecision {
        quantity,
        constraint_events,
    }
}

fn summarize_paths(paths: &[InventoryPathResult]) -> InventorySimulationSummary {
    InventorySimulationSummary {
        cycle_service_level: summarize_metric(paths, |path| path.cycle_service_level),
        fill_rate: summarize_metric(paths, |path| path.fill_rate),
        average_on_hand: summarize_metric(paths, |path| path.average_on_hand),
        stockout_events: summarize_metric(paths, |path| path.stockout_events as f64),
        total_cost: summarize_metric(paths, |path| path.total_cost),
        ending_on_hand: summarize_metric(paths, |path| path.ending_on_hand),
        ending_on_order: summarize_metric(paths, |path| path.ending_on_order),
        ending_backorder: summarize_metric(paths, |path| path.ending_backorder),
    }
}

fn summarize_metric(
    paths: &[InventoryPathResult],
    value: impl Fn(&InventoryPathResult) -> f64,
) -> InventoryMetricSummary {
    let mut values: Vec<f64> = paths.iter().map(value).collect();
    values.sort_by(f64::total_cmp);
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    InventoryMetricSummary {
        mean,
        min: values[0],
        p05: interpolated_quantile(&values, 0.05),
        p50: interpolated_quantile(&values, 0.50),
        p95: interpolated_quantile(&values, 0.95),
        max: values[values.len() - 1],
    }
}

fn interpolated_quantile(sorted_values: &[f64], probability: f64) -> f64 {
    let index = probability * (sorted_values.len() - 1) as f64;
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;
    if lower == upper {
        sorted_values[lower]
    } else {
        let weight = index - lower as f64;
        sorted_values[lower] * (1.0 - weight) + sorted_values[upper] * weight
    }
}

fn path_seed(base_seed: u64, path_index: usize) -> u64 {
    let mut value = base_seed.wrapping_add((path_index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

fn non_negative(value: f64) -> f64 {
    value.max(0.0)
}

fn materially_different(left: f64, right: f64) -> bool {
    (left - right).abs() > f64::EPSILON * left.abs().max(right.abs()).max(1.0)
}

impl InventoryConstraintEvents {
    fn has_adjustment(&self) -> bool {
        self.minimum_order_quantity_adjustments > 0
            || self.case_pack_roundups > 0
            || self.supplier_capacity_clips > 0
            || self.warehouse_capacity_clips > 0
    }

    fn add_assign(&mut self, other: Self) {
        self.limited_order_attempts += other.limited_order_attempts;
        self.minimum_order_quantity_adjustments += other.minimum_order_quantity_adjustments;
        self.case_pack_roundups += other.case_pack_roundups;
        self.supplier_capacity_clips += other.supplier_capacity_clips;
        self.warehouse_capacity_clips += other.warehouse_capacity_clips;
        self.blocked_order_attempts += other.blocked_order_attempts;
    }
}

fn diagnostic(
    code: &str,
    field: &str,
    message: &str,
    suggestion: &str,
) -> InventoryValidationDiagnostic {
    InventoryValidationDiagnostic {
        code: code.to_string(),
        field: field.to_string(),
        message: message.to_string(),
        suggestion: suggestion.to_string(),
    }
}

fn validate_non_negative(
    diagnostics: &mut Vec<InventoryValidationDiagnostic>,
    value: f64,
    code: &str,
    field: &str,
) {
    if !value.is_finite() || value < 0.0 {
        diagnostics.push(diagnostic(
            code,
            field,
            &format!("{field} must be finite and non-negative"),
            "Provide a finite value greater than or equal to zero.",
        ));
    }
}

fn validate_optional_non_negative(
    diagnostics: &mut Vec<InventoryValidationDiagnostic>,
    value: Option<f64>,
    code: &str,
    field: &str,
) {
    if let Some(value) = value {
        validate_non_negative(diagnostics, value, code, field);
    }
}
