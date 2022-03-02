use crate::base_types::{NodeId, UnitId};
use crate::schedule::Schedule;
use crate::schedule::path::Segment;

/// An elementary modification. Defining the "neighborhood" for the local search.
pub(crate) trait Swap {
    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String>;
    // TODO maybe add something like, get_improvement()
}

/// Computes for a given schedule all Swaps in the neighborhood.
pub(crate) trait LocalModifier {
    fn compute_neighborhood(&self, schedule: &Schedule) -> Vec<Schedule>;
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
    pub(crate) fn new(start: NodeId, end: NodeId, provider: UnitId, receiver: UnitId) -> PathExchange {
        let segment = Segment::new(start, end);
        PathExchange{segment, provider, receiver}
    }
}

impl Swap for PathExchange {
    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String> {
        let (intermediateSchedule, new_dummy_opt) = schedule.override_reassign(self.segment, self.provider, self.receiver)?;

        match new_dummy_opt {
            None => Ok(intermediateSchedule),
            Some(new_dummy) => Ok(intermediateSchedule.fit_reassign_all(new_dummy, self.provider)?)
        }
    }
}






