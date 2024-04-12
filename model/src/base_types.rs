use derive_more::Display;
use derive_more::From;

pub mod distance;
pub mod location;

pub use distance::Distance;
pub use location::Location;

pub type Idx = u16;

// TODO rename xxxId to xxxIdx
#[derive(Display, From, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocationId(pub Idx);

#[derive(Display, From, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VehicleTypeId(pub Idx);

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VehicleId {
    #[display(fmt = "veh{}", _0)]
    Vehicle(Idx),
    #[display(fmt = "dummy{}", _0)]
    Dummy(Idx),
}

impl VehicleId {
    pub fn vehicle_from(id: Idx) -> VehicleId {
        VehicleId::Vehicle(id)
    }
    pub fn dummy_from(id: Idx) -> VehicleId {
        VehicleId::Dummy(id)
    }
}

#[derive(Display, From, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DepotId(pub Idx);

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeId {
    #[display(fmt = "sdep_{}", _0)]
    StartDepot(Idx),
    #[display(fmt = "serv_{}_{}", departure_id, order)]
    Service { departure_id: Idx, order: u8 },
    #[display(fmt = "main_{}", _0)]
    Maintenance(Idx),
    #[display(fmt = "edep_{}", _0)]
    EndDepot(Idx),
}

impl NodeId {
    pub fn start_depot_from(id: Idx) -> NodeId {
        NodeId::StartDepot(id)
    }
    pub fn end_depot_from(id: Idx) -> NodeId {
        NodeId::EndDepot(id)
    }
    pub fn service_from(departure_id: Idx, order: u8) -> NodeId {
        NodeId::Service {
            departure_id,
            order,
        }
    }
    pub fn maintenance_from(id: Idx) -> NodeId {
        NodeId::Maintenance(id)
    }
    pub fn smallest() -> NodeId {
        NodeId::StartDepot(0)
    }
}

pub type VehicleCount = u32;
pub type PassengerCount = u32;
pub type Meter = u64;
pub type Cost = u64;
pub type MaintenanceCounter = i64;
