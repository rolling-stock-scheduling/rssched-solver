use crate::location::Location;
use crate::time::Time;
use std::fmt;

pub(crate) struct MaintenanceSlot<'a> {
    location: &'a Location,
    start: Time,
    end: Time,
}
// methods
impl<'a> MaintenanceSlot<'a> {
    pub(crate) fn location(&self) -> &Location {
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
impl<'a> MaintenanceSlot<'a> {
    pub(super) fn new(location: &'a Location, start: Time, end: Time) -> MaintenanceSlot {
        MaintenanceSlot{
            location,
            start,
            end
        }
    }
}

impl<'a> fmt::Display for MaintenanceSlot<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Maintenance at {} (from {} to {})", self.location, self.start, self.end)
    }
}
