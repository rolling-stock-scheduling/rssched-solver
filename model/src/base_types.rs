use crate::utilities::CopyStr;

pub mod distance;
pub mod location;
pub mod time;

pub use distance::Distance;
pub use location::Location;
pub use location::LocationId;
pub use location::StationSide;
pub use time::{DateTime, Duration};

pub type VehicleTypeId = CopyStr<10>;
pub type PassengerCount = u8;
pub type TrainLength = u8;

pub type VehicleId = CopyStr<10>;
pub type DummyId = CopyStr<8>;

pub type VehicleCount = u32;

pub type DepotId = CopyStr<10>;

pub type NodeId = CopyStr<32>;

pub type Meter = u64;

pub type Cost = f32;

pub const COST_ZERO: f32 = 0.0f32;
