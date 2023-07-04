use crate::base_types::UnitId;
use crate::distance::Distance;
use crate::locations::{Location, Locations};
use crate::time::{Duration, Time};
use std::collections::HashMap;
use std::fmt;
use std::iter::Iterator;
use std::sync::Arc;

pub(crate) struct Units {
    units: HashMap<UnitId, Unit>,
    ids_sorted: Vec<UnitId>,
}

#[derive(Clone)]
pub(crate) struct Unit {
    id: UnitId,
    unit_type: UnitType,
    start_time: Time,
    start_location: Location,
    initial_duration_counter: Duration, // time passed since last maintenance (at start_node)
    initial_dist_counter: Distance,     // distance since last maintenance (at start_node)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnitType {
    Standard,
    // Giruno,
    // FVDosto,
    // Astoro
}

/////////////////////////////////////////////////////////////////////
//////////////////////////////// Units //////////////////////////////
/////////////////////////////////////////////////////////////////////

impl Units {
    pub(crate) fn load_from_csv(path: &str, loc: Arc<Locations>) -> Units {
        let mut units: HashMap<UnitId, Unit> = HashMap::new();
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_path(path)
            .expect("csv-file for loading units not found");
        for result in reader.records() {
            let record = result.expect("Some recond cannot be read while reading units");
            let id = UnitId::from(record.get(0).unwrap());
            let unit_type = UnitType::Standard;
            let start_time = Time::new(record.get(1).unwrap());
            let start_location = loc.get_location(record.get(2).unwrap());
            let initial_duration_counter = Duration::from_iso(record.get(3).unwrap());
            let initial_dist_counter = Distance::from_km(record.get(4).unwrap().parse().unwrap());
            units.insert(
                id,
                Unit {
                    id,
                    unit_type,
                    start_time,
                    start_location,
                    initial_duration_counter,
                    initial_dist_counter,
                },
            );
        }

        let mut ids_sorted: Vec<UnitId> = units.keys().copied().collect();
        ids_sorted.sort();
        Units { units, ids_sorted }
    }
}

impl Units {
    // pub(crate) fn len(&self) -> usize {
    // self.units.len()
    // }

    pub(crate) fn iter(&self) -> impl Iterator<Item = UnitId> + '_ {
        self.ids_sorted.iter().copied()
    }

    pub(crate) fn get_unit(&self, id: UnitId) -> &Unit {
        self.units.get(&id).unwrap()
    }

    pub(crate) fn contains(&self, id: UnitId) -> bool {
        self.units.contains_key(&id)
    }
}

/////////////////////////////////////////////////////////////////////
//////////////////////////////// Unit ///////////////////////////////
/////////////////////////////////////////////////////////////////////

// methods
impl Unit {
    // pub(crate) fn id(&self) -> UnitId {
    // self.id
    // }

    pub(crate) fn unit_type(&self) -> UnitType {
        self.unit_type
    }

    pub(crate) fn start_time(&self) -> Time {
        self.start_time
    }

    pub(crate) fn start_location(&self) -> Location {
        self.start_location
    }

    pub(crate) fn initial_duration_counter(&self) -> Duration {
        self.initial_duration_counter
    }

    pub(crate) fn initial_dist_counter(&self) -> Distance {
        self.initial_dist_counter
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "unit {} ({:?}; {}; {})",
            self.id, self.unit_type, self.initial_dist_counter, self.initial_duration_counter
        )
    }
}
