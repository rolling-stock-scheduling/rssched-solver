use crate::locations::Location;
use crate::units::UnitType;
use crate::time::Time;
use crate::base_types::UnitId;
use std::fmt;

pub(crate) struct StartNode {
    unit_id : UnitId,
    location: Location,
    time: Time,
}

// methods
impl StartNode {
    pub(crate) fn unit_id(&self) -> UnitId {
        self.unit_id
    }

    pub(crate) fn location(&self) -> Location {
        self.location
    }

    pub(crate) fn time(&self) -> Time {
        self.time
    }
}

// static functions
impl StartNode {
    pub(super) fn new(unit_id: UnitId, location: Location, time: Time) -> StartNode {
        StartNode{
            unit_id,
            location,
            time
        }
    }
}

impl fmt::Display for StartNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"start of {} at {} ({})", self.unit_id, self.location, self.time)
    }
}



pub(crate) struct EndNode {
    unit_type: UnitType,
    location: Location,
    time: Time,
    covered_by: Option<UnitId>
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

    pub(crate) fn covered_by(&self) -> Option<UnitId> {
        self.covered_by
    }

    pub(crate) fn cover(&mut self, unit_id: UnitId) {
        self.covered_by = Some(unit_id)
    }

    pub(crate) fn remove_cover(&mut self) {
        self.covered_by = None
    }
}

// static functions
impl EndNode {
    pub(super) fn new(unit_type: UnitType , location: Location, time: Time) -> EndNode {
        EndNode{
            unit_type,
            location,
            time,
            covered_by: None
        }
    }
}

impl fmt::Display for EndNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"end for {:?} at {} ({})", self.unit_type, self.location, self.time)
    }
}
