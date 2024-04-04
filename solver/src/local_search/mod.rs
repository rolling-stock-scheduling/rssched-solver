mod neighborhood;
pub mod objective; // TODO make this private
use std::sync::Arc;

use heuristic_framework::local_search::{local_improver::TakeAnyParallelRecursion, LocalSearch};
use heuristic_framework::Solver;
use model::network::Network;
use objective_framework::EvaluatedSolution;
use solution::Schedule;

use neighborhood::LimitedExchanges;
// use time::Duration;

pub struct RollingStockLocalSearch {
    solver: LocalSearch<Schedule>,
}

impl RollingStockLocalSearch {
    pub fn initialize(network: Arc<Network>) -> RollingStockLocalSearch {
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

        let solver = LocalSearch::with_local_improver(neighborhood, objective, take_any);

        RollingStockLocalSearch { solver }
    }

    pub fn solve(&self, initial_solution: Schedule) -> EvaluatedSolution<Schedule> {
        self.solver.solve(initial_solution)
    }
}
