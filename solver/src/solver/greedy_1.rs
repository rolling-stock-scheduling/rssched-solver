use crate::solver::Solver;
use sbb_model::base_types::{NodeId, VehicleId};
use sbb_model::config::Config;
use sbb_model::network::Network;
use sbb_model::vehicles::Vehicles;
use sbb_solution::Schedule;
use std::sync::Arc;

pub struct Greedy1 {
    config: Arc<Config>,
    vehicles: Arc<Vehicles>,
    nw: Arc<Network>,
}

impl Solver for Greedy1 {
    fn initialize(config: Arc<Config>, vehicles: Arc<Vehicles>, nw: Arc<Network>) -> Greedy1 {
        Greedy1 {
            config,
            vehicles,
            nw,
        }
    }

    fn solve(&self) -> Schedule {
        let mut schedule =
            Schedule::initialize(self.config.clone(), self.vehicles.clone(), self.nw.clone());
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
