#![allow(unused_imports)]
mod test_objective;

use solver::min_cost_flow_solver::MinCostFlowSolver;
use solver::objective;

use model::json_serialisation::load_rolling_stock_problem_instance_from_json;

use std::sync::Arc;
use std::time as stdtime;

pub fn run(input_data: serde_json::Value) -> serde_json::Value {
    let start_time = stdtime::Instant::now();
    let network = load_rolling_stock_problem_instance_from_json(input_data);
    println!(
        "Instance with {} vehicle types and {} trips loaded (elapsed time: {:0.2}sec)",
        network.vehicle_types().iter().count(),
        network.size(),
        start_time.elapsed().as_secs_f32()
    );

    let objective = Arc::new(objective::build());

    // use min_cost_flow_solver as start solution
    println!("Solve with MinCostFlowSolver:");
    let min_cost_flow_solver = MinCostFlowSolver::initialize(network.clone());
    let start_schedule = min_cost_flow_solver.solve();
    println!(
        "MinCostFlowSolver computed start schedule (elapsed time: {:0.2}sec)",
        start_time.elapsed().as_secs_f32()
    );

    println!("\nStarting local search:\n");
    println!("Initial objective value:");
    objective.print_objective_value(objective.evaluate(start_schedule.clone()).objective_value());
    println!();
    // initialize local search
    let local_search_solver = solver::local_search::build_local_search_solver(network.clone());
    let final_solution = local_search_solver.solve(start_schedule);

    let end_time = stdtime::Instant::now();
    let runtime_duration = end_time.duration_since(start_time);

    // println!("\nfinal schedule (long version):");
    // final_solution.solution().print_tours_long();

    println!("Final schedule:");
    final_solution.solution().print_tours();

    // println!("\n\nFinal train formations:");
    // final_solution.solution().print_train_formations();
    println!("Objective value:");
    objective.print_objective_value(final_solution.objective_value());

    // final_solution.solution().print_depot_balances();
    println!(
        "Total depot balance violations: {}",
        final_solution.solution().total_depot_balance_violation()
    );

    println!("Running time: {:0.2}sec", runtime_duration.as_secs_f32());

    server::create_output_json(&final_solution, &objective, runtime_duration)
}
