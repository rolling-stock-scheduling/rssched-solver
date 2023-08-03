pub mod nodes;
use nodes::Node;

use crate::base_types::{Distance, Duration, NodeId};
use crate::config::Config;
use crate::locations::Locations;

use std::collections::HashMap;
use std::fmt;

use std::iter::Iterator;

use std::sync::Arc;

pub struct Network {
    nodes: HashMap<NodeId, Node>,

    // nodes are by default sorted by start_time
    service_nodes: Vec<NodeId>,
    maintenance_nodes: Vec<NodeId>,
    depot_nodes: Vec<NodeId>,

    nodes_sorted_by_start: Vec<NodeId>,
    nodes_sorted_by_end: Vec<NodeId>,

    // for convenience
    config: Arc<Config>,
    loc: Arc<Locations>,
}

// methods
impl Network {
    pub fn node(&self, id: NodeId) -> &Node {
        self.nodes.get(&id).unwrap()
    }

    /// return the number of nodes in the network.
    pub fn size(&self) -> usize {
        self.nodes.len()
    }

    pub fn service_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.service_nodes.iter().copied()
    }

    pub fn maintenance_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.maintenance_nodes.iter().copied()
    }

    pub fn depot_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.depot_nodes.iter().copied()
    }

    pub fn idle_time_between(&self, node1: NodeId, node2: NodeId) -> Duration {
        let idle_start = self.node(node1).end_time() + self.dead_head_time_between(node1, node2);
        let idle_end = self.node(node2).start_time();
        if idle_start <= idle_end {
            idle_end - idle_start
        } else {
            println!("negative idle time!");
            Duration::zero()
        }
    }

    pub fn dead_head_time_between(&self, node1: NodeId, node2: NodeId) -> Duration {
        self.loc.travel_time(
            self.node(node1).end_location(),
            self.node(node2).start_location(),
        )
    }

    pub fn dead_head_distance_between(&self, node1: NodeId, node2: NodeId) -> Distance {
        self.loc.distance(
            self.node(node1).end_location(),
            self.node(node2).start_location(),
        )
    }

    /// returns True iff node1 can reach node2
    pub fn can_reach(&self, node1: NodeId, node2: NodeId) -> bool {
        let n1 = self.nodes.get(&node1).unwrap();
        let n2 = self.nodes.get(&node2).unwrap();

        n1.end_time() + self.required_duration_between_activities(n1, n2) <= n2.start_time()
    }

    fn required_duration_between_activities(&self, n1: &Node, n2: &Node) -> Duration {
        if n1.end_location() == n2.start_location() {
            // no dead_head_trip
            self.shunting_duration_between_activities_without_dead_head_trip(n1, n2)
        } else {
            // dead_head_trip
            self.loc.travel_time(n1.end_location(), n2.start_location())
                + self.shunting_duration_between_activities_with_dead_head_trip(n1, n2)
        }
    }

    fn shunting_duration_between_activities_without_dead_head_trip(
        &self,
        n1: &Node,
        n2: &Node,
    ) -> Duration {
        if let Node::Service(_) = n1 {
            if let Node::Service(_) = n2 {
                // both nodes are service trips

                self.config.durations_between_activities.minimal
            } else {
                // n2 is no service trip
                Duration::zero()
            }
        } else {
            // n1 is no service trip
            Duration::zero()
        }
    }

    fn shunting_duration_between_activities_with_dead_head_trip(
        &self,
        n1: &Node,
        n2: &Node,
    ) -> Duration {
        // let (dht_start_side, dht_end_side) = self
        // .loc
        // .station_sides(n1.end_location(), n2.start_location());
        let previous: Duration = match n1 {
            Node::Service(_) => self.config.durations_between_activities.dead_head_trip,
            Node::Maintenance(_) => self.config.durations_between_activities.dead_head_trip,
            _ => Duration::zero(),
        };

        let next: Duration = match n2 {
            Node::Service(_) => self.config.durations_between_activities.dead_head_trip,
            Node::Maintenance(_) => self.config.durations_between_activities.dead_head_trip,
            _ => Duration::zero(),
        };

        previous + next
    }

    /// provides all nodes that are reachable from node (in increasing order according to the
    /// starting time)
    pub fn all_successors(&self, node: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        // TODO: Could use binary_search for speed up
        self.nodes_sorted_by_start
            .iter()
            .copied()
            .filter(move |&n| self.can_reach(node, n))
    }

    /// provides all nodes that are can reach node (in decreasing order according to the
    /// end time)
    pub fn all_predecessors(&self, node: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes_sorted_by_end
            .iter()
            .rev()
            .copied()
            .filter(move |&n| self.can_reach(n, node))
    }

    pub fn all_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes_sorted_by_start.iter().copied()
    }
}

impl Network {
    pub fn new(all_nodes: Vec<Node>, config: Arc<Config>, loc: Arc<Locations>) -> Network {
        let mut nodes = HashMap::new();
        let mut service_nodes = Vec::new();
        let mut maintenance_nodes = Vec::new();
        let mut depot_nodes = Vec::new();
        for node in all_nodes {
            let id = node.id();
            nodes.insert(id, node);
            match nodes.get(&id).unwrap() {
                Node::Service(_) => service_nodes.push(id),
                Node::Maintenance(_) => maintenance_nodes.push(id),
                Node::Depot(_) => depot_nodes.push(id),
            }
        }
        let mut nodes_sorted_by_start: Vec<NodeId> = nodes.keys().copied().collect();
        nodes_sorted_by_start.sort_by_key(|&n| nodes.get(&n).unwrap().start_time());

        let mut nodes_sorted_by_end: Vec<NodeId> = nodes_sorted_by_start.clone();
        nodes_sorted_by_end.sort_by_key(|&n| nodes.get(&n).unwrap().end_time());

        Network {
            nodes,
            service_nodes,
            maintenance_nodes,
            depot_nodes,
            nodes_sorted_by_start,
            nodes_sorted_by_end,
            config,
            loc,
        }
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "** network with {} nodes:", self.size())?;
        for (i, n) in self.nodes_sorted_by_start.iter().enumerate() {
            writeln!(f, "\t{}: {}", i, self.nodes.get(n).unwrap())?;
        }
        Ok(())
    }
}
