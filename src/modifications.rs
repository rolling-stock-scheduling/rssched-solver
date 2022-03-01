use crate::base_types::UnitId;
use crate::schedule::Schedule;
use crate::schedule::path::Path;

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
    path: Path,
    provider: UnitId,
    receiver: UnitId,
}


