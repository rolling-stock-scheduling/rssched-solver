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
struct JsonInput {
    vehicle_types: Vec<VehicleType>,
    locations: Vec<Location>,
    depots: Vec<Depot>,
    routes: Vec<Route>,
    service_trips: Vec<ServiceTrip>,
    dead_head_trips: DeadHeadTrips,
    parameters: Parameters,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct VehicleType {
    id: String,
    name: String,
    seats: Integer,
    capacity: Integer,
    length: Integer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Location {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Depot {
    id: String,
    location: String,
    capacities: Vec<Capacities>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Capacities {
    vehicle_type: String,
    upper_bound: Integer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Route {
    id: String,
    line: String,
    origin: String,
    destination: String,
    distance: Integer,
    duration: Integer,
    maximal_formation_length: Option<Integer>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ServiceTrip {
    id: String,
    route: String,
    name: String,
    departure: String,
    passengers: Integer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeadHeadTrips {
    indices: Vec<String>,
    durations: Vec<Vec<Integer>>,
    distances: Vec<Vec<Integer>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Parameters {
    shunting: Shunting,
    defaults: Defaults,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Shunting {
    minimal_duration: Integer,
    dead_head_trip_duration: Integer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Defaults {
    maximal_formation_length: Integer,
}

pub fn load_rolling_stock_problem_instance_from_json(
    path: &str,
) -> (Arc<Locations>, Arc<VehicleTypes>, Arc<Network>, Arc<Config>) {
    let json_input = load_json_input(path);
    let locations = Arc::new(create_locations(&json_input));
    let vehicle_types = Arc::new(create_vehicle_types(&json_input));
    let config = Arc::new(create_config(&json_input));
    let network = Arc::new(create_network(
        &json_input,
        locations.clone(),
        &vehicle_types,
        config.clone(),
    ));
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
                    Distance::from_meter(json_input.dead_head_trips.distances[i][j] as u64),
                    Duration::from_seconds(json_input.dead_head_trips.durations[i][j]),
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
    let vehicle_types: Vec<ModelVehicleType> = json_input
        .vehicle_types
        .iter()
        .map(|unit_type| {
            ModelVehicleType::new(
                VehicleTypeId::from(&unit_type.id),
                unit_type.name.clone(),
                unit_type.seats as PassengerCount,
                unit_type.capacity as PassengerCount,
                unit_type.length as TrainLength,
            )
        })
        .collect();

    VehicleTypes::new(vehicle_types)
}

fn create_config(json_input: &JsonInput) -> Config {
    Config::new(
        Duration::from_seconds(json_input.parameters.shunting.minimal_duration),
        Duration::from_seconds(json_input.parameters.shunting.dead_head_trip_duration),
        Distance::from_meter(json_input.parameters.defaults.maximal_formation_length as u64),
    )
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
                .capacities
                .iter()
                .map(|vehicle_type_bound| {
                    let vehicle_id = VehicleTypeId::from(&vehicle_type_bound.vehicle_type);
                    assert!(vehicle_types.get(vehicle_id).is_some());
                    let vehicle_type = vehicle_types.get(vehicle_id).unwrap();
                    Node::create_depot_node(
                        id,
                        location,
                        vehicle_id,
                        Some(vehicle_type_bound.upper_bound),
                        String::from(format!("{}-{}", vehicle_type.name(), location)),
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
            let departure = DateTime::new(&service_trip.departure);
            Node::create_service_node(
                NodeId::from(&service_trip.id),
                locations.get_location(LocationId::from(&route.origin)),
                locations.get_location(LocationId::from(&route.destination)),
                departure,
                departure + Duration::from_seconds(route.duration),
                StationSide::Front, // TODO: Read this from json
                StationSide::Front, // TODO: Read this from json
                Distance::from_meter(route.distance as Meter),
                service_trip.passengers as PassengerCount,
                service_trip.name.clone(),
            )
        })
        .collect()
}

//TODO create static function for writing schedule to json

#[cfg(test)]
mod test {
    use crate::{
        base_types::{
            DateTime, DepotId, Distance, Duration, Location, LocationId, NodeId, StationSide,
            VehicleTypeId,
        },
        json_serialisation::load_rolling_stock_problem_instance_from_json,
        locations::Locations,
        network::nodes::Node,
        vehicle_types::VehicleType,
    };

    //add a test that reads a json file
    #[test]
    fn test_load_from_json() {
        let (locations, vehicle_types, network, config) =
            load_rolling_stock_problem_instance_from_json("resources/small_test_input.json");

        let loc1 = locations.get_location(LocationId::from("loc1"));
        let loc2 = locations.get_location(LocationId::from("loc2"));
        let loc3 = locations.get_location(LocationId::from("loc3"));

        assert_eq!(
            *vehicle_types.get(VehicleTypeId::from("vt1")).unwrap(),
            VehicleType::new(
                VehicleTypeId::from("vt1"),
                String::from("Vehicle Type 1"),
                50,
                100,
                10,
            )
        );
        assert_eq!(
            *vehicle_types.get(VehicleTypeId::from("vt2")).unwrap(),
            VehicleType::new(
                VehicleTypeId::from("vt2"),
                String::from("Vehicle Type 2"),
                40,
                80,
                8,
            )
        );

        assert_eq!(loc1, Location::of(LocationId::from("loc1")));
        assert_eq!(loc2, Location::of(LocationId::from("loc2")));
        assert_eq!(loc3, Location::of(LocationId::from("loc3")));

        assert_eq!(network.all_nodes().count(), 5);
        assert_eq!(
            *network.node(NodeId::from("depot1_vt1")),
            Node::create_depot_node(
                DepotId::from("depot1"),
                loc1,
                VehicleTypeId::from("vt1"),
                Some(7),
                String::from("Vehicle Type 1-loc1"),
            )
        );
        assert_eq!(
            *network.node(NodeId::from("depot1_vt2")),
            Node::create_depot_node(
                DepotId::from("depot1"),
                loc1,
                VehicleTypeId::from("vt2"),
                Some(5),
                String::from("Vehicle Type 2-loc1"),
            )
        );
        assert_eq!(
            *network.node(NodeId::from("depot2_vt2")),
            Node::create_depot_node(
                DepotId::from("depot2"),
                loc2,
                VehicleTypeId::from("vt2"),
                Some(8),
                String::from("Vehicle Type 2-loc2"),
            )
        );

        assert_eq!(
            *network.node(NodeId::from("trip1")),
            Node::create_service_node(
                NodeId::from("trip1"),
                loc1,
                loc2,
                DateTime::new("2023-07-24T11:59:41"),
                DateTime::new("2023-07-24T12:59:41"),
                StationSide::Front,
                StationSide::Front,
                Distance::from_meter(1000),
                50,
                String::from("Trip 1"),
            )
        );

        assert_eq!(
            *network.node(NodeId::from("trip2")),
            Node::create_service_node(
                NodeId::from("trip2"),
                loc2,
                loc3,
                DateTime::new("2023-07-24T11:59:41"),
                DateTime::new("2023-07-24T13:59:41"),
                StationSide::Front,
                StationSide::Front,
                Distance::from_meter(2000),
                80,
                String::from("Trip 2"),
            )
        );

        assert_travel_time(loc1, loc1, 0, &locations);
        assert_travel_time(loc1, loc2, 600, &locations);
        assert_travel_time(loc1, loc3, 300, &locations);
        assert_travel_time(loc2, loc1, 6000, &locations);
        assert_travel_time(loc2, loc2, 0, &locations);
        assert_travel_time(loc2, loc3, 400, &locations);
        assert_travel_time(loc3, loc1, 3000, &locations);
        assert_travel_time(loc3, loc2, 4000, &locations);
        assert_travel_time(loc3, loc3, 0, &locations);

        assert_travel_distance(loc1, loc1, 0, &locations);
        assert_travel_distance(loc1, loc2, 1000, &locations);
        assert_travel_distance(loc1, loc3, 500, &locations);
        assert_travel_distance(loc2, loc1, 10000, &locations);
        assert_travel_distance(loc2, loc2, 0, &locations);
        assert_travel_distance(loc2, loc3, 700, &locations);
        assert_travel_distance(loc3, loc1, 5000, &locations);
        assert_travel_distance(loc3, loc2, 7000, &locations);
        assert_travel_distance(loc3, loc3, 0, &locations);

        assert_eq!(
            config.durations_between_activities.minimal,
            Duration::from_seconds(600)
        );
        assert_eq!(
            config.durations_between_activities.dead_head_trip,
            Duration::from_seconds(300)
        );
        assert_eq!(
            config.default_maximal_formation_length,
            Distance::from_meter(20)
        );
    }

    fn assert_travel_time(from: Location, to: Location, expected: u32, locations: &Locations) {
        assert_eq!(
            locations.travel_time(from, to),
            Duration::from_seconds(expected),
            "Travel time from {} to {} should be {}",
            from,
            to,
            expected
        );
    }

    fn assert_travel_distance(from: Location, to: Location, expected: u64, locations: &Locations) {
        assert_eq!(
            locations.distance(from, to),
            Distance::from_meter(expected),
            "Travel distance from {} to {} should be {}",
            from,
            to,
            expected
        );
    }
}
