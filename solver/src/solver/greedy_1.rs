use crate::solver::Solver;
use objective_framework::EvaluatedSolution;
use sbb_model::base_types::{NodeId, VehicleId};
use sbb_model::config::Config;
use sbb_model::network::Network;
use sbb_model::vehicle_types::VehicleTypes;
use sbb_solution::Schedule;
use std::sync::Arc;

pub struct Greedy1 {
    vehicles: Arc<VehicleTypes>,
    nw: Arc<Network>,
    config: Arc<Config>,
}

impl Solver for Greedy1 {
    fn initialize(vehicles: Arc<VehicleTypes>, nw: Arc<Network>, config: Arc<Config>) -> Greedy1 {
        Greedy1 {
            vehicles,
            nw,
            config,
        }
    }

    fn solve(&self) -> EvaluatedSolution<Schedule> {
        let mut schedule =
            Schedule::empty(self.vehicles.clone(), self.nw.clone(), self.config.clone());
        for vehicle in self.vehicles.iter() {
            let mut node = self.nw.start_node_of(vehicle);
            let mut new_node_opt = get_fitting_node(&schedule, node, vehicle);

            while new_node_opt.is_some() {
                let (new_node, dummy) = new_node_opt.unwrap();
                node = new_node;
                schedule = schedule.reassign_all(dummy, vehicle).unwrap();
                new_node_opt = get_fitting_node(&schedule, node, vehicle);
            }
        }
        schedule
    }

    fn foo(&self) -> Schedule {
        println!("hi");
        self.solve()
    }
}

fn get_fitting_node(
    schedule: &Schedule,
    node: NodeId,
    vehicle_id: VehicleId,
) -> Option<(NodeId, VehicleId)> {
    schedule.uncovered_successors(node).find(|(n, _)| {
        schedule
            .conflict_single_node(*n, vehicle_id)
            .map(|c| c.is_empty())
            .unwrap_or(false)
    })
}
