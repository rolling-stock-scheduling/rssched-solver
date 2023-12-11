use crate::solver::Solver;
use crate::Solution;
use objective_framework::Objective;
use sbb_model::config::Config;
use sbb_model::network::Network;
use sbb_model::vehicle_types::VehicleTypes;
use sbb_solution::Schedule;
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

        let vehicle_type = self.vehicles.ids_iter().next().unwrap();

        while let Some(service_trip) = self
            .network
            .service_nodes()
            .filter(|s| !schedule.is_fully_covered(*s))
            .next()
        {
            schedule = schedule
                .spawn_vehicle_for_path(vehicle_type, vec![service_trip])
                .unwrap();
        }

        self.objective.evaluate(schedule)
    }
}
