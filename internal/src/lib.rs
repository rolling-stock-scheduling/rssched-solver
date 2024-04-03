#![allow(unused_imports)]
mod test_objective;

use solver::local_search::objective;
use solver::local_search::RollingStockLocalSearch;
use solver::min_cost_flow_solver::MinCostFlowSolver;

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
    // let objective = Arc::new(test_objective::build());

    /*
    // use min_cost_max_matching_solver as start solution
    let min_cost_max_matching_solver = MinCostMaxMatchingSolver::initialize(
        network.clone(),
        objective.clone(),
    );
    let start_solution = min_cost_max_matching_solver.solve();
    println!(
        "\n*** MinCostMaxMatchingSolver computed initial schedule (elapsed time: {:0.2}sec) ***",
        start_time.elapsed().as_secs_f32()
    );
    // */

    // /*
    // use min_cost_flow_solver as start solution
    println!("Solve with MinCostFlowSolver:");
    let min_cost_flow_solver = MinCostFlowSolver::initialize(network.clone());
    let start_solution = objective.evaluate(min_cost_flow_solver.solve());
    println!(
        "MinCostFlowSolver computed optimal schedule (elapsed time: {:0.2}sec)",
        start_time.elapsed().as_secs_f32()
    );
    // */
    let final_solution = start_solution;
    /*
    // initialize local search
    let mut local_search_solver = LocalSearch::initialize(
        network.clone(),
        objective.clone(),
    );
    local_search_solver.set_initial_solution(start_solution);
    let final_solution = local_search_solver.solve();
    // */
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
