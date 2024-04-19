#![allow(unused_imports)]
mod test_objective;

use solver::local_search::ScheduleWithInfo;
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

    println!("Solve with MinCostFlowSolver:");
    let min_cost_flow_solver = MinCostFlowSolver::initialize(network.clone());
    let start_schedule = min_cost_flow_solver.solve();
    println!(
        "MinCostFlowSolver computed schedule (elapsed time: {:0.2}sec)",
        start_time.elapsed().as_secs_f32()
    );

    let start_schedule_with_info = ScheduleWithInfo::new(
        start_schedule.clone(),
        None,
        "Result from min cost flow solver".to_string(),
    );

    let final_solution = if network.maintenance_considered() {
        println!("\nStarting local search:\n");
        println!("Initial objective value:");
        objective.print_objective_value(
            objective
                .evaluate(start_schedule_with_info.clone())
                .objective_value(),
        );
        println!();

        let local_search_solver = solver::local_search::build_local_search_solver(network.clone());

        local_search_solver.solve(start_schedule_with_info)
    } else {
        println!("\nMaintenance is not considered, returning MinCostFlowSolver solution as final solution");
        objective.evaluate(start_schedule_with_info.clone())
    };

    let end_time = stdtime::Instant::now();
    let runtime_duration = end_time.duration_since(start_time);

    // println!("\nfinal schedule (long version):");
    // final_solution.solution().print_tours_long();

    println!("\nFinal schedule:");
    final_solution.solution().get_schedule().print_tours();

    // println!("\n\nFinal train formations:");
    // final_solution.solution().print_train_formations();
    println!("\nObjective value:");
    objective.print_objective_value(final_solution.objective_value());

    // final_solution.solution().print_depot_balances();
    println!(
        "Total depot balance violations: {}",
        final_solution
            .solution()
            .get_schedule()
            .total_depot_balance_violation()
    );

    println!("Running time: {:0.2}sec", runtime_duration.as_secs_f32());

    server::create_output_json(&final_solution, &objective, runtime_duration)
}
