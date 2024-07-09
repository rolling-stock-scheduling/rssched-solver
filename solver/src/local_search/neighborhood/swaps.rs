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

mod add_trip_for_hitch_hiking;
mod path_exchange;
mod remove_single_node;
mod spawn_vehicle_for_maintenance;
pub use add_trip_for_hitch_hiking::AddTripForHitchHiking;
pub use path_exchange::PathExchange;
pub use remove_single_node::RemoveSingleNode;
pub use spawn_vehicle_for_maintenance::SpawnVehicleForMaintenance;

use std::fmt;

use model::base_types::VehicleIdx;
use solution::Schedule;

/// An elementary modification. Defining the "neighborhood" for the local search.
pub trait Swap: fmt::Display + Send + Sync {
    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String>;
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum SwapInfo {
    SpawnVehicleForMaintenance(VehicleIdx), // last receiver
    PathExchange(VehicleIdx),               // last provider
    AddTripForHitchHiking(VehicleIdx),      // last vehicle
    RemoveSingleNode(VehicleIdx),           // last vehicle
    NoSwap,
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
