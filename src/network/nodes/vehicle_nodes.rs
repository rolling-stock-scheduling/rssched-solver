use crate::locations::Location;
use crate::vehicle::{Vehicle,VehicleType};
use crate::time::Time;
use std::fmt;

pub(crate) struct StartNode<'a> {
    vehicle : &'a Vehicle,
    location: &'a Location,
    time: Time,
}

// methods
impl<'a> StartNode<'a> {
    pub(crate) fn vehicle(&self) -> &Vehicle {
        self.vehicle
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
    pub(super) fn new(vehicle: &'a Vehicle, location: &'a Location, time: Time) -> StartNode<'a> {
        StartNode{
            vehicle,
            location,
            time
        }
    }
}

impl<'a> fmt::Display for StartNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"start of {} at {} ({})", self.vehicle, self.location, self.time)
    }
}



pub(crate) struct EndNode<'a> {
    vehicle_type: VehicleType,
    location: &'a Location,
    time: Time,
}

// methods
impl<'a> EndNode<'a> {
    pub(crate) fn vehicle_type(&self) -> VehicleType {
        self.vehicle_type
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
    pub(super) fn new(vehicle: &'a Vehicle, location: &'a Location, time: Time) -> EndNode<'a> {
        EndNode{
            vehicle_type:vehicle.get_type(),
            location,
            time
        }
    }
}

impl<'a> fmt::Display for EndNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"end for {:?} at {} ({})", self.vehicle_type, self.location, self.time)
    }
}
