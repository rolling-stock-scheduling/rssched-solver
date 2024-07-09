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

use model::base_types::{NodeIdx, VehicleIdx};
use solution::{path::Path, Schedule};

use super::{improve_depot_and_recompute_transitions, Swap};

/// Adds a trip for hitch hiking to a vehicle.
pub struct AddTripForHitchHiking {
    node: NodeIdx,
    vehicle: VehicleIdx,
}

impl AddTripForHitchHiking {
    pub(crate) fn new(node: NodeIdx, vehicle: VehicleIdx) -> AddTripForHitchHiking {
        AddTripForHitchHiking { node, vehicle }
    }
}

impl Swap for AddTripForHitchHiking {
    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String> {
        if let Some(max_formation_count) = schedule
            .get_network()
            .maximal_formation_count_for(self.node)
        {
            if schedule.train_formation_of(self.node).vehicle_count() >= max_formation_count {
                return Err("node is already fully occupied".to_string());
            }
        }
        let (sched, conflict) = schedule.add_path_to_vehicle_tour(
            self.vehicle,
            Path::new_from_single_node(self.node, schedule.get_network()),
        )?;
        match conflict {
            Some(_) => Err("node causes conflict".to_string()),
            None => Ok(improve_depot_and_recompute_transitions(
                sched,
                vec![self.vehicle],
            )),
        }
    }
}

impl fmt::Display for AddTripForHitchHiking {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AddTripForHitchHiking {} to {}", self.node, self.vehicle)
    }
}
