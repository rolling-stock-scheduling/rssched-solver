use crate::Solution;
use crate::Solver;
use model::base_types::NodeId;
use model::base_types::VehicleId;
use model::config::Config;
use model::network::nodes::Node;
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
use std::iter::repeat;
use std::sync::Arc;
use std::time;

const UPPER_BOUND_FOR_FORMATION_LENGTH: i64 = 100;
const COST_FOR_SPAWNING_VEHICLE: i64 = 1000000000000;

pub struct MinCostFlowSolver {
    vehicles: Arc<VehicleTypes>,
    network: Arc<Network>,
    config: Arc<Config>,
    objective: Arc<Objective<Schedule>>,
}

impl Solver for MinCostFlowSolver {
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

        let vehicle_type = self.vehicles.iter().last().unwrap();
        let seat_count = self.vehicles.get(vehicle_type).unwrap().seats();

        let mut builder = LinkedListGraph::<u32>::new_builder();

        let mut left_rsnode_to_node: HashMap<RsNode, NodeId> = HashMap::new();
        let mut right_rsnode_to_node: HashMap<RsNode, NodeId> = HashMap::new();
        let mut node_to_rsnode: HashMap<NodeId, (RsNode, RsNode)> = HashMap::new();

        // (lower_bound, upper_bound, cost)
        let mut edges: HashMap<RsEdge, (i64, i64, i64)> = HashMap::new();

        let mut max_cost = 0;

        let node_count = self.network.service_nodes().count() + self.network.depots_iter().count();

        // create two nodes for each service trip and depot (represented by their end nodes)
        for node in self.network.all_nodes().filter(|&node| {
            self.network.node(node).is_service() || self.network.node(node).is_end_depot()
        }) {
            let left_rsnode = builder.add_node();
            let right_rsnode = builder.add_node();
            left_rsnode_to_node.insert(left_rsnode, node);
            right_rsnode_to_node.insert(right_rsnode, node);
            node_to_rsnode.insert(node, (left_rsnode, right_rsnode));
            match self.network.node(node) {
                Node::Service(service_trip) => {
                    let required_vehicles_count = service_trip.demand().div_ceil(seat_count) as i64;
                    edges.insert(
                        builder.add_edge(left_rsnode, right_rsnode),
                        (required_vehicles_count, UPPER_BOUND_FOR_FORMATION_LENGTH, 0),
                    );
                }
                Node::EndDepot(depot_node) => {
                    let capacity = self
                        .network
                        .get_depot(depot_node.depot_id())
                        .capacity_for(vehicle_type)
                        .map(|c| c as i64)
                        .unwrap_or(i64::MAX);
                    edges.insert(
                        builder.add_edge(left_rsnode, right_rsnode),
                        (0, capacity, COST_FOR_SPAWNING_VEHICLE),
                    );
                }
                _ => {}
            }
        }

        // create the edges
        for (counter, node) in self
            .network
            .all_nodes()
            .filter(|&node| {
                self.network.node(node).is_service() || self.network.node(node).is_end_depot()
            })
            .enumerate()
        {
            let left_rsnode = node_to_rsnode[&node].0;
            self.network
                .all_predecessors(node)
                .filter(|&pred| self.network.node(pred).is_service())
                .chain(self.network.end_depot_nodes()) // all depots are predecesors of all ttrips (here depots are represented by their end nodes)
                .for_each(|pred| {
                    let pred_right_node = node_to_rsnode[&pred].1;
                    let cost: i64 = self
                        .network
                        .dead_head_distance_between(pred, node) // TODO a bit unclean for depots
                        .in_meter() as i64
                        * seat_count as i64;
                    max_cost = i64::max(max_cost, cost);
                    edges.insert(
                        builder.add_edge(pred_right_node, left_rsnode),
                        (0, UPPER_BOUND_FOR_FORMATION_LENGTH, cost),
                    );
                });
            if counter % 100 == 99 {
                println!(
                    "  service trips added to matching graph: {}/{}",
                    counter + 1,
                    node_count
                );
            }
        }

