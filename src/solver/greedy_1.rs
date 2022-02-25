use crate::schedule::Schedule;
use crate::locations::Locations;
use crate::units::Units;
use crate::network::Network;
use crate::solver::Solver;

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
        for unit_id in self.units.get_all() {
            let mut node = self.nw.start_node_of(unit_id);
            let mut new_node_opt = schedule.uncovered_successors(node).find(|&n| schedule.conflict_node(unit_id,n).is_ok());
            while new_node_opt.is_some() {
                node = new_node_opt.unwrap();
                schedule = schedule.assign_node(unit_id, node).unwrap();
                new_node_opt = schedule.uncovered_successors(node).find(|&n| schedule.conflict_node(unit_id,n).is_ok());
            }
        }
        schedule
    }
}
