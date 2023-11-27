use crate::solver::Solver;
use objective_framework::{EvaluatedSolution, Objective};
use sbb_model::base_types::{NodeId, VehicleId};
use sbb_model::config::Config;
use sbb_model::network::Network;
use sbb_model::vehicle_types::VehicleTypes;
use sbb_solution::Schedule;
use std::sync::Arc;

pub struct Greedy {
    vehicles: Arc<VehicleTypes>,
    nw: Arc<Network>,
    config: Arc<Config>,
    objective: Arc<Objective<Schedule>>,
}

impl Solver for Greedy {
    fn initialize(
        vehicles: Arc<VehicleTypes>,
        nw: Arc<Network>,
        config: Arc<Config>,
        objective: Arc<Objective<Schedule>>,
    ) -> Greedy {
        Greedy {
            vehicles,
            nw,
            config,
            objective,
        }
    }

    fn solve(&self) -> EvaluatedSolution<Schedule> {
        let mut schedule =
            Schedule::empty(self.vehicles.clone(), self.nw.clone(), self.config.clone());
        self.objective.evaluate(schedule)
    }
}
