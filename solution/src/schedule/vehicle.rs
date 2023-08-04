use std::{fmt, sync::Arc};

use sbb_model::{
    base_types::{PassengerCount, TrainLength, VehicleId, VehicleTypeId},
    vehicle_types::VehicleTypes,
};

#[derive(Clone)]
pub(super) struct Vehicle {
    id: VehicleId,
    type_id: VehicleTypeId,
    vehicle_types: Arc<VehicleTypes>,
}

impl Vehicle {
    pub(super) fn new(
        id: VehicleId,
        type_id: VehicleTypeId,
        vehicle_types: Arc<VehicleTypes>,
    ) -> Vehicle {
        Vehicle {
            id,
            type_id,
            vehicle_types,
        }
    }

    pub fn id(&self) -> VehicleId {
        self.id
    }

    pub fn type_id(&self) -> VehicleTypeId {
        self.type_id
    }

    pub fn seats(&self) -> PassengerCount {
        self.vehicle_types.get(self.type_id).unwrap().seats()
    }

    pub fn capacity(&self) -> PassengerCount {
        self.vehicle_types.get(self.type_id).unwrap().capacity()
    }

    pub fn length(&self) -> TrainLength {
        self.vehicle_types.get(self.type_id).unwrap().length()
    }
}

impl fmt::Display for Vehicle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.id, self.type_id)?;
        Ok(())
    }
}
