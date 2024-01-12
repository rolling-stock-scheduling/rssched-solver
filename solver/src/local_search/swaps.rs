use model::base_types::VehicleId;
use solution::{segment::Segment, Schedule};

use std::fmt;

/// An elementary modification. Defining the "neighborhood" for the local search.
pub trait Swap: fmt::Display + Send + Sync {
    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String>;

    fn provider(&self) -> VehicleId;
}

/// Removes the path from the provider's tour and insert it into the receiver's tour.
/// All removed nodes that are removed from receiver's tour (due to conflicts) are tried to insert conflict-free into
/// the provider's tour.
pub struct PathExchange {
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
    fn provider(&self) -> VehicleId {
        self.provider
    }

    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String> {
        let (first_intermediate_schedule, new_dummy_opt) =
            schedule.override_reassign(self.segment, self.provider, self.receiver)?;

        let mut changed_tours = vec![self.receiver];

        let last_intermediate_schedule: Schedule = match new_dummy_opt {
            None => first_intermediate_schedule,
            Some(new_dummy) => {
                if schedule.is_dummy(self.provider)
                    && !first_intermediate_schedule.is_dummy(self.provider)
                {
                    // provider was dummy but got removed -> so no need for fit_reassign
                    first_intermediate_schedule
                } else {
                    // try to fit the full tour of the new dummy into provider's tour (if provider
                    // still exists)
                    let second_intermediate_schedule =
                        match first_intermediate_schedule.get_vehicle(self.provider) {
                            Ok(_) => {
                                let tour = first_intermediate_schedule.tour_of(new_dummy).unwrap();
                                changed_tours.push(self.provider);
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
                        // new_dummy could not be fit fully into provider's tour -> try spawning a new vehicle (of provider's type) for left over nodes
                        match second_intermediate_schedule.spawn_vehicle_to_replace_dummy_tour(
                            new_dummy,
                            schedule.vehicle_type_of(self.provider),
                        ) {
                            Ok(final_schedule) => final_schedule,
                            Err(_) => second_intermediate_schedule, // could not spawn new
                                                                    // vehicle -> keep tour as
                                                                    // dummy tour
                        }
                    } else {
                        second_intermediate_schedule
                    }
                }
            }
        };

        // finally improve the depots of receiver (and provider if still present).
        Ok(last_intermediate_schedule.improve_depots(Some(changed_tours)))
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
