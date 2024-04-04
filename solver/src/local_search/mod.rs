mod neighborhood;
pub mod objective; // TODO make this private
use std::sync::Arc;

use heuristic_framework::local_search::local_improver::TakeAnyParallelRecursion;
use heuristic_framework::local_search::LocalSearchSolver;
use model::network::Network;
use solution::Schedule;

use neighborhood::LimitedExchanges;
// use time::Duration;

pub fn build_local_search_solver(network: Arc<Network>) -> LocalSearchSolver<Schedule> {
    let objective = Arc::new(objective::build());

    // let segment_limit = Duration::new("3:00:00");
    // let overhead_threshold = Duration::new("0:05:00"); // tours of real-vehicle-providers are not splitted at nodes under these duration

    let neighborhood = Arc::new(LimitedExchanges::new(
        // Some(segment_limit),
        // Some(overhead_threshold),
        None, None, false, network,
    ));

    let take_any = Box::new(TakeAnyParallelRecursion::new(
        0,
        Some(0),
        neighborhood.clone(),
        objective.clone(),
    ));

    LocalSearchSolver::with_local_improver(neighborhood, objective, take_any)
}
