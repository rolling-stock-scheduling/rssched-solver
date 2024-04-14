use model::base_types::DepotIdx;
use model::base_types::NodeIdx;
use model::base_types::VehicleTypeIdx;
use model::base_types::COST_FOR_INF_DURATION;
use model::config::Config;
use model::network::nodes::Node;
use model::network::Network;
use model::vehicle_types::VehicleTypes;
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
use std::io;
use std::io::Write;
use std::iter::repeat;
use std::sync::Arc;
use std::time;

#[derive(Clone, Hash, Eq, PartialEq, Debug, Copy)]
enum TripNode {
    Service(NodeIdx),
    Depot(DepotIdx),
}

type NetworkNumberType = i64;

type LowerBound = NetworkNumberType;
type UpperBound = NetworkNumberType;
type Cost = NetworkNumberType;

struct EdgeLabel {
    lower_bound: LowerBound,
    upper_bound: UpperBound,
    cost: Cost,
}

pub struct MinCostFlowSolver {
    vehicle_types: Arc<VehicleTypes>,
    config: Arc<Config>,
    network: Arc<Network>,
}

impl MinCostFlowSolver {
    pub fn initialize(network: Arc<Network>) -> Self {
        Self {
            vehicle_types: network.vehicle_types(),
            config: network.config(),
            network,
        }
    }

    pub fn solve(&self) -> Schedule {
        // split into vehicle types
        let mut tours: HashMap<VehicleTypeIdx, Vec<Vec<NodeIdx>>> = HashMap::new();
        for vehicle_type in self.vehicle_types.iter() {
            tours.insert(vehicle_type, self.solve_for_vehicle_type(vehicle_type));
        }

        Schedule::from_tours(tours, self.network.clone()).unwrap()
    }
}

