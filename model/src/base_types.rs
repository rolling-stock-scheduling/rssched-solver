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

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VehicleId {
    #[display(fmt = "veh{}", _0)]
    Vehicle(Id),
    #[display(fmt = "dummy{}", _0)]
    Dummy(Id),
}

impl VehicleId {
    pub fn vehicle_from(id: Id) -> VehicleId {
        VehicleId::Vehicle(id)
    }
    pub fn dummy_from(id: Id) -> VehicleId {
        VehicleId::Dummy(id)
    }
}

#[derive(Display, From, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DepotId(pub Id);

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeId {
    #[display(fmt = "sdep_{}", _0)]
    StartDepot(Id),
    #[display(fmt = "serv_{}_{}", departure_id, order)]
    Service { departure_id: Id, order: u8 },
    #[display(fmt = "main_{}", _0)]
    Maintenance(Id),
    #[display(fmt = "edep_{}", _0)]
    EndDepot(Id),
}

impl NodeId {
    pub fn start_depot_from(id: Id) -> NodeId {
        NodeId::StartDepot(id)
    }
    pub fn end_depot_from(id: Id) -> NodeId {
        NodeId::EndDepot(id)
    }
    pub fn service_from(departure_id: Id, order: u8) -> NodeId {
        NodeId::Service {
            departure_id,
            order,
        }
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
pub type Cost = u64;
