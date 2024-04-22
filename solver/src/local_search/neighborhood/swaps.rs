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

        let mut changed_vehicles = vec![];

        let first_schedule = if occupants.len() as VehicleCount
            >= schedule
                .get_network()
                .track_count_of_maintenance_slot(self.maintenance_slot)
        {
            // maintenance slot is already fully occupied
            // remove the last occupant and spawn a new vehicle
            let last_occupant = *occupants.last().unwrap();

            let (sched, vehicle) = schedule
                .remove_segment(
                    Segment::new(self.maintenance_slot, self.maintenance_slot),
                    last_occupant,
                )?
                .spawn_vehicle_for_path(self.vehicle_type, vec![self.maintenance_slot])?;

            if sched.is_vehicle(last_occupant) {
                changed_vehicles.push(last_occupant);
            }
            changed_vehicles.push(vehicle);
            sched
        } else {
            let (sched, vehicle) =
                schedule.spawn_vehicle_for_path(self.vehicle_type, vec![self.maintenance_slot])?;
            changed_vehicles.push(vehicle);
            sched
        };

        Ok(improve_depot_and_recompute_transitions(
            first_schedule,
            changed_vehicles,
        ))
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
            "swap {} from {} to {}",
            self.segment, self.provider, self.receiver
        )
    }
}

// assumes that all vehicles are real vehicles in the given schedule
fn improve_depot_and_recompute_transitions(
    schedule: Schedule,
    changed_vehicles: Vec<VehicleIdx>,
) -> Schedule {
    let changed_vehicle_types = changed_vehicles
        .iter()
        .map(|&v| schedule.vehicle_type_of(v).unwrap())
        .collect::<Vec<_>>();

    let schedule_with_improved_depots = schedule.improve_depots(Some(changed_vehicles));

    let mut changed_vehicle_types_with_positive_violation = changed_vehicle_types
        .iter()
        .filter_map(|&vt| {
            if schedule_with_improved_depots
                .next_day_transition_of(vt)
                .maintenance_violation()
                > 0
            {
                Some(vt)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // make vehicle types unique
    changed_vehicle_types_with_positive_violation.sort();
    changed_vehicle_types_with_positive_violation.dedup();

    schedule_with_improved_depots
        .recompute_transitions_for(Some(changed_vehicle_types_with_positive_violation))
}
