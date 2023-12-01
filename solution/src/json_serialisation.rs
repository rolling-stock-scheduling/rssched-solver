use itertools::Itertools;
use objective_framework::{EvaluatedSolution, Objective};
use sbb_model::{
    base_types::NodeId,
    network::{nodes::Node, Network},
};
//TODO create static function for writing schedule to json
use serde::{Deserialize, Serialize};
use time::DateTime;

use crate::Schedule;

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

pub fn write_solution_to_json(
    solution: &EvaluatedSolution<Schedule>,
    objective: &Objective<Schedule>,
    path: &str,
) -> Result<(), std::io::Error> {
    let json_output = schedule_to_json(solution.solution());
    let json_objective_value = objective.objective_value_to_json(solution.objective_value());
    let json_output = serde_json::json!({
        "objective_value": json_objective_value,
        "schedule": json_output,
    });
    let file = std::fs::File::create(path)?;
    serde_json::to_writer_pretty(file, &json_output)?;
    Ok(())
}

fn schedule_to_json(schedule: &Schedule) -> serde_json::Value {
    let nw = schedule.get_network();

    let mut json_output = vec![];

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
                let (departure_time, arrival_time) =
                    schedule_dead_head_trip(node1_id, node2_id, &nw);
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
            vehicle_type: vehicle_type_id.to_string(),
            start_depot: nw.node(start_depot).as_depot().depot_id().to_string(),
            end_depot: nw.node(end_depot).as_depot().depot_id().to_string(),
            tour,
        };
        json_output.push(json_tour);
    }
    serde_json::to_value(json_output).unwrap()
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
// TODO why does endDepot appeads before startDepot in json?
// TODO only serialization needed not deserialization
// TODO include line_id and route_id in json if needed
// TODO add vehicle_demand to json
// TODO add tests
