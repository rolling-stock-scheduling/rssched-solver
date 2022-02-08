use std::fmt;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::distance::Distance;
use crate::time::Duration;

type Station = String; // Stations are represented by String codes of length 2 to 4.

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
pub(crate) struct Locations {
    stations: HashSet<Location>,
    distances: HashMap<Station,HashMap<Station,Distance>>,
    travel_times: HashMap<Station,HashMap<Station,Duration>>
}

#[derive(Hash,Eq,PartialEq)]
pub(crate) enum Location {
    Location(Station),
    Infinity // distance to Infinity is always infinity
}


/////////////////////////////////////////////////////////////////////
////////////////////////////// Locations ////////////////////////////
/////////////////////////////////////////////////////////////////////

// static functions
impl Locations {

    pub(crate) fn load_from_csv(path: String) -> Locations {
        let mut stations: HashSet<Location> = HashSet::new();
        let mut distances: HashMap<Station,HashMap<Station,Distance>> = HashMap::new();
        let mut travel_times: HashMap<Station,HashMap<Station,Duration>> = HashMap::new();
        let mut reader = csv::ReaderBuilder::new().delimiter(b';').from_path(path).expect("csv-file for loading locations not found");
        for result in reader.records() {
            let record = result.expect("Some recond cannot be read while reading locations");
            let origin = String::from(record.get(0).unwrap());
            let destination = String::from(record.get(1).unwrap());
            
            let travel_time_string = String::from(record.get(2).unwrap());
            let travel_time_formatted = travel_time_string.split('T').last().unwrap().split('M').next().unwrap().replace("H",":");
            let travel_time = Duration::new(&travel_time_formatted);

            let distance_string = String::from(record.get(3).unwrap());
            let distance = Distance::from_km(distance_string.parse().unwrap());

            fn insert<A>(distances: &mut HashMap<Station,HashMap<Station,A>>, origin: &Station, destination: &Station, value: A) {
                match distances.get_mut(origin){
                    Some(hm) => hm,
                    None => {distances.insert(origin.clone(),HashMap::new());
                             distances.get_mut(origin).unwrap()}
                }.insert(destination.clone(), value);
            }

            stations.insert(Location::of(&origin));
            stations.insert(Location::of(&destination));

            insert(&mut distances,&origin,&destination, distance);
            insert(&mut distances,&destination,&origin, distance);
            insert(&mut distances,&origin,&origin, Distance::zero());
            insert(&mut distances,&destination,&destination, Distance::zero());
            
            insert(&mut travel_times,&origin,&destination, travel_time);
            insert(&mut travel_times,&destination,&origin, travel_time);
            insert(&mut travel_times,&origin,&origin, Duration::zero());
            insert(&mut travel_times,&destination,&destination, Duration::zero());
            

        }
        Locations{stations, distances, travel_times}

    }

    pub(crate) fn create() -> Locations {
        // TODO: Read stations and distance matrix from file (?)

        let zue = String::from("ZUE");
        let bs = String::from("BS");
        let zas = String::from("ZAS");

        let stations = HashSet::from([Location::of(&zue), Location::of(&bs), Location::of(&zas)]);

        let from_zuerich = HashMap::from([(zue.clone(), Distance::from_km(0.0)), (bs.clone(), Distance::from_km(150.0)), (zas.clone(), Distance::from_km(5.0))]);
        let from_basel = HashMap::from([(zue.clone(), Distance::from_km(150.0)), (bs.clone(), Distance::from_km(0.0)), (zas.clone(), Distance::from_km(145.0))]);
        let from_altstetten = HashMap::from([(zue.clone(), Distance::from_km(5.0)), (bs.clone(), Distance::from_km(145.0)), (zas.clone(), Distance::from_km(0.0))]);

        let distances = HashMap::from([(zue.clone(), from_zuerich), (bs.clone(), from_basel), (zas.clone(), from_altstetten)]);

        let tt_zuerich = HashMap::from([(zue.clone(), Duration::new("0:00")), (bs.clone(), Duration::new("1:40")), (zas.clone(), Duration::new("0:03"))]);
        let tt_basel = HashMap::from([(zue.clone(), Duration::new("1:40")), (bs.clone(), Duration::new("0:00")), (zas.clone(), Duration::new("1:30"))]);
        let tt_altstetten = HashMap::from([(zue.clone(), Duration::new("0:03")), (bs.clone(), Duration::new("1:30")), (zas.clone(), Duration::new("0:00"))]);

        let travel_times = HashMap::from([(zue.clone(), tt_zuerich), (bs.clone(), tt_basel), (zas.clone(), tt_altstetten)]);


        Locations{stations, distances, travel_times}

    }
}

// methods
impl Locations {
    pub(crate) fn get_all_stations(&self) -> Vec<&Location> {
        self.stations.iter().collect()
    }

    pub(crate) fn distance(&self, a: &Location, b: &Location) -> Distance {
        match a {
            Location::Infinity => Distance::Infinity,
            Location::Location(station_a) =>
                match b {
                    Location::Infinity => Distance::Infinity,
                    Location::Location(station_b) =>
                        *self.distances.get(station_a).unwrap().get(station_b).unwrap()
                }
        }
    }

    pub(crate) fn travel_time(&self, a: &Location, b: &Location) -> Duration {
        match a {
            Location::Infinity => Duration::Infinity,
            Location::Location(station_a) => {
                match b {
                    Location::Infinity => Duration::Infinity,
                    Location::Location(station_b) =>
                        *self.travel_times.get(station_a).unwrap().get(station_b).unwrap()
                }
            }
        }
    }
}


/////////////////////////////////////////////////////////////////////
////////////////////////////// Location /////////////////////////////
/////////////////////////////////////////////////////////////////////

impl Location {
    fn of(station: &str) -> Location {
        Location::Location(String::from(station))
    }
}

impl Location {
    fn as_station(&self) -> Station {
        match self {
            Location::Location(s) => s.clone(),
            Location::Infinity => {panic!("Location is infinity!")},
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Location::Location(s) => write!(f, "{}", s),
            Location::Infinity => write!(f, "INFINITY!"),
        }
    }
}

