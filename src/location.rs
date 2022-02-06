use std::fmt;
use crate::distance::Distance;
use crate::time::Duration;

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



/// a type for storing the pair-wise distances and travel times between all stations.
/// Distances are stored as a Vec<Vec<Distance>>-matrix.
/// Travel times are stored as a Vec<Vec<Duration>>-matrix.
/// The indices in the matrix equal the indices in the station vector equal the index stored in
/// each station.
/// The distance can be obtained by the dist function which has two &Location as input and provides
/// a Distance.
/// The travel time can be obtained by the tt function which has two &Location as input and
/// provides a Duration.
///
/// Distances and travel times should satisfy the triangle-inequality. This is not asserted.
///
/// A DeadHeadMetrics instance can only be created together with the Vec<Distance> of wrapped
/// stations. Use loactions::create_locations for that. Hence, the indices should always be consistent.
pub(crate) struct DeadHeadMetrics {
    distances: Vec<Vec<Distance>>,
    travel_times: Vec<Vec<Duration>>
}

// methods:
impl DeadHeadMetrics {
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

    pub(crate) fn tt(&self, a: &Location, b: &Location) -> Duration {
        match a {
            Location::Infinity => Duration::Infinity,
            Location::Location(station_a) =>
                match b {
                    Location::Infinity => Duration::Infinity,
                    Location::Location(station_b) =>
                        *self.travel_times.get(station_a.idx).unwrap().get(station_b.idx).unwrap()
                }
        }


    }
}

pub(crate) fn create_locations() -> (Vec<Location>, DeadHeadMetrics) {
    // TODO: Read stations and distance matrix from file (?)

    let stations = vec!(Location::new(0, "Zuerich"),
                        Location::new(1, "Basel"),
                        Location::new(2, "Altstetten"));

    let from_zuerich = vec!(Distance::from_km(0), Distance::from_km(150), Distance::from_km(5));
    let from_basel = vec!(Distance::from_km(150), Distance::from_km(0), Distance::from_km(145));
    let from_altstetten = vec!(Distance::from_km(5), Distance::from_km(145), Distance::from_km(0));

    let distances = vec!(from_zuerich, from_basel, from_altstetten);

    let tt_zuerich = vec!(Duration::new("0:00"), Duration::new("0:00"), Duration::new("0:00"));
    let tt_basel = vec!(Duration::new("0:00"), Duration::new("0:00"), Duration::new("0:00"));
    let tt_altstetten = vec!(Duration::new("0:00"), Duration::new("0:00"), Duration::new("0:00"));

    let travel_times = vec!(tt_zuerich, tt_basel, tt_altstetten);


    (stations, DeadHeadMetrics{ distances, travel_times})

}
