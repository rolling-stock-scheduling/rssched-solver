use model::base_types::{NodeIdx, VehicleCount, VehicleIdx, VehicleTypeIdx};
use solution::{segment::Segment, Schedule};

use std::fmt;

/// An elementary modification. Defining the "neighborhood" for the local search.
pub trait Swap: fmt::Display + Send + Sync {
    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String>;
}

#[derive(Clone)]
pub struct SpawnVehicleForMaintenance {
    maintenance_slot: NodeIdx,
    vehicle_type: VehicleTypeIdx,
}

impl SpawnVehicleForMaintenance {
    pub(crate) fn new(
        maintenance_slot: NodeIdx,
        vehicle_type: VehicleTypeIdx,
    ) -> SpawnVehicleForMaintenance {
        SpawnVehicleForMaintenance {
            maintenance_slot,
            vehicle_type,
        }
    }
}

impl Swap for SpawnVehicleForMaintenance {
    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String> {
        let occupants = schedule.train_formation_of(self.maintenance_slot).ids();

        if occupants.len() as VehicleCount
            >= schedule
                .get_network()
                .track_count_of_maintenance_slot(self.maintenance_slot)
        {
            // maintenance slot is already fully occupied
            // remove the last occupant and spawn a new vehicle
            let last_occupant = *occupants.last().unwrap();
            Ok(schedule
                .remove_segment(
                    Segment::new(self.maintenance_slot, self.maintenance_slot),
                    last_occupant,
                )?
                .spawn_vehicle_for_path(self.vehicle_type, vec![self.maintenance_slot])?
                .0)
        } else {
            Ok(schedule
                .spawn_vehicle_for_path(self.vehicle_type, vec![self.maintenance_slot])?
                .0)
        }
    }
}

impl fmt::Display for SpawnVehicleForMaintenance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "spawn vehicle for maintenance at {}",
            self.maintenance_slot
        )
    }
}

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

        let mut changed_tours = vec![self.receiver];

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
                let (new_schedule, new_vehicle) = first_schedule
                    .spawn_vehicle_to_replace_dummy_tour(
                        new_dummy,
                        schedule.vehicle_type_of(self.provider).unwrap(),
                    )?;
                changed_tours.push(new_vehicle);
                new_schedule
            }
            (Some(new_dummy), _, true) => {
                // provider still present -> try to fit the full tour of the new dummy into receiver's tour
                let tour = first_schedule.tour_of(new_dummy).unwrap();
                changed_tours.push(self.provider);
                let full_tour_segment = Segment::new(tour.first_node(), tour.last_node());
                first_schedule.fit_reassign(full_tour_segment, new_dummy, self.provider)?
            }
        };

        changed_tours.retain(|&v| schedule.is_vehicle(v));

        // finally improve the depots of receiver (and provider if still present).
        Ok(second_schedule.improve_depots(Some(changed_tours)))
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
