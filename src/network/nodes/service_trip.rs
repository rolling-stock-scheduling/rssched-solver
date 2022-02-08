use crate::distance::Distance;
use crate::locations::Location;
use crate::time::{Time,Duration};
use std::fmt;

pub(crate) struct ServiceTrip<'a> {
    origin: &'a Location,
    destination: &'a Location,
    departure: Time,
    arrival: Time,
    length: Distance,
}
// methods
impl<'a> ServiceTrip<'a> {
    pub(crate) fn origin(&self) -> &Location {
        self.origin
    }

    pub(crate) fn destination(&self) -> &Location {
        self.destination
    }

    pub(crate) fn departure(&self) -> Time {
        self.departure
    }

    pub(crate) fn arrival(&self) -> Time {
        self.arrival
    }

    pub(crate) fn length(&self) -> Distance {
        self.length
    }

    pub(crate) fn travel_time(&self) -> Duration {
        self.arrival - self.departure //could in principle also be something else if there are long stops
    }

}

// static functions
impl<'a> ServiceTrip<'a> {
    pub(super) fn new(origin: &'a Location, destination: &'a Location, departure: Time, arrival: Time, length: Distance) -> ServiceTrip<'a> {
        ServiceTrip{
            origin,
            destination,
            departure,
            arrival,
            length
        }
    }
}

impl<'a> fmt::Display for ServiceTrip<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"service from {} ({}) to {} ({}), {}", self.origin, self.departure, self.destination, self.arrival, self.length)
    }
}
