use derive_more::Display;
use derive_more::From;

pub mod distance;
pub mod location;

pub use distance::Distance;
pub use location::Location;
pub use location::StationSide;

#[derive(Display, From, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocationId(pub u16);

#[derive(Display, From, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VehicleTypeId(pub u16);

#[derive(Display, From, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VehicleId(pub u16);

#[derive(Display, From, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DepotId(pub u16);

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeId {
    StartDepot(u16),
    Service(u16),
    Maintenance(u16),
    EndDepot(u16),
}

impl NodeId {
    pub fn start_depot_from(id: u16) -> NodeId {
        NodeId::StartDepot(id)
    }
    pub fn end_depot_from(id: u16) -> NodeId {
        NodeId::EndDepot(id)
    }
    pub fn service_from(id: u16) -> NodeId {
        NodeId::Service(id)
    }
    pub fn maintenance_from(id: u16) -> NodeId {
        NodeId::Maintenance(id)
    }
    pub fn smallest() -> NodeId {
        NodeId::StartDepot(0)
    }
}

pub type VehicleCount = u32;
pub type PassengerCount = u32;
pub type TrainLength = u16;
pub type SeatDistance = u64;
pub type Meter = u64;
pub type Cost = f32;

pub const COST_ZERO: f32 = 0.0f32;
