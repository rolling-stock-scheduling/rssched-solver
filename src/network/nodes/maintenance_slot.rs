use crate::placeholder::{Location, DayTime};
use std::fmt;

pub(crate) struct MaintenanceSlot {
    location: Location,
    start_time: DayTime,
    end_time: DayTime,
}

impl MaintenanceSlot {
    pub(super) fn new(location: Location, start_time: DayTime, end_time: DayTime) -> MaintenanceSlot {
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
