use crate::Solution;
use crate::Solver;
use model::base_types::NodeId;
use model::base_types::VehicleId;
use model::config::Config;
use model::network::Network;
use model::vehicle_types::VehicleTypes;
use objective_framework::Objective;
use rs_graph::linkedlistgraph::Node as RsNode;
use rs_graph::maxflow::pushrelabel;
use rs_graph::traits::Directed;
use rs_graph::Buildable;
use rs_graph::Builder;
use rs_graph::IndexGraph;
use rs_graph::LinkedListGraph;
use solution::path::Path;
use solution::Schedule;
use std::collections::HashMap;
use std::sync::Arc;

/// Solving the problem by finding a maximum-cardinally matching in a unweighted bipartit graph.
/// For each service trip, we create two nodes, one on the left and one on the right.
/// If trip A can reach trip B, we add an edge from the left node of A to the right node of B.
/// Via a maximum flow computation we find a maximum-cardinally matching.
/// A matching edge means that the corresponding service trip are assigned in succession to a vehicle.
/// As each matching edge reduces the number of vehicle by one, a maximum-cardinally matching
/// corresponds to a feasible solution with a minimum number of vehicles.
pub struct MaxMatchingSolver {
    vehicles: Arc<VehicleTypes>,
    network: Arc<Network>,
    config: Arc<Config>,
    objective: Arc<Objective<Schedule>>,
}

impl Solver for MaxMatchingSolver {
    fn initialize(
        vehicles: Arc<VehicleTypes>,
        network: Arc<Network>,
        config: Arc<Config>,
        objective: Arc<Objective<Schedule>>,
    ) -> Self {
        Self {
            vehicles,
            network,
            config,
            objective,
        }
    }

    fn solve(&self) -> Solution {
        // TODO decide on which vehicle type (biggest or best fitting)
        // for now: take biggest vehicles (good for a small count, as it might be reused for
        // later trips)
        let vehicle_type = self.vehicles.iter().last().unwrap();
        let seat_count = self.vehicles.get(vehicle_type).unwrap().seats();

        let mut builder = LinkedListGraph::<u32>::new_builder();
        // let mut builder = VecGraph::<u32>::new_builder();

        let mut left_node_to_trip: HashMap<RsNode, (NodeId, u8)> = HashMap::new();
        let mut trip_to_node: HashMap<(NodeId, u8), (RsNode, RsNode)> = HashMap::new();
        let source = builder.add_node();
        let sink = builder.add_node();
        let num_service_trips = self.network.service_nodes().count();
        for (counter, service_trip) in self.network.service_nodes().enumerate() {
            println!("Adding service trip {}/{}", counter + 1, num_service_trips);
            let demand = self.network.node(service_trip).as_service_trip().demand();
            for i in 0..demand.div_ceil(seat_count) as u8 {
                let left_node = builder.add_node();
                let right_node = builder.add_node();
                left_node_to_trip.insert(left_node, (service_trip, i));
                trip_to_node.insert((service_trip, i), (left_node, right_node));
                builder.add_edge(source, left_node);
                builder.add_edge(right_node, sink);
                self.network
                    .all_predecessors(service_trip)
                    .filter(|&pred| self.network.node(pred).is_service())
                    .for_each(|pred| {
                        let pred_demand = self.network.node(pred).as_service_trip().demand();
                        for j in 0..pred_demand.div_ceil(seat_count) as u8 {
                            let pred_left_node = trip_to_node[&(pred, j)].0;
                            builder.add_edge(pred_left_node, right_node);
                        }
                    });
            }
        }

        let graph = builder.into_graph();

        let (_, flow, _) = pushrelabel(&graph, source, sink, |_| 1);

        let mut schedule = Schedule::empty(
            self.vehicles.clone(),
            self.network.clone(),
            self.config.clone(),
        );

        let mut last_trip_to_vehicle: HashMap<(NodeId, u8), VehicleId> = HashMap::new();

        for service_trip in self.network.service_nodes() {
            let demand = self.network.node(service_trip).as_service_trip().demand();
            for i in 0..demand.div_ceil(seat_count) as u8 {
                let right_node = trip_to_node[&(service_trip, i)].1;
                let pred_left_node = graph.inedges(right_node).find_map(|(edge, node)| {
                    if flow[graph.edge_id(edge)].1 == 1 {
                        Some(node)
                    } else {
                        None
                    }
                });

                let candidate =
                    pred_left_node.map(|n| last_trip_to_vehicle[&left_node_to_trip[&n]]);

                match candidate {
                    Some(v) => {
                        schedule = schedule
                            .add_path_to_vehicle_tour(
                                v,
                                Path::new_from_single_node(service_trip, self.network.clone()),
                            )
                            .unwrap();
                        last_trip_to_vehicle.insert((service_trip, i), v);
                    }
                    None => {
                        // no vehicle can reach the service trip, spawn a new one

                        let result =
                            schedule.spawn_vehicle_for_path(vehicle_type, vec![service_trip]);

                        match result {
                            Ok((new_schedule, v)) => {
                                schedule = new_schedule;
                                last_trip_to_vehicle.insert((service_trip, i), v);
                            }
                            Err(_) => {
                                println!(
                                    "Greedy: not enough depots space to cover service trip {}",
                                    service_trip
                                );
                            }
                        }
                    }
                }
            }
        }

        schedule = schedule.reassign_end_depots_greedily().unwrap();

        self.objective.evaluate(schedule)
    }
}
