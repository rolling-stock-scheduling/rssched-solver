mod transition_neighborhood;
mod transition_objective;
use std::sync::Arc;

use heuristic_framework::local_search::LocalSearchSolver;
use model::network::Network;
use solution::{transition::Transition, Schedule};

use self::transition_neighborhood::TransitionNeighborhood;

pub fn build_transition_local_search_solver(
    schedule: &Schedule,
    network: Arc<Network>,
) -> LocalSearchSolver<Transition> {
    let objective = Arc::new(transition_objective::build());

    let neighborhood = Arc::new(TransitionNeighborhood::new(
        schedule.get_tours().clone(),
        network.clone(),
    ));

    LocalSearchSolver::initialize(neighborhood, objective)
}
