use model::base_types::DepotIdx;
use model::base_types::Distance;
use model::base_types::NodeIdx;
use model::base_types::VehicleCount;
use model::base_types::VehicleTypeIdx;
use model::config::Config;
use model::network::nodes::Node;
use model::network::Network;
use model::vehicle_types::VehicleTypes;
use rs_graph::traits::FiniteGraph;
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
    ServiceOrMaintenance(NodeIdx),
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
        // distribute maintenance slots proportional to the total distance of the fleet
        let mut maintenance_slots = self.distribute_maintenance_slots();

        // split into vehicle types
        let mut tours: HashMap<VehicleTypeIdx, Vec<Vec<NodeIdx>>> = HashMap::new();
        // TODO: parallelize this
        for vehicle_type in self.vehicle_types.iter() {
            println!(
                " solving sub-instance for vehicle type {}",
                self.network.vehicle_types().get(vehicle_type).unwrap()
            );
            tours.insert(
                vehicle_type,
                self.solve_for_vehicle_type(
                    vehicle_type,
                    maintenance_slots.remove(&vehicle_type).unwrap(),
                ),
            );
        }

        Schedule::from_tours(tours, self.network.clone()).unwrap()
    }
}

impl MinCostFlowSolver {
    fn distribute_maintenance_slots(
        &self,
    ) -> HashMap<VehicleTypeIdx, HashMap<NodeIdx, VehicleCount>> {
        let mut maintenance_slots: HashMap<VehicleTypeIdx, HashMap<NodeIdx, VehicleCount>> =
            HashMap::new();
        let mut total_distances: HashMap<VehicleTypeIdx, Distance> = HashMap::new();

        for vehicle_type in self.vehicle_types.iter() {
            maintenance_slots.insert(vehicle_type, HashMap::new());

            let dist = self
                .network
                .service_nodes(vehicle_type)
                .map(|node| self.network.node(node).travel_distance())
                .sum();

            total_distances.insert(vehicle_type, dist);
        }

        let priority_increment: HashMap<VehicleTypeIdx, f32> = self
            .vehicle_types
            .iter()
            .map(|vehicle_type| {
                (
                    vehicle_type,
                    1.0 // x% of the maintenance limit is used
                        * self.config.maintenance.maximal_distance.in_meter().unwrap() as f32
                        / total_distances[&vehicle_type].in_meter().unwrap() as f32,
                )
            })
            .collect();

        // Priority counter starts at 0.0 and ends at >=1.0.
        // 0.0 means that the vehicle type has no maintenance slot and 1.0 means that the vehicle
        // type has enough maintenance slot.
        // The next maintenance slot is assigned to the vehicle type with the smallest priority
        // counter.
        let mut priority_counter: Vec<(VehicleTypeIdx, f32)> = self
            .vehicle_types
            .iter()
            .map(|vehicle_type| (vehicle_type, 0.0))
            .collect();

        for maintenance_node in self.network.maintenance_nodes() {
            for _ in 0..self
                .network
                .track_count_of_maintenance_slot(maintenance_node)
            {
                let (vehicle_type, priority) = priority_counter
                    .iter_mut()
                    .min_by(|a, b| {
                        a.1.partial_cmp(&b.1).unwrap().then(
                            priority_increment[&a.0]
                                .partial_cmp(&priority_increment[&b.0])
                                .unwrap(),
                        )
                    })
                    .unwrap();

                let entry = maintenance_slots
                    .get_mut(vehicle_type)
                    .unwrap()
                    .entry(maintenance_node)
                    .or_insert(0);
                *entry += 1;

                *priority += priority_increment[&vehicle_type];

                if priority_counter.iter().all(|(_, p)| *p >= 1.0) {
                    break;
                }
            }
        }

        maintenance_slots
    }

