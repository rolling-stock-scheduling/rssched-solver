// © 2023-2024 ETH Zurich
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// any later version.
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::fmt;

use model::base_types::{NodeIdx, VehicleCount, VehicleIdx};
use solution::{path::Path, segment::Segment, Schedule};

use super::{improve_depot_and_recompute_transitions, Swap};

/// Forces a maintenance slot to a given vehicle and spawns a new vehicle for the conflict path.
/// If the maintenance slot is already fully occupied, the last occupant is removed.
#[derive(Clone)]
pub struct SpawnVehicleForMaintenance {
    maintenance_slot: NodeIdx,
    vehicle: VehicleIdx,
}

impl SpawnVehicleForMaintenance {
    pub(crate) fn new(
        maintenance_slot: NodeIdx,
        vehicle: VehicleIdx,
    ) -> SpawnVehicleForMaintenance {
        SpawnVehicleForMaintenance {
            maintenance_slot,
            vehicle,
        }
    }
}

impl Swap for SpawnVehicleForMaintenance {
    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String> {
        if schedule.tour_of(self.vehicle).unwrap().visits_maintenance() {
            return Err(format!(
                "Vehicle {} already visits maintenance slot",
                self.vehicle
            ));
        }

        let occupants = schedule.train_formation_of(self.maintenance_slot).ids();

        let mut changed_vehicles = vec![];
        let vehicle_type = schedule.vehicle_type_of(self.vehicle).unwrap();

        let schedule1 = if occupants.len() as VehicleCount
            >= schedule
                .get_network()
                .track_count_of_maintenance_slot(self.maintenance_slot)
        {
            // maintenance slot is already fully occupied
            // remove the last occupant and spawn a new vehicle
            let last_occupant = *occupants.last().unwrap();

            let sched = schedule.remove_segment(
                Segment::new(self.maintenance_slot, self.maintenance_slot),
                last_occupant,
            )?;
            if sched.is_vehicle(last_occupant) {
                changed_vehicles.push(last_occupant);
            }
            sched
        } else {
            schedule.clone()
        };

        // add the maintenance slot to the vehicle's tour
        let (schedule2, conflict_path) = schedule1.add_path_to_vehicle_tour(
            self.vehicle,
            Path::new_from_single_node(self.maintenance_slot, schedule.get_network()),
        )?;
        changed_vehicles.push(self.vehicle);

        // spawn a new vehicle for the conflict path
        let schedule3 = if let Some(path) = conflict_path {
            let (sched, new_vehicle) =
                schedule2.spawn_vehicle_for_path(vehicle_type, path.consume())?;
            changed_vehicles.push(new_vehicle);
            sched
        } else {
            schedule2
        };

        Ok(improve_depot_and_recompute_transitions(
            schedule3,
            changed_vehicles,
        ))
    }
}

impl fmt::Display for SpawnVehicleForMaintenance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SpawnVehicleForMaintenance {} forced onto {}",
            self.maintenance_slot, self.vehicle
        )
    }
}
