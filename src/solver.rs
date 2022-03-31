pub mod greedy_1;
pub mod greedy_2;
pub mod greedy_3;
pub mod local_search;

use crate::schedule::Schedule;
use crate::config::Config;
use crate::locations::Locations;
use crate::units::Units;
use crate::network::Network;
use std::sync::Arc;

pub(crate) trait Solver {
    fn initialize(config: Arc<Config>, units: Arc<Units>, nw: Arc<Network>) -> Self;

    fn solve(&self) -> Schedule;
}
