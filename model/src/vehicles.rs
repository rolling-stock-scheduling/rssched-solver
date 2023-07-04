use crate::base_types::{DateTime, Distance, Duration, Location, LocationId, VehicleId};
use crate::locations::Locations;
use std::collections::HashMap;
use std::fmt;
use std::iter::Iterator;
use std::sync::Arc;

pub struct Vehicles {
    vehicles: HashMap<VehicleId, Vehicle>,
    ids_sorted: Vec<VehicleId>,
}

#[derive(Clone)]
pub struct Vehicle {
    id: VehicleId,
    vehicle_type: VehicleType,
    start_time: DateTime,
    start_location: Location,
    initial_duration_counter: Duration, // time passed since last maintenance (at start_node)
    initial_dist_counter: Distance,     // distance since last maintenance (at start_node)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleType {
    Standard,
    // Giruno,
    // FVDosto,
    // Astoro
}

/////////////////////////////////////////////////////////////////////
///////////////////////////// Vehicles //////////////////////////////
/////////////////////////////////////////////////////////////////////

impl Vehicles {
    pub fn load_from_csv(path: &str, loc: Arc<Locations>) -> Vehicles {
        let mut vehicles: HashMap<VehicleId, Vehicle> = HashMap::new();
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_path(path)
            .expect("csv-file for loading Vehicles not found");
        for result in reader.records() {
            let record = result.expect("Some recond cannot be read while reading Vehicles");
            let id = VehicleId::from(record.get(0).unwrap());
            let vehicle_type = VehicleType::Standard;
            let start_time = DateTime::new(record.get(1).unwrap());
            let start_location = loc.get_location(LocationId::from(record.get(2).unwrap()));
            let initial_duration_counter = Duration::from_iso(record.get(3).unwrap());
            let initial_dist_counter = Distance::from_km(record.get(4).unwrap().parse().unwrap());
            vehicles.insert(
                id,
                Vehicle {
                    id,
                    vehicle_type,
                    start_time,
                    start_location,
                    initial_duration_counter,
                    initial_dist_counter,
                },
            );
        }

        let mut ids_sorted: Vec<VehicleId> = vehicles.keys().copied().collect();
        ids_sorted.sort();
        Vehicles {
            vehicles,
            ids_sorted,
        }
    }
}

impl Vehicles {
    // pub fn len(&self) -> usize {
    // self.Vehicles.len()
    // }

    pub fn iter(&self) -> impl Iterator<Item = VehicleId> + '_ {
        self.ids_sorted.iter().copied()
    }

    pub fn get_vehicle(&self, id: VehicleId) -> &Vehicle {
        self.vehicles.get(&id).unwrap()
    }

    pub fn contains(&self, id: VehicleId) -> bool {
        self.vehicles.contains_key(&id)
    }
}

/////////////////////////////////////////////////////////////////////
//////////////////////////////// vehicle ///////////////////////////////
/////////////////////////////////////////////////////////////////////

// methods
impl Vehicle {
    // pub fn id(&self) -> VehicleId {
    // self.id
    // }

    pub fn vehicle_type(&self) -> VehicleType {
        self.vehicle_type
    }

    pub fn start_time(&self) -> DateTime {
        self.start_time
    }

    pub fn start_location(&self) -> Location {
        self.start_location
    }

    pub fn initial_duration_counter(&self) -> Duration {
        self.initial_duration_counter
    }

    pub fn initial_dist_counter(&self) -> Distance {
        self.initial_dist_counter
    }
}

impl fmt::Display for Vehicle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "vehicle {} ({:?}; {}; {})",
            self.id, self.vehicle_type, self.initial_dist_counter, self.initial_duration_counter
        )
    }
}
