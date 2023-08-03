use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;

use crate::base_types::{
    DateTime, DepotId, Distance, Duration, LocationId, Meter, NodeId, PassengerCount, StationSide,
    TrainLength, VehicleTypeId,
};
use crate::config::Config;
use crate::locations::{DeadHeadTrip, Locations};
use crate::network::nodes::Node;
use crate::network::Network;
use crate::vehicle_types::VehicleType as ModelVehicleType;
use crate::vehicle_types::VehicleTypes;

type Integer = u32;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct VehicleType {
    id: String,
    name: String,
    number_of_seats: Integer,
    capacity_of_passengers: Integer,
    vehicle_length_in_meter: Integer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Location {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct UpperBoundForVehicleTypes {
    unit_type: String,
    upper_bound: Integer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Depot {
    id: String,
    location: String,
    upper_bound_for_unit_types: Vec<UpperBoundForVehicleTypes>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Route {
    id: String,
    line: String,
    origin: String,
    destination: String,
    travel_distance_in_meter: Integer,
    travel_duration_in_seconds: Integer,
    maximal_formation_length_in_meter: Option<Integer>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ServiceTrip {
    id: String,
    route: String,
    name: String,
    departure_time: String,
    passenger_demand: Integer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeadHeadTrips {
    indices: Vec<String>,
    travel_time_durations_in_seconds: Vec<Vec<Integer>>,
    distances_in_meter: Vec<Vec<Integer>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ShuntingDurationsInSeconds {
    minimal_duration: Integer,
    dead_head_trip_duration: Integer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Parameters {
    shunting_durations_in_seconds: ShuntingDurationsInSeconds,
    default_maximal_formation_length_in_meter: Integer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonInput {
    unit_types: Vec<VehicleType>,
    locations: Vec<Location>,
    depots: Vec<Depot>,
    routes: Vec<Route>,
    service_trips: Vec<ServiceTrip>,
    dead_head_trips: DeadHeadTrips,
    parameters: Parameters,
}

pub fn load_rolling_stock_problem_instance_from_json(
    path: &str,
) -> (Arc<Locations>, VehicleTypes, Network, Arc<Config>) {
    let json_input = load_json_input(path);
    let locations = Arc::new(create_locations(&json_input));
    let vehicle_types = create_vehicle_types(&json_input);
    let config = Arc::new(create_config(&json_input));
    let network = create_network(
        &json_input,
        locations.clone(),
        &vehicle_types,
        config.clone(),
    );
    (locations, vehicle_types, network, config)
}

fn load_json_input(path: &str) -> JsonInput {
    let mut file = File::open(path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    serde_json::from_str(&data).unwrap()
}

fn create_locations(json_input: &JsonInput) -> Locations {
    let mut stations: HashSet<LocationId> = HashSet::new();
    let mut dead_head_trips: HashMap<LocationId, HashMap<LocationId, DeadHeadTrip>> =
        HashMap::new();

    // add stations
    for location in &json_input.locations {
        stations.insert(LocationId::from(&location.id));
    }

    // add dead head trips
    for (i, origin) in json_input.dead_head_trips.indices.iter().enumerate() {
        let origin_station = LocationId::from(&origin);
        let mut destination_map: HashMap<LocationId, DeadHeadTrip> = HashMap::new();
        for (j, destination) in json_input.dead_head_trips.indices.iter().enumerate() {
            destination_map.insert(
                LocationId::from(&destination),
                DeadHeadTrip::new(
                    Distance::from_meter(
                        json_input.dead_head_trips.distances_in_meter[i][j] as u64,
                    ),
                    Duration::from_seconds(
                        json_input.dead_head_trips.travel_time_durations_in_seconds[i][j],
                    ),
                    StationSide::Back,  // TODO: Read this from json
                    StationSide::Front, // TODO: Read this from json
                ),
            );
        }
        dead_head_trips.insert(origin_station, destination_map);
    }

    Locations::new(stations, dead_head_trips)
}

fn create_vehicle_types(json_input: &JsonInput) -> VehicleTypes {
    let mut vehicle_types: Vec<ModelVehicleType> = json_input
        .unit_types
        .iter()
        .map(|unit_type| {
            ModelVehicleType::new(
                VehicleTypeId::from(&unit_type.id),
                unit_type.name.clone(),
                unit_type.number_of_seats as PassengerCount,
                unit_type.capacity_of_passengers as PassengerCount,
                unit_type.vehicle_length_in_meter as TrainLength,
            )
        })
        .collect();

    VehicleTypes::new(vehicle_types)
}

fn create_network(
    json_input: &JsonInput,
    locations: Arc<Locations>,
    vehicle_types: &VehicleTypes,
    config: Arc<Config>,
) -> Network {
    let mut nodes: Vec<Node> = create_depot_nodes(json_input, &locations, vehicle_types);
    nodes.append(&mut create_service_trip_nodes(json_input, &locations));
    //TODO: add maintenance nodes
    Network::new(nodes, config, locations)
}

fn create_depot_nodes(
    json_input: &JsonInput,
    loc: &Locations,
    vehicle_types: &VehicleTypes,
) -> Vec<Node> {
    json_input
        .depots
        .iter()
        .map(|depot| {
            let id = DepotId::from(&depot.id);
            let location = loc.get_location(LocationId::from(&depot.location));
            depot
                .upper_bound_for_unit_types
                .iter()
                .map(|unit_type_bound| {
                    let vehicle_id = VehicleTypeId::from(&unit_type_bound.unit_type);
                    assert!(vehicle_types.get(&vehicle_id).is_some());
                    Node::create_depot_node(
                        id,
                        location,
                        vehicle_id,
                        Some(unit_type_bound.upper_bound),
                    )
                })
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect()
}

fn create_service_trip_nodes(json_input: &JsonInput, locations: &Locations) -> Vec<Node> {
    json_input
        .service_trips
        .iter()
        .map(|service_trip| {
            let route = json_input
                .routes
                .iter()
                .find(|route| route.id == service_trip.route)
                .unwrap();
            let departure = DateTime::new(&service_trip.departure_time);
            Node::create_service_node(
                NodeId::from(&service_trip.id),
                locations.get_location(LocationId::from(&route.origin)),
                locations.get_location(LocationId::from(&route.destination)),
                departure,
                departure + Duration::from_seconds(route.travel_duration_in_seconds),
                StationSide::Front, // TODO: Read this from json
                StationSide::Front, // TODO: Read this from json
                Distance::from_meter(route.travel_distance_in_meter as Meter),
                service_trip.passenger_demand as PassengerCount,
                service_trip.name,
            )
        })
        .collect()
}

//TODO create config from JsonInput
//TODO create static function for writing schedule to json
