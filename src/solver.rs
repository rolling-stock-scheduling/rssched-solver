pub mod greedy_1;

use crate::schedule::Schedule;
use crate::locations::Locations;
use crate::units::Units;
use crate::network::Network;
use std::rc::Rc;

pub(crate) trait Solver {
    fn initialize(loc: Rc<Locations>, units: Rc<Units>, nw: Rc<Network>) -> Self;

    fn solve(&self) -> Schedule;
}
