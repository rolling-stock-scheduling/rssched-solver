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

use itertools::Itertools;
use model::{
    base_types::{DepotIdx, NodeIdx, VehicleIdx, VehicleTypeIdx},
    network::{nodes::Node, Network},
};
use rapid_time::DateTime;
use serde::{Deserialize, Serialize};

use crate::Schedule;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ScheduleJson {
    depot_loads: Vec<DepotLoad>,
    fleet: Vec<JsonFleet>,
    departure_segments: Vec<JsonDepartureSegmentWithFormation>,
    maintenance_slots: Vec<JsonFleetMaintenanceSlotWithFormation>,
    dead_head_trips: Vec<JsonFleetDeadHeadTripWithFormation>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DepotLoad {
    depot: String,
    load: Vec<Load>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Load {
    vehicle_type: String,
    spawn_count: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonFleet {
    vehicle_type: String,
    vehicles: Vec<JsonVehicle>,
    vehicle_cycles: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonVehicle {
    id: String,
    start_depot: String,
    end_depot: String,
    departure_segments: Vec<JsonFleetDepartureSegment>,
    maintenance_slots: Vec<JsonFleetMaintenanceSlot>,
    dead_head_trips: Vec<JsonFleetDeadHeadTrip>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonFleetDepartureSegment {
    departure_segment: String,
    origin: String,
    destination: String,
    departure: String,
    arrival: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonFleetMaintenanceSlot {
    maintenance_slot: String,
    location: String,
    start: String,
    end: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonFleetDeadHeadTrip {
    id: String,
    origin: String,
    destination: String,
    departure: String,
    arrival: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonDepartureSegmentWithFormation {
    departure_segment: String,
    origin: String,
    destination: String,
    departure: String,
    arrival: String,
    vehicle_type: String,
    formation: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonFleetMaintenanceSlotWithFormation {
    maintenance_slot: String,
    location: String,
    start: String,
    end: String,
    formation: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonFleetDeadHeadTripWithFormation {
    id: String,
    origin: String,
    destination: String,
    departure: String,
    arrival: String,
    formation: Vec<String>,
}

pub fn schedule_to_json(schedule: &Schedule) -> serde_json::Value {
    let mut dead_head_trips: Vec<JsonFleetDeadHeadTripWithFormation> = vec![];
    let mut fleet = vec![];
    for vehicle_type in schedule.get_network().vehicle_types().iter() {
        fleet.push(fleet_to_json(schedule, vehicle_type, &mut dead_head_trips));
    }
    let schedule_json = ScheduleJson {
        depot_loads: depots_usage_to_json(schedule),
        fleet,
        departure_segments: departure_segments_to_json(schedule),
        maintenance_slots: maintenance_slots_to_json(schedule),
        dead_head_trips,
    };
    serde_json::to_value(schedule_json).unwrap()
}

fn depots_usage_to_json(schedule: &Schedule) -> Vec<DepotLoad> {
    let mut depot_loads = vec![];
    let network = schedule.get_network();
    for depot_idx in network.depots_iter() {
        let depot = network.get_depot(depot_idx);
        depot_loads.push(DepotLoad {
            depot: depot.id().to_string(),
            load: depot_usage_to_json(schedule, depot_idx),
        });
    }
    depot_loads
}

fn depot_usage_to_json(schedule: &Schedule, depot_idx: DepotIdx) -> Vec<Load> {
    let mut loads = vec![];
    let network = schedule.get_network();
    for vehicle_type in schedule.get_vehicle_types().iter() {
        let spawn_count =
            schedule.number_of_vehicles_of_same_type_spawned_at(depot_idx, vehicle_type);
        if spawn_count > 0 {
            loads.push(Load {
                vehicle_type: network
                    .vehicle_types()
                    .get(vehicle_type)
                    .unwrap()
                    .id()
                    .clone(),
                spawn_count,
            })
        }
    }
    loads
}

fn fleet_to_json(
    schedule: &Schedule,
    vehicle_type: VehicleTypeIdx,
    dead_head_trips_with_formation: &mut Vec<JsonFleetDeadHeadTripWithFormation>,
) -> JsonFleet {
    let mut vehicles = vec![];
    for vehicle_idx in schedule.vehicles_iter(vehicle_type) {
        vehicles.push(vehicle_to_json(
            schedule,
            vehicle_idx,
            dead_head_trips_with_formation,
        ));
    }
    let mut vehicle_cycles = vec![];
    for transtion_cylce in schedule.next_day_transition_of(vehicle_type).cycles_iter() {
        vehicle_cycles.push(
            transtion_cylce
                .iter()
                .map(|vehicle_id| vehicle_id.to_string())
                .collect(),
        );
    }
    JsonFleet {
        vehicle_type: schedule
            .get_network()
            .vehicle_types()
            .get(vehicle_type)
            .unwrap()
            .id()
            .clone(),
        vehicles,
        vehicle_cycles,
    }
}

fn vehicle_to_json(
    schedule: &Schedule,
    vehicle_idx: VehicleIdx,
    dead_head_trips_with_formation: &mut Vec<JsonFleetDeadHeadTripWithFormation>,
) -> JsonVehicle {
    let network = schedule.get_network();
    let start_depot_node = schedule.tour_of(vehicle_idx).unwrap().first_node();
    let start_depot_id = network.get_depot_idx(start_depot_node);
    let start_depot = network.get_depot(start_depot_id);
    let end_depot_node = schedule.tour_of(vehicle_idx).unwrap().last_node();
    let end_depot_id = network.get_depot_idx(end_depot_node);
    let end_depot = network.get_depot(end_depot_id);
    let mut departure_segments = vec![];
    let mut maintenance_slots = vec![];
    let mut dead_head_trips = vec![];
    let mut dead_head_trips_counter = 0;
    for (node1_idx, node2_idx) in schedule
        .tour_of(vehicle_idx)
        .unwrap()
        .all_nodes_iter()
        .tuple_windows()
    {
        let node1 = network.node(node1_idx);
        let node2 = network.node(node2_idx);
        if network.node(node1_idx).end_location() != node2.start_location() {
            let (departure_time, arrival_time) =
                schedule_dead_head_trip(node1_idx, node2_idx, &network);
            let dead_head_trip = JsonFleetDeadHeadTrip {
                id: "dht_".to_string() + &dead_head_trips_counter.to_string(),
                origin: network.locations().get_id(node1.end_location()).unwrap(),
                destination: network.locations().get_id(node2.start_location()).unwrap(),
                departure: departure_time.as_iso(),
                arrival: arrival_time.as_iso(),
            };
            let dead_head_trip_with_formation = JsonFleetDeadHeadTripWithFormation {
                id: dead_head_trip.id.clone(),
                origin: dead_head_trip.origin.clone(),
                destination: dead_head_trip.destination.clone(),
                departure: dead_head_trip.departure.clone(),
                arrival: dead_head_trip.arrival.clone(),
                formation: vec![vehicle_idx.to_string()],
            };
            dead_head_trips_counter += 1;
            dead_head_trips.push(dead_head_trip);
            dead_head_trips_with_formation.push(dead_head_trip_with_formation);
        }
        match node2 {
            Node::Service((_, s)) => {
                let departure_segment = JsonFleetDepartureSegment {
                    departure_segment: s.id().to_string(),
                    origin: network.locations().get_id(node2.start_location()).unwrap(),
                    destination: network.locations().get_id(node2.end_location()).unwrap(),
                    departure: node2.start_time().as_iso(),
                    arrival: node2.end_time().as_iso(),
                };
                departure_segments.push(departure_segment);
            }
            Node::Maintenance((_, m)) => {
                let maintenance_slot = JsonFleetMaintenanceSlot {
                    maintenance_slot: m.id().clone(),
                    location: network.locations().get_id(node2.start_location()).unwrap(),
                    start: node2.start_time().as_iso(),
                    end: node2.end_time().as_iso(),
                };
                maintenance_slots.push(maintenance_slot);
            }
            _ => {}
        }
    }
    JsonVehicle {
        id: vehicle_idx.to_string(),
        start_depot: start_depot.id().to_string(),
        end_depot: end_depot.id().to_string(),
        departure_segments,
        maintenance_slots,
        dead_head_trips,
    }
}

fn departure_segments_to_json(schedule: &Schedule) -> Vec<JsonDepartureSegmentWithFormation> {
    let network = schedule.get_network();
    let mut departure_segments = vec![];
    for vehicle_type in network.vehicle_types().iter() {
        for service_trip_node_idx in network.service_nodes(vehicle_type) {
            let service_trip_node = network.node(service_trip_node_idx);
            let service_trip = service_trip_node.as_service_trip();

            let formation = schedule.train_formation_of(service_trip_node_idx);
            let departure_segment = JsonDepartureSegmentWithFormation {
                departure_segment: service_trip.id().to_string(),
                origin: network
                    .locations()
                    .get_id(service_trip_node.start_location())
                    .unwrap(),
                destination: network
                    .locations()
                    .get_id(service_trip_node.end_location())
                    .unwrap(),
                departure: service_trip_node.start_time().as_iso(),
                arrival: service_trip_node.end_time().as_iso(),
                vehicle_type: network
                    .vehicle_types()
                    .get(vehicle_type)
                    .unwrap()
                    .id()
                    .clone(),
                formation: formation
                    .iter()
                    .map(|vehicle| vehicle.idx().to_string())
                    .collect(),
            };
            departure_segments.push(departure_segment);
        }
    }
    departure_segments
}

fn maintenance_slots_to_json(schedule: &Schedule) -> Vec<JsonFleetMaintenanceSlotWithFormation> {
    let network = schedule.get_network();
    let mut maintenance_slots = vec![];
    for maintenance_node_idx in network.maintenance_nodes() {
        let maintenance_node = network.node(maintenance_node_idx);
        let maintenance_slot = maintenance_node.as_maintenance_slot();
        let formation = schedule.train_formation_of(maintenance_node_idx);
        let maintenance_slot = JsonFleetMaintenanceSlotWithFormation {
            maintenance_slot: maintenance_slot.id().clone(),
            location: network
                .locations()
                .get_id(maintenance_node.start_location())
                .unwrap(),
            start: maintenance_node.start_time().as_iso(),
            end: maintenance_node.end_time().as_iso(),
            formation: formation
                .iter()
                .map(|vehicle| vehicle.idx().to_string())
                .collect(),
        };
        maintenance_slots.push(maintenance_slot);
    }
    maintenance_slots
}

fn schedule_dead_head_trip(
    node1_idx: NodeIdx,
    node2_idx: NodeIdx,
    nw: &Network,
) -> (DateTime, DateTime) {
    let node1 = nw.node(node1_idx);
    let node2 = nw.node(node2_idx);
    if node1.is_depot() {
        let departure_time =
            node2.start_time() - nw.minimal_duration_between_nodes(node1_idx, node2_idx);
        let arrival_time = node2.start_time();
        return (departure_time, arrival_time);
    }
    let departure_time = node1.end_time();
    let arrival_time = node1.end_time() + nw.minimal_duration_between_nodes(node1_idx, node2_idx);
    (departure_time, arrival_time)
}
