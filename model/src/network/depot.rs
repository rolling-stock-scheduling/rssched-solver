use std::collections::HashMap;

use crate::base_types::{DepotIdx, Location, VehicleCount, VehicleTypeIdx};

pub struct Depot {
    idx: DepotIdx,
    id: String,
    location: Location,
    total_capacity: VehicleCount,
    allowed_types: HashMap<VehicleTypeIdx, Option<VehicleCount>>, // number of vehicles that can be
                                                                 // spawned. None means no limit.
}

// methods
impl Depot {
    pub fn idx(&self) -> DepotIdx {
        self.idx
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn location(&self) -> Location {
        self.location
    }

    pub fn total_capacity(&self) -> VehicleCount {
        self.total_capacity
    }

    /// takes the minimum of vehicle specific capacity (None means no limit) and depot capacity
    pub fn capacity_for(&self, vehicle_type_id: VehicleTypeIdx) -> VehicleCount {
        match self.allowed_types.get(&vehicle_type_id) {
            Some(Some(capacity)) => VehicleCount::min(*capacity, self.total_capacity),
            Some(None) => self.total_capacity, // no vehicle specific limit
            None => 0,                         // vehicle type not allowed
        }
    }
}

// static
impl Depot {
    pub fn new(
        depot_id: DepotIdx,
        name: String,
        location: Location,
        total_capacity: VehicleCount,
        allowed_types: HashMap<VehicleTypeIdx, Option<VehicleCount>>,
    ) -> Self {
        Self {
            idx: depot_id,
            id: name,
            location,
            total_capacity,
            allowed_types,
        }
    }
}
