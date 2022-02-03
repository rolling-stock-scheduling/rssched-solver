use std::fmt;
use crate::distance::Distance;
use crate::time::Duration;
use crate::placeholder::VehicleId;

pub(crate) struct Vehicle {
    id: VehicleId,
    vehicle_type: VehicleType,
    initial_dist_counter: Distance, // distance since last maintenance (at start_node)
    initial_tt_counter: Duration // travel time since last maintenance (at start_node)
}

impl Vehicle {
    pub(crate) fn new(id: VehicleId, vehicle_type: VehicleType, initial_dist_counter: Distance, initial_tt_counter: Duration) -> Vehicle {
        Vehicle{
            id,
            vehicle_type,
            initial_dist_counter,
            initial_tt_counter
        }
    }
}



impl fmt::Display for Vehicle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vehicle {} ({:?}; {}; {})", self.id, self.vehicle_type, self.initial_dist_counter, self.initial_tt_counter)
    }
}

#[derive(Debug)]
pub(crate) enum VehicleType {
    Giruno,
    FVDosto,
    Astoro
}

