use sbb_model::base_types::VehicleId;
use sbb_solution::{path::Segment, Schedule};

use std::fmt;

/// An elementary modification. Defining the "neighborhood" for the local search.
pub(crate) trait Swap: fmt::Display {
    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String>;
}

/// Removes the path from the provider's tour and insert it into the receiver's tour.
/// All removed nodes that are removed from receiver's tour (due to conflicts) are tried to insert conflict-free into
/// the provider's tour.
pub(crate) struct PathExchange {
    segment: Segment,
    provider: VehicleId,
    receiver: VehicleId,
}

impl PathExchange {
    pub(crate) fn new(segment: Segment, provider: VehicleId, receiver: VehicleId) -> PathExchange {
        PathExchange {
            segment,
            provider,
            receiver,
        }
    }
}

impl Swap for PathExchange {
    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String> {
        let (first_intermediate_schedule, new_dummy_opt) =
            schedule.override_reassign(self.segment, self.provider, self.receiver)?;

        match new_dummy_opt {
            None => Ok(first_intermediate_schedule),
            Some(new_dummy) => {
                if schedule.is_dummy(self.provider)
                    && !first_intermediate_schedule.is_dummy(self.provider)
                {
                    // provider was dummy but got removed -> so no need for fit_reassign
                    Ok(first_intermediate_schedule)
                } else {
                    // try to fit the full tour of the new dummy into provider's tour (if provider
                    // still exists)
                    let second_intermediate_schedule =
                        match first_intermediate_schedule.get_vehicle(self.provider) {
                            Ok(_) => {
                                let tour = first_intermediate_schedule.tour_of(new_dummy).unwrap();
                                let full_tour_segment =
                                    Segment::new(tour.first_node(), tour.last_node());
                                first_intermediate_schedule.fit_reassign(
                                    full_tour_segment,
                                    new_dummy,
                                    self.provider,
                                )?
                            }
                            Err(_) => first_intermediate_schedule, // provider was removed -> no need for fit_reassign
                        };

                    if second_intermediate_schedule.is_dummy(new_dummy)
                        && !second_intermediate_schedule.is_dummy(self.provider)
                    {
                        // new_dummy could not be fit fully into provider's tour -> spawn a new vehicle (of provider's type) for left over nodes
                        second_intermediate_schedule.spawn_vehicle_to_replace_dummy_tour(
                            new_dummy,
                            schedule.get_vehicle(self.provider).unwrap().type_id(),
                        )
                    } else {
                        Ok(second_intermediate_schedule)
                    }
                }
            }
        }
    }
}

impl fmt::Display for PathExchange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "swap {} from {} to {}",
            self.segment, self.provider, self.receiver
        )
    }
}
