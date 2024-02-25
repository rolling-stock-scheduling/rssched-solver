use crate::Solution;
use crate::Solver;
use model::base_types::NodeId;
use model::base_types::VehicleId;
use model::config::Config;
use model::network::Network;
use model::vehicle_types::VehicleTypes;
use objective_framework::Objective;
use solution::path::Path;
use solution::Schedule;

use rs_graph::linkedlistgraph::Edge as RsEdge;
use rs_graph::linkedlistgraph::Node as RsNode;
use rs_graph::mcf::network_simplex;
use rs_graph::traits::Directed;
use rs_graph::Buildable;
use rs_graph::Builder;
use rs_graph::IndexGraph;
use rs_graph::LinkedListGraph;

use std::collections::HashMap;
use std::sync::Arc;
use std::time;

/// Solving the problem by finding a min-cost maximum-cardinally matching in a bipartit graph.
/// For each service trip, we create two nodes, one on the left and one on the right.
/// If more than one vehicle is needed to cover the service trip, we create multiple nodes for this
/// trip.
/// If trip A can reach trip B, we add an edge from the left node of A to the right node of B.
/// The cost of this edge is the dead-head distance between A and B multiplied by the number of
/// seats of the vehicle type.
///
/// Via a maximum flow computation we find a maximum-cardinally matching:
/// We add a source node and connect it to all left nodes with an edge of cost 0 and capacity 1.
/// We add a sink node and connect all right nodes to it with an edge of cost 0 and capacity 1.
/// We add an edge from source to sink with very large cost and capacity equal to the number of service trips.
/// The flow value is equal to the number of service trips.
///
/// A matching edge means that the corresponding service trip are assigned in succession to a vehicle.
/// As each matching edge reduces the number of vehicle by one, a min-cost maximum-cardinally matching
/// corresponds to a feasible solution with a minimum number of vehicles, where the seat-distance
/// is minimized
pub struct MinCostMaxMatchingSolver {
    vehicles: Arc<VehicleTypes>,
    network: Arc<Network>,
    config: Arc<Config>,
    objective: Arc<Objective<Schedule>>,
}

impl Solver for MinCostMaxMatchingSolver {
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
        let start_time = time::Instant::now();
        // TODO split trips into vehicle types
        // for now: take biggest vehicles (good for a small count, as it might be reused for
        // later trips)
        let vehicle_type = self.vehicles.iter().last().unwrap();
        let seat_count = self.vehicles.get(vehicle_type).unwrap().seats();

        let mut builder = LinkedListGraph::<u32>::new_builder();

        let mut left_node_to_trip: HashMap<RsNode, (NodeId, u8)> = HashMap::new();
        let mut trip_to_node: HashMap<(NodeId, u8), (RsNode, RsNode)> = HashMap::new();

        // (lower_bound, upper_bound, cost)
        let mut edges: HashMap<RsEdge, (i64, i64, i64)> = HashMap::new();

        let source = builder.add_node();
        let sink = builder.add_node();

        let mut node_counter: i64 = 0;
        let mut max_cost = 0;

        let num_service_trips = self.network.all_service_nodes().count();
        for (counter, service_trip) in self.network.all_service_nodes().enumerate() {
            let demand = self.network.passengers_of(service_trip);
            for i in 0..demand.div_ceil(seat_count) as u8 {
                node_counter += 1;
                let left_node = builder.add_node();
                let right_node = builder.add_node();
                left_node_to_trip.insert(left_node, (service_trip, i));
                trip_to_node.insert((service_trip, i), (left_node, right_node));
                edges.insert(builder.add_edge(source, left_node), (0, 1, 0));
                edges.insert(builder.add_edge(right_node, sink), (0, 1, 0));
                self.network
                    .all_predecessors(service_trip)
                    .filter(|&pred| self.network.node(pred).is_service())
                    .for_each(|pred| {
                        let pred_demand = self.network.passengers_of(pred);
                        for j in 0..pred_demand.div_ceil(seat_count) as u8 {
                            let pred_left_node = trip_to_node[&(pred, j)].0;
                            let cost: i64 = self
                                .network
                                .dead_head_distance_between(pred, service_trip)
                                .in_meter() as i64
                                * seat_count as i64;
                            max_cost = i64::max(max_cost, cost);
                            edges
                                .insert(builder.add_edge(pred_left_node, right_node), (0, 1, cost));
                        }
                    });
            }
            if counter % 100 == 99 {
                println!(
                    "  service trips added to matching graph: {}/{}",
                    counter + 1,
                    num_service_trips
                );
            }
        }
        let st_cost: i64 = node_counter
            .checked_mul(max_cost)
            .expect("overflow")
            .checked_add(1)
            .expect("overflow");
        assert!(
            st_cost.checked_mul(node_counter).is_some(),
            "overflow could happen"
        );
        edges.insert(builder.add_edge(source, sink), (0, i64::MAX, st_cost));

        let graph = builder.into_graph();

        println!(
            "Min-Cost Matching graph loaded (elapsed time for matching: {:0.2}sec)",
            start_time.elapsed().as_secs_f32()
        );

        let balance = |n| {
            if n == source {
                node_counter
            } else if n == sink {
                -node_counter
            } else {
                0
            }
        };

        let (_, flow) = network_simplex(
            &graph,
            balance,
            |e| edges[&e].0,
            |e| edges[&e].1,
            |e| edges[&e].2,
        )
        .unwrap();

        println!(
            "Min-Cost Max-Matching computed (elapsed time for matching: {:0.2}sec)",
            start_time.elapsed().as_secs_f32()
        );

        let mut schedule = Schedule::empty(
            self.vehicles.clone(),
            self.network.clone(),
            self.config.clone(),
        );

        let mut last_trip_to_vehicle: HashMap<(NodeId, u8), VehicleId> = HashMap::new();

        for service_trip in self.network.all_service_nodes() {
            let demand = self.network.passengers_of(service_trip);
            for i in 0..demand.div_ceil(seat_count) as u8 {
                let right_node = trip_to_node[&(service_trip, i)].1;
                let pred_left_node = graph.inedges(right_node).find_map(|(edge, node)| {
                    if flow[graph.edge_id(edge)].1 == 1 {
                        Some(node)
                    } else {
                        None
                    }
                });

                let candidate = pred_left_node
                    .map(|n| last_trip_to_vehicle.remove(&left_node_to_trip[&n]).unwrap());

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
        println!(
            "Min-Cost Max-Matching turned into schedule. (matching running time: {:0.2}sec)",
            start_time.elapsed().as_secs_f32()
        );

        self.objective.evaluate(schedule)
    }
}
