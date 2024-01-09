use std::{fmt, sync::Arc};

use model::{
    base_types::{PassengerCount, TrainLength, VehicleId, VehicleTypeId},
    vehicle_types::{VehicleType, VehicleTypes},
};

#[derive(Clone)]
pub struct Vehicle {
    id: VehicleId,
    vehicle_type: Arc<VehicleType>,
}

impl Vehicle {
    pub(super) fn new(
        id: VehicleId,
        type_id: VehicleTypeId,
        vehicle_types: Arc<VehicleTypes>,
    ) -> Vehicle {
        Vehicle {
            id,
            vehicle_type: vehicle_types.get(type_id).unwrap().clone(),
        }
    }

    pub fn id(&self) -> VehicleId {
        self.id
    }

    pub fn type_id(&self) -> VehicleTypeId {
        self.vehicle_type.id()
    }

    pub fn seats(&self) -> PassengerCount {
        self.vehicle_type.seats()
    }

    pub fn capacity(&self) -> PassengerCount {
        self.vehicle_type.capacity()
    }

    pub fn length(&self) -> TrainLength {
        self.vehicle_type.length()
    }
}

impl fmt::Display for Vehicle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.id, self.vehicle_type.name())?;
        Ok(())
    }
}
