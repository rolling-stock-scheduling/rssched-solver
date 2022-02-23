use crate::schedule::Schedule;
use crate::locations::Locations;
use crate::units::Units;
use crate::network::Network;
use crate::solver::Solver;

pub struct Greedy1<'a> {
    loc: &'a Locations,
    units: &'a Units,
    nw: &'a Network<'a>
}

impl<'a> Solver<'a> for Greedy1<'a> {

    fn initialize(loc: &'a Locations, units: &'a Units, nw: &'a Network<'a>) -> Greedy1<'a> {
        Greedy1{loc, units, nw}
    }

    fn solve(&self) -> Schedule<'a> {
        let mut schedule = Schedule::initialize(self.loc, self.units, self.nw);
        for unit_id in self.units.get_all() {
            let mut node = self.nw.start_node_of(unit_id);
            let mut new_node_opt = schedule.uncovered_successors(node).find(|&n| schedule.assign_test(unit_id,vec!(n)).is_ok());
            while new_node_opt.is_some() {
                node = new_node_opt.unwrap();
                schedule.assign(unit_id, vec!(node)).unwrap();
                new_node_opt = schedule.uncovered_successors(node).find(|&n| schedule.assign_test(unit_id,vec!(n)).is_ok());
            }
        }
        schedule
    }
}
