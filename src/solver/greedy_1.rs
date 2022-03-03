use crate::schedule::Schedule;
use crate::locations::Locations;
use crate::units::Units;
use crate::network::Network;
use crate::solver::Solver;
use crate::base_types::{UnitId, NodeId};

use std::rc::Rc;

pub struct Greedy1 {
    loc: Rc<Locations>,
    units: Rc<Units>,
    nw: Rc<Network>
}

impl Solver for Greedy1 {

    fn initialize(loc: Rc<Locations>, units: Rc<Units>, nw: Rc<Network>) -> Greedy1 {
        Greedy1{loc, units, nw}
    }

    fn solve(&self) -> Schedule {
        let mut schedule = Schedule::initialize(self.loc.clone(), self.units.clone(), self.nw.clone());
        for unit in self.units.get_all() {
            let mut node = self.nw.start_node_of(unit);
            let mut new_node_opt = get_fitting_node(&schedule, node, unit);

            while new_node_opt.is_some() {
                let (new_node, dummy) = new_node_opt.unwrap();
                node = new_node;
                schedule = schedule.reassign_all(dummy, unit).unwrap();
                new_node_opt = get_fitting_node(&schedule, node, unit);
            }
        }
        schedule
    }
}

fn get_fitting_node(schedule: &Schedule, node: NodeId, unit_id: UnitId) -> Option<(NodeId,UnitId)> {
    schedule.uncovered_successors(node)
        .find(|(n,_)| schedule.conflict_single_node(*n, unit_id)
              .map(|c| c.is_empty()).unwrap_or(false))

}
