mod test_objective;

use solution::json_serialisation::schedule_to_json;
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

    // use greedy algorithm
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

    println!("\n\nFinal schedule (long version):");
    final_solution.solution().print_tours_long();

    // println!("\n\nFinal schedule:");
    // final_solution.solution().print_tours();

    // println!("\n\nFinal train formations:");
    // final_solution.solution().print_train_formations();
    println!();
    println!("\nFinal objective value:");
    objective.print_objective_value(final_solution.objective_value());

    final_solution.solution().print_depot_balances();

    println!("Running time: {:0.2}sec", runtime_duration.as_secs_f32());

    let json_output = schedule_to_json(final_solution.solution());
    let json_objective_value = objective.objective_value_to_json(final_solution.objective_value());
    serde_json::json!({
        "objectiveValue": json_objective_value,
        "schedule": json_output,
    })
}
