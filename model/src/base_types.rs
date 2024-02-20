use derive_more::Display;
use derive_more::From;

pub mod distance;
pub mod location;

pub use distance::Distance;
pub use location::Location;

pub type Id = u16;

#[derive(Display, From, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocationId(pub Id);

#[derive(Display, From, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VehicleTypeId(pub Id);

#[derive(Display, From, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VehicleId(pub Id);

#[derive(Display, From, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DepotId(pub Id);

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeId {
    StartDepot(Id),
    Service(Id),
    Maintenance(Id),
    EndDepot(Id),
}

impl NodeId {
    pub fn start_depot_from(id: Id) -> NodeId {
        NodeId::StartDepot(id)
    }
    pub fn end_depot_from(id: Id) -> NodeId {
        NodeId::EndDepot(id)
    }
    pub fn service_from(id: Id) -> NodeId {
        NodeId::Service(id)
    }
    pub fn maintenance_from(id: Id) -> NodeId {
        NodeId::Maintenance(id)
    }
    pub fn smallest() -> NodeId {
        NodeId::StartDepot(0)
    }
}

pub type VehicleCount = u32;
pub type PassengerCount = u32;
pub type SeatDistance = u64; // TODO remove this
pub type Meter = u64;