        // TEMP:
        assert!(
            (node_count as i64).checked_mul(max_cost).expect("overflow")
                < COST_FOR_SPAWNING_VEHICLE,
            "max_cost * node_count = {} * {} = {} > {} = COST_FOW_SPAWNING_VEHICLE",
            max_cost,
            node_count,
            (node_count as i64).checked_mul(max_cost).unwrap(),
            COST_FOR_SPAWNING_VEHICLE
        );
        assert!(
            COST_FOR_SPAWNING_VEHICLE
                .checked_mul(node_count as i64)
                .is_some(),
            "overflow could happen"
        );

        let graph = builder.into_graph();

        println!(
            "Min-Cost Matching graph loaded (elapsed time for matching: {:0.2}sec)",
            start_time.elapsed().as_secs_f32()
        );

        let (_, flow) = network_simplex(
            &graph,
            |_| 0, // balance is 0 everywhere -> circulation
            // |_| 0,
            |e| edges[&e].0,
            |e| edges[&e].1,
            |e| edges[&e].2,
        )
        .unwrap();

        println!(
            "Min-Cost-Flow computed (elapsed time for matching: {:0.2}sec)",
            start_time.elapsed().as_secs_f32()
        );

        let mut schedule = Schedule::empty(
            self.vehicles.clone(),
            self.network.clone(),
            self.config.clone(),
        );

        let mut last_trip_to_vehicle: HashMap<NodeId, Vec<VehicleId>> = HashMap::new();

        for node in self
            .network
            .service_nodes()
            .chain(self.network.end_depot_nodes())
        {
            let left_rsnode = node_to_rsnode[&node].0;
            for pred in graph
                .inedges(left_rsnode)
                .filter_map(|(e, n)| {
                    if flow[graph.edge_id(e)].1 == 0 {
                        None
                    } else {
                        // take rs_node flow-value often and turn into a node_id
                        Some(
                            repeat(right_rsnode_to_node[&n])
                                .take(flow[graph.edge_id(e)].1 as usize),
                        )
                    }
                })
                .flatten()
            {
                match (self.network.node(pred), self.network.node(node)) {
                    (Node::Service(_), Node::Service(_)) => {
                        // Flow goes from service trip to service trip -> Use old vehicle
                        let vehicle = last_trip_to_vehicle
                            .get_mut(&pred)
                            .expect("pred not found")
                            .pop()
                            .unwrap();
                        schedule = schedule
                            .add_path_to_vehicle_tour(
                                vehicle,
                                Path::new_from_single_node(node, self.network.clone()),
                            )
                            .unwrap();
                        // remember the vehicle for the next service trip
                        last_trip_to_vehicle.entry(node).or_default().push(vehicle);
                    }
                    (Node::EndDepot(depot_node), Node::Service(_)) => {
                        // Flow goes from depot to service trip -> Spawn new vehicle
                        let start_depot_node =
                            self.network.get_start_depot_node(depot_node.depot_id());

                        let schedule_vehicle_pair = schedule
                            .spawn_vehicle_for_path(vehicle_type, vec![start_depot_node, node])
                            .unwrap();
                        schedule = schedule_vehicle_pair.0;
                        let vehicle = schedule_vehicle_pair.1;

                        // remember the vehicle for the next service trip
                        last_trip_to_vehicle.entry(node).or_default().push(vehicle);
                    }
                    (Node::Service(_), Node::EndDepot(_)) => {
                        // Flow goes from service trip to depot -> Change end depot of vehicle
                        let vehicle = last_trip_to_vehicle
                            .get_mut(&pred)
                            .expect("pred not found")
                            .pop()
                            .unwrap();
                        schedule = schedule
                            .add_path_to_vehicle_tour(
                                vehicle,
                                Path::new(vec![pred, node], self.network.clone())
                                    .unwrap()
                                    .unwrap(),
                            )
                            .unwrap();
                    }
                    (Node::EndDepot(_), Node::EndDepot(_)) => {
                        panic!("flow should not go from depot to depot");
                    }
                    _ => {
                        panic!("unexpected node type");
                    }
                }
            }
        }

        // to be continied: path decomposition needed

        self.objective.evaluate(schedule)
    }
}
