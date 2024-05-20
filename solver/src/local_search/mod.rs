mod neighborhood;
use std::sync::Arc;
use std::time::{self as stdtime, Instant};

use crate::objective;
use heuristic_framework::local_search::local_improver::{
    TakeAnyParallelRecursion, TakeFirstRecursion,
};
use heuristic_framework::local_search::LocalSearchSolver;
use model::base_types::VehicleIdx;
use model::network::Network;
use objective_framework::{EvaluatedSolution, Objective};
use solution::Schedule;

use neighborhood::SpawnForMaintenanceAndPathExchange;
use time::Duration;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ScheduleWithInfo {
    schedule: Schedule,
    last_provider: Option<VehicleIdx>,
    print_text: String,
}

impl ScheduleWithInfo {
    pub fn new(
        schedule: Schedule,
        last_provider: Option<VehicleIdx>,
        print_text: String,
    ) -> ScheduleWithInfo {
        ScheduleWithInfo {
            schedule,
            last_provider,
            print_text,
        }
    }

    pub fn get_schedule(&self) -> &Schedule {
        &self.schedule
    }

    pub fn get_last_provider(&self) -> Option<VehicleIdx> {
        self.last_provider
    }

    pub fn get_print_text(&self) -> &str {
        &self.print_text
    }
}

pub fn build_local_search_solver(network: Arc<Network>) -> LocalSearchSolver<ScheduleWithInfo> {
    let objective = Arc::new(objective::build());

    let segment_limit = Duration::new("3:00:00");
    let overhead_threshold = Duration::new("0:10:00"); // tours of real-vehicle-providers are not splitted at nodes under these duration

    let neighborhood = Arc::new(SpawnForMaintenanceAndPathExchange::new(
        Some(segment_limit),
        // None,
        Some(overhead_threshold),
        // None,
        false,
        network,
    ));

    let _take_first = Box::new(TakeFirstRecursion::new(
        0,
        Some(0),
        neighborhood.clone(),
        objective.clone(),
    ));

    let _take_any = Box::new(TakeAnyParallelRecursion::new(
        0,
        Some(0),
        neighborhood.clone(),
        objective.clone(),
    ));

    let function_between_steps = Box::new(
        |iteration_counter: u32,
         current_solution: &EvaluatedSolution<ScheduleWithInfo>,
         previous_solution: Option<&EvaluatedSolution<ScheduleWithInfo>>,
         objective: Arc<Objective<ScheduleWithInfo>>,
         start_time: Option<Instant>| {
            println!(
                "Iteration {} - Swap: {}",
                iteration_counter,
                current_solution.solution().get_print_text()
            );
            println!(
                // TEMP
                "number of hitchhikers: {}",
                current_solution
                    .solution()
                    .get_schedule()
                    .count_hitch_hikers()
            );
            println!("Objective value:");
            match previous_solution {
                Some(prev_solution) => {
                    objective.print_objective_value_with_comparison(
                        current_solution.objective_value(),
                        prev_solution.objective_value(),
                    );
                }
                None => {
                    objective.print_objective_value(current_solution.objective_value());
                }
            }
            if let Some(start_time) = start_time {
                println!(
                    "elapsed time for local search: {:0.2}sec",
                    stdtime::Instant::now()
                        .duration_since(start_time)
                        .as_secs_f32()
                );
            }
            println!();
        },
    );

    LocalSearchSolver::with_local_improver_and_function(
        neighborhood,
        objective,
        Some(_take_any),
        Some(function_between_steps),
    )
}
