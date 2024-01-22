use crate::Solution;
use crate::Solver;
use model::config::Config;
use model::network::Network;
use model::vehicle_types::VehicleTypes;
use objective_framework::Objective;
use rs_graph::maxflow::pushrelabel;
use rs_graph::traits::Directed;
use rs_graph::Buildable;
use rs_graph::Builder;
use rs_graph::IndexGraph;
use rs_graph::LinkedListGraph;
use rs_graph::VecGraph;
use solution::Schedule;
use std::collections::HashMap;
use std::sync::Arc;

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
        let mut schedule = Schedule::empty(
            self.vehicles.clone(),
            self.network.clone(),
            self.config.clone(),
        );

        let mut builder = LinkedListGraph::<u32>::new_builder();
        // let mut builder = VecGraph::<u32>::new_builder();

        let mut left_node_to_trip = HashMap::new();
        let mut right_node_to_trip = HashMap::new();
        let mut trip_to_node = HashMap::new();
        let source = builder.add_node();
        let sink = builder.add_node();
        let num_service_trips = self.network.service_nodes().count();
        for (counter, service_trip) in self.network.service_nodes().enumerate() {
            println!("Adding service trip {}/{}", counter, num_service_trips);
            let left_node = builder.add_node();
            let right_node = builder.add_node();
            left_node_to_trip.insert(left_node, service_trip);
            right_node_to_trip.insert(right_node, service_trip);
            trip_to_node.insert(service_trip, (left_node, right_node));
            builder.add_edge(source, left_node);
            builder.add_edge(right_node, sink);
            self.network
                .all_predecessors(service_trip)
                .filter(|&pred| self.network.node(pred).is_service())
                .for_each(|pred| {
                    let pred_left_node = trip_to_node[&pred].0;
                    builder.add_edge(pred_left_node, right_node);
                });
        }

        let graph = builder.into_graph();

        println!("Start push-relabel");

        let (value, flow, _) = pushrelabel(&graph, source, sink, |_| 1);

        println!("value: {}", value);

        for left_node in graph.outedges(source).filter_map(|(edge, node)| {
            if flow[graph.edge_id(edge)].1 == 1 {
                Some(node)
            } else {
                None
            }
        }) {
            println!("left_node: {:?}", left_node);
        }
        /*
        for service_trip in self.network.service_nodes() {
            let (left_node, right_node) = trip_to_node[&service_trip];
            if flow_carrying.contains(&(left_node, right_node)) {
                schedule = schedule.add_trip(service_trip).unwrap();
            }
        }
        */

        schedule = schedule.reassign_end_depots_greedily().unwrap();

        self.objective.evaluate(schedule)
    }
}
