use itertools::Itertools;
use model::{
    base_types::{DepotId, NodeId, VehicleId},
    network::{nodes::Node, Network},
};
use serde::{Deserialize, Serialize};
use time::DateTime;

use crate::Schedule;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ScheduleJson {
    depot_loads: Vec<DepotLoad>,
    tours: Vec<JsonTour>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonTour {
    vehicle_id: String,
    vehicle_type: String,
    start_depot: String,
    end_depot: String,
    tour: Vec<JsonTourStop>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
enum JsonTourStop {
    DeadHeadTrip {
        origin: String,
        destination: String,
        departure_time: String,
        arrival_time: String,
    },
    ServiceTrip {
        id: String,
        origin: String,
        destination: String,
        departure_time: String,
        arrival_time: String,
        // line_id: String,
        // route_id: String,
    },
    Maintenance {
        id: String,
        location: String,
        start_time: String,
        end_time: String,
    },
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

pub fn schedule_to_json(schedule: &Schedule) -> serde_json::Value {
    let schedule_json = ScheduleJson {
        tours: tours_to_json(schedule),
        depot_loads: depots_usage_to_json(schedule),
    };
    serde_json::to_value(schedule_json).unwrap()
}

fn depots_usage_to_json(schedule: &Schedule) -> Vec<DepotLoad> {
    let mut depot_loads = vec![];
    for depot_id in schedule.get_network().depots_iter() {
        depot_loads.push(DepotLoad {
            depot: depot_id.to_string(),
            load: depot_usage_to_json(schedule, depot_id),
        });
    }
    depot_loads
}

fn depot_usage_to_json(schedule: &Schedule, depot_id: DepotId) -> Vec<Load> {
    let mut loads = vec![];
    for vehicle_type_id in schedule.get_vehicle_types().iter() {
        let spawn_count =
            schedule.number_of_vehicles_of_same_type_spawned_at(depot_id, vehicle_type_id);
        if spawn_count > 0 {
            loads.push(Load {
                vehicle_type: vehicle_type_id.to_string(),
                spawn_count,
            })
        }
    }
    loads
}

fn tours_to_json(schedule: &Schedule) -> Vec<JsonTour> {
    let mut tours = vec![];

    for vehicle_id in schedule.vehicles_iter() {
        tours.push(tour_to_json(schedule, vehicle_id));
    }
    tours
}

fn tour_to_json(schedule: &Schedule, vehicle_id: VehicleId) -> JsonTour {
    let nw = schedule.get_network();
    let vehicle_type_id = schedule.vehicle_type_of(vehicle_id);
    let start_depot = schedule.tour_of(vehicle_id).unwrap().first_node();
    let end_depot = schedule.tour_of(vehicle_id).unwrap().last_node();
    let mut tour = Vec::new();
    for (node1_id, node2_id) in schedule
        .tour_of(vehicle_id)
        .unwrap()
        .all_nodes_iter()
        .tuple_windows()
    {
        if nw.node(node1_id).end_location() != nw.node(node2_id).start_location() {
            let (departure_time, arrival_time) = schedule_dead_head_trip(node1_id, node2_id, nw);
            let dead_head_trip = JsonTourStop::DeadHeadTrip {
                origin: nw.node(node1_id).end_location().to_string(),
                destination: nw.node(node2_id).start_location().to_string(),
                departure_time: departure_time.as_iso(),
                arrival_time: arrival_time.as_iso(),
            };
            tour.push(dead_head_trip);
        }
        let node2 = nw.node(node2_id);
        match node2 {
            Node::Service(_) => {
                let service_trip = JsonTourStop::ServiceTrip {
                    id: node2.id().to_string(),
                    origin: node2.start_location().to_string(),
                    destination: node2.end_location().to_string(),
                    departure_time: node2.start_time().as_iso(),
                    arrival_time: node2.end_time().as_iso(),
                };
                tour.push(service_trip);
            }
            Node::Maintenance(_) => {
                let maintenance_trip = JsonTourStop::Maintenance {
                    id: node2.id().to_string(),
                    location: node2.start_location().to_string(),
                    start_time: node2.start_time().as_iso(),
                    end_time: node2.end_time().as_iso(),
                };
                tour.push(maintenance_trip);
            }
            _ => {}
        }
    }
    let json_tour = JsonTour {
        vehicle_id: vehicle_id.to_string(),
        vehicle_type: vehicle_type_id.to_string(),
        start_depot: nw.node(start_depot).as_depot().depot_id().to_string(),
        end_depot: nw.node(end_depot).as_depot().depot_id().to_string(),
        tour,
    };
    json_tour
}

fn schedule_dead_head_trip(
    node1_id: NodeId,
    node2_id: NodeId,
    nw: &Network,
) -> (DateTime, DateTime) {
    let node1 = nw.node(node1_id);
    let node2 = nw.node(node2_id);
    if node1.is_depot() {
        let departure_time =
            node2.start_time() - nw.minimal_duration_between_nodes(node1_id, node2_id);
        let arrival_time = node2.start_time();
        return (departure_time, arrival_time);
    }
    let departure_time = node1.end_time();
    let arrival_time = node1.end_time() + nw.minimal_duration_between_nodes(node1_id, node2_id);
    (departure_time, arrival_time)
}
