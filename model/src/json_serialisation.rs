#[cfg(test)]
#[path = "json_serialisation_tests.rs"]
mod json_serialisation_tests;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use time::{DateTime, Duration};

use crate::base_types::{
    DepotId, Distance, Id, LocationId, Meter, NodeId, PassengerCount, VehicleCount, VehicleTypeId,
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

type Integer = u64;

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
    id: Integer,
    name: Option<String>,
    seats: Integer,
    capacity: Integer,
    maximal_formation_count: Option<Integer>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Location {
    id: Integer,
    name: Option<String>,
    day_limit: Option<Integer>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Depot {
    id: Integer,
    location: Integer,
    capacity: Integer,
    allowed_types: Vec<TypeCapacities>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TypeCapacities {
    vehicle_type: Integer,
    capacity: Option<Integer>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Route {
    id: Integer,
    name: Option<String>,
    vehicle_type: Integer,
    segments: Vec<RouteSegment>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RouteSegment {
    order: Integer,
    origin: Integer,
    destination: Integer,
    distance: Integer,
    duration: Integer,
    maximal_formation_count: Option<Integer>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Departures {
    id: Integer,
    route: Integer,
    name: Option<String>,
    segments: Vec<DepartureSegment>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DepartureSegment {
    order: Integer,
    departure: String,
    passengers: Integer,
    seated: Integer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct MaintenanceSlots {
    id: Integer,
    name: Option<String>,
    location: Integer,
    start: String,
    end: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeadHeadTrips {
    indices: Vec<Integer>,
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
    service_trip_first_vehicle_cost: Integer,
    service_trip_additional_vehicle_cost: Integer,
    dead_head_trip_cost: Integer,
    idle_cost: Integer,
}

pub fn load_rolling_stock_problem_instance_from_json(
    input_data: serde_json::Value,
) -> (Arc<VehicleTypes>, Arc<Network>, Arc<Config>) {
    let json_input = serde_json::from_value(input_data).unwrap();
    let locations = Arc::new(create_locations(&json_input));
    let vehicle_types = Arc::new(create_vehicle_types(&json_input));
    let config = Arc::new(create_config(&json_input));
    let network = Arc::new(create_network(
        &json_input,
        locations.clone(),
        vehicle_types.clone(),
        config.clone(),
    ));
    (vehicle_types, network, config)
}

fn create_locations(json_input: &JsonInput) -> Locations {
    let mut stations: HashMap<LocationId, (String, Option<VehicleCount>)> = HashMap::new();
    let mut dead_head_trips: HashMap<LocationId, HashMap<LocationId, DeadHeadTrip>> =
        HashMap::new();

    // add stations
    for location in &json_input.locations {
        stations.insert(
            LocationId::from(location.id as Id),
            (
                location
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("station_{}", location.id)),
                location.day_limit.map(|x| x as VehicleCount),
            ),
        );
    }

    // add dead head trips
    for (i, &origin) in json_input.dead_head_trips.indices.iter().enumerate() {
        let origin_station = LocationId::from(origin as Id);
        let mut destination_map: HashMap<LocationId, DeadHeadTrip> = HashMap::new();
        for (j, &destination) in json_input.dead_head_trips.indices.iter().enumerate() {
            destination_map.insert(
                LocationId::from(destination as Id),
                DeadHeadTrip::new(
                    Distance::from_meter(json_input.dead_head_trips.distances[i][j]),
                    Duration::from_seconds(json_input.dead_head_trips.durations[i][j]),
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
        .map(|vehicle_type| {
            ModelVehicleType::new(
                VehicleTypeId::from(vehicle_type.id as Id),
                vehicle_type
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("vehicle_{}", vehicle_type.id)),
                vehicle_type.seats as PassengerCount,
                vehicle_type.capacity as PassengerCount,
                vehicle_type
                    .maximal_formation_count
                    .map(|x| x as VehicleCount),
            )
        })
        .collect();

    VehicleTypes::new(vehicle_types)
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
        json_input.parameters.costs.service_trip_first_vehicle_cost,
        json_input
            .parameters
            .costs
            .service_trip_additional_vehicle_cost,
        json_input.parameters.costs.dead_head_trip_cost,
        json_input.parameters.costs.idle_cost,
    )
}

fn create_network(
    json_input: &JsonInput,
    locations: Arc<Locations>,
    vehicle_types: Arc<VehicleTypes>,
    config: Arc<Config>,
) -> Network {
    let depots = create_depots(json_input, &locations);
    let service_trips = create_service_trips(json_input, &locations, &vehicle_types);
    let maintenance_slots = create_maintenance_slots(json_input, &locations);
    Network::new(
        depots,
        service_trips,
        maintenance_slots,
        config,
        locations,
        vehicle_types,
    )
}

fn create_depots(json_input: &JsonInput, loc: &Locations) -> Vec<ModelDepot> {
    json_input
        .depots
        .iter()
        .map(|depot| {
            let id = DepotId::from(depot.id as Id);
            let location = loc
                .get_location(LocationId::from(depot.location as Id))
                .unwrap();
            let capacity: VehicleCount = depot.capacity as VehicleCount;
            let mut allowed_types: HashMap<VehicleTypeId, Option<VehicleCount>> = HashMap::new();
            for allowed_type in &depot.allowed_types {
                allowed_types.insert(
                    VehicleTypeId::from(allowed_type.vehicle_type as Id),
                    allowed_type.capacity.map(|x| x as VehicleCount),
                );
            }
            ModelDepot::new(id, location, capacity, allowed_types.clone())
        })
        .collect()
}

fn create_service_trips(
    json_input: &JsonInput,
    locations: &Locations,
    vehicle_types: &VehicleTypes,
) -> HashMap<VehicleTypeId, Vec<ModelServiceTrip>> {
    let mut service_trips: HashMap<VehicleTypeId, Vec<ModelServiceTrip>> = HashMap::new();
    for vehicle_type in vehicle_types.iter() {
        service_trips.insert(vehicle_type, Vec::new());
    }

    for departure in json_input.departures.iter() {
        let route = json_input
            .routes
            .iter()
            .find(|route| route.id == departure.route)
            .unwrap();
        let vehicle_type = VehicleTypeId::from(route.vehicle_type as Id);
        let name = departure
            .name
            .clone()
            .unwrap_or_else(|| format!("service_trip_{}", departure.id));

        let segment_count = departure.segments.len() as Integer;

        for order in 0..segment_count {
            let route_segment = &route
                .segments
                .iter()
                .find(|segment| segment.order == order)
                .unwrap();
            let departure_segment = &departure
                .segments
                .iter()
                .find(|segment| segment.order == order)
                .unwrap();
            let origin = locations
                .get_location(LocationId::from(route_segment.origin as Id))
                .unwrap();
            let destination = locations
                .get_location(LocationId::from(route_segment.destination as Id))
                .unwrap();
            let distance = Distance::from_meter(route_segment.distance as Meter);
            let departure_time = DateTime::new(&departure_segment.departure);
            let arrival_time = departure_time + Duration::from_seconds(route_segment.duration);
            let passengers = departure_segment.passengers as PassengerCount;
            let seated = departure_segment.seated as PassengerCount;
            let name = if segment_count == 1 {
                name.clone()
            } else {
                format!("{}-{}", name, order)
            };

            let service_trip = Node::create_service_trip(
                NodeId::service_from(departure.id as Id, order as u8),
                vehicle_type,
                origin,
                destination,
                departure_time,
                arrival_time,
                distance,
                passengers,
                seated,
                name,
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
) -> Vec<ModelMaintenanceSlot> {
    json_input
        .maintenance_slots
        .iter()
        .map(|maintenance_slot| {
            let location = locations
                .get_location(LocationId::from(maintenance_slot.location as Id))
                .unwrap();
            let start = DateTime::new(&maintenance_slot.start);
            let end = DateTime::new(&maintenance_slot.end);
            let name = maintenance_slot
                .name
                .clone()
                .unwrap_or_else(|| format!("maintenance_{}", maintenance_slot.id));

            Node::create_maintenance(
                NodeId::maintenance_from(maintenance_slot.id as Id),
                location,
                start,
                end,
                name,
            )
        })
        .collect()
}
