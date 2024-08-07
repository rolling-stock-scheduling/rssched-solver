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

use std::{fmt, sync::Arc};

use model::{
    base_types::{PassengerCount, VehicleCount, VehicleIdx, VehicleTypeIdx},
    vehicle_types::{VehicleType, VehicleTypes},
};

#[derive(Clone, Debug)]
pub struct Vehicle {
    idx: VehicleIdx,
    vehicle_type: Arc<VehicleType>,
}

impl Vehicle {
    pub(super) fn new(
        idx: VehicleIdx,
        type_idx: VehicleTypeIdx,
        vehicle_types: Arc<VehicleTypes>,
    ) -> Vehicle {
        Vehicle {
            idx,
            vehicle_type: vehicle_types.get(type_idx).unwrap().clone(),
        }
    }

    pub fn idx(&self) -> VehicleIdx {
        self.idx
    }

    pub fn type_idx(&self) -> VehicleTypeIdx {
        self.vehicle_type.idx()
    }

    pub fn seats(&self) -> PassengerCount {
        self.vehicle_type.seats()
    }

    pub fn capacity(&self) -> PassengerCount {
        self.vehicle_type.capacity()
    }

    pub fn maximal_formation_count(&self) -> Option<VehicleCount> {
        self.vehicle_type.maximal_formation_count()
    }
}

impl fmt::Display for Vehicle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", self.idx, self.vehicle_type.id())?;
        Ok(())
    }
}
