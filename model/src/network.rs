pub mod depot;
pub mod nodes;

use depot::Depot;
use nodes::Node;
use nodes::{MaintenanceSlot, ServiceTrip};
use time::{DateTime, Duration};

use crate::base_types::{
    DepotId, Distance, Location, NodeId, PassengerCount, SeatDistance, VehicleTypeId,
};
use crate::config::Config;
use crate::locations::Locations;

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use std::iter::Iterator;

use std::sync::Arc;

pub struct Network {
    nodes: HashMap<NodeId, Node>,
    depots: HashMap<DepotId, Depot>,

    // nodes are by default sorted by start_time (ties are broken by end_time then id)
    service_nodes: Vec<NodeId>,
    maintenance_nodes: Vec<NodeId>,
    start_depot_nodes: Vec<NodeId>,
    end_depot_nodes: Vec<NodeId>,

    nodes_sorted_by_start: BTreeMap<(DateTime, NodeId), NodeId>,
    nodes_sorted_by_end: BTreeMap<(DateTime, NodeId), NodeId>,

    // redundant information
    passenger_distance_demand: SeatDistance,
    latest_end_time: DateTime,

    // for convenience
    config: Arc<Config>,
    locations: Arc<Locations>,
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

    pub fn start_depot_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.start_depot_nodes.iter().copied()
    }

    pub fn end_depot_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.end_depot_nodes.iter().copied()
    }

    pub fn depots_iter(&self) -> impl Iterator<Item = DepotId> + '_ {
        self.depots.keys().copied()
    }

    /// service and maintenance_nodes
    pub fn coverable_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.service_nodes().chain(self.maintenance_nodes())
    }

    pub fn locations(&self) -> &Locations {
        &self.locations
    }

    pub fn capacity_of(
        &self,
        depot_id: DepotId,
        vehicle_type_id: VehicleTypeId,
    ) -> Option<PassengerCount> {
        self.depots[&depot_id].capacity_for(vehicle_type_id)
    }

    pub fn get_depot_id(&self, node_id: NodeId) -> DepotId {
        self.node(node_id).as_depot().depot_id()
    }

    pub fn get_depot(&self, depot_id: DepotId) -> &Depot {
        self.depots.get(&depot_id).unwrap()
    }

    // TODO this can be done more efficiently
    pub fn get_start_depot_node(&self, depot_id: DepotId) -> NodeId {
        *self
            .start_depot_nodes
            .iter()
            .find(|&&n| self.get_depot_id(n) == depot_id)
            .unwrap()
    }

    /// sum over all service trips: number of passenger * distance
    pub fn passenger_distance_demand(&self) -> SeatDistance {
        self.passenger_distance_demand
    }

    pub fn latest_end_time(&self) -> DateTime {
        self.latest_end_time
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
        self.locations.travel_time(
            self.node(node1).end_location(),
            self.node(node2).start_location(),
        )
    }

    pub fn dead_head_distance_between(&self, node1: NodeId, node2: NodeId) -> Distance {
        self.locations.distance(
            self.node(node1).end_location(),
            self.node(node2).start_location(),
        )
    }

    /// returns True iff node1 can reach node2
    /// but alswats False from start depot to start depot and end depot to end depot
    pub fn can_reach(&self, node1: NodeId, node2: NodeId) -> bool {
        let n1 = self.nodes.get(&node1).unwrap();
        let n2 = self.nodes.get(&node2).unwrap();

        if n1.is_start_depot() && n2.is_start_depot() {
            // start depots cannot reach each other
            return false;
        }
        if n1.is_end_depot() && n2.is_end_depot() {
            // end depots cannot reach each other
            return false;
        }

        n1.end_time() + self.minimal_duration_between_nodes_as_ref(n1, n2) <= n2.start_time()
    }

    /// provides all nodes that are reachable from node (in increasing order according to the
    /// starting time)
    pub fn all_successors(&self, node: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes_sorted_by_start
            .range((self.node(node).end_time(), NodeId::from(""))..)
            .map(|(_, n)| n)
            .copied()
            .filter(move |&n| self.can_reach(node, n))
    }

    /// provides all nodes that are can reach node
    pub fn all_predecessors(&self, node: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes_sorted_by_end
            .range(..(self.node(node).start_time(), NodeId::from("")))
            .map(|(_, n)| n)
            .copied()
            .filter(move |&n| self.can_reach(n, node))
    }

    /// Assume that node1 can reach node2.
    pub fn minimal_duration_between_nodes(&self, node1: NodeId, node2: NodeId) -> Duration {
        let n1 = self.nodes.get(&node1).unwrap();
        let n2 = self.nodes.get(&node2).unwrap();

        self.minimal_duration_between_nodes_as_ref(n1, n2)
    }

    fn minimal_duration_between_nodes_as_ref(&self, n1: &Node, n2: &Node) -> Duration {
        if n1.end_location() == n2.start_location() {
            // no dead_head_trip
            self.shunting_duration_between_activities_if_no_dead_head_trip(n1, n2)
        } else {
            // dead_head_trip
            self.locations
                .travel_time(n1.end_location(), n2.start_location())
                + self.shunting_duration_between_activities_if_dead_head_trip(n1, n2)
        }
    }

    fn shunting_duration_between_activities_if_no_dead_head_trip(
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

    fn shunting_duration_between_activities_if_dead_head_trip(
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

    pub fn all_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes_sorted_by_start.values().copied()
    }

    pub fn start_depots_sorted_by_distance_to(&self, location: Location) -> Vec<NodeId> {
        let mut depots = self.start_depot_nodes.clone();
        depots.sort_by_key(|&d| {
            self.locations
                .distance(self.node(d).start_location(), location)
        });
        depots
    }

    pub fn end_depots_sorted_by_distance_from(&self, location: Location) -> Vec<NodeId> {
        let mut depots = self.end_depot_nodes.clone();
        depots.sort_by_key(|&d| {
            self.locations
                .distance(location, self.node(d).start_location())
        });
        depots
    }
}

