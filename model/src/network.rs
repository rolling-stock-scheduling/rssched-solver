pub mod depot;
pub mod nodes;

use depot::Depot;
use nodes::Node;
use nodes::{MaintenanceSlot, ServiceTrip};
use time::{DateTime, Duration};

use crate::base_types::{
    DepotId, Distance, Location, NodeId, PassengerCount, VehicleCount, VehicleTypeId,
};
use crate::config::Config;
use crate::locations::Locations;
use crate::vehicle_types::VehicleTypes;

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use std::iter::Iterator;

use std::sync::Arc;

type SortedNodes = BTreeMap<(DateTime, NodeId), NodeId>;

pub struct Network {
    nodes: HashMap<NodeId, Node>,
    depots: HashMap<DepotId, (Depot, NodeId, NodeId)>, // depot, start_node, end_node

    // nodes are by default sorted by start_time (ties are broken by end_time then id)
    service_nodes: HashMap<VehicleTypeId, Vec<NodeId>>,
    maintenance_nodes: Vec<NodeId>,
    start_depot_nodes: Vec<NodeId>,
    end_depot_nodes: Vec<NodeId>,

    nodes_sorted_by_start: SortedNodes,

    vehicle_type_nodes_sorted_by_start: HashMap<VehicleTypeId, SortedNodes>,
    vehicle_type_nodes_sorted_by_end: HashMap<VehicleTypeId, SortedNodes>,

    config: Arc<Config>,
    locations: Arc<Locations>,
    vehicle_types: Arc<VehicleTypes>,

    // redundant information
    number_of_service_nodes: usize,
}

// methods
impl Network {
    pub fn vehicle_types(&self) -> Arc<VehicleTypes> {
        self.vehicle_types.clone()
    }

    pub fn config(&self) -> Arc<Config> {
        self.config.clone()
    }

    pub fn node(&self, id: NodeId) -> &Node {
        self.nodes.get(&id).unwrap()
    }

    /// return the number of nodes in the network.
    pub fn size(&self) -> usize {
        self.nodes.len()
    }

