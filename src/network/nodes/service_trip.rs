use crate::placeholder::{Location, DayTime, Distance};
use std::fmt;

pub(crate) struct ServiceTrip {
    start_station: Location,
    end_station: Location,
    departure_time: DayTime,
    arrival_time: DayTime,
    length: Distance,
}

impl ServiceTrip {
    pub(super) fn new(start_station: Location, end_station: Location, departure_time: DayTime, arrival_time: DayTime, length: Distance) -> ServiceTrip {
        ServiceTrip{
            start_station,
            end_station,
            departure_time,
            arrival_time,
            length
        }
    }
}

impl fmt::Display for ServiceTrip {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Service from {} ({}) to {} ({}), d:{} km", self.start_station, self.departure_time, self.end_station, self.arrival_time, self.length)
    }
}
