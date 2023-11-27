pub mod greedy;
// pub(crate) mod local_search;

use objective_framework::{EvaluatedSolution, Objective};
use sbb_model::{config::Config, network::Network, vehicle_types::VehicleTypes};
use sbb_solution::Schedule;
use std::sync::Arc;

pub trait Solver {
    fn initialize(
        vehicle_types: Arc<VehicleTypes>,
        network: Arc<Network>,
        config: Arc<Config>,
        objective: Arc<Objective<Schedule>>,
    ) -> Self;

    fn solve(&self) -> EvaluatedSolution<Schedule>;
}
