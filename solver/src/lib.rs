pub mod local_search;
pub mod min_cost_flow_solver;
pub mod objective;
pub mod one_node_per_tour;

use model::{config::Config, network::Network, vehicle_types::VehicleTypes};
use objective_framework::{EvaluatedSolution, Objective};
use std::sync::Arc;

pub trait Solver<S> {
    fn initialize(
        vehicle_types: Arc<VehicleTypes>,
        network: Arc<Network>,
        config: Arc<Config>,
        objective: Arc<Objective<S>>,
    ) -> Self;

    fn solve(&self) -> EvaluatedSolution<S>;
}
