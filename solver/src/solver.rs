pub mod greedy;
pub mod local_search;

use objective_framework::Objective;
use sbb_model::{config::Config, network::Network, vehicle_types::VehicleTypes};
use sbb_solution::Schedule;
use std::sync::Arc;

use crate::Solution;

pub trait Solver {
    fn initialize(
        vehicle_types: Arc<VehicleTypes>,
        network: Arc<Network>,
        config: Arc<Config>,
        objective: Arc<Objective<Schedule>>,
    ) -> Self;

    fn solve(&self) -> Solution;
}
