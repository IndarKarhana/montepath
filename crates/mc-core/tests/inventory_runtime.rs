use mc_core::{
    simulate_inventory_policy_cpu, validate_inventory_config, InventoryConstraints,
    InventoryCostConfig, InventoryDemandDistribution, InventoryPolicy, InventoryShortagePolicy,
    InventorySimulationConfig, InventoryTraceConfig,
};

fn deterministic_config(demand_units: f64) -> InventorySimulationConfig {
    InventorySimulationConfig {
        n_paths: 1,
        n_periods: 3,
        warmup_periods: 0,
        seed: 42,
        n_threads: 1,
        initial_on_hand: 10.0,
        initial_backorder: 0.0,
        lead_time_periods: 1,
        demand: InventoryDemandDistribution::Deterministic {
            units: demand_units,
        },
        shortage_policy: InventoryShortagePolicy::LostSales,
        policy: InventoryPolicy {
            reorder_point: 4.0,
            order_up_to: 10.0,
        },
        constraints: InventoryConstraints::default(),
        costs: InventoryCostConfig::default(),
        trace: InventoryTraceConfig::default(),
    }
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1.0e-12,
        "actual={actual}, expected={expected}"
    );
}

#[test]
fn inventory_validation_returns_structured_diagnostics() {
    let cfg = InventorySimulationConfig {
        n_paths: 0,
        n_periods: 0,
        warmup_periods: 1,
        initial_on_hand: -1.0,
        demand: InventoryDemandDistribution::Normal {
            mean: 10.0,
            std_dev: -2.0,
        },
        policy: InventoryPolicy {
            reorder_point: 10.0,
            order_up_to: 5.0,
        },
        constraints: InventoryConstraints {
            case_pack: 0.0,
            ..InventoryConstraints::default()
        },
        ..InventorySimulationConfig::default()
    };

    let diagnostics = validate_inventory_config(&cfg);
    let codes: Vec<&str> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect();

    assert!(codes.contains(&"inventory.paths.invalid"));
    assert!(codes.contains(&"inventory.periods.invalid"));
    assert!(codes.contains(&"inventory.warmup.invalid"));
    assert!(codes.contains(&"inventory.initial_on_hand.invalid"));
    assert!(codes.contains(&"inventory.demand.normal_std_dev.invalid"));
    assert!(codes.contains(&"inventory.policy.order_up_to.invalid"));
    assert!(codes.contains(&"inventory.constraints.case_pack.invalid"));
    assert!(simulate_inventory_policy_cpu(&cfg).is_err());
}

#[test]
fn zero_demand_preserves_inventory_and_reconciles_costs() {
    let cfg = InventorySimulationConfig {
        costs: InventoryCostConfig {
            holding_cost_per_unit_period: 2.0,
            ..InventoryCostConfig::default()
        },
        ..deterministic_config(0.0)
    };

    let result = simulate_inventory_policy_cpu(&cfg).expect("valid deterministic config");
    let path = &result.paths[0];

    assert_eq!(result.manifest.schema_version, "inventory-simulation.v1");
    assert_eq!(
        result.manifest.period_order,
        "receive-demand-fulfill-review-order-record"
    );
    assert_eq!(result.manifest.config, cfg);
    assert_eq!(
        result.manifest.quantile_method,
        "linear_interpolation_sorted_path_values"
    );
    assert_eq!(path.orders_placed, 0);
    assert_eq!(path.stockout_events, 0);
    assert_close(path.ending_on_hand, 10.0);
    assert_close(path.ending_on_order, 0.0);
    assert_close(path.ending_backorder, 0.0);
    assert_close(path.cycle_service_level, 1.0);
    assert_close(path.fill_rate, 1.0);
    assert_close(path.average_on_hand, 10.0);
    assert_close(path.holding_cost, 60.0);
    assert_close(path.total_cost, 60.0);
}

