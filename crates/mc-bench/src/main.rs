use std::env;
use std::fs;

use mc_bench::{build_competitiveness_plan, run_default_benchmarks};

fn main() {
    let output = parse_output_path();
    let report = run_default_benchmarks();
    let json = serde_json::to_string_pretty(&report)
        .expect("benchmark report serialization should succeed");

    if let Some(path) = output {
        fs::write(&path, json).expect("writing benchmark output should succeed");
        let plan_path = "benchmarks/improvement-plan.md";
        let plan = build_competitiveness_plan(&report);
        fs::write(plan_path, plan).expect("writing competitiveness plan should succeed");
        println!("Benchmark report written to {path}");
        println!("Competitiveness plan written to {plan_path}");
    } else {
        println!("{json}");
    }
}

fn parse_output_path() -> Option<String> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--output" {
            return args.next();
        }
    }
    None
}
