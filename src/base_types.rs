use crate::utilities::CopyStr;

/// the side on which units are leaving or entering the station
#[derive(Copy,Clone)]
pub(crate) enum StationSide {
    Back, // corresponds to 0
    Front // corresponds to 1
}

impl StationSide {
    pub(crate) fn from(string: &str) -> StationSide {
        match string {
            "0" => StationSide::Back,
            "1" => StationSide::Front,
            _ => panic!("StationSide is neither '0' nor '1'")
        }
    }
}

pub(crate) type UnitId = CopyStr<10>;


pub(crate) type NodeId = CopyStr<32>;

pub(crate) type Meter = u32;
pub(crate) type Penalty = u32;
// TODO: Integrate this into the Penalty type
pub(crate) const PENALTY_ZERO: Penalty = 0;
pub(crate) const PENALTY_INF: Penalty = 100000;
pub(crate) const PENALTY_UNUSED_MAINTENANCE: Penalty = 1;