    fn solve_for_vehicle_type(
        &self,
        vehicle_type: VehicleTypeIdx,
        maintenance_slots: HashMap<NodeIdx, VehicleCount>,
    ) -> Vec<Vec<NodeIdx>> {
        let start_time_creating_network = time::Instant::now();

        print!("  1) creating min-cost-flow network - \x1b[93m 0%\x1b[0m");
        io::stdout().flush().unwrap();

        let mut builder = LinkedListGraph::<u32>::new_builder();

        let mut left_rsnode_to_node: HashMap<RsNode, TripNode> = HashMap::new();
        let mut right_rsnode_to_node: HashMap<RsNode, TripNode> = HashMap::new();
        let mut node_to_rsnode: HashMap<TripNode, (RsNode, RsNode)> = HashMap::new();

        let mut edges: HashMap<RsEdge, EdgeLabel> = HashMap::new();

        let mut total_lower_bound: LowerBound = 0;
        let mut cost_overflow_checker: Cost = 0; // computes the maximal cost for the worst
                                                 // feasible flow

        let maximal_formation_count_for_vehicle_type = self
            .vehicle_types
            .get(vehicle_type)
            .unwrap()
            .maximal_formation_count()
            .unwrap_or(100) as UpperBound;

        let trip_node_count =
            self.network.service_nodes(vehicle_type).count() + self.network.depots_iter().count();
        // number of nodes in the flow network will be twice this number

        // create two nodes for each service trip and connect them with an edge
        for service_trip in self.network.service_nodes(vehicle_type) {
            let maximal_formation_count = self
                .network
                .maximal_formation_count_for(service_trip)
                .unwrap_or(100) as UpperBound;
            let left_rsnode = builder.add_node();
            let right_rsnode = builder.add_node();
            let trip_node = TripNode::ServiceOrMaintenance(service_trip);
            left_rsnode_to_node.insert(left_rsnode, trip_node);
            right_rsnode_to_node.insert(right_rsnode, trip_node);
            node_to_rsnode.insert(trip_node, (left_rsnode, right_rsnode));
            let number_of_vehicles_required = self
                .network
                .number_of_vehicles_required_to_serve(vehicle_type, service_trip)
                as LowerBound;

            let lower_bound = number_of_vehicles_required.min(maximal_formation_count); // if more vehicles are required than maximal_formation_count, use maximal_formation_count

            let cost = self.network.node(service_trip).duration().in_sec().unwrap() as Cost
                * self.config.costs.service_trip as Cost;

            total_lower_bound += lower_bound;

            cost_overflow_checker = cost_overflow_checker
                .checked_add(cost.checked_mul(maximal_formation_count).unwrap())
                .expect("overflow in cost_overflow_checker");

            edges.insert(
                builder.add_edge(left_rsnode, right_rsnode),
                EdgeLabel {
                    lower_bound,
                    upper_bound: maximal_formation_count,
                    cost,
                },
            );
        }

        for (maintenance_node, count) in maintenance_slots.iter() {
            let lower_bound = *count as LowerBound;
            let left_rsnode = builder.add_node();
            let right_rsnode = builder.add_node();
            let trip_node = TripNode::ServiceOrMaintenance(*maintenance_node);
            left_rsnode_to_node.insert(left_rsnode, trip_node);
            right_rsnode_to_node.insert(right_rsnode, trip_node);
            node_to_rsnode.insert(trip_node, (left_rsnode, right_rsnode));
            let cost = self
                .network
                .node(*maintenance_node)
                .duration()
                .in_sec()
                .unwrap() as Cost
                * self.config.costs.maintenance as Cost;

            total_lower_bound += lower_bound;

            cost_overflow_checker = cost_overflow_checker
                .checked_add(cost.checked_mul(lower_bound).unwrap())
                .expect("overflow in cost_overflow_checker");

            edges.insert(
                builder.add_edge(left_rsnode, right_rsnode),
                EdgeLabel {
                    lower_bound,
                    upper_bound: lower_bound,
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
                TripNode::ServiceOrMaintenance(s) => *s,
                TripNode::Depot(d) => self.network.get_end_depot_node(*d),
            };
            for pred in self.network.predecessors(vehicle_type, node_id) {
                let pred_node = match self.network.node(pred) {
                    Node::Service(_) => TripNode::ServiceOrMaintenance(pred),
                    Node::StartDepot((_, d)) => TripNode::Depot(d.depot_idx()),
                    Node::Maintenance(_) if maintenance_slots.contains_key(&pred) => {
                        TripNode::ServiceOrMaintenance(pred)
                    }
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
                        .unwrap() as Cost
                        * self.config.costs.idle as Cost
                };

                let cost: Cost = self
                    .network
                    .dead_head_time_between(pred, node_id)
                    .in_sec()
                    .unwrap_or(self.network.planning_days().in_sec().unwrap())
                    as Cost
                    * self.config.costs.dead_head_trip as Cost
                    + idle_time_cost;

                cost_overflow_checker = cost_overflow_checker
                    .checked_add(
                        cost.checked_mul(maximal_formation_count_for_vehicle_type)
                            .unwrap(),
                    )
                    .expect("overflow in cost_overflow_checker");

                edges.insert(
                    builder.add_edge(pred_right_rsnode, *left_rsnode),
                    EdgeLabel {
                        lower_bound: 0,
                        upper_bound: maximal_formation_count_for_vehicle_type,
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

        let max_cost_per_sec = [
            self.config.costs.staff,
            self.config.costs.service_trip,
            self.config.costs.maintenance,
            self.config.costs.dead_head_trip,
            self.config.costs.idle,
        ]
        .into_iter()
        .max()
        .unwrap();

        // spawning cost = costliest activity * (3 * planning days) * total_lower_bound.
        // This suffices, as the total non-spawning costs for the trivial schedule, where each vehicle do exactly one
        // service trip is smaller than that.
        // To see this, note that activity duration of each vehicle in this trival schedule is
        // bounded by the sum of:
        // - Dead head trip from a depot to the start of the single service trips <= planning days
        // - Service trip duration <= planning days
        // - Dead head trip to the depot <= planning days
        // Hence, each vehicle costs at most costliest activity * 3 * planning days.
        let spawning_cost = (max_cost_per_sec as Cost)
            .checked_mul(3)
            .unwrap()
            .checked_mul(self.network.planning_days().in_sec().unwrap() as Cost)
            .unwrap()
            .checked_mul(total_lower_bound)
            .unwrap();

        for depot in self.network.depots_iter() {
            let (left_rsnode, right_rsnode) = node_to_rsnode[&TripNode::Depot(depot)];
            let capacity = self.network.get_depot(depot).capacity_for(vehicle_type) as UpperBound;

            cost_overflow_checker = cost_overflow_checker
                .checked_add(spawning_cost.checked_mul(capacity).unwrap())
                .unwrap_or_else(|| {
                    println!("\x1b[93mwarning:\x1b[0m overflow in min_cost_flow_solver possible. Increase Cost type to i128 in solver/src/min_cost_flow_solver.rs");
                    0 as Cost
                });

            edges.insert(
                builder.add_edge(left_rsnode, right_rsnode),
                EdgeLabel {
                    lower_bound: 0,
                    upper_bound: capacity,
                    cost: spawning_cost,
                },
            );
        }
        let graph = builder.into_graph();

        let start_time_computing_min_cost_flow = time::Instant::now();
        print!(
            "  2) computing min-cost-flow in network with {} nodes and {} edges",
            graph.num_nodes(),
            graph.num_edges()
        );
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
            "\r  2) computing min-cost-flow in network with {} nodes and {} edges - \x1b[32mdone ({:0.2}sec)\x1b[0m",
            graph.num_nodes(),
            graph.num_edges(),
            start_time_computing_min_cost_flow.elapsed().as_secs_f32()
        );

        let time_at_building_schedule = time::Instant::now();
        print!("  3) building schedule");
        io::stdout().flush().unwrap();

        let mut tours: Vec<Vec<NodeIdx>> = Vec::new();

        let mut last_trip_to_tour: HashMap<NodeIdx, Vec<usize>> = HashMap::new();
        // for each service trip or maintenance slot store the tours (as index of tours) that are currently ending there

        let mut print_overflow_depot_warning = false;
        for node in self
            .network
            .nodes_of_vehicle_type_sorted_by_start(vehicle_type)
        {
            // for each service trip, mainteance slot and end depot in chronological order
            let trip_node = match self.network.node(node) {
                Node::Service(_) => TripNode::ServiceOrMaintenance(node),
                Node::EndDepot((_, d)) => TripNode::Depot(d.depot_idx()),
                Node::Maintenance(_) if maintenance_slots.contains_key(&node) => {
                    TripNode::ServiceOrMaintenance(node)
                }
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
                    (TripNode::ServiceOrMaintenance(pred_service_trip), _) => {
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
                    (TripNode::Depot(pred_depot_id), TripNode::ServiceOrMaintenance(_)) => {
                        // Flow goes from depot to service trip -> Create new tour
                        let start_depot_node = self.network.get_start_depot_node(pred_depot_id);

                        if start_depot_node == self.network.overflow_depot_ids().1 {
                            print_overflow_depot_warning = true;
                        }

                        tours.push(vec![start_depot_node, node]);

                        // remember the tour for the next service trip
                        last_trip_to_tour
                            .entry(node)
                            .or_default()
                            .push(tours.len() - 1);
                    }
                    (TripNode::Depot(_), TripNode::Depot(_)) => {
                        println!("\x1b[93mwarning:\x1b[0m flow should not go from depot to depot");
                    }
                }
            }
        }
        println!(
            "\r  3) building schedule - \x1b[32mdone ({:0.2}sec)\x1b[0m",
            time_at_building_schedule.elapsed().as_secs_f32()
        );
        if print_overflow_depot_warning {
            println!(
                "\x1b[93mwarning:\x1b[0m Flow uses overflow depot for vehicle type {}.",
                vehicle_type
            );
        }
        tours
    }
}
