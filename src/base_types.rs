use crate::utilities::CopyStr;

/// the side on which units are leaving or entering the station
#[derive(Copy,Clone,PartialEq,Eq)]
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

pub(crate) type Meter = u64;

pub(crate) type Cost = f32;
