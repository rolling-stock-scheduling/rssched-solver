use model::json_serialisation::load_rolling_stock_problem_instance_from_json;
use objective_framework::EvaluatedSolution;
use objective_framework::Objective;
use solution::json_serialisation::schedule_to_json;
use solver::local_search::neighborhood::swaps::SwapInfo;
use solver::local_search::ScheduleWithInfo;
use solver::min_cost_flow_solver::MinCostFlowSolver;
use solver::objective;
use time::{DateTime, Duration};

use gethostname::gethostname;
use std::sync::Arc;
use std::time as stdtime;

pub fn solve_instance(input_data: serde_json::Value) -> serde_json::Value {
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
        start_schedule.improve_depots(None),
        SwapInfo::NoSwap,
        "Result from min cost flow solver".to_string(),
    );

    let solution = if network.maintenance_considered() {
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

    // TODO add transition local search here

    // reassign end depots to be consistent with transitions
    let final_schedule = solution
        .solution()
        .get_schedule()
        .reassign_end_depots_consistent_with_transitions();
    let final_schedule_with_info = ScheduleWithInfo::new(
        final_schedule,
        SwapInfo::NoSwap,
        "Final schedule after reassigning end depots".to_string(),
    );
    let final_solution = objective.evaluate(final_schedule_with_info);

    let end_time = stdtime::Instant::now();
    let runtime_duration = end_time.duration_since(start_time);

    let final_schedule = final_solution.solution().get_schedule();

    let overflow_depot = network.overflow_depot_idxs().0;
    for vehicle_type in network.vehicle_types().iter() {
        if final_schedule.number_of_vehicles_of_same_type_spawned_at(overflow_depot, vehicle_type)
            > 0
        {
            println!(
            "\x1b[93mnote:\x1b[0m vehicle type {} uses the overflow depot. Consider adding more depot capacity for this type.",
            vehicle_type
        );
        }
    }

    println!("\nObjective value:");
    objective.print_objective_value(final_solution.objective_value());

    println!("Running time: {:0.2}sec", runtime_duration.as_secs_f32());

    create_output_json(&final_solution, &objective, runtime_duration)
}

pub fn create_output_json(
    final_solution: &EvaluatedSolution<ScheduleWithInfo>,
    objective: &Objective<ScheduleWithInfo>,
    runtime_duration: stdtime::Duration,
) -> serde_json::Value {
    let json_output = schedule_to_json(final_solution.solution().get_schedule());
    let json_objective_value = objective.objective_value_to_json(final_solution.objective_value());
    let today = DateTime::new("1970-01-01T00:00:00")
        + Duration::from_seconds(
            stdtime::SystemTime::now()
                .duration_since(stdtime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    serde_json::json!({
        "info": {
            "runningTime": format!("{:0.2}sec", runtime_duration.as_secs_f32()),
            "numberOfThreads": rayon::current_num_threads(),
            "timestampUTC": today.as_iso(),
            "hostname": gethostname().into_string().unwrap_or("unknown".to_string()),
        },
        "objectiveValue": json_objective_value,
        "schedule": json_output,
    })
}
