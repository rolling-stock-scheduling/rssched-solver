use std::collections::HashMap;

use crate::base_types::{PassengerCount, TrainLength, VehicleTypeId};

pub struct VehicleTypes {
    vehicle_types: HashMap<VehicleTypeId, VehicleType>,
}

impl VehicleTypes {
    pub fn new(vehicle_types: Vec<VehicleType>) -> VehicleTypes {
        VehicleTypes {
            vehicle_types: vehicle_types.into_iter().map(|vt| (vt.id, vt)).collect(),
        }
    }

    pub fn get(&self, id: &VehicleTypeId) -> Option<&VehicleType> {
        self.vehicle_types.get(id)
    }
}

pub struct VehicleType {
    id: VehicleTypeId,
    name: String,
    number_of_seats: PassengerCount,
    capacity_of_passengers: PassengerCount,
    vehicle_length_in_meters: TrainLength,
}

impl VehicleType {
    pub fn new(
        id: VehicleTypeId,
        name: String,
        number_of_seats: PassengerCount,
        capacity_of_passengers: PassengerCount,
        vehicle_length_in_meters: TrainLength,
    ) -> VehicleType {
        VehicleType {
            id,
            name,
            number_of_seats,
            capacity_of_passengers,
            vehicle_length_in_meters,
        }
    }
}
