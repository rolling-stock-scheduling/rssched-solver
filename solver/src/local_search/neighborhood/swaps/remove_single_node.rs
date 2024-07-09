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
use solution::{segment::Segment, Schedule};

use super::Swap;

pub struct RemoveSingleNode {
    node: NodeIdx,
    vehicle: VehicleIdx,
}

impl RemoveSingleNode {
    pub(crate) fn new(node: NodeIdx, vehicle: VehicleIdx) -> RemoveSingleNode {
        RemoveSingleNode { node, vehicle }
    }
}

impl Swap for RemoveSingleNode {
    fn apply(&self, schedule: &Schedule) -> Result<Schedule, String> {
        schedule.remove_segment(Segment::new(self.node, self.node), self.vehicle)
        /* let first_schedule =
            schedule.remove_segment(Segment::new(self.node, self.node), self.vehicle)?;
        if schedule.get_network().node(self.node).is_maintenance() {
            // if a maintenance node is removed, we need to recompute transitions for all vehicles
            // of that type
            let vehicle_types = vec![schedule.vehicle_type_of(self.vehicle).unwrap()];
            Ok(schedule.recompute_transitions_for(Some(vehicle_types)))
        } else {
            Ok(first_schedule)
        } */
    }
}

impl fmt::Display for RemoveSingleNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RemoveSingleNode {} from {}", self.node, self.vehicle)
    }
}
