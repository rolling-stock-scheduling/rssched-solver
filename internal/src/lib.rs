mod test_objective;

#[allow(unused_imports)]
use solver::greedy::Greedy;
use solver::local_search::LocalSearch;
use solver::max_matching_solver::MaxMatchingSolver;
use solver::{first_phase_objective, Solver};

use model::json_serialisation::load_rolling_stock_problem_instance_from_json;

use std::sync::Arc;
use std::time as stdtime;

pub fn run(input_data: serde_json::Value) -> serde_json::Value {
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
    // let objective = Arc::new(test_objective::build());

    /*
    // use greedy algorithm as start solution
    let greedy = Greedy::initialize(
        vehicle_types.clone(),
        network.clone(),
        config.clone(),
        objective.clone(),
    );
    let start_solution = greedy.solve();
    // */
    // /*
    // use matching_solver as start solution
    let matching_solver = MaxMatchingSolver::initialize(
        vehicle_types.clone(),
        network.clone(),
        config.clone(),
        objective.clone(),
    );
    let start_solution = matching_solver.solve();
    println!(
        "\n*** Matching-Solver computed initial schedule (elapsed time: {:0.2}sec) ***",
        start_time.elapsed().as_secs_f32()
    );
    // */
    objective.print_objective_value(start_solution.objective_value());

    // let final_solution = start_solution;
    // /*
    // initialize local search
    let mut local_search_solver = LocalSearch::initialize(
        vehicle_types.clone(),
        network.clone(),
        config.clone(),
        objective.clone(),
    );
    local_search_solver.set_initial_solution(start_solution);
    let final_solution = local_search_solver.solve();
    // */
    let end_time = stdtime::Instant::now();
    let runtime_duration = end_time.duration_since(start_time);

    println!("\n*** Solved ***");
    // println!("\nfinal schedule (long version):");
    // final_solution.solution().print_tours_long();

    // println!("\n\nFinal schedule:");
    // final_solution.solution().print_tours();

    // println!("\n\nFinal train formations:");
    // final_solution.solution().print_train_formations();
    println!("\nfinal objective value:");
    objective.print_objective_value(final_solution.objective_value());

    // final_solution.solution().print_depot_balances();

    println!("running time: {:0.2}sec", runtime_duration.as_secs_f32());

    server::create_output_json(&final_solution, &objective, runtime_duration)
}
