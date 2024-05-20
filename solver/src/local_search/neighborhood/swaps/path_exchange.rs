use std::fmt;

use model::base_types::VehicleIdx;
use solution::{segment::Segment, Schedule};

use super::{improve_depot_and_recompute_transitions, Swap};

/// Removes the path from the provider's tour and insert it into the receiver's tour.
/// All removed nodes that are removed from receiver's tour (due to conflicts) are tried to insert conflict-free into
/// the provider's tour.
#[derive(Clone)]
pub struct PathExchange {
    segment: Segment,
    provider: VehicleIdx,
    receiver: VehicleIdx,
}

impl PathExchange {
    pub(crate) fn new(
        segment: Segment,
        provider: VehicleIdx,
        receiver: VehicleIdx,
    ) -> PathExchange {
        PathExchange {
            segment,
            provider,
            receiver,
        }
    }
}

impl Swap for PathExchange {
    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String> {
        let (first_schedule, new_dummy_opt) =
            schedule.override_reassign(self.segment, self.provider, self.receiver)?;

        let mut vehicle_of_changed_tours = vec![];
        if schedule.is_vehicle(self.receiver) {
            vehicle_of_changed_tours.push(self.receiver);
        }

        let second_schedule = match (
            new_dummy_opt,
            schedule.is_vehicle(self.provider),
            first_schedule.is_vehicle_or_dummy(self.provider),
        ) {
            (None, _, _) => {
                // no nodes were removed from receiver's tour -> no need for fit_reassign
                first_schedule
            }
            (Some(_), false, false) => {
                // provider (dummy) got removed -> no need for fit_reassign, no new vehicle
                first_schedule
            }
            (Some(new_dummy), true, false) => {
                // provider (real) got removed -> no need for fit_reassign, but spawn new vehicle
                let vehicle_type_of_provider = schedule.vehicle_type_of(self.provider).unwrap();
                let (new_schedule, new_vehicle) = first_schedule
                    .spawn_vehicle_to_replace_dummy_tour(new_dummy, vehicle_type_of_provider)?;
                vehicle_of_changed_tours.push(new_vehicle);
                new_schedule
            }
            (Some(new_dummy), _, true) => {
                // provider still present -> try to fit the full tour of the new dummy into receiver's tour
                let tour = first_schedule.tour_of(new_dummy).unwrap();
                vehicle_of_changed_tours.push(self.provider);
                let full_tour_segment = Segment::new(tour.first_node(), tour.last_node());
                first_schedule.fit_reassign(full_tour_segment, new_dummy, self.provider)?
            }
        };

        vehicle_of_changed_tours.retain(|&v| second_schedule.is_vehicle(v));
        vehicle_of_changed_tours.dedup();

        // finally improve the depots of receiver (and provider if still present).
        Ok(improve_depot_and_recompute_transitions(
            second_schedule,
            vehicle_of_changed_tours,
        ))
    }
}

impl fmt::Display for PathExchange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PathExchange {} from {} to {}",
            self.segment, self.provider, self.receiver
        )
    }
}
