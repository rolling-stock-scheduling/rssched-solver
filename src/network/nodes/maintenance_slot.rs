use crate::placeholder::{Location};
use crate::time::Time;
use std::fmt;

pub(crate) struct MaintenanceSlot {
    location: Location,
    start_time: Time,
    end_time: Time,
}

impl MaintenanceSlot {
    pub(super) fn new(location: Location, start_time: Time, end_time: Time) -> MaintenanceSlot {
        MaintenanceSlot{
            location,
            start_time,
            end_time
        }
    }
}

impl fmt::Display for MaintenanceSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Maintenance at {} (from {} to {})", self.location, self.start_time, self.end_time)
    }
}