impl MinCostFlowSolver {
    fn solve_for_vehicle_type(&self, vehicle_type: VehicleTypeIdx) -> Vec<Vec<NodeIdx>> {
        let start_time_creating_network = time::Instant::now();

        print!("  1) creating min-cost-flow network - \x1b[93m 0%\x1b[0m");
        io::stdout().flush().unwrap();

        let mut builder = LinkedListGraph::<u32>::new_builder();

        let mut left_rsnode_to_node: HashMap<RsNode, TripNode> = HashMap::new();
        let mut right_rsnode_to_node: HashMap<RsNode, TripNode> = HashMap::new();
        let mut node_to_rsnode: HashMap<TripNode, (RsNode, RsNode)> = HashMap::new();

        let mut edges: HashMap<RsEdge, EdgeLabel> = HashMap::new();

        let mut max_cost: Cost = 0;

        let trip_node_count =
            self.network.service_nodes(vehicle_type).count() + self.network.depots_iter().count();
        // number of nodes in the flow network will be twice this number

        let maximal_formation_count: UpperBound = self
            .vehicle_types
            .get(vehicle_type)
            .unwrap()
            .maximal_formation_count()
            .map_or(UpperBound::MAX, |x| x as UpperBound);

        // create two nodes for each service trip and connect them with an edge
        for service_trip in self.network.service_nodes(vehicle_type) {
            let left_rsnode = builder.add_node();
            let right_rsnode = builder.add_node();
            let trip_node = TripNode::Service(service_trip);
            left_rsnode_to_node.insert(left_rsnode, trip_node);
            right_rsnode_to_node.insert(right_rsnode, trip_node);
            node_to_rsnode.insert(trip_node, (left_rsnode, right_rsnode));
            let number_of_vehicles_required = self
                .network
                .number_of_vehicles_required_to_serve(vehicle_type, service_trip)
                as LowerBound;

            let cost = self
                .network
                .node(service_trip)
                .duration()
                .in_sec()
                .unwrap_or(COST_FOR_INF_DURATION) as Cost
                * self.config.costs.service_trip as Cost;
            edges.insert(
                builder.add_edge(left_rsnode, right_rsnode),
                EdgeLabel {
                    lower_bound: number_of_vehicles_required,
                    upper_bound: number_of_vehicles_required,
                    cost,
                },
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
        let mut time_since_last_print = time::Instant::now();
        for (counter, (trip_node, (left_rsnode, _))) in node_to_rsnode.iter().enumerate() {
            let node_id = match trip_node {
                TripNode::Service(s) => *s,
                TripNode::Depot(d) => self.network.get_end_depot_node(*d),
            };
            for pred in self.network.predecessors(vehicle_type, node_id) {
                let pred_node = match self.network.node(pred) {
                    Node::Service(_) => TripNode::Service(pred),
                    Node::StartDepot(d) => TripNode::Depot(d.depot_idx()),
                    _ => continue,
                };
                let pred_right_rsnode = node_to_rsnode[&pred_node].1;

                let idle_time_cost = if self.network.node(pred).is_depot()
                    || self.network.node(node_id).is_depot()
                {
                    0
                } else {
                    self.network
                        .idle_time_between(pred, node_id)
                        .in_sec()
                        .unwrap_or(COST_FOR_INF_DURATION) as Cost
                        * self.config.costs.idle as Cost
                };

                let cost: Cost = self
                    .network
                    .dead_head_time_between(pred, node_id)
                    .in_sec()
                    .unwrap_or(COST_FOR_INF_DURATION) as Cost
                    * self.config.costs.dead_head_trip as Cost
                    + idle_time_cost;
                max_cost = max_cost.max(cost);
                edges.insert(
                    builder.add_edge(pred_right_rsnode, *left_rsnode),
                    EdgeLabel {
                        lower_bound: 0,
                        upper_bound: maximal_formation_count,
                        cost,
                    },
                );
            }
            if time_since_last_print.elapsed().as_secs_f32() >= 5.0 {
                print!(
                    "\r  1) creating min-cost-flow network - \x1b[93m{:>2}%\x1b[0m",
                    (counter + 1) * 100 / trip_node_count,
                );
                io::stdout().flush().unwrap();
                time_since_last_print = time::Instant::now();
            }
        }

        println!(
            "\r  1) creating min-cost-flow network - \x1b[32mdone ({:0.2}sec)\x1b[0m",
            start_time_creating_network.elapsed().as_secs_f32()
        );

        let spawn_cost = max_cost
            .checked_mul(trip_node_count as Cost)
            .expect("overflow")
            .checked_add(1)
            .expect("overflow");
        if spawn_cost.checked_mul(trip_node_count as Cost).is_none() {
            // worst case one vehicle per trip would cause overflow
            println!("\x1b[93mWARNING: overflow could happen");
            println!("   spawn_cost: {}", spawn_cost);
            println!("   trip_node_count: {}\x1b[0m", trip_node_count);
        }

        for depot in self.network.depots_iter() {
            let (left_rsnode, right_rsnode) = node_to_rsnode[&TripNode::Depot(depot)];
            let capacity = self.network.get_depot(depot).capacity_for(vehicle_type) as UpperBound;
            edges.insert(
                builder.add_edge(left_rsnode, right_rsnode),
                EdgeLabel {
                    lower_bound: 0,
                    upper_bound: capacity,
                    cost: spawn_cost,
                },
            );
        }
        let graph = builder.into_graph();

        let start_time_computing_min_cost_flow = time::Instant::now();
        print!("  2) computing min-cost-flow");
        io::stdout().flush().unwrap();

        let (_, flow) = network_simplex(
            &graph,
            |_| 0,                     // balance is 0 everywhere -> circulation
            |e| edges[&e].lower_bound, // lower bounds
            |e| edges[&e].upper_bound, // upper bounds
            |e| edges[&e].cost,        // costs
        )
        .unwrap();

        println!(
            "\r  2) computing min-cost-flow - \x1b[32mdone ({:0.2}sec)\x1b[0m",
            start_time_computing_min_cost_flow.elapsed().as_secs_f32()
        );

        let time_at_building_schedule = time::Instant::now();
        print!("  3) building schedule");
        io::stdout().flush().unwrap();

        let mut tours: Vec<Vec<NodeIdx>> = Vec::new();

        let mut last_trip_to_tour: HashMap<NodeIdx, Vec<usize>> = HashMap::new();
        // for each service trip store the tours (as index of tours) that are currently ending there

        for node in self
            .network
            .service_nodes(vehicle_type)
            .chain(self.network.end_depot_nodes())
        {
            // for each service trip and each depot in chronological order

            let trip_node = match self.network.node(node) {
                Node::Service(_) => TripNode::Service(node),
                Node::EndDepot(d) => TripNode::Depot(d.depot_idx()),
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
                    (TripNode::Service(pred_service_trip), _) => {
                        // Flow goes from service trip to service trip -> Use existing tour
                        let existing_tour_index = last_trip_to_tour
                            .get_mut(&pred_service_trip)
                            .expect("pred not found")
                            .pop()
                            .unwrap();
                        tours[existing_tour_index].push(node);
                        // remember the tour for the next service trip
                        last_trip_to_tour
                            .entry(node)
                            .or_default()
                            .push(existing_tour_index);
                    }
                    (TripNode::Depot(pred_depot_id), TripNode::Service(_)) => {
                        // Flow goes from depot to service trip -> Create new tour
                        let start_depot_node = self.network.get_start_depot_node(pred_depot_id);

                        tours.push(vec![start_depot_node, node]);

                        // remember the tour for the next service trip
                        last_trip_to_tour
                            .entry(node)
                            .or_default()
                            .push(tours.len() - 1);
                    }
                    (TripNode::Depot(_), TripNode::Depot(_)) => {
                        println!("\x1b[93mWARNING: flow should not go from depot to depot\x1b[0m");
                    }
                }
            }
        }
        println!(
            "\r  3) building schedule - \x1b[32mdone ({:0.2}sec)\x1b[0m",
            time_at_building_schedule.elapsed().as_secs_f32()
        );
        tours
    }
}
