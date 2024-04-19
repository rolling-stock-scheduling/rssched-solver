mod neighborhood;
use std::sync::Arc;

use crate::objective;
use heuristic_framework::local_search::local_improver::{
    TakeAnyParallelRecursion, TakeFirstRecursion,
};
use heuristic_framework::local_search::LocalSearchSolver;
use model::base_types::VehicleIdx;
use model::network::Network;
use objective_framework::EvaluatedSolution;
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
    // TODO: Caplse schedule into new struct ScheduleWithInfo that also contains additional infos:
    // - last swap
    // then use last provider to start the next neighborhood
    // print swap between each step (function must be provided to the local search solver)

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

    // TODO: implement function_between_steps, that prints the swap
    let function_between_steps = Box::new(
        |_evaluated_solution: &EvaluatedSolution<ScheduleWithInfo>| {
            // TODO: parameter should
            // contain: old solution, new
            // solution, iteration
            // number, Arc<Objective>
            println!("Swap: {}", _evaluated_solution.solution().get_print_text());
        },
    );

    LocalSearchSolver::with_local_improver_and_function(
        neighborhood,
        objective,
        Some(_take_any),
        // None,
        Some(function_between_steps),
    )
}
