use super::LocationId;
use std::fmt;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Location {
    Station(LocationId),
    Nowhere, // distance to Nowhere is always infinity
}

impl Location {
    pub fn id(&self) -> LocationId {
        match self {
            Location::Station(s) => *s,
            Location::Nowhere => panic!("Location::Nowhere has no id."),
        }
    }

    pub fn of(id: LocationId) -> Location {
        Location::Station(id)
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Location::Station(s) => write!(f, "{}", s),
            Location::Nowhere => write!(f, "NOWHERE!"),
        }
    }
}
