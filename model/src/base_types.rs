use crate::utilities::CopyStr;

pub mod distance;
pub mod location;

pub use distance::Distance;
pub use location::Location;
pub use location::StationSide;

pub type LocationId = CopyStr<32>;

pub type VehicleTypeId = CopyStr<32>;
pub type PassengerCount = u32;
pub type TrainLength = u16;
pub type SeatDistance = u64;

pub type VehicleId = CopyStr<6>;

pub type VehicleCount = u32;

pub type NodeId = CopyStr<34>; // two chars for "s_depot" and "e_depot"

pub type DepotId = CopyStr<32>;

pub type Meter = u64;

pub type Cost = f32;

pub const COST_ZERO: f32 = 0.0f32;
