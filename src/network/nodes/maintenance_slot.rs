use crate::placeholder::{Location, DayTime};
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
