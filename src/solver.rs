pub(crate) mod greedy_1;
pub(crate) mod greedy_2;
pub(crate) mod greedy_3;
pub(crate) mod local_search;

use crate::config::Config;
use crate::network::Network;
use crate::schedule::Schedule;
use crate::units::Units;
use std::sync::Arc;

pub(crate) trait Solver {
    fn initialize(config: Arc<Config>, units: Arc<Units>, nw: Arc<Network>) -> Self;

    fn solve(&self) -> Schedule;

    fn foo(&self) -> Schedule {
        self.solve()
    }
}
