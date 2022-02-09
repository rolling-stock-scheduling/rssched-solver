use crate::utilities::CopyStr;

/// the side on which units are leaving or entering the station
#[derive(Copy,Clone)]
pub(crate) enum StationSide {
    Back, // corresponds to 0
    Front // corresponds to 1
}

pub(crate) type UnitId = CopyStr<10>;

pub(crate) type Kilometer = f32;
