use crate::utilities::CopyStr;

pub type LocationId = CopyStr<10>; // Stations are represented by String codes of length up to
                                   // 10.

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub enum Location {
    Station(LocationId),
    Nowhere, // distance to Nowhere is always infinity
             // Everywhere // distance to Everywehre is always zero (except for Nowhere)
}

/// the side on which Vehicles are leaving or entering the station
#[derive(Copy, Clone, PartialEq, Eq)]
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
