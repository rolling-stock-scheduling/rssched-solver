use crate::schedule::Schedule;
use crate::locations::Locations;
use crate::units::Units;
use crate::network::Network;
use crate::solver::Solver;
use crate::base_types::{UnitId,NodeId,StationSide};

use std::rc::Rc;

pub struct Greedy2 {
    loc: Rc<Locations>,
    units: Rc<Units>,
    nw: Rc<Network>
}

impl Solver for Greedy2 {

    fn initialize(loc: Rc<Locations>, units: Rc<Units>, nw: Rc<Network>) -> Greedy2 {
        Greedy2{loc, units, nw}
    }

    fn solve(&self) -> Schedule {
        let mut schedule = Schedule::initialize(self.loc.clone(), self.units.clone(), self.nw.clone());

        // Sort service and maintanence nodes by start time
        let mut nodes_sorted_by_start: Vec<NodeId> = self.nw.service_nodes().chain(self.nw.maintenance_nodes()).collect();
        nodes_sorted_by_start.sort_by(|n1, n2| self.nw.nodes.get(n1).unwrap().cmp_start_time(&self.nw.nodes.get(n2).unwrap()));

        // Last node in each non-dummy tour excluding end node. Initialize to start nodes.
        let mut last_nodes: Vec<(UnitId, NodeId)> = Vec::new();
        for unit_id in self.units.get_all(){
            last_nodes.push( (unit_id, self.nw.start_node_of(unit_id)) );
        }

        //  For each node find an existing tour that can cover it while minimizing the wasted time
        for node in nodes_sorted_by_start{
            for dummy_id in schedule.clone().covered_by(node).iter() {
                // Sort last_nodes by end time of nodes (i.e. second component in the tuple)
                last_nodes.sort_by(|n1, n2| self.nw.nodes.get(&n1.1).unwrap().cmp_end_time(&self.nw.nodes.get(&n2.1).unwrap()));
                // Find an existing tour that can cover 'node' while minimizing the wasted time
                let candidate = last_nodes.iter().find(|(u,_n)| {
                let conflict_result = schedule.conflict_single_node(node, *u);
                conflict_result.is_ok() && conflict_result.unwrap().is_empty()
                });
                // update tour
                if candidate.is_some() {
                    let (new_unit, _dummy) = candidate.unwrap();
                    schedule = schedule.reassign_all(dummy_id, *new_unit).unwrap();
                }
            }
        }
        schedule
    }
}
