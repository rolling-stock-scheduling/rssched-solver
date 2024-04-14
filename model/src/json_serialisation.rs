#[cfg(test)]
#[path = "json_serialisation_tests.rs"]
mod json_serialisation_tests;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use time::{DateTime, Duration};

use crate::base_types::{
    DepotId, Distance, Idx, LocationId, Meter, NodeId, PassengerCount, VehicleCount, VehicleTypeId,
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
    depots: Vec<Depot>,
    routes: Vec<Route>,
    departures: Vec<Departures>,
    maintenance_slots: Vec<MaintenanceSlots>,
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
    capacity: Integer,
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
    maintenance: Maintenance,
    costs: Costs,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Shunting {
    minimal_duration: Integer,
    dead_head_trip_duration: Integer,
    coupling_duration: Integer,
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
    maintenance: Integer,
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

fn create_locations(json_input: &JsonInput) -> (Locations, HashMap<IdType, LocationId>) {
    let mut stations: HashMap<LocationId, (String, Option<VehicleCount>)> = HashMap::new(); // TODO: use vec instead
    let mut dead_head_trips: HashMap<LocationId, HashMap<LocationId, DeadHeadTrip>> =
        HashMap::new();

    let mut location_lookup: HashMap<IdType, LocationId> = HashMap::new();

    // add stations
    for (idx, location_json) in json_input.locations.iter().enumerate() {
        let location_idx = LocationId::from(idx as Idx);
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
    for (i, origin_json) in json_input.dead_head_trips.indices.iter().enumerate() {
        let origin_station = location_lookup[origin_json];
        let mut destination_map: HashMap<LocationId, DeadHeadTrip> = HashMap::new();
        for (j, destination_json) in json_input.dead_head_trips.indices.iter().enumerate() {
            destination_map.insert(
                location_lookup[destination_json],
                DeadHeadTrip::new(
                    Distance::from_meter(json_input.dead_head_trips.distances[i][j]),
                    Duration::from_seconds(json_input.dead_head_trips.durations[i][j]),
                ),
            );
        }
        dead_head_trips.insert(origin_station, destination_map);
    }

    (Locations::new(stations, dead_head_trips), location_lookup)
}

fn create_vehicle_types(json_input: &JsonInput) -> (VehicleTypes, HashMap<IdType, VehicleTypeId>) {
    let mut vehicle_type_lookup: HashMap<IdType, VehicleTypeId> = HashMap::new();
    let vehicle_types: Vec<ModelVehicleType> = json_input
        .vehicle_types
        .iter()
        .enumerate()
        .map(|(idx, vehicle_type)| {
            let vehicle_type_idx = VehicleTypeId::from(idx as Idx);
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
    Config::new(
        json_input
            .parameters
            .forbid_dead_head_trips
            .unwrap_or(false),
        Duration::from_seconds(json_input.parameters.day_limit_threshold.unwrap_or(0)),
        Duration::from_seconds(json_input.parameters.shunting.minimal_duration),
        Duration::from_seconds(json_input.parameters.shunting.dead_head_trip_duration),
        Duration::from_seconds(json_input.parameters.shunting.coupling_duration),
        Distance::from_meter(json_input.parameters.maintenance.maximal_distance),
        json_input.parameters.costs.staff,
        json_input.parameters.costs.service_trip,
        json_input.parameters.costs.maintenance,
        json_input.parameters.costs.dead_head_trip,
        json_input.parameters.costs.idle,
    )
}

fn create_network(
    json_input: &JsonInput,
    locations: Locations,
    vehicle_types: VehicleTypes,
    config: Config,
    location_lookup: HashMap<IdType, LocationId>,
    vehicle_type_lookup: HashMap<IdType, VehicleTypeId>,
) -> Network {
    let depots = create_depots(
        json_input,
        &locations,
        &location_lookup,
        &vehicle_type_lookup,
    );
    let service_trips = create_service_trips(
        json_input,
        &locations,
        &vehicle_types,
        &location_lookup,
        &vehicle_type_lookup,
    );
    let maintenance_slots = create_maintenance_slots(
        json_input,
        &locations,
        service_trips.len(),
        &location_lookup,
    );
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
    location_lookup: &HashMap<IdType, LocationId>,
    vehicle_type_lookup: &HashMap<IdType, VehicleTypeId>,
) -> Vec<ModelDepot> {
    json_input
        .depots
        .iter()
        .enumerate()
        .map(|(idx, depot)| {
            let idx = DepotId::from(idx as Idx);
            let location = loc.get_location(location_lookup[&depot.location]).unwrap();
            let capacity: VehicleCount = depot.capacity as VehicleCount;
            let mut allowed_types: HashMap<VehicleTypeId, Option<VehicleCount>> = HashMap::new();
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
        .collect()
}

fn create_service_trips(
    json_input: &JsonInput,
    locations: &Locations,
    vehicle_types: &VehicleTypes,
    location_lookup: &HashMap<IdType, LocationId>,
    vehicle_type_lookup: &HashMap<IdType, VehicleTypeId>,
) -> HashMap<VehicleTypeId, Vec<ModelServiceTrip>> {
    let mut service_trips: HashMap<VehicleTypeId, Vec<ModelServiceTrip>> = HashMap::new();
    for vehicle_type in vehicle_types.iter() {
        service_trips.insert(vehicle_type, Vec::new());
    }

    let mut idx_counter = 0;

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
            let origin = locations
                .get_location(location_lookup[&route_segment.origin])
                .unwrap();
            let destination = locations
                .get_location(location_lookup[&route_segment.destination])
                .unwrap();
            let distance = Distance::from_meter(route_segment.distance as Meter);
            let departure_time = DateTime::new(&departure_segment.departure);
            let arrival_time = departure_time + Duration::from_seconds(route_segment.duration);
            let passengers = departure_segment.passengers as PassengerCount;
            let seated = departure_segment.seated as PassengerCount;
            let id = departure_segment.id.clone();

            let service_trip = Node::create_service_trip(
                NodeId::service_from(idx_counter as Idx, 0), // TODO change to index only
                id,
                vehicle_type,
                origin,
                destination,
                departure_time,
                arrival_time,
                distance,
                passengers,
                seated,
            );
            idx_counter += 1;

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
    start_idx: usize,
    location_lookup: &HashMap<IdType, LocationId>,
) -> Vec<ModelMaintenanceSlot> {
    let mut idx_counter = start_idx;
    json_input
        .maintenance_slots
        .iter()
        .map(|maintenance_slot| {
            let location = locations
                .get_location(location_lookup[&maintenance_slot.location])
                .unwrap();
            let start = DateTime::new(&maintenance_slot.start);
            let end = DateTime::new(&maintenance_slot.end);
            let id = maintenance_slot.id.clone();

            let maintenance_node = Node::create_maintenance(
                NodeId::maintenance_from(idx_counter as Idx),
                id,
                location,
                start,
                end,
            );
            idx_counter += 1;
            maintenance_node
        })
        .collect()
}
