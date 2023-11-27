use crate::solver::Solver;
use objective_framework::{EvaluatedSolution, Objective};
use sbb_model::config::Config;
use sbb_model::network::Network;
use sbb_model::vehicle_types::VehicleTypes;
use sbb_solution::Schedule;
use std::sync::Arc;

pub struct Greedy {
    vehicles: Arc<VehicleTypes>,
    network: Arc<Network>,
    config: Arc<Config>,
    objective: Arc<Objective<Schedule>>,
}

impl Solver for Greedy {
    fn initialize(
        vehicles: Arc<VehicleTypes>,
        network: Arc<Network>,
        config: Arc<Config>,
        objective: Arc<Objective<Schedule>>,
    ) -> Greedy {
        Greedy {
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

        while let Some(service_trip) = self
            .network
            .service_nodes()
            .filter(|s| !schedule.is_fully_covered(*s))
            .next()
        {
            // TODO find best vehicle to cover it, or otherwise spawn new vehicle.
        }

        self.objective.evaluate(schedule)
    }
}
