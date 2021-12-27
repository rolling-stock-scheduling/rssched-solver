use std::fmt;
use crate::distance::Distance;
use crate::placeholder::VehicleId;

pub(crate) struct Vehicle {
    id: VehicleId,
    vehicle_type: VehicleType,
    initial_dist_counter: Distance
}

impl Vehicle {
    pub(crate) fn new(id: VehicleId, vehicle_type: VehicleType, initial_dist_counter: Distance) -> Vehicle {
        Vehicle{
            id,
            vehicle_type,
            initial_dist_counter
        }
    }
}



impl fmt::Display for Vehicle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vehicle {} ({:?}; {})", self.id, self.vehicle_type, self.initial_dist_counter)
    }
}

#[derive(Debug)]
pub(crate) enum VehicleType {
    Giruno,
    FVDosto,
    Astoro
}

