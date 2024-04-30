use model::json_serialisation::load_rolling_stock_problem_instance_from_json;
use objective_framework::EvaluatedSolution;
use objective_framework::Objective;
use solution::json_serialisation::schedule_to_json;
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

    let min_cost_flow_solver = MinCostFlowSolver::initialize(network.clone());
    println!("Solve with MinCostFlowSolver:");
    let schedule = min_cost_flow_solver.solve();
    let schedule_with_info = ScheduleWithInfo::new(schedule, None, "MinCostFlowSolver".to_string());

    let final_solution = objective.evaluate(schedule_with_info);
    println!(
        "MinCostFlowSolver computed optimal schedule (elapsed time: {:0.2}sec)",
        start_time.elapsed().as_secs_f32()
    );

    let end_time = stdtime::Instant::now();
    let runtime_duration = end_time.duration_since(start_time);

    println!("Objective value:");
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
