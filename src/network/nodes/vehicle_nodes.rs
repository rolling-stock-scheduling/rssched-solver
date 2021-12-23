use crate::placeholder::{VehicleId, Location, DayTime};

pub(crate) struct StartNode {
    vehicle_id : VehicleId,
    location: Location,
    time: DayTime,
}

impl StartNode{
    pub(super) fn new(vehicle_id: VehicleId, location: Location, time: DayTime) -> StartNode {
        StartNode{
            vehicle_id,
            location,
            time
        }
    }
}




pub(crate) struct EndNode {
    vehicle_id: VehicleId,
    location: Location,
    time: DayTime,
}

impl EndNode{
    pub(super) fn new(vehicle_id: VehicleId, location: Location, time: DayTime) -> EndNode {
        EndNode{
            vehicle_id,
            location,
            time
        }
    }
}
