pub mod greedy_1;

use crate::schedule::Schedule;
use crate::locations::Locations;
use crate::units::Units;
use crate::network::Network;

pub(crate) trait Solver<'a> {
    fn initialize(loc: &'a Locations, units: &'a Units, nw: &'a Network<'a>) -> Self;

    fn solve(&self) -> Schedule<'a>;
}
