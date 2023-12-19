mod first_phase_objective;
mod solver;
mod test_objective;

use objective_framework::{EvaluatedSolution, Objective};
use sbb_solution::json_serialisation::schedule_to_json;
use sbb_solution::Schedule;
use solver::greedy::Greedy;
use solver::local_search::LocalSearch;
use solver::one_node_per_tour::OneNodePerTour;
use solver::Solver;

use sbb_model::json_serialisation::load_rolling_stock_problem_instance_from_json;

use std::path::Path;
use std::sync::Arc;
use std::{fs, time as stdtime};

type Solution = EvaluatedSolution<Schedule>;

pub fn run(path: &str) {
    println!("\n\n********** RUN: {} **********\n", path);

    let (vehicle_types, network, config) = load_rolling_stock_problem_instance_from_json(path);
    let start_time = stdtime::Instant::now();

    let objective = Arc::new(first_phase_objective::build());
    // let objective = Arc::new(test_objective::build());

    // initialize local search
    let mut local_search_solver = LocalSearch::initialize(
        vehicle_types.clone(),
        network.clone(),
        config.clone(),
        objective.clone(),
    );

    // use greedy algorithm
    let greedy = Greedy::initialize(
        vehicle_types.clone(),
        network.clone(),
        config.clone(),
        objective.clone(),
    );

    let one_node_per_tour = OneNodePerTour::initialize(
        vehicle_types.clone(),
        network.clone(),
        config.clone(),
        objective.clone(),
    );

    // solve
    let start_solution = greedy.solve();
    // let start_solution = one_node_per_tour.solve();
    local_search_solver.set_initial_solution(start_solution);
    let final_solution = local_search_solver.solve();

    let end_time = stdtime::Instant::now();
    let runtime_duration = end_time.duration_since(start_time);

    println!("\n\nFinal schedule (long version):");
    final_solution.solution().print_tours_long();

    // println!("\n\nFinal schedule:");
    // final_solution.solution().print_tours();

    // println!("\n\nFinal train formations:");
    // final_solution.solution().print_train_formations();
    println!();
    println!("\nFinal objective value:");
    objective.print_objective_value(final_solution.objective_value());

    // print the depot balance of all depots
    println!("\nDepot balances:");
    for depot in network.depots_iter() {
        for vehicle_type in vehicle_types.iter() {
            println!(
                "  depot {}, vehicle type {}: {}",
                depot,
                vehicle_type,
                final_solution.solution().depot_balance(depot, vehicle_type)
            );
        }
    }
    println!(
        "  total depot balance violation: {}",
        final_solution.solution().total_depot_balance_violation()
    );

    println!("Running time: {:0.2}sec", runtime_duration.as_secs_f32());

    // output path with sub-directory creation
    let output_dir_name = "output";
    let output_path = ensure_output_path(path, output_dir_name);
    write_solution_to_json(&final_solution, &objective, &output_path).expect("Error writing json");
}

pub fn write_solution_to_json(
    solution: &Solution,
    objective: &Objective<Schedule>,
    path: &str,
) -> Result<(), std::io::Error> {
    let json_output = schedule_to_json(solution.solution());
    let json_objective_value = objective.objective_value_to_json(solution.objective_value());
    let json_output = serde_json::json!({
        "objectiveValue": json_objective_value,
        "schedule": json_output,
    });
    let file = std::fs::File::create(path)?;
    serde_json::to_writer_pretty(file, &json_output)?;
    Ok(())
}

fn ensure_output_path(input_path: &str, output_dir_name: &str) -> String {
    let file_name = Path::new(input_path)
        .file_name()
        .expect("Error getting file name")
        .to_str()
        .expect("Error converting file name to string");
    let output_path = format!("{}/output_{}", output_dir_name, file_name);
    if let Some(parent_dir) = Path::new(&output_path).parent() {
        fs::create_dir_all(parent_dir).expect("Error creating directories");
    }
    output_path
}
