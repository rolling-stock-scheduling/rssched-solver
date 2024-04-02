use crate::Solver;
use model::config::Config;
use model::network::Network;
use model::vehicle_types::VehicleTypes;
use objective_framework::{EvaluatedSolution, Objective};
use solution::Schedule;
use std::sync::Arc;

pub struct OneNodePerTour {
    vehicles: Arc<VehicleTypes>,
    network: Arc<Network>,
    config: Arc<Config>,
    objective: Arc<Objective<Schedule>>,
}

impl Solver<Schedule> for OneNodePerTour {
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

    fn solve(&self) -> EvaluatedSolution<Schedule> {
        let mut schedule = Schedule::empty(
            self.vehicles.clone(),
            self.network.clone(),
            self.config.clone(),
        );

        for service_trip in self.network.all_service_nodes() {
            while !schedule.is_fully_covered(service_trip) {
                let vehicle_type = self.network.vehicle_type_for(service_trip);

                schedule = schedule
                    .spawn_vehicle_for_path(vehicle_type, vec![service_trip])
                    .unwrap()
                    .0;
            }
        }

        self.objective.evaluate(schedule)
    }
}
