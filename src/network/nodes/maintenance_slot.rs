use crate::locations::Location;
use crate::time::Time;
use std::fmt;

pub(crate) struct MaintenanceSlot {
    location: Location,
    start: Time,
    end: Time,
    // used_by: UnitIdx
}
// methods
impl MaintenanceSlot {
    pub(crate) fn location(&self) -> Location {
        self.location
    }

    pub(crate) fn start(&self) -> Time {
        self.start
    }

    pub(crate) fn end(&self) -> Time {
        self.end
    }
}

// static functions:
impl MaintenanceSlot {
    pub(super) fn new(location: Location, start: Time, end: Time) -> MaintenanceSlot {
        MaintenanceSlot{
            location,
            start,
            end
        }
    }
}

impl fmt::Display for MaintenanceSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"maintenance at {} (from {} to {})", self.location, self.start, self.end)
    }
}
