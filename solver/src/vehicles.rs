use crate::base_types::{DateTime, Distance, Duration, Location, VehicleId};
use std::collections::HashMap;
use std::fmt;
use std::iter::Iterator;

pub struct Vehicles {
    vehicles: HashMap<VehicleId, Vehicle>,
    ids_sorted: Vec<VehicleId>,
}

#[derive(Clone)]
pub struct Vehicle {
    id: VehicleId,
    vehicle_type: VehicleType,
    start_depot: Location,
}

impl Vehicle {
    pub fn new(id: VehicleId, vehicle_type: VehicleType, start_depot: Location) -> Vehicle {
        Vehicle {
            id,
            vehicle_type,
            start_depot,
        }
    }
}

/////////////////////////////////////////////////////////////////////
///////////////////////////// Vehicles //////////////////////////////
/////////////////////////////////////////////////////////////////////

impl Vehicles {
    pub fn iter(&self) -> impl Iterator<Item = VehicleId> + '_ {
        self.ids_sorted.iter().copied()
    }

    pub fn get_vehicle(&self, id: VehicleId) -> &Vehicle {
        self.vehicles.get(&id).unwrap()
    }

    pub fn contains(&self, id: VehicleId) -> bool {
        self.vehicles.contains_key(&id)
    }

    pub fn spawn_vehicle(
        &mut self,
        id: VehicleId,
        vehicle_type: VehicleType,
        start_time: DateTime,
        start_depot: Location,
    ) {
        let vehicle = Vehicle::new(id, vehicle_type, start_depot);
        self.vehicles.insert(id, vehicle);
        self.ids_sorted.push(id);
        self.ids_sorted.sort();
    }
}

impl Vehicles {
    pub fn new(vehicle_vector: Vec<Vehicle>) -> Vehicles {
        let mut vehicles = HashMap::new();
        let mut ids_sorted = Vec::new();
        for vehicle in vehicle_vector {
            let id = vehicle.id;
            vehicles.insert(id, vehicle);
            ids_sorted.push(id);
        }
        ids_sorted.sort();
        Vehicles {
            vehicles,
            ids_sorted,
        }
    }
}

/////////////////////////////////////////////////////////////////////
///////////////////////////// Vehicle ///////////////////////////////
/////////////////////////////////////////////////////////////////////

// methods
impl Vehicle {
    // pub fn id(&self) -> VehicleId {
    // self.id
    // }

    pub fn vehicle_type(&self) -> VehicleType {
        self.vehicle_type
    }

    pub fn start_depot(&self) -> Location {
        self.start_depot
    }
}

impl fmt::Display for Vehicle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "vehicle {} ({:?}; {};)",
            self.id, self.vehicle_type, self.start_depot,
        )
    }
}
