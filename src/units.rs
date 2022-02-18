use std::fmt;
use crate::distance::Distance;
use crate::time::{Time,Duration};
use crate::locations::{Locations,Location};
use crate::base_types::UnitId;
use std::collections::HashMap;
use std::iter::Iterator;

pub(crate) struct Units {
    units: HashMap<UnitId, Unit>,
}


pub(crate) struct Unit {
    id: UnitId,
    unit_type: UnitType,
    start_time: Time,
    start_location: Location,
    initial_time_counter: Duration, // time passed since last maintenance (at start_node)
    initial_dist_counter: Distance, // distance since last maintenance (at start_node)
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(crate) enum UnitType {
    Standard,
    Giruno,
    FVDosto,
    Astoro
}

/////////////////////////////////////////////////////////////////////
//////////////////////////////// Units //////////////////////////////
/////////////////////////////////////////////////////////////////////

impl Units {
    pub(crate) fn load_from_csv(path: &str, locations: &Locations) -> Units {
        let mut units: HashMap<UnitId, Unit> = HashMap::new();
        let mut reader = csv::ReaderBuilder::new().delimiter(b';').from_path(path).expect("csv-file for loading units not found");
        for result in reader.records() {
            let record = result.expect("Some recond cannot be read while reading units");
            let id = UnitId::from(record.get(0).unwrap());
            let unit_type = UnitType::Standard;
            let start_time = Time::new(record.get(1).unwrap());
            let start_location = locations.get_location(record.get(2).unwrap());
            let initial_time_counter = Duration::from_iso(record.get(3).unwrap());
            let initial_dist_counter =  Distance::from_km(record.get(4).unwrap().parse().unwrap());
            units.insert(id,Unit{id,unit_type,start_time,start_location,initial_time_counter,initial_dist_counter});
        }

        Units{units}
    }
}

impl Units {
    pub(crate) fn len(&self) -> usize {
        self.units.len()
    }

    pub(crate) fn get_all(&self) -> Vec<UnitId> {
        let mut ids: Vec<UnitId> = self.units.keys().cloned().collect();
        ids.sort();
        ids
    }

    pub(crate) fn get_unit(&self, id: UnitId) -> &Unit {
        self.units.get(&id).unwrap()
    }
}

/////////////////////////////////////////////////////////////////////
//////////////////////////////// Unit ///////////////////////////////
/////////////////////////////////////////////////////////////////////

// methods
impl Unit {
    pub(crate) fn id(&self) -> UnitId {
        self.id
    }

    pub(crate) fn unit_type(&self) -> UnitType {
        self.unit_type
    }

    pub(crate) fn start_time(&self) -> Time {
        self.start_time
    }

    pub(crate) fn start_location(&self) -> Location {
        self.start_location
    }

    pub(crate) fn initial_time_counter(&self) -> Duration {
        self.initial_time_counter
    }

    pub(crate) fn initial_dist_counter(&self) -> Distance {
        self.initial_dist_counter
    }
}



impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unit {} ({:?}; {}; {})", self.id, self.unit_type, self.initial_dist_counter, self.initial_time_counter)
    }
}