#[test]
fn fixed_demand_and_lead_time_produce_auditable_replenishment_trace() {
    let result =
        simulate_inventory_policy_cpu(&deterministic_config(4.0)).expect("valid deterministic run");
    let path = &result.paths[0];

    assert_eq!(path.orders_placed, 1);
    assert_close(path.ordered_units, 8.0);
    assert_close(path.received_units, 8.0);
    assert_close(path.total_demand, 12.0);
    assert_close(path.fulfilled_demand, 12.0);
    assert_close(path.unmet_demand, 0.0);
    assert_close(path.ending_on_hand, 6.0);
    assert_close(path.ending_on_order, 0.0);
    assert_close(path.average_on_hand, 14.0 / 3.0);
    assert_close(path.cycle_service_level, 1.0);
    assert_close(path.fill_rate, 1.0);
}

#[test]
fn backorder_and_lost_sales_shortage_modes_are_explicitly_different() {
    let base = InventorySimulationConfig {
        n_periods: 2,
        initial_on_hand: 2.0,
        policy: InventoryPolicy {
            reorder_point: -100.0,
            order_up_to: 0.0,
        },
        ..deterministic_config(5.0)
    };
    let lost_sales = simulate_inventory_policy_cpu(&base).expect("valid lost-sales run");
    let backorder = simulate_inventory_policy_cpu(&InventorySimulationConfig {
        shortage_policy: InventoryShortagePolicy::Backorder,
        ..base
    })
    .expect("valid backorder run");

    assert_close(lost_sales.paths[0].ending_backorder, 0.0);
    assert_close(backorder.paths[0].ending_backorder, 8.0);
    assert_close(lost_sales.paths[0].unmet_demand, 8.0);
    assert_close(backorder.paths[0].unmet_demand, 8.0);
    assert_close(lost_sales.paths[0].fill_rate, 0.2);
    assert_close(backorder.paths[0].fill_rate, 0.2);
}

#[test]
fn order_constraints_apply_in_documented_order() {
    let cfg = InventorySimulationConfig {
        n_periods: 1,
        initial_on_hand: 0.0,
        lead_time_periods: 0,
        policy: InventoryPolicy {
            reorder_point: 100.0,
            order_up_to: 200.0,
        },
        constraints: InventoryConstraints {
            minimum_order_quantity: 120.0,
            case_pack: 50.0,
            supplier_capacity_per_period: Some(130.0),
            warehouse_capacity: Some(100.0),
        },
        ..deterministic_config(0.0)
    };

    let result = simulate_inventory_policy_cpu(&cfg).expect("valid constrained run");
    let path = &result.paths[0];

    assert_eq!(path.orders_placed, 1);
    assert_eq!(path.constraint_events.limited_order_attempts, 1);
    assert_eq!(path.constraint_events.supplier_capacity_clips, 1);
    assert_eq!(path.constraint_events.warehouse_capacity_clips, 1);
    assert_close(path.ordered_units, 100.0);
    assert_close(path.ending_on_order, 100.0);
    assert_close(path.ending_on_hand, 0.0);
}

#[test]
fn each_order_constraint_has_an_auditable_effect() {
    let base = InventorySimulationConfig {
        n_periods: 1,
        initial_on_hand: 0.0,
        policy: InventoryPolicy {
            reorder_point: 100.0,
            order_up_to: 101.0,
        },
        ..deterministic_config(0.0)
    };

    let moq = simulate_inventory_policy_cpu(&InventorySimulationConfig {
        constraints: InventoryConstraints {
            minimum_order_quantity: 120.0,
            ..InventoryConstraints::default()
        },
        ..base.clone()
    })
    .expect("valid MOQ run");
    let case_pack = simulate_inventory_policy_cpu(&InventorySimulationConfig {
        constraints: InventoryConstraints {
            case_pack: 50.0,
            ..InventoryConstraints::default()
        },
        ..base.clone()
    })
    .expect("valid case-pack run");
    let supplier = simulate_inventory_policy_cpu(&InventorySimulationConfig {
        constraints: InventoryConstraints {
            supplier_capacity_per_period: Some(80.0),
            ..InventoryConstraints::default()
        },
        ..base.clone()
    })
    .expect("valid supplier-capacity run");
    let warehouse = simulate_inventory_policy_cpu(&InventorySimulationConfig {
        constraints: InventoryConstraints {
            warehouse_capacity: Some(90.0),
            ..InventoryConstraints::default()
        },
        ..base
    })
    .expect("valid warehouse-capacity run");

    assert_close(moq.paths[0].ordered_units, 120.0);
    assert_eq!(
        moq.paths[0]
            .constraint_events
            .minimum_order_quantity_adjustments,
        1
    );
    assert_close(case_pack.paths[0].ordered_units, 150.0);
    assert_eq!(case_pack.paths[0].constraint_events.case_pack_roundups, 1);
    assert_close(supplier.paths[0].ordered_units, 80.0);
    assert_eq!(
        supplier.paths[0].constraint_events.supplier_capacity_clips,
        1
    );
    assert_close(warehouse.paths[0].ordered_units, 90.0);
    assert_eq!(
        warehouse.paths[0]
            .constraint_events
            .warehouse_capacity_clips,
        1
    );
}

