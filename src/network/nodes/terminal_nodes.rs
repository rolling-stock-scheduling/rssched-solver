use crate::locations::Location;
use crate::unit::{Unit,UnitType};
use crate::time::Time;
use std::fmt;

pub(crate) struct StartNode<'a> {
    unit : &'a Unit,
    location: &'a Location,
    time: Time,
}

// methods
impl<'a> StartNode<'a> {
    pub(crate) fn unit(&self) -> &Unit {
        self.unit
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
    pub(super) fn new(unit: &'a Unit, location: &'a Location, time: Time) -> StartNode<'a> {
        StartNode{
            unit,
            location,
            time
        }
    }
}

impl<'a> fmt::Display for StartNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"start of {} at {} ({})", self.unit, self.location, self.time)
    }
}



pub(crate) struct EndNode<'a> {
    unit_type: UnitType,
    location: &'a Location,
    time: Time,
}

// methods
impl<'a> EndNode<'a> {
    pub(crate) fn unit_type(&self) -> UnitType {
        self.unit_type
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
    pub(super) fn new(unit: &'a Unit, location: &'a Location, time: Time) -> EndNode<'a> {
        EndNode{
            unit_type:unit.get_type(),
            location,
            time
        }
    }
}

impl<'a> fmt::Display for EndNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"end for {:?} at {} ({})", self.unit_type, self.location, self.time)
    }
}
