use model::network::Network;
use objective_framework::{EvaluatedSolution, Objective};
use solution::Schedule;
use std::sync::Arc;

pub struct OneNodePerTour {
    network: Arc<Network>,
    objective: Arc<Objective<Schedule>>,
}
impl OneNodePerTour {
    pub fn initialize(network: Arc<Network>, objective: Arc<Objective<Schedule>>) -> Self {
        Self { network, objective }
    }

    pub fn solve(&self) -> EvaluatedSolution<Schedule> {
        let mut schedule = Schedule::empty(self.network.clone());

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
