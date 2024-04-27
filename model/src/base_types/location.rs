use super::LocationIdx;
use std::fmt;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Location {
    Station(LocationIdx),
    Nowhere, // distance to Nowhere is always infinity
}

impl Location {
    pub fn idx(&self) -> LocationIdx {
        match self {
            Location::Station(s) => *s,
            Location::Nowhere => panic!("Location::Nowhere has no idx."),
        }
    }

    pub fn of(idx: LocationIdx) -> Location {
        Location::Station(idx)
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
