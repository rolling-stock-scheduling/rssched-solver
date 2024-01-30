use crate::Solution;
use crate::Solver;
use model::base_types::NodeId;
use model::base_types::PassengerCount;
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
use std::sync::Arc;
use std::time;

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
        let upper_bound_for_formation_length = 10;

        let mut builder = LinkedListGraph::<u32>::new_builder();

        let mut left_rsnode_to_node: HashMap<RsNode, NodeId> = HashMap::new();
        let mut right_rsnode_to_node: HashMap<RsNode, NodeId> = HashMap::new();
        let mut node_to_rsnode: HashMap<NodeId, (RsNode, RsNode)> = HashMap::new();

        // (lower_bound, upper_bound, cost)
        let mut edges: HashMap<RsEdge, (i64, i64, i64)> = HashMap::new();

        let mut max_cost = 0;

        let node_count = self.network.service_nodes().count() + self.network.depots_iter().count();

        // let num_service_trips = self.network.service_nodes().count();
        for (counter, node) in self
            .network
            .all_nodes()
            .filter(|&node| {
                self.network.node(node).is_service() || self.network.node(node).is_end_depot()
            })
            .enumerate()
        {
            let left_rsnode = builder.add_node();
            let right_rsnode = builder.add_node();
            left_rsnode_to_node.insert(left_rsnode, node);
            right_rsnode_to_node.insert(left_rsnode, node);
            node_to_rsnode.insert(node, (left_rsnode, right_rsnode));
            match self.network.node(node) {
                Node::Service(service_trip) => {
                    let required_vehicles = service_trip.demand().div_ceil(seat_count) as i64;
                    edges.insert(
                        builder.add_edge(left_rsnode, right_rsnode),
                        (required_vehicles, upper_bound_for_formation_length, 0),
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
                        (0, capacity, 0),
                    );
                }
                _ => {}
            }

            self.network
                .all_predecessors(node)
                .filter(|&pred| {
                    self.network.node(pred).is_service() || self.network.node(pred).is_end_depot()
                })
                .for_each(|pred| {
                    let pred_right_node = node_to_rsnode[&pred].1;
                    let cost: i64 = self
                        .network
                        .dead_head_distance_between(pred, node)
                        .in_meter() as i64
                        * seat_count as i64;
                    max_cost = i64::max(max_cost, cost);
                    edges.insert(
                        builder.add_edge(pred_right_node, left_rsnode),
                        (0, upper_bound_for_formation_length, cost),
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

        let graph = builder.into_graph();

        println!(
            "Min-Cost Matching graph loaded (elapsed time for matching: {:0.2}sec)",
            start_time.elapsed().as_secs_f32()
        );

        let (_, flow) = network_simplex(
            &graph,
            |_| 0, // balance is 0 everywhere -> circulation
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

        // to be continied: path decomposition needed

        self.objective.evaluate(schedule)
    }
}
