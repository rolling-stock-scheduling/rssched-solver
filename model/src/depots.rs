use crate::base_types::{DepotId, Location, VehicleCount, VehicleTypeId};

pub struct Depots {
    depots: Vec<Depot>,
}

// we have a seperate depot for each vehicle type
// (id, vehicle_type) is unique
pub struct Depot {
    id: DepotId,
    location: Location,
    vehicle_type: VehicleTypeId,
    upper_bound: Option<VehicleCount>, // number of vehicles that can be spawned. None means no limit.
}

impl Depot {
    pub fn id(&self) -> DepotId {
        self.id
    }

    pub fn location(&self) -> Location {
        self.location
    }

    pub fn vehicle_type(&self) -> VehicleTypeId {
        self.vehicle_type
    }
}

impl Depot {
    pub fn new(
        id: DepotId,
        location: Location,
        vehicle_type: VehicleTypeId,
        upper_bound: Option<VehicleCount>,
    ) -> Depot {
        Depot {
            id,
            location,
            vehicle_type,
            upper_bound,
        }
    }
}

impl Depots {
    pub fn new(depots: Vec<Depot>) -> Depots {
        Depots { depots }
    }
}
