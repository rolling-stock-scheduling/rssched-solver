use crate::Solution;
use crate::Solver;
use model::config::Config;
use model::network::Network;
use model::vehicle_types::VehicleTypes;
use objective_framework::Objective;
use solution::Schedule;
use std::sync::Arc;

pub struct OneNodePerTour {
    vehicles: Arc<VehicleTypes>,
    network: Arc<Network>,
    config: Arc<Config>,
    objective: Arc<Objective<Schedule>>,
}

impl Solver for OneNodePerTour {
    fn initialize(
        vehicles: Arc<VehicleTypes>,
        network: Arc<Network>,
        config: Arc<Config>,
        objective: Arc<Objective<Schedule>>,
    ) -> Self {
        Self {
            vehicles,
            network,
            config,
            objective,
        }
    }

    fn solve(&self) -> Solution {
        let mut schedule = Schedule::empty(
            self.vehicles.clone(),
            self.network.clone(),
            self.config.clone(),
        );

        let vehicle_type = self.vehicles.iter().next().unwrap();

        for service_trip in self.network.service_nodes() {
            while !schedule.is_fully_covered(service_trip) {
                schedule = schedule
                    .spawn_vehicle_for_path(vehicle_type, vec![service_trip])
                    .unwrap()
                    .0;
            }
        }

        self.objective.evaluate(schedule)
    }
}
