use crate::placeholder::{VehicleId, Location};
use crate::time::Time;
use std::fmt;

pub(crate) struct StartNode {
    vehicle_id : VehicleId,
    location: Location,
    time: Time,
}

impl StartNode{
    pub(super) fn new(vehicle_id: VehicleId, location: Location, time: Time) -> StartNode {
        StartNode{
            vehicle_id,
            location,
            time
        }
    }
}

impl fmt::Display for StartNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Start of {} at {} ({})", self.vehicle_id, self.location, self.time)
    }
}



pub(crate) struct EndNode {
    vehicle_id: VehicleId,
    location: Location,
    time: Time,
}

impl EndNode{
    pub(super) fn new(vehicle_id: VehicleId, location: Location, time: Time) -> EndNode {
        EndNode{
            vehicle_id,
            location,
            time
        }
    }
}

impl fmt::Display for EndNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"End of {} at {} ({})", self.vehicle_id, self.location, self.time)
    }
}
