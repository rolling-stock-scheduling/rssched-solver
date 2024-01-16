mod test_objective;

use solver::greedy::Greedy;
use solver::local_search::LocalSearch;
use solver::Solver;

use model::json_serialisation::load_rolling_stock_problem_instance_from_json;

use std::sync::Arc;
use std::time as stdtime;

pub fn run(input_data: serde_json::Value) -> serde_json::Value {
    let (vehicle_types, network, config) =
        load_rolling_stock_problem_instance_from_json(input_data);
    let start_time = stdtime::Instant::now();

    let objective = Arc::new(server::first_phase_objective::build());
    // let objective = Arc::new(test_objective::build());

    // initialize local search
    let mut local_search_solver = LocalSearch::initialize(
        vehicle_types.clone(),
        network.clone(),
        config.clone(),
        objective.clone(),
    );

    // use greedy algorithm as start solution
    let greedy = Greedy::initialize(
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

    println!("\n*** Solved ***");
    // println!("\nfinal schedule (long version):");
    // final_solution.solution().print_tours_long();

    // println!("\n\nFinal schedule:");
    // final_solution.solution().print_tours();

    // println!("\n\nFinal train formations:");
    // final_solution.solution().print_train_formations();
    println!("\nfinal objective value:");
    objective.print_objective_value(final_solution.objective_value());

    final_solution.solution().print_depot_balances();

    println!("running time: {:0.2}sec", runtime_duration.as_secs_f32());

    server::create_output_json(&final_solution, &objective, runtime_duration)
}
