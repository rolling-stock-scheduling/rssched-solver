pub mod first_phase_objective;
pub mod greedy;
pub mod local_search;
pub mod one_node_per_tour;

use model::{config::Config, network::Network, vehicle_types::VehicleTypes};
use objective_framework::{EvaluatedSolution, Objective};
use solution::Schedule;
use std::sync::Arc;

pub type Solution = EvaluatedSolution<Schedule>;

pub trait Solver {
    fn initialize(
        vehicle_types: Arc<VehicleTypes>,
        network: Arc<Network>,
        config: Arc<Config>,
        objective: Arc<Objective<Schedule>>,
    ) -> Self;

    fn solve(&self) -> Solution;
}
