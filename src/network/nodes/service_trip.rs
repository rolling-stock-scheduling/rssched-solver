use crate::placeholder::{Location, DayTime, Distance};

pub(crate) struct ServiceTrip {
    start_station: Location,
    end_station: Location,
    departure_time: DayTime,
    length: Distance,
}

impl ServiceTrip {
    pub(super) fn new(start_station: Location, end_station: Location, departure_time: DayTime, length: Distance) -> ServiceTrip {
        ServiceTrip{
            start_station,
            end_station,
            departure_time,
            length
        }
    }
}