#[test]
fn fully_blocked_orders_and_shortage_costs_remain_visible() {
    let blocked = simulate_inventory_policy_cpu(&InventorySimulationConfig {
        n_periods: 1,
        initial_on_hand: 0.0,
        constraints: InventoryConstraints {
            supplier_capacity_per_period: Some(0.0),
            ..InventoryConstraints::default()
        },
        ..deterministic_config(0.0)
    })
    .expect("valid blocked-order run");
    assert_eq!(blocked.paths[0].orders_placed, 0);
    assert_eq!(blocked.paths[0].constraint_events.blocked_order_attempts, 1);

    let base = InventorySimulationConfig {
        n_periods: 2,
        initial_on_hand: 2.0,
        policy: InventoryPolicy {
            reorder_point: -100.0,
            order_up_to: 0.0,
        },
        ..deterministic_config(5.0)
    };
    let lost_sales = simulate_inventory_policy_cpu(&InventorySimulationConfig {
        costs: InventoryCostConfig {
            lost_sale_cost_per_unit: 2.0,
            ..InventoryCostConfig::default()
        },
        ..base.clone()
    })
    .expect("valid lost-sales cost run");
    let backorder = simulate_inventory_policy_cpu(&InventorySimulationConfig {
        shortage_policy: InventoryShortagePolicy::Backorder,
        costs: InventoryCostConfig {
            backorder_cost_per_unit_period: 2.0,
            ..InventoryCostConfig::default()
        },
        ..base
    })
    .expect("valid backorder cost run");

    assert_close(lost_sales.paths[0].shortage_cost, 16.0);
    assert_close(backorder.paths[0].shortage_cost, 22.0);
}

#[test]
fn zero_lead_time_orders_arrive_next_period_not_same_period() {
    let cfg = InventorySimulationConfig {
        n_periods: 2,
        initial_on_hand: 0.0,
        lead_time_periods: 0,
        policy: InventoryPolicy {
            reorder_point: 0.0,
            order_up_to: 10.0,
        },
        ..deterministic_config(0.0)
    };

    let result = simulate_inventory_policy_cpu(&cfg).expect("valid zero-lead-time run");
    let path = &result.paths[0];

    assert_close(path.ordered_units, 10.0);
    assert_close(path.received_units, 10.0);
    assert_close(path.ending_on_hand, 10.0);
    assert_close(path.ending_on_order, 0.0);
}

#[test]
fn warmup_periods_update_state_but_do_not_contribute_to_metrics() {
    let cfg = InventorySimulationConfig {
        n_periods: 2,
        warmup_periods: 1,
        initial_on_hand: 10.0,
        policy: InventoryPolicy {
            reorder_point: -100.0,
            order_up_to: 0.0,
        },
        ..deterministic_config(4.0)
    };

    let result = simulate_inventory_policy_cpu(&cfg).expect("valid warmup run");
    let path = &result.paths[0];

    assert_eq!(result.manifest.observed_periods, 1);
    assert_close(path.total_demand, 4.0);
    assert_close(path.fulfilled_demand, 4.0);
    assert_close(path.average_on_hand, 2.0);
    assert_close(path.ending_on_hand, 2.0);
}

