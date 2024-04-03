mod neighborhood;
mod objective;
use std::sync::Arc;

use heuristic_framework::{local_search::LocalSearch, Solver};
use model::network::Network;
use objective_framework::EvaluatedSolution;
use solution::Schedule;

use neighborhood::LimitedExchanges;
// use time::Duration;

pub struct RollingStockLocalSearch {
    solver: LocalSearch<Schedule>,
}

impl RollingStockLocalSearch {
    pub fn initialize(
        initial_solution: Schedule,
        network: Arc<Network>,
    ) -> RollingStockLocalSearch {
        let objective = Arc::new(objective::build());

        // let segment_limit = Duration::new("3:00:00");
        // let overhead_threshold = Duration::new("0:05:00"); // tours of real-vehicle-providers are not splitted at nodes under these duration

        let neighborhood = Arc::new(LimitedExchanges::new(
            // Some(segment_limit),
            // Some(overhead_threshold),
            None, None, false, network,
        ));

        let solver = LocalSearch::initialize(initial_solution, neighborhood, objective);

        RollingStockLocalSearch { solver }
    }

    pub fn solve(&self) -> EvaluatedSolution<Schedule> {
        self.solver.solve()
    }
}
