use crate::base_types::{NodeId, UnitId};
use crate::schedule::path::Segment;
use crate::schedule::Schedule;
use super::Swap;

use std::fmt;


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
            Some(new_dummy) =>
                if schedule.is_dummy(self.provider) && !intermediate_schedule.is_dummy(self.provider) {
                    // provider was dummy but got removed -> so no need for fit_reassign
                    Ok(intermediate_schedule)
                } else {
                    Ok(intermediate_schedule.fit_reassign_all(new_dummy, self.provider)?)
                }
        }
    }
}

impl fmt::Display for PathExchange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "swap {} from {} to {}", self.segment, self.provider, self.receiver)
    }
}