#[test]
fn stochastic_inventory_execution_is_reproducible_and_bounded() {
    let cfg = InventorySimulationConfig {
        n_paths: 64,
        n_periods: 24,
        demand: InventoryDemandDistribution::Normal {
            mean: 4.0,
            std_dev: 1.5,
        },
        ..deterministic_config(0.0)
    };

    let first = simulate_inventory_policy_cpu(&cfg).expect("valid stochastic run");
    let second = simulate_inventory_policy_cpu(&cfg).expect("valid stochastic run");

    assert_eq!(first, second);
    assert_eq!(first.paths.len(), 64);
    assert!(first.paths.iter().all(|path| {
        path.cycle_service_level >= 0.0
            && path.cycle_service_level <= 1.0
            && path.fill_rate >= 0.0
            && path.fill_rate <= 1.0
            && path.total_cost.is_finite()
            && path.ending_on_hand >= 0.0
            && path.ending_on_order >= 0.0
            && path.ending_backorder >= 0.0
    }));

    for summary in [
        &first.summary.cycle_service_level,
        &first.summary.fill_rate,
        &first.summary.average_on_hand,
        &first.summary.total_cost,
    ] {
        assert!(summary.min <= summary.p05);
        assert!(summary.p05 <= summary.p50);
        assert!(summary.p50 <= summary.p95);
        assert!(summary.p95 <= summary.max);
        assert!(summary.mean.is_finite());
    }

    for path in &first.paths {
        assert_close(
            path.total_cost,
            path.holding_cost + path.shortage_cost + path.ordering_cost,
        );
    }
}

#[test]
fn inventory_execution_is_identical_across_thread_counts() {
    let single_threaded = InventorySimulationConfig {
        n_paths: 128,
        n_periods: 24,
        n_threads: 1,
        demand: InventoryDemandDistribution::Normal {
            mean: 4.0,
            std_dev: 1.5,
        },
        ..deterministic_config(0.0)
    };
    let parallel = InventorySimulationConfig {
        n_threads: 4,
        ..single_threaded.clone()
    };

    let single = simulate_inventory_policy_cpu(&single_threaded).expect("valid single-thread run");
    let multi = simulate_inventory_policy_cpu(&parallel).expect("valid parallel run");

    assert_eq!(single.paths, multi.paths);
    assert_eq!(single.summary, multi.summary);
}

#[test]
fn bounded_trace_reports_auditable_period_state_without_changing_results() {
    let base = deterministic_config(4.0);
    let traced = InventorySimulationConfig {
        trace: InventoryTraceConfig {
            path_indices: vec![0],
            max_periods: 2,
        },
        ..base.clone()
    };

    let without_trace = simulate_inventory_policy_cpu(&base).expect("valid base run");
    let with_trace = simulate_inventory_policy_cpu(&traced).expect("valid traced run");

    assert_eq!(with_trace.paths, without_trace.paths);
    assert_eq!(with_trace.summary, without_trace.summary);
    assert_eq!(with_trace.traces.len(), 1);
    assert_eq!(with_trace.traces[0].path_index, 0);
    assert_eq!(with_trace.traces[0].periods.len(), 2);
    assert_eq!(with_trace.traces[0].periods[0].period, 0);
    assert_close(with_trace.traces[0].periods[0].demand, 4.0);
    assert_close(with_trace.traces[0].periods[0].ending_on_hand, 6.0);
    assert_close(with_trace.traces[0].periods[1].order_quantity, 8.0);
    assert_eq!(with_trace.manifest.trace_paths, vec![0]);
    assert_eq!(with_trace.manifest.trace_period_limit, 2);
}

#[test]
fn trace_validation_enforces_bounded_unique_existing_paths() {
    let cfg = InventorySimulationConfig {
        n_paths: 2,
        trace: InventoryTraceConfig {
            path_indices: vec![0, 0, 2],
            max_periods: 10_001,
        },
        ..deterministic_config(4.0)
    };

    let diagnostics = validate_inventory_config(&cfg);
    let codes: Vec<&str> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect();

    assert!(codes.contains(&"inventory.trace.path_duplicate"));
    assert!(codes.contains(&"inventory.trace.path_out_of_range"));
    assert!(codes.contains(&"inventory.trace.period_limit"));
}
