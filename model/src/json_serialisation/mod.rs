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

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use rapid_time::{DateTime, Duration};

use crate::base_types::{
    DepotIdx, Distance, Idx, LocationIdx, Meter, PassengerCount, VehicleCount, VehicleTypeIdx,
    MAX_DISTANCE,
};
use crate::config::Config;
use crate::locations::{DeadHeadTrip, Locations};
use crate::network::depot::Depot as ModelDepot;
use crate::network::nodes::MaintenanceSlot as ModelMaintenanceSlot;
use crate::network::nodes::Node;
use crate::network::nodes::ServiceTrip as ModelServiceTrip;
use crate::network::Network;
use crate::vehicle_types::VehicleType as ModelVehicleType;
use crate::vehicle_types::VehicleTypes;

type IdType = String;
type Integer = u64;
type DateTimeString = String;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonInput {
    vehicle_types: Vec<VehicleType>,
    locations: Vec<Location>,
    depots: Option<Vec<Depot>>,
    routes: Vec<Route>,
    departures: Vec<Departures>,
    maintenance_slots: Option<Vec<MaintenanceSlots>>,
    dead_head_trips: DeadHeadTrips,
    parameters: Parameters,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct VehicleType {
    id: IdType,
    capacity: Integer,
    seats: Integer,
    maximal_formation_count: Option<Integer>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Location {
    id: IdType,
    day_limit: Option<Integer>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Depot {
    id: IdType,
    location: IdType,
    capacity: Integer,
    allowed_types: Vec<TypeCapacities>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TypeCapacities {
    vehicle_type: IdType,
    capacity: Option<Integer>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Route {
    id: IdType,
    vehicle_type: IdType,
    segments: Vec<RouteSegment>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RouteSegment {
    id: IdType,
    order: Integer,
    origin: IdType,
    destination: IdType,
    distance: Integer,
    duration: Integer,
    maximal_formation_count: Option<Integer>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Departures {
    id: IdType,
    route: IdType,
    segments: Vec<DepartureSegment>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DepartureSegment {
    id: IdType,
    route_segment: IdType,
    departure: DateTimeString,
    passengers: Integer,
    seated: Integer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct MaintenanceSlots {
    id: IdType,
    location: IdType,
    start: DateTimeString,
    end: DateTimeString,
    track_count: Integer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeadHeadTrips {
    indices: Vec<IdType>,
    durations: Vec<Vec<Integer>>,
    distances: Vec<Vec<Integer>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Parameters {
    forbid_dead_head_trips: Option<bool>,
    day_limit_threshold: Option<Integer>,
    shunting: Shunting,
    maintenance: Option<Maintenance>,
    costs: Costs,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Shunting {
    minimal_duration: Integer,
    dead_head_trip_duration: Integer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Maintenance {
    maximal_distance: Integer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Costs {
    staff: Integer,
    service_trip: Integer,
    maintenance: Option<Integer>,
    dead_head_trip: Integer,
    idle: Integer,
}

pub fn load_rolling_stock_problem_instance_from_json(
    input_data: serde_json::Value,
) -> Arc<Network> {
    let json_input = serde_json::from_value(input_data).expect(
        "Could not parse input data. Please check if the input data is in the correct format",
    );

    let (locations, location_lookup) = create_locations(&json_input);
    let (vehicle_types, vehicle_type_lookup) = create_vehicle_types(&json_input);
    let config = create_config(&json_input);

    Arc::new(create_network(
        &json_input,
        locations,
        vehicle_types,
        config,
        location_lookup,
        vehicle_type_lookup,
    ))
}

fn create_locations(json_input: &JsonInput) -> (Locations, HashMap<IdType, LocationIdx>) {
    let planning_days = determine_planning_days(json_input);
    let mut stations: HashMap<LocationIdx, (String, Option<VehicleCount>)> = HashMap::new(); // PpRF: use vec instead
    let mut dead_head_trips: HashMap<LocationIdx, HashMap<LocationIdx, DeadHeadTrip>> =
        HashMap::new();

    let mut location_lookup: HashMap<IdType, LocationIdx> = HashMap::new();

    // add stations
    for (idx, location_json) in json_input.locations.iter().enumerate() {
        let location_idx = LocationIdx::from(idx as Idx);
        stations.insert(
            location_idx,
            (
                location_json.id.clone(),
                location_json.day_limit.map(|x| x as VehicleCount),
            ),
        );
        location_lookup.insert(location_json.id.clone(), location_idx);
    }

    // add dead head trips
    let mut duration_warning_printed = false;
    let mut distance_warning_printed = false;
    for (i, origin_json) in json_input.dead_head_trips.indices.iter().enumerate() {
        let origin_station = location_lookup[origin_json];
        let mut destination_map: HashMap<LocationIdx, DeadHeadTrip> = HashMap::new();
        for (j, destination_json) in json_input.dead_head_trips.indices.iter().enumerate() {
            let mut duration = Duration::from_seconds(json_input.dead_head_trips.durations[i][j]);
            if duration > planning_days {
                if !duration_warning_printed {
                    println!(
                        "\x1b[93mwarning:\x1b[0m Some dead head trip durations exceed planning duration of {} day(s). \
                        Taking planning duration instead.",
                        planning_days.in_min().unwrap() / 1440
                    );
                    duration_warning_printed = true;
                }
                duration = planning_days;
            }
            let mut distance = Distance::from_meter(json_input.dead_head_trips.distances[i][j]);
            if distance > Distance::from_meter(MAX_DISTANCE) {
                if !distance_warning_printed {
                    println!(
                        "\x1b[93mwarning:\x1b[0m Some dead head trip distances exceed {}m. \
                        This might be a mistake. Distance reduced to {}m.",
                        MAX_DISTANCE, MAX_DISTANCE
                    );
                    distance_warning_printed = true;
                }
                distance = Distance::from_meter(MAX_DISTANCE);
            }
            destination_map.insert(
                location_lookup[destination_json],
                DeadHeadTrip::new(distance, duration),
            );
        }
        dead_head_trips.insert(origin_station, destination_map);
    }

    (Locations::new(stations, dead_head_trips), location_lookup)
}

fn determine_planning_days(json_input: &JsonInput) -> Duration {
    let mut earliest_datetime = DateTime::Latest;
    let mut latest_datetime = DateTime::Earliest;

    if let Some(maintenance_slots) = &json_input.maintenance_slots {
        for maintenance_slot in maintenance_slots {
            earliest_datetime = earliest_datetime.min(DateTime::new(&maintenance_slot.start));
            latest_datetime = latest_datetime.max(DateTime::new(&maintenance_slot.end));
        }
    }

    for departure in &json_input.departures {
        for departure_segment in &departure.segments {
            let departure_time = DateTime::new(&departure_segment.departure);
            let arrival_time = departure_time
                + Duration::from_seconds(
                    json_input
                        .routes
                        .iter()
                        .find(|route| route.id == departure.route)
                        .unwrap()
                        .segments
                        .iter()
                        .find(|segment| segment.id == departure_segment.route_segment)
                        .unwrap()
                        .duration,
                );
            earliest_datetime = earliest_datetime.min(departure_time);
            latest_datetime = latest_datetime.max(arrival_time);
        }
    }
    // round to multiple of day
    Duration::from_seconds(
        (latest_datetime - earliest_datetime)
            .in_sec()
            .unwrap()
            .div_ceil(86400)
            * 86400,
    )
}

fn create_vehicle_types(json_input: &JsonInput) -> (VehicleTypes, HashMap<IdType, VehicleTypeIdx>) {
    let mut vehicle_type_lookup: HashMap<IdType, VehicleTypeIdx> = HashMap::new();
    let vehicle_types: Vec<ModelVehicleType> = json_input
        .vehicle_types
        .iter()
        .enumerate()
        .map(|(idx, vehicle_type)| {
            let vehicle_type_idx = VehicleTypeIdx::from(idx as Idx);
            vehicle_type_lookup.insert(vehicle_type.id.clone(), vehicle_type_idx);
            ModelVehicleType::new(
                vehicle_type_idx,
                vehicle_type.id.clone(),
                vehicle_type.capacity as PassengerCount,
                vehicle_type.seats as PassengerCount,
                vehicle_type
                    .maximal_formation_count
                    .map(|x| x as VehicleCount),
            )
        })
        .collect();

    (VehicleTypes::new(vehicle_types), vehicle_type_lookup)
}

fn create_config(json_input: &JsonInput) -> Config {
    if json_input.parameters.costs.dead_head_trip <= json_input.parameters.costs.service_trip {
        println!(
            "\x1b[93mwarning:\x1b[0m Dead head trip costs are lower than service trip costs. \
            Vehicle will not hitch-hike on service trips."
        );
    }
    Config::new(
        json_input
            .parameters
            .forbid_dead_head_trips
            .unwrap_or(false),
        Duration::from_seconds(json_input.parameters.day_limit_threshold.unwrap_or(0)),
        Duration::from_seconds(json_input.parameters.shunting.minimal_duration),
        Duration::from_seconds(json_input.parameters.shunting.dead_head_trip_duration),
        Distance::from_meter(
            json_input
                .parameters
                .maintenance
                .as_ref()
                .map(|m| m.maximal_distance)
                .unwrap_or(0),
        ),
        json_input.parameters.costs.staff,
        json_input.parameters.costs.service_trip,
        json_input.parameters.costs.maintenance.unwrap_or(0),
        json_input.parameters.costs.dead_head_trip,
        json_input.parameters.costs.idle,
    )
}

fn create_network(
    json_input: &JsonInput,
    locations: Locations,
    vehicle_types: VehicleTypes,
    config: Config,
    location_lookup: HashMap<IdType, LocationIdx>,
    vehicle_type_lookup: HashMap<IdType, VehicleTypeIdx>,
) -> Network {
    let service_trips = create_service_trips(
        json_input,
        &locations,
        &vehicle_types,
        &location_lookup,
        &vehicle_type_lookup,
    );

    let number_of_service_trips: VehicleCount = service_trips
        .values()
        .map(|trips| trips.len() as VehicleCount)
        .sum();
    let depots = create_depots(
        json_input,
        &locations,
        &location_lookup,
        &vehicle_type_lookup,
        number_of_service_trips,
    );

    let maintenance_slots = create_maintenance_slots(json_input, &locations, &location_lookup);

    Network::new(
        depots,
        service_trips,
        maintenance_slots,
        config,
        locations,
        vehicle_types,
    )
}

fn create_depots(
    json_input: &JsonInput,
    loc: &Locations,
    location_lookup: &HashMap<IdType, LocationIdx>,
    vehicle_type_lookup: &HashMap<IdType, VehicleTypeIdx>,
    vehicle_upper_limit: VehicleCount,
) -> Vec<ModelDepot> {
    match &json_input.depots {
        None => {
            // add a depot at every location with unlimited capacity for each type
            let allowed_vehicle_types: HashMap<VehicleTypeIdx, Option<VehicleCount>> =
                vehicle_type_lookup
                    .values()
                    .map(|vehicle_type_idx| (*vehicle_type_idx, None))
                    .collect();
            loc.iter()
                .enumerate()
                .map(|(idx, location)| {
                    ModelDepot::new(
                        DepotIdx::from(idx as Idx),
                        format!("depot_{}", loc.get_id(location).unwrap()),
                        location,
                        VehicleCount::from(vehicle_upper_limit),
                        allowed_vehicle_types.clone(),
                    )
                })
                .collect()
        }
        Some(depots) => depots
            .iter()
            .enumerate()
            .map(|(idx, depot)| {
                let idx = DepotIdx::from(idx as Idx);
                let location = loc.get(location_lookup[&depot.location]).unwrap();
                let capacity: VehicleCount = depot.capacity as VehicleCount;
                let mut allowed_types: HashMap<VehicleTypeIdx, Option<VehicleCount>> =
                    HashMap::new();
                for allowed_type in &depot.allowed_types {
                    allowed_types.insert(
                        vehicle_type_lookup[&allowed_type.vehicle_type],
                        allowed_type.capacity.map(|x| x as VehicleCount),
                    );
                }
                ModelDepot::new(
                    idx,
                    depot.id.clone(),
                    location,
                    capacity,
                    allowed_types.clone(),
                )
            })
            .collect(),
    }
}

fn create_service_trips(
    json_input: &JsonInput,
    locations: &Locations,
    vehicle_types: &VehicleTypes,
    location_lookup: &HashMap<IdType, LocationIdx>,
    vehicle_type_lookup: &HashMap<IdType, VehicleTypeIdx>,
) -> HashMap<VehicleTypeIdx, Vec<ModelServiceTrip>> {
    let mut service_trips: HashMap<VehicleTypeIdx, Vec<ModelServiceTrip>> = HashMap::new();
    for vehicle_type in vehicle_types.iter() {
        service_trips.insert(vehicle_type, Vec::new());
    }

    let mut warnings_printed = false;

    for departure in json_input.departures.iter() {
        let route = json_input
            .routes
            .iter()
            .find(|route| route.id == departure.route)
            .unwrap();
        let vehicle_type = vehicle_type_lookup[&route.vehicle_type];

        for departure_segment in departure.segments.iter() {
            let route_segment = &route
                .segments
                .iter()
                .find(|segment| segment.id == departure_segment.route_segment)
                .unwrap();
            let id = departure_segment.id.clone();
            let origin = locations
                .get(location_lookup[&route_segment.origin])
                .unwrap();
            let destination = locations
                .get(location_lookup[&route_segment.destination])
                .unwrap();
            let departure_time = DateTime::new(&departure_segment.departure);
            let arrival_time = departure_time + Duration::from_seconds(route_segment.duration);
            let distance = Distance::from_meter(route_segment.distance as Meter);
            let mut passengers = departure_segment.passengers as PassengerCount;
            let seated = departure_segment.seated as PassengerCount;

            if passengers == 0 {
                passengers = 1;
                if !warnings_printed {
                    println!(
                    "\x1b[93mwarning:\x1b[0m Some service trips have no passengers. Setting passengers to 1, so that at least one vehicle is needed.",
                );
                    warnings_printed = true;
                }
            }

            let maximal_formation_count = route_segment
                .maximal_formation_count
                .map(|x| x as VehicleCount);

            let service_trip = Node::create_service_trip(
                id,
                vehicle_type,
                origin,
                destination,
                departure_time,
                arrival_time,
                distance,
                passengers,
                seated,
                maximal_formation_count,
            );
            service_trips
                .get_mut(&vehicle_type)
                .unwrap()
                .push(service_trip);
        }
    }
    service_trips
}

fn create_maintenance_slots(
    json_input: &JsonInput,
    locations: &Locations,
    location_lookup: &HashMap<IdType, LocationIdx>,
) -> Vec<ModelMaintenanceSlot> {
    match &json_input.maintenance_slots {
        None => Vec::new(),
        Some(maintenance_slots) => maintenance_slots
            .iter()
            .map(|maintenance_slot| {
                let location = locations
                    .get(location_lookup[&maintenance_slot.location])
                    .unwrap();
                let start = DateTime::new(&maintenance_slot.start);
                let end = DateTime::new(&maintenance_slot.end);
                let id = maintenance_slot.id.clone();

                Node::create_maintenance(
                    id,
                    location,
                    start,
                    end,
                    maintenance_slot.track_count as VehicleCount,
                )
            })
            .collect(),
    }
}
