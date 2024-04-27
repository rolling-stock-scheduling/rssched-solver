use std::{fmt, sync::Arc};

use model::{
    base_types::{PassengerCount, VehicleCount, VehicleIdx, VehicleTypeIdx},
    vehicle_types::{VehicleType, VehicleTypes},
};

#[derive(Clone, Debug)]
pub struct Vehicle {
    id: VehicleIdx,
    vehicle_type: Arc<VehicleType>,
}

impl Vehicle {
    pub(super) fn new(
        id: VehicleIdx,
        type_id: VehicleTypeIdx,
        vehicle_types: Arc<VehicleTypes>,
    ) -> Vehicle {
        Vehicle {
            id,
            vehicle_type: vehicle_types.get(type_id).unwrap().clone(),
        }
    }

    pub fn idx(&self) -> VehicleIdx {
        self.id
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
        write!(f, "{}({})", self.id, self.vehicle_type.id())?;
        Ok(())
    }
}
