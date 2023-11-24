use itertools::Itertools;
use sbb_model::network::nodes::Node;
//TODO create static function for writing schedule to json
use serde::{Deserialize, Serialize};

use crate::Schedule;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonOutput {
    schedule: Vec<JsonTour>,
    // vehicle_demand: Vec<JsonVehicleDemand>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonTour {
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
        // line_id: String,
        // route_id: String,
    },
    Maintenance {
        location: String,
        start_time: String,
        end_time: String,
    },
}

pub fn write_schedule_to_json(schedule: &Schedule, path: &str) -> Result<(), std::io::Error> {
    let nw = schedule.get_network();
    let mut json_output = JsonOutput {
        schedule: Vec::new(),
    };

    for vehicle_id in schedule.vehicles_iter() {
        let vehicle_type_id = schedule.get_vehicle(vehicle_id).unwrap().type_id();
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
                let dead_head_trip = JsonTourStop::DeadHeadTrip {
                    origin: nw.node(node1_id).end_location().to_string(),
                    destination: nw.node(node2_id).start_location().to_string(),
                    departure_time: nw.node(node1_id).end_time().as_iso(),
                    arrival_time: nw.node(node2_id).start_time().as_iso(),
                };
                tour.push(dead_head_trip);
            }
            let node2 = nw.node(node2_id);
            match node2 {
                Node::Service(_) => {
                    let service_trip = JsonTourStop::ServiceTrip {
                        id: node2.id().to_string(),
                    };
                    tour.push(service_trip);
                }
                Node::Maintenance(_) => {
                    let maintenance_trip = JsonTourStop::Maintenance {
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
            vehicle_type: vehicle_type_id.to_string(),
            start_depot: start_depot.to_string(),
            end_depot: end_depot.to_string(),
            tour,
        };
        json_output.schedule.push(json_tour);
    }

    let json = serde_json::to_string(&json_output).unwrap();
    println!("{}", json);
    std::fs::write(path, json)
}

// TODO only serialization needed not deserialization
// TODO include line_id and route_id in json if needed
// TODO add vehicle_demand to json
// TODO add tests
