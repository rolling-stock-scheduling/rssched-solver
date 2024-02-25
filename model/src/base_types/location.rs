use super::LocationId;
use std::fmt;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Location {
    Station(LocationId),
    Nowhere, // distance to Nowhere is always infinity
             // Everywhere // distance to Everywehre is always zero (except for Nowhere)
}

/// the side on which Vehicles are leaving or entering the station
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum StationSide {
    Back,  // corresponds to 0
    Front, // corresponds to 1
}

impl StationSide {
    pub fn from(string: &str) -> StationSide {
        match string {
            "0" => StationSide::Back,
            "1" => StationSide::Front,
            _ => panic!("StationSide is neither '0' nor '1'"),
        }
    }
}

impl Location {
    pub fn of(station: LocationId) -> Location {
        Location::Station(station)
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Location::Station(s) => write!(f, "{}", s),
            Location::Nowhere => write!(f, "NOWHERE!"),
            // Location::Everywhere => write!(f, "EVERYWHERE!")
        }
    }
}
