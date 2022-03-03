use crate::base_types::{NodeId, UnitId};
use crate::schedule::Schedule;
use crate::schedule::path::Segment;
use std::fmt;

use itertools::Itertools;

/// An elementary modification. Defining the "neighborhood" for the local search.
pub(crate) trait Swap: fmt::Display {
    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String>;
    // TODO maybe add something like, get_improvement()
}

/// Computes for a given schedule all Swaps in the neighborhood.
pub(crate) trait SwapFactory {
    fn create_swaps(&self, schedule: &Schedule) -> Vec<Box<dyn Swap>>;
}

/// Computes for a given schedule the best new schedule that has better objective function.
/// Returns None if there is no better schedule in the neighborhood.
pub(crate) trait LocalImprover {
    fn improve(&self, schedule: &Schedule) -> Option<Schedule>;
}




/////////////////////////////////////////////////////////////
////////////////////////// Swaps ////////////////////////////
/////////////////////////////////////////////////////////////

/// Removes the path from the provider's Tour and insert it into the receiver's Tour.
/// All removed nodes that are removed from receiver's Tour (due to conflicts) are tried to insert conflict-free into
/// the provider's Tour.
pub(crate) struct PathExchange {
    segment: Segment,
    provider: UnitId,
    receiver: UnitId,
}

impl PathExchange {
    pub(crate) fn new(segment: Segment, provider: UnitId, receiver: UnitId) -> PathExchange {
        PathExchange{segment, provider, receiver}
    }
}

impl Swap for PathExchange {
    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String> {
        let (intermediate_schedule, new_dummy_opt) = schedule.override_reassign(self.segment, self.provider, self.receiver)?;

        match new_dummy_opt {
            None => Ok(intermediate_schedule),
            Some(new_dummy) => Ok(intermediate_schedule.fit_reassign_all(new_dummy, self.provider)?)
        }
    }
}

impl fmt::Display for PathExchange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "swap {} from {} to {}", self.segment, self.provider, self.receiver)
    }
}



/////////////////////////////////////////////////////////////
//////////////////////// SwapFactory ////////////////////////
/////////////////////////////////////////////////////////////

pub(crate) struct AllExchanges {
}

impl AllExchanges {
    pub(crate) fn new () -> AllExchanges {
        AllExchanges{}
    }
}

impl SwapFactory for AllExchanges {
    fn create_swaps(&self, schedule: &Schedule) -> Vec<Box<dyn Swap>> {
        let mut swaps: Vec<Box<dyn Swap>> = Vec::new();
        for provider in schedule.dummy_iter().chain(schedule.real_units_iter()) {
            let tour = schedule.tour_of(provider);
            let nodes: Vec<NodeId> = tour.nodes_iter().copied().collect();
            for segment in nodes.iter().tuple_combinations().chain(nodes.iter().map(|n| (n, n)))
                .map(|(s, e)| Segment::new(*s, *e))
                .filter(|seg| tour.removable(*seg)) {

                    for receiver in schedule.real_units_iter().chain(schedule.dummy_iter())
                    .filter(|&u| u != provider && schedule.conflict(segment, u).is_ok()) {

                        swaps.push(Box::new(PathExchange::new(segment, provider, receiver)));
                }
            }
        }
        swaps
    }
}



