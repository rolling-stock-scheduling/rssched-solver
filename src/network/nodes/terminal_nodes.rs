use crate::locations::Location;
use crate::unit::{Unit,UnitType};
use crate::time::Time;
use std::fmt;

pub(crate) struct StartNode<'a> {
    unit : &'a Unit,
    location: Location,
    time: Time,
}

// methods
impl<'a> StartNode<'a> {
    pub(crate) fn unit(&self) -> &Unit {
        self.unit
    }

    pub(crate) fn location(&self) -> Location {
        self.location
    }

    pub(crate) fn time(&self) -> Time {
        self.time
    }
}

// static functions
impl<'a> StartNode<'a> {
    pub(super) fn new(unit: &'a Unit, location: Location, time: Time) -> StartNode {
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



pub(crate) struct EndNode {
    unit_type: UnitType,
    location: Location,
    time: Time,
    //covered_by: UnitIdx
}

// methods
impl EndNode {
    pub(crate) fn unit_type(&self) -> UnitType {
        self.unit_type
    }

    pub(crate) fn location(&self) -> Location {
        self.location
    }

    pub(crate) fn time(&self) -> Time {
        self.time
    }
}

// static functions
impl EndNode {
    pub(super) fn new(unit_type: UnitType , location: Location, time: Time) -> EndNode {
        EndNode{
            unit_type,
            location,
            time
        }
    }
}

impl fmt::Display for EndNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"end for {:?} at {} ({})", self.unit_type, self.location, self.time)
    }
}
