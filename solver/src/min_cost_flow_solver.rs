use crate::Solution;
use crate::Solver;
use model::base_types::DepotId;
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

#[derive(Clone, Hash, Eq, PartialEq, Debug, Copy)]
enum TripNode {
    Service(NodeId),
    Depot(DepotId),
}

type NetworkNumberType = i64;

type LowerBound = NetworkNumberType;
type UpperBound = NetworkNumberType;
type Cost = NetworkNumberType;

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

        let mut left_rsnode_to_node: HashMap<RsNode, TripNode> = HashMap::new();
        let mut right_rsnode_to_node: HashMap<RsNode, TripNode> = HashMap::new();
        let mut node_to_rsnode: HashMap<TripNode, (RsNode, RsNode)> = HashMap::new();

        let mut edges: HashMap<RsEdge, (LowerBound, UpperBound, Cost)> = HashMap::new();

        let mut max_cost: Cost = 0;

        let trip_node_count =
            self.network.service_nodes().count() + self.network.depots_iter().count();
        // number of nodes in the flow network will be twice this number

        let maximal_formation_count: UpperBound = 10; //TODO read this from vehicle type

        // create two nodes for each service trip and connect them with an edge
        for service_trip in self.network.service_nodes() {
            let left_rsnode = builder.add_node();
            let right_rsnode = builder.add_node();
            let trip_node = TripNode::Service(service_trip);
            left_rsnode_to_node.insert(left_rsnode, trip_node);
            right_rsnode_to_node.insert(right_rsnode, trip_node);
            node_to_rsnode.insert(trip_node, (left_rsnode, right_rsnode));
            let required_vehicles_count = self
                .network
                .node(service_trip)
                .as_service_trip()
                .demand()
                .div_ceil(seat_count) as LowerBound;
            let cost = self.network.node(service_trip).travel_distance().in_meter() as Cost
                * seat_count as Cost;
            edges.insert(
                builder.add_edge(left_rsnode, right_rsnode),
                (required_vehicles_count, required_vehicles_count, cost),
            );
        }

        for depot in self.network.depots_iter() {
            let left_rsnode = builder.add_node();
            let right_rsnode = builder.add_node();
            let depot_node = TripNode::Depot(depot);
            left_rsnode_to_node.insert(left_rsnode, depot_node);
            right_rsnode_to_node.insert(right_rsnode, depot_node);
            node_to_rsnode.insert(depot_node, (left_rsnode, right_rsnode));
        }

        // create the edges between trips
        for (counter, (trip_node, (left_rsnode, _))) in node_to_rsnode.iter().enumerate() {
            let node_id = match trip_node {
                TripNode::Service(s) => *s,
                TripNode::Depot(d) => self.network.get_end_depot_node(*d),
            };
            for pred in self.network.all_predecessors(node_id) {
                let pred_node = match self.network.node(pred) {
                    Node::Service(_) => TripNode::Service(pred),
                    Node::StartDepot(d) => TripNode::Depot(d.depot_id()),
                    _ => continue,
                };
                let pred_right_rsnode = node_to_rsnode[&pred_node].1;

                let cost: Cost = self
                    .network
                    .dead_head_distance_between(pred, node_id)
                    .in_meter() as Cost
                    * seat_count as Cost;
                max_cost = max_cost.max(cost);
                edges.insert(
                    builder.add_edge(pred_right_rsnode, *left_rsnode),
                    (0, maximal_formation_count, cost),
                );
            }
            if counter % 100 == 99 {
                println!(
                    "  adding edges to flow network; node {}/{}",
                    counter + 1,
                    trip_node_count
                );
            }
        }

        let spawn_cost = max_cost
            .checked_mul(trip_node_count as Cost)
            .expect("overflow")
            .checked_add(1)
            .expect("overflow");
        if spawn_cost.checked_mul(trip_node_count as Cost).is_none() {
            // worst case one vehicle per trip would cause overflow
            println!("WARNING: overflow could happen");
            println!("   spawn_cost: {}", spawn_cost);
            println!("   trip_node_count: {}", trip_node_count);
        }

        for depot in self.network.depots_iter() {
            let (left_rsnode, right_rsnode) = node_to_rsnode[&TripNode::Depot(depot)];
            let capacity = self
                .network
                .get_depot(depot)
                .capacity_for(vehicle_type)
                .map(|c| c as UpperBound)
                .unwrap_or(UpperBound::MAX);
            edges.insert(
                builder.add_edge(left_rsnode, right_rsnode),
                (0, capacity, spawn_cost),
            );
        }

        let graph = builder.into_graph();

        println!(
            "Min-Cost-Flow graph loaded (elapsed time for solver: {:0.2}sec)",
            start_time.elapsed().as_secs_f32()
        );

        let (_, flow) = network_simplex(
            &graph,
            |_| 0,           // balance is 0 everywhere -> circulation
            |e| edges[&e].0, // lower bounds
            |e| edges[&e].1, // upper bounds
            |e| edges[&e].2, // costs
        )
        .unwrap();

        println!(
            "Min-Cost-Flow computed (elapsed time for solver: {:0.2}sec)",
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
            let trip_node = match self.network.node(node) {
                Node::Service(_) => TripNode::Service(node),
                Node::EndDepot(d) => TripNode::Depot(d.depot_id()),
                _ => continue,
            };
            let left_rsnode = node_to_rsnode[&trip_node].0;

            for pred_trip_node in graph
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
                match (pred_trip_node, trip_node) {
                    (TripNode::Service(pred_service_trip), TripNode::Service(_)) => {
                        // Flow goes from service trip to service trip -> Use old vehicle
                        let vehicle = last_trip_to_vehicle
                            .get_mut(&pred_service_trip)
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
                    (TripNode::Depot(pred_depot_id), TripNode::Service(_)) => {
                        // Flow goes from depot to service trip -> Spawn new vehicle
                        let start_depot_node = self.network.get_start_depot_node(pred_depot_id);

                        let schedule_vehicle_pair = schedule
                            .spawn_vehicle_for_path(vehicle_type, vec![start_depot_node, node])
                            .unwrap();
                        schedule = schedule_vehicle_pair.0;
                        let vehicle = schedule_vehicle_pair.1;

                        // remember the vehicle for the next service trip
                        last_trip_to_vehicle.entry(node).or_default().push(vehicle);
                    }
                    (TripNode::Service(pred_service_trip), TripNode::Depot(_)) => {
                        // Flow goes from service trip to depot -> Change end depot of vehicle
                        let vehicle = last_trip_to_vehicle
                            .get_mut(&pred_service_trip)
                            .expect("pred not found")
                            .pop()
                            .unwrap();
                        schedule = schedule
                            .add_path_to_vehicle_tour(
                                vehicle,
                                Path::new(vec![pred_service_trip, node], self.network.clone())
                                    .unwrap()
                                    .unwrap(),
                            )
                            .unwrap();
                    }
                    (TripNode::Depot(_), TripNode::Depot(_)) => {
                        println!("WARNING: flow should not go from depot to depot");
                    }
                }
            }
        }
        self.objective.evaluate(schedule)
    }
}