impl Network {
    pub fn new(
        depots: Vec<Depot>,
        service_trips: Vec<ServiceTrip>,
        maintenance_slots: Vec<MaintenanceSlot>,
        config: Arc<Config>,
        loc: Arc<Locations>,
    ) -> Network {
        let mut nodes = HashMap::new();
        let mut depots_lookup = HashMap::new();
        let mut service_nodes = Vec::new();
        let mut maintenance_nodes = Vec::new();
        let mut start_depot_nodes = Vec::new();
        let mut end_depot_nodes = Vec::new();

        for depot in depots {
            let depot_id = depot.depot_id();

            let start_node_id = NodeId::from(&format!("s_{}", depot_id));
            let start_node_name = format!("start_depot({},{})", depot_id, depot.location());
            let start_node = Node::create_start_depot_node(
                start_node_id,
                depot_id,
                depot.location(),
                start_node_name,
            );
            nodes.insert(start_node_id, start_node);
            start_depot_nodes.push(start_node_id);

            let end_node_id = NodeId::from(&format!("e_{}", depot_id));
            let end_node_name = format!("end_depot({},{})", depot_id, depot.location());
            let end_node =
                Node::create_end_depot_node(end_node_id, depot_id, depot.location(), end_node_name);
            nodes.insert(end_node_id, end_node);
            end_depot_nodes.push(end_node_id);
            depots_lookup.insert(depot_id, depot);
        }

        for service_trip in service_trips.into_iter() {
            let id = service_trip.id();
            let node = Node::create_service_trip_node(service_trip);
            nodes.insert(id, node);
            service_nodes.push(id);
        }

        for maintenance_slot in maintenance_slots.into_iter() {
            let id = maintenance_slot.id();
            let node = Node::create_maintenance_node(maintenance_slot);
            nodes.insert(id, node);
            maintenance_nodes.push(id);
        }

        // sort nodes:
        service_nodes.sort_by(|&n1, &n2| {
            nodes
                .get(&n1)
                .unwrap()
                .cmp_start_time(nodes.get(&n2).unwrap())
        });

        maintenance_nodes.sort_by(|&n1, &n2| {
            nodes
                .get(&n1)
                .unwrap()
                .cmp_start_time(nodes.get(&n2).unwrap())
        });

        start_depot_nodes.sort_by(|&n1, &n2| {
            nodes
                .get(&n1)
                .unwrap()
                .cmp_start_time(nodes.get(&n2).unwrap())
        });

        end_depot_nodes.sort_by(|&n1, &n2| {
            nodes
                .get(&n1)
                .unwrap()
                .cmp_start_time(nodes.get(&n2).unwrap())
        });

        let nodes_sorted_by_start: BTreeMap<(DateTime, NodeId), NodeId> = nodes
            .keys()
            .map(|&n| {
                let node = nodes.get(&n).unwrap();
                ((node.start_time(), n), n)
            })
            .collect();

        let nodes_sorted_by_end: BTreeMap<(DateTime, NodeId), NodeId> = nodes
            .keys()
            .map(|&n| {
                let node = nodes.get(&n).unwrap();
                ((node.end_time(), n), n)
            })
            .collect();

        let passenger_distance_demand = service_nodes
            .iter()
            .map(|&n| {
                let node = nodes.get(&n).unwrap();
                node.as_service_trip().demand() as SeatDistance
                    * node.travel_distance().in_meter() as SeatDistance
            })
            .sum();

        let latest_end_time = service_nodes
            .iter()
            .map(|&n| nodes.get(&n).unwrap().end_time())
            .max()
            .unwrap();

        Network {
            nodes,
            depots: depots_lookup,
            service_nodes,
            maintenance_nodes,
            start_depot_nodes,
            end_depot_nodes,
            nodes_sorted_by_start,
            nodes_sorted_by_end,
            passenger_distance_demand,
            latest_end_time,
            config,
            locations: loc,
        }
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "** network with {} nodes:", self.size())?;
        for (i, n) in self.nodes_sorted_by_start.values().enumerate() {
            writeln!(f, "\t{}: {}", i, self.nodes.get(n).unwrap())?;
        }
        Ok(())
    }
}
