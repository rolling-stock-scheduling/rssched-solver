use crate::schedule::Schedule;
use crate::config::Config;
use crate::units::Units;
use crate::network::Network;
use crate::solver::Solver;
use crate::base_types::{UnitId,NodeId};

use std::sync::Arc;

pub struct Greedy3 {
    config: Arc<Config>,
    units: Arc<Units>,
    nw: Arc<Network>
}

impl Solver for Greedy3 {

    fn initialize(config: Arc<Config>, units: Arc<Units>, nw: Arc<Network>) -> Greedy3 {
        Greedy3{config, units, nw}
    }

    fn solve(&self) -> Schedule {
        let mut schedule = Schedule::initialize(self.config.clone(), self.units.clone(), self.nw.clone());

        // Sort service and maintanence nodes by start time
        let mut nodes_sorted_by_start: Vec<NodeId> = self.nw.service_nodes().chain(self.nw.maintenance_nodes()).collect();
        nodes_sorted_by_start.sort_by(|n1, n2| self.nw.node(*n1).cmp_start_time(self.nw.node(*n2)));

        // Last node in each non-dummy tour excluding end node. Initialize to start nodes.
        let mut last_nodes: Vec<(UnitId, NodeId)> = Vec::new();
        for unit_id in self.units.iter(){
            last_nodes.push( (unit_id, self.nw.start_node_of(unit_id)) );
        }

        //  For each node find an existing tour that can cover it while minimizing the added deadhead trip distance
        for node in nodes_sorted_by_start{
            for dummy_id in schedule.clone().covered_by(node).iter() {
                // Sort last_nodes by deadhead trip distance to 'node' in increasing order
                // Use a tie breaking rule given by candidate's end time or/and unit ID
                last_nodes.sort_by(|n1, n2| {
                    self.nw.dead_head_time_between(n1.1,node).cmp(
                        &self.nw.dead_head_time_between(n2.1,node))
                        // break ties by candidate's end time
                        .then(self.nw.node(n2.1).cmp_end_time(self.nw.node(n1.1)))
                        // and/or break ties by candidate's unit ID
                        .then(self.nw.node(n1.1).id().cmp(&self.nw.node(n2.1).id()))
                });
                // Find an existing tour that can cover 'node' while minimizing the deadhead distance
                let candidate = last_nodes.iter().enumerate().find(|(_,(u,_))| {
                    let conflict_result = schedule.conflict_single_node(node, *u);
                    conflict_result.is_ok() && conflict_result.unwrap().is_empty()
                });
                // update tour
                if let Some((index,&(new_unit, _))) = candidate {
                    schedule = schedule.reassign_all(dummy_id, new_unit).unwrap();
                    last_nodes[index] = (new_unit, node);
                }
            }
        }
        schedule
    }
}
