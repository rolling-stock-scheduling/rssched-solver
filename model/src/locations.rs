// © 2023-2024 ETH Zurich
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// any later version.
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::collections::HashMap;

use rapid_time::Duration;

use crate::base_types::VehicleCount;
use crate::base_types::{Distance, Location, LocationIdx};

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
pub struct Locations {
    stations: HashMap<LocationIdx, (String, Option<VehicleCount>)>, // values: (id, daylimit)
    dead_head_trips: HashMap<LocationIdx, HashMap<LocationIdx, DeadHeadTrip>>,
}

pub struct DeadHeadTrip {
    distance: Distance,
    travel_time: Duration,
}

impl DeadHeadTrip {
    pub fn new(distance: Distance, travel_time: Duration) -> DeadHeadTrip {
        DeadHeadTrip {
            distance,
            travel_time,
        }
    }
}

/////////////////////////////////////////////////////////////////////
////////////////////////////// Locations ////////////////////////////
/////////////////////////////////////////////////////////////////////

// static functions
impl Locations {
    pub fn new(
        stations: HashMap<LocationIdx, (String, Option<VehicleCount>)>,
        dead_head_trips: HashMap<LocationIdx, HashMap<LocationIdx, DeadHeadTrip>>,
    ) -> Locations {
        Locations {
            stations,
            dead_head_trips,
        }
    }
}

// methods
impl Locations {
    pub fn get(&self, location_idx: LocationIdx) -> Result<Location, &'static str> {
        match self.stations.get(&location_idx) {
            Some(_) => Ok(Location::Station(location_idx)),
            None => Err("Location Id is invalid."),
        }
    }

    pub fn get_id(&self, location: Location) -> Result<String, &'static str> {
        match location {
            Location::Nowhere => Ok("NOWHERE".to_string()),
            Location::Station(idx) => match self.stations.get(&idx) {
                Some((name, _)) => Ok(name.clone()),
                None => Err("Location Idx is invalid."),
            },
        }
    }

    pub fn get_daylimit(&self, location: Location) -> Result<Option<VehicleCount>, &'static str> {
        match self.stations.get(&location.idx()) {
            Some((_, daylimit)) => Ok(*daylimit),
            None => Err("Location Id is invalid."),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Location> + '_ {
        self.stations.keys().map(|idx| Location::Station(*idx))
    }

    pub fn distance(&self, a: Location, b: Location) -> Distance {
        match self.get_dead_head_trip(a, b) {
            Some(d) => d.distance,
            None => {
                if a == Location::Nowhere || b == Location::Nowhere {
                    Distance::Infinity
                } else {
                    Distance::ZERO
                }
            }
        }
    }

    pub fn travel_time(&self, a: Location, b: Location) -> Duration {
        match self.get_dead_head_trip(a, b) {
            Some(d) => d.travel_time,
            None => {
                if a == Location::Nowhere || b == Location::Nowhere {
                    Duration::Infinity
                } else {
                    Duration::ZERO
                }
            }
        }
    }

    fn get_dead_head_trip(&self, a: Location, b: Location) -> Option<&DeadHeadTrip> {
        match a {
            Location::Station(station_a) => match b {
                Location::Station(station_b) => Some(
                    self.dead_head_trips
                        .get(&station_a)
                        .unwrap()
                        .get(&station_b)
                        .unwrap(),
                ),
                _ => None,
            },
            _ => None,
        }
    }
}