    pub fn service_nodes(&self, vehicle_type: VehicleTypeId) -> impl Iterator<Item = NodeId> + '_ {
        self.service_nodes[&vehicle_type].iter().copied()
    }

    pub fn all_service_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes_sorted_by_start
            .values()
            .filter(move |&n| self.node(*n).is_service())
            .copied()
    }

    pub fn number_of_service_nodes(&self) -> usize {
        self.number_of_service_nodes
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
        self.all_service_nodes().chain(self.maintenance_nodes())
    }

    pub fn locations(&self) -> &Locations {
        &self.locations
    }

    pub fn capacity_of(&self, depot_id: DepotId, vehicle_type_id: VehicleTypeId) -> PassengerCount {
        self.depots[&depot_id].0.capacity_for(vehicle_type_id)
    }

    pub fn total_capacity_of(&self, depot_id: DepotId) -> PassengerCount {
        self.depots[&depot_id].0.total_capacity()
    }

    pub fn vehicle_type_for(&self, service_trip: NodeId) -> VehicleTypeId {
        self.node(service_trip).as_service_trip().vehicle_type()
    }

    pub fn passengers_of(&self, service_trip: NodeId) -> PassengerCount {
        self.node(service_trip).as_service_trip().passengers()
    }

    pub fn seated_passengers_of(&self, service_trip: NodeId) -> PassengerCount {
        self.node(service_trip).as_service_trip().seated()
    }

    pub fn number_of_vehicles_required_to_serve(
        &self,
        vehicle_type: VehicleTypeId,
        service_trip: NodeId,
    ) -> VehicleCount {
        let service_trip = self.node(service_trip).as_service_trip();
        let vehicle_type = self.vehicle_types.get(vehicle_type).unwrap();
        service_trip
            .passengers()
            .div_ceil(vehicle_type.capacity())
            .max(service_trip.seated().div_ceil(vehicle_type.seats()))
            .max(1) // one vehicle is always required
    }

    pub fn get_depot_id(&self, node_id: NodeId) -> DepotId {
        self.node(node_id).as_depot().depot_id()
    }

    pub fn get_depot(&self, depot_id: DepotId) -> &Depot {
        &self.depots.get(&depot_id).unwrap().0
    }

    pub fn get_start_depot_node(&self, depot_id: DepotId) -> NodeId {
        self.depots.get(&depot_id).unwrap().1
    }

    pub fn get_end_depot_node(&self, depot_id: DepotId) -> NodeId {
        self.depots.get(&depot_id).unwrap().2
    }

    pub fn idle_time_between(&self, node1: NodeId, node2: NodeId) -> Duration {
        if self.node(node1).is_start_depot() || self.node(node2).is_end_depot() {
            return Duration::ZERO;
        }
        let idle_start = self.node(node1).end_time() + self.dead_head_time_between(node1, node2);
        let idle_end = self.node(node2).start_time();
        if idle_start <= idle_end {
            idle_end - idle_start
        } else {
            println!("negative idle time!");
            Duration::ZERO
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

    // TODO store predecssor of service trips within the same route
    // TODO connected by a route service trip can always be reached

    /// returns True iff node1 can reach node2
    /// but alswats False from start depot to start depot and end depot to end depot
    pub fn can_reach(&self, node1: NodeId, node2: NodeId) -> bool {
        let n1 = self.nodes.get(&node1).unwrap();
        let n2 = self.nodes.get(&node2).unwrap();

        if n2.is_start_depot() || n1.is_end_depot() {
            // start depots cannot be reached
            // end depots cannot reach anything
            return false;
        }

        n1.end_time() + self.minimal_duration_between_nodes_as_ref(n1, n2) <= n2.start_time()
    }

    /// provides all nodes of the given vehicle_type that are can be reached by node
    pub fn successors(
        &self,
        vehicle_type: VehicleTypeId,
        node: NodeId,
    ) -> impl Iterator<Item = NodeId> + '_ {
        self.vehicle_type_nodes_sorted_by_start[&vehicle_type]
            .range((self.node(node).end_time(), NodeId::smallest())..)
            .filter_map(move |(_, &n)| {
                if self.can_reach(node, n) {
                    Some(n)
                } else {
                    None
                }
            })
    }

    /// provides all nodes of the given vehicle_type that are can reach node
    pub fn predecessors(
        &self,
        vehicle_type: VehicleTypeId,
        node: NodeId,
    ) -> impl Iterator<Item = NodeId> + '_ {
        self.vehicle_type_nodes_sorted_by_end[&vehicle_type]
            .range(..(self.node(node).start_time(), NodeId::smallest()))
            .filter_map(move |(_, &n)| {
                if self.can_reach(n, node) {
                    Some(n)
                } else {
                    None
                }
            })
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

                self.config.shunting.minimal
            } else {
                // n2 is no service trip
                Duration::ZERO
            }
        } else {
            // n1 is no service trip
            Duration::ZERO
        }
    }

    fn shunting_duration_between_activities_if_dead_head_trip(
        &self,
        n1: &Node,
        n2: &Node,
    ) -> Duration {
        let previous: Duration = match n1 {
            Node::Service(_) => self.config.shunting.dead_head_trip,
            Node::Maintenance(_) => self.config.shunting.dead_head_trip,
            _ => Duration::ZERO,
        };

        let next: Duration = match n2 {
            Node::Service(_) => self.config.shunting.dead_head_trip,
            Node::Maintenance(_) => self.config.shunting.dead_head_trip,
            _ => Duration::ZERO,
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
        service_trips: HashMap<VehicleTypeId, Vec<ServiceTrip>>,
        maintenance_slots: Vec<MaintenanceSlot>,
        config: Arc<Config>,
        locations: Arc<Locations>,
        vehicle_types: Arc<VehicleTypes>,
    ) -> Network {
        let mut nodes = HashMap::new();
        let mut depots_lookup = HashMap::new();
        let mut service_nodes = HashMap::new();
        let mut maintenance_nodes = Vec::new();
        let mut start_depot_nodes = Vec::new();
        let mut end_depot_nodes = Vec::new();

        for depot in depots {
            let depot_id = depot.depot_id();

            let start_node_id = NodeId::start_depot_from(depot_id.0);
            let start_node_name = format!("Start Depot {} (at {})", depot_id, depot.location());
            let start_node = Node::create_start_depot_node(
                start_node_id,
                depot_id,
                depot.location(),
                start_node_name,
            );
            nodes.insert(start_node_id, start_node);
            start_depot_nodes.push(start_node_id);

            let end_node_id = NodeId::end_depot_from(depot_id.0);
            let end_node_name = format!("End Depot {} (at {})", depot_id, depot.location());
            let end_node =
                Node::create_end_depot_node(end_node_id, depot_id, depot.location(), end_node_name);
            nodes.insert(end_node_id, end_node);
            end_depot_nodes.push(end_node_id);
            depots_lookup.insert(depot_id, (depot, start_node_id, end_node_id));
        }

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

        for (vehicle_type, service_trips_of_type) in service_trips.into_iter() {
            let mut trips = Vec::new();
            for service_trip in service_trips_of_type.into_iter() {
                let id = service_trip.id();
                let node = Node::create_service_trip_node(service_trip);
                nodes.insert(id, node);
                trips.push(id);
            }
            trips.sort_by(|&n1, &n2| {
                nodes
                    .get(&n1)
                    .unwrap()
                    .cmp_start_time(nodes.get(&n2).unwrap())
            });
            service_nodes.insert(vehicle_type, trips);
        }

        for maintenance_slot in maintenance_slots.into_iter() {
            let id = maintenance_slot.id();
            let node = Node::create_maintenance_node(maintenance_slot);
            nodes.insert(id, node);
            maintenance_nodes.push(id);
        }

        maintenance_nodes.sort_by(|&n1, &n2| {
            nodes
                .get(&n1)
                .unwrap()
                .cmp_start_time(nodes.get(&n2).unwrap())
        });

        let nodes_sorted_by_start: SortedNodes = nodes
            .keys()
            .map(|&n| {
                let node = nodes.get(&n).unwrap();
                ((node.start_time(), n), n)
            })
            .collect();

        let vehicle_type_nodes_sorted_by_start: HashMap<VehicleTypeId, SortedNodes> = vehicle_types
            .iter()
            .map(|vt| {
                let nodes = service_nodes[&vt]
                    .iter()
                    .chain(maintenance_nodes.iter())
                    .chain(start_depot_nodes.iter())
                    .chain(end_depot_nodes.iter())
                    .map(|&n| {
                        let node = nodes.get(&n).unwrap();
                        ((node.start_time(), n), n)
                    })
                    .collect();
                (vt, nodes)
            })
            .collect();

        let vehicle_type_nodes_sorted_by_end: HashMap<VehicleTypeId, SortedNodes> = vehicle_types
            .iter()
            .map(|vt| {
                let nodes = service_nodes[&vt]
                    .iter()
                    .chain(maintenance_nodes.iter())
                    .chain(start_depot_nodes.iter())
                    .chain(end_depot_nodes.iter())
                    .map(|&n| {
                        let node = nodes.get(&n).unwrap();
                        ((node.end_time(), n), n)
                    })
                    .collect();
                (vt, nodes)
            })
            .collect();

        let number_of_service_nodes = service_nodes.values().map(|v| v.len()).sum();

        Network {
            nodes,
            depots: depots_lookup,
            service_nodes,
            maintenance_nodes,
            start_depot_nodes,
            end_depot_nodes,
            nodes_sorted_by_start,
            vehicle_type_nodes_sorted_by_start,
            vehicle_type_nodes_sorted_by_end,
            config,
            locations,
            vehicle_types,
            number_of_service_nodes,
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
