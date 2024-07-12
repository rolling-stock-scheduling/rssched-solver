// © 2023-2024 ETH Zurich
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// any later version.
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use im::HashMap;
use model::base_types::VehicleTypeIdx;
use rapid_solve::heuristics::Solver;
use solution::transition::Transition;
use solver::local_search::neighborhood::swaps::SwapInfo;
use solver::local_search::ScheduleWithInfo;
use solver::min_cost_flow_solver::MinCostFlowSolver;
use solver::objective;

use model::json_serialisation::load_rolling_stock_problem_instance_from_json;
use solver::transition_local_search::{build_transition_local_search_solver, TransitionWithInfo};

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

    // optimize transitions
    println!("\nOptimizing transitions:");
    let start_time_transition_optimization = stdtime::Instant::now();
    let mut optimized_transitions: HashMap<VehicleTypeIdx, Transition> = HashMap::new();
    let schedule = solution.solution().get_schedule();
    let transition_local_search_solver =
        build_transition_local_search_solver(schedule, network.clone());
    for vehicle_type in network.vehicle_types().iter() {
        println!(
            "\nOptimizing transitions for vehicle type {}",
            network.vehicle_types().get(vehicle_type).unwrap()
        );
        let start_transition = TransitionWithInfo::new(
            schedule.next_day_transition_of(vehicle_type).clone(),
            "Initial transition".to_string(),
        );
        let improved_transition = transition_local_search_solver
            .solve(start_transition)
            .unwrap()
            .unwrap_transition();

        optimized_transitions.insert(vehicle_type, improved_transition);
    }
    let schedule_with_optimized_transitions =
        schedule.set_next_day_transitions(optimized_transitions);
    println!(
        "Transition optimized (elapsed time: {:0.2}sec)",
        start_time_transition_optimization.elapsed().as_secs_f32()
    );
    schedule_with_optimized_transitions.print_next_day_transitions();

    // reassign end depots to be consistent with transitions
    let final_schedule =
        schedule_with_optimized_transitions.reassign_end_depots_consistent_with_transitions();
    let final_schedule_with_info = ScheduleWithInfo::new(
        final_schedule,
        SwapInfo::NoSwap,
        "Final schedule after reassigning end depots".to_string(),
    );
    let final_solution = objective.evaluate(final_schedule_with_info);

    let end_time = stdtime::Instant::now();
    let runtime_duration = end_time.duration_since(start_time);

    // println!("\nfinal schedule (long version):");
    // final_solution.solution().print_tours_long();

    let final_schedule = final_solution.solution().get_schedule();
    println!("\nFinal schedule:");
    final_schedule.print_tours();

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
