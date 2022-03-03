use crate::base_types::{NodeId, UnitId};
use crate::schedule::Schedule;
use crate::schedule::path::Segment;

/// An elementary modification. Defining the "neighborhood" for the local search.
pub(crate) trait Swap {
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
    pub(crate) fn new(start: NodeId, end: NodeId, provider: UnitId, receiver: UnitId) -> PathExchange {
        let segment = Segment::new(start, end);
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



/////////////////////////////////////////////////////////////
//////////////////////// SwapFactory ////////////////////////
/////////////////////////////////////////////////////////////

pub(crate) struct AllExchanges {

}

// impl SwapFactory for AllExchanges {
    // fn create_swaps(&self, schedule: &Schedule) -> Vec<Box<dyn Swap>> {
        // let mut swaps: Vec<Box<dyn Swap>> = Vec::new();
        // for dummy in schedule.all_dummy_units() {
            // let tour = schedule.tour_of(unit)

        // }

        // swaps
    // }
// }



