use std::fmt;
use crate::distance::Distance;

type StationIdx = usize; // connecting the station to the index in the DeadHeadDistance-matrix

pub(crate) enum Location {
    Location(Station),
    Infinity // distance to Infinity is always infinity
}

impl Location {
    fn new(idx: StationIdx, name: &str) -> Location {
        Location::Location(Station{
            idx,
            name : String::from(name)
        })
    }
}

impl Location {
    fn as_station(&self) -> &Station {
        match self {
            Location::Location(s) => s,
            Location::Infinity => {panic!("Location is infinity!")},
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Location::Location(s) => write!(f, "{}", s.name),
            Location::Infinity => write!(f, "INFINITY!"),
        }
    }
}


#[derive(Hash, PartialEq, Eq)]
pub(crate) struct Station {
    idx: StationIdx,
    name: String
}





pub(crate) struct DeadHeadDistances {
    distances: Vec<Vec<Distance>>
}

// methods:
impl DeadHeadDistances {
    pub(crate) fn dist(&self, a: &Location, b: &Location) -> Distance {
        match a {
            Location::Infinity => Distance::Infinity,
            Location::Location(station_a) =>
                match b {
                    Location::Infinity => Distance::Infinity,
                    Location::Location(station_b) =>
                        *self.distances.get(station_a.idx).unwrap().get(station_b.idx).unwrap()
                }
        }
    }
}

pub(crate) fn create_locations() -> (Vec<Location>, DeadHeadDistances) {
    // TODO: Read stations and distance matrix from file (?)

    let stations = vec!(Location::new(0, "Zuerich"),
                        Location::new(1, "Basel"),
                        Location::new(2, "Altstetten"));

    let from_zuerich = vec!(Distance::from_km(0), Distance::from_km(150), Distance::from_km(5));
    let from_basel = vec!(Distance::from_km(150), Distance::from_km(0), Distance::from_km(145));
    let from_altstetten = vec!(Distance::from_km(5), Distance::from_km(145), Distance::from_km(0));

    let distances = vec!(from_zuerich, from_basel, from_altstetten);

    (stations, DeadHeadDistances{ distances })

}
