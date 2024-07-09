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

use std::collections::HashMap;

use crate::base_types::{DepotIdx, Location, VehicleCount, VehicleTypeIdx};

pub struct Depot {
    idx: DepotIdx,
    id: String,
    location: Location,
    total_capacity: VehicleCount,
    allowed_types: HashMap<VehicleTypeIdx, Option<VehicleCount>>, // number of vehicles that can be
                                                                 // spawned. None means no limit.
}

// methods
impl Depot {
    pub fn idx(&self) -> DepotIdx {
        self.idx
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn location(&self) -> Location {
        self.location
    }

    pub fn total_capacity(&self) -> VehicleCount {
        self.total_capacity
    }

    /// takes the minimum of vehicle specific capacity (None means no limit) and depot capacity
    pub fn capacity_for(&self, vehicle_type_idx: VehicleTypeIdx) -> VehicleCount {
        match self.allowed_types.get(&vehicle_type_idx) {
            Some(Some(capacity)) => VehicleCount::min(*capacity, self.total_capacity),
            Some(None) => self.total_capacity, // no vehicle specific limit
            None => 0,                         // vehicle type not allowed
        }
    }
}

// static
impl Depot {
    pub fn new(
        depot_idx: DepotIdx,
        name: String,
        location: Location,
        total_capacity: VehicleCount,
        allowed_types: HashMap<VehicleTypeIdx, Option<VehicleCount>>,
    ) -> Self {
        Self {
            idx: depot_idx,
            id: name,
            location,
            total_capacity,
            allowed_types,
        }
    }
}
