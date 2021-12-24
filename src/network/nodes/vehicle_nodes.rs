use crate::placeholder::VehicleId;
use crate::location::Location;
use crate::time::Time;
use std::fmt;

pub(crate) struct StartNode<'a> {
    vehicle_id : VehicleId,
    location: &'a Location,
    time: Time,
}

// methods
impl<'a> StartNode<'a> {
    pub(crate) fn vehicle_id(&self) -> VehicleId {
        self.vehicle_id
    }

    pub(crate) fn location(&self) -> &Location {
        self.location
    }

    pub(crate) fn time(&self) -> Time {
        self.time
    }
}

// static functions
impl<'a> StartNode<'a> {
    pub(super) fn new(vehicle_id: VehicleId, location: &'a Location, time: Time) -> StartNode {
        StartNode{
            vehicle_id,
            location,
            time
        }
    }
}

impl<'a> fmt::Display for StartNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Start of {} at {} ({})", self.vehicle_id, self.location, self.time)
    }
}



pub(crate) struct EndNode<'a> {
    vehicle_id: VehicleId,
    location: &'a Location,
    time: Time,
}

// methods
impl<'a> EndNode<'a> {
    pub(crate) fn vehicle_id(&self) -> VehicleId {
        self.vehicle_id
    }

    pub(crate) fn location(&self) -> &Location {
        self.location
    }

    pub(crate) fn time(&self) -> Time {
        self.time
    }
}

// static functions
impl<'a> EndNode<'a> {
    pub(super) fn new(vehicle_id: VehicleId, location: &'a Location, time: Time) -> EndNode {
        EndNode{
            vehicle_id,
            location,
            time
        }
    }
}

impl<'a> fmt::Display for EndNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"End of {} at {} ({})", self.vehicle_id, self.location, self.time)
    }
}
