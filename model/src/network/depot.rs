use std::collections::HashMap;

use crate::base_types::{DepotId, Location, PassengerCount, VehicleTypeId};

pub struct Depot {
    depot_id: DepotId,
    location: Location,
    capacities: HashMap<VehicleTypeId, Option<PassengerCount>>, // number of vehicles that can be
                                                                // spawned. None means no limit.
}

// methods
impl Depot {
    pub fn depot_id(&self) -> DepotId {
        self.depot_id
    }

    pub fn location(&self) -> Location {
        self.location
    }

    /// None means no limit
    pub fn capacity_for(&self, vehicle_type_id: VehicleTypeId) -> Option<PassengerCount> {
        *self.capacities.get(&vehicle_type_id).unwrap_or(&Some(0))
    }
}

// static
impl Depot {
    pub fn new(
        depot_id: DepotId,
        location: Location,
        capacities: HashMap<VehicleTypeId, Option<PassengerCount>>,
    ) -> Self {
        Self {
            depot_id,
            location,
            capacities,
        }
    }
}
