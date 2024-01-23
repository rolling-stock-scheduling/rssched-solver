use model::json_serialisation::load_rolling_stock_problem_instance_from_json;
use objective_framework::Objective;
use solution::json_serialisation::schedule_to_json;
use solution::Schedule;
use solver::first_phase_objective;
use solver::greedy::Greedy;
use solver::local_search::LocalSearch;
use solver::Solution;
use solver::Solver;
use time::{DateTime, Duration};

use gethostname::gethostname;
use std::sync::Arc;
use std::time as stdtime;

pub fn solve_instance(input_data: serde_json::Value) -> serde_json::Value {
    let start_time = stdtime::Instant::now();
    let (vehicle_types, network, config) =
        load_rolling_stock_problem_instance_from_json(input_data);
    println!(
        "*** Instance with {} vehicle types and {} trips loaded (elapsed time: {:0.2}sec) ***",
        vehicle_types.iter().count(),
        network.size(),
        start_time.elapsed().as_secs_f32()
    );

    let objective = Arc::new(first_phase_objective::build());

    // initialize local search
    let mut local_search_solver = LocalSearch::initialize(
        vehicle_types.clone(),
        network.clone(),
        config.clone(),
        objective.clone(),
    );

    // use greedy algorithm as start solution
    println!(
        "\n*** Run Greedy (elapsed time: {:0.2}sec) ***",
        start_time.elapsed().as_secs_f32()
    );
    let greedy = Greedy::initialize(
        vehicle_types.clone(),
        network.clone(),
        config.clone(),
        objective.clone(),
    );
    let start_solution = greedy.solve();
    objective.print_objective_value(start_solution.objective_value());

    // run local search
    println!(
        "\n*** Run Local Search (elapsed time: {:0.2}sec) ***",
        start_time.elapsed().as_secs_f32()
    );
    local_search_solver.set_initial_solution(start_solution);
    let final_solution = local_search_solver.solve();

    let end_time = stdtime::Instant::now();
    let runtime_duration = end_time.duration_since(start_time);

    println!("\n*** Solved ***");

    println!("\nfinal objective value:");
    objective.print_objective_value(final_solution.objective_value());

    println!("\nrunning time: {:0.2}sec", runtime_duration.as_secs_f32());

    create_output_json(&final_solution, &objective, runtime_duration)
}

pub fn create_output_json(
    final_solution: &Solution,
    objective: &Objective<Schedule>,
    runtime_duration: stdtime::Duration,
) -> serde_json::Value {
    let json_output = schedule_to_json(final_solution.solution());
    let json_objective_value = objective.objective_value_to_json(final_solution.objective_value());
    let today = DateTime::new("1970-01-01T00:00:00")
        + Duration::from_seconds(
            stdtime::SystemTime::now()
                .duration_since(stdtime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    serde_json::json!({
        "info:": {
            "runningTime": format!("{:0.2}sec", runtime_duration.as_secs_f32()),
            "numberOfThreads": rayon::current_num_threads(),
            "timestamp(UTC)": today.as_iso(),
            "hostname": gethostname().into_string().unwrap_or("unknown".to_string()),
        },
        "objectiveValue": json_objective_value,
        "schedule": json_output,
    })
}
