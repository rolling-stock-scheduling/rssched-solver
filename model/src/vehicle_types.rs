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

    pub fn get(&self, id: VehicleTypeId) -> Option<&VehicleType> {
        self.vehicle_types.get(&id)
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct VehicleType {
    id: VehicleTypeId,
    name: String,
    seats: PassengerCount,
    capacity: PassengerCount,
    length: TrainLength,
}

impl VehicleType {
    pub fn new(
        id: VehicleTypeId,
        name: String,
        number_of_seats: PassengerCount,
        capacity_of_passengers: PassengerCount,
        vehicle_length: TrainLength,
    ) -> VehicleType {
        VehicleType {
            id,
            name,
            seats: number_of_seats,
            capacity: capacity_of_passengers,
            length: vehicle_length,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn seats(&self) -> PassengerCount {
        self.seats
    }

    pub fn capacity(&self) -> PassengerCount {
        self.capacity
    }

    pub fn length(&self) -> TrainLength {
        self.length
    }
}
