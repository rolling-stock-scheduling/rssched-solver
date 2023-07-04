pub mod nodes;
use nodes::Node;

pub mod demand;
use demand::Demand;

use crate::base_types::{DateTime, Distance, Duration, LocationId, NodeId, StationSide, VehicleId};
use crate::config::Config;
use crate::locations::Locations;
use crate::vehicles::{VehicleType, Vehicles};

use std::collections::HashMap;
use std::fmt;

use std::iter::Iterator;

use std::sync::Arc;

pub struct Network {
    nodes: HashMap<NodeId, Node>,

    // nodes are by default sorted by start_time
    service_nodes: Vec<NodeId>,
    maintenance_nodes: Vec<NodeId>,
    start_nodes: HashMap<VehicleId, NodeId>,
    end_nodes: Vec<NodeId>,
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

    pub fn end_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.end_nodes.iter().copied()
    }

    pub fn start_node_of(&self, vehicle_id: VehicleId) -> NodeId {
        *self.start_nodes.get(&vehicle_id).unwrap()
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
        // TODO: Adjust if it is a recommended activity_link
        self.loc.travel_time(
            self.node(node1).end_location(),
            self.node(node2).start_location(),
        )
    }

    pub fn dead_head_distance_between(&self, node1: NodeId, node2: NodeId) -> Distance {
        // TODO: Adjust if it is a recommended activity_link
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
        // TODO: Check if the nodes are present as activity links: JOINT or REFERENCE.
        // if yes, return Duration::zero()

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
        if let Node::Service(s1) = n1 {
            if let Node::Service(s2) = n2 {
                // both nodes are service trips

                if n1.demand() != n2.demand() {
                    // different demands mean a new coupling is needed
                    self.config.durations_between_activities.coupling
                } else {
                    if s1.arrival_side() == s2.departure_side() {
                        // turn
                        self.config.durations_between_activities.turn
                    } else {
                        // minimum
                        self.config.durations_between_activities.minimal
                    }
                }
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
        let (dht_start_side, dht_end_side) = self
            .loc
            .station_sides(n1.end_location(), n2.start_location());
        let previous: Duration = match n1 {
            Node::Service(s1) => {
                if dht_start_side == s1.departure_side() {
                    // turn
                    self.config.durations_between_activities.dead_head_trip
                        + self.config.durations_between_activities.turn
                } else {
                    // no turn
                    self.config.durations_between_activities.dead_head_trip
                }
            }
            Node::Maintenance(_) => self.config.durations_between_activities.dead_head_trip,
            _ => Duration::zero(),
        };

        let next: Duration = match n2 {
            Node::Service(s2) => {
                if dht_end_side == s2.arrival_side() {
                    // turn
                    self.config.durations_between_activities.dead_head_trip
                        + self.config.durations_between_activities.turn
                } else {
                    // no turn
                    self.config.durations_between_activities.dead_head_trip
                }
            }
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

    pub fn minimal_overhead(&self) -> Duration {
        let earliest_start_time = self
            .start_nodes
            .values()
            .map(|n| self.node(*n).end_time())
            .min()
            .unwrap();
        self.end_nodes
            .iter()
            .map(|n| self.node(*n).start_time() - earliest_start_time)
            .sum::<Duration>()
            - self
                .start_nodes
                .values()
                .map(|n| self.node(*n).end_time() - earliest_start_time)
                .sum()
            - self.total_useful_duration()
    }

    fn total_useful_duration(&self) -> Duration {
        self.service_nodes
            .iter()
            .chain(self.maintenance_nodes.iter())
            .map(|n| {
                (0..self.node(*n).demand().number_of_vehicles())
                    .map(|_| self.node(*n).duration())
                    .sum()
            })
            .sum()
        // node that service trips are counted as big as their demand is
    }
}

// static functions
impl Network {
    pub fn load_from_csv(
        path_service: &str,
        path_maintenance: &str,
        path_endpoints: &str,
        config: Arc<Config>,
        loc: Arc<Locations>,
        vehicles: Arc<Vehicles>,
    ) -> Network {
        let mut nodes: HashMap<NodeId, Node> = HashMap::new();

        let mut service_nodes: Vec<NodeId> = Vec::new();
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_path(path_service)
            .expect("csv-file for loading service_trips not found");
        for (i, result) in reader.records().enumerate() {
            let record = result.expect("Some recond cannot be read while reading service_trips");
            let _driving_day = record.get(0).unwrap();
            let _train_number = record.get(1).unwrap();
            let start_time = DateTime::new(record.get(2).unwrap());
            let start_location = loc.get_location(LocationId::from(record.get(3).unwrap()));
            let start_side = StationSide::from(record.get(4).unwrap());
            let end_time = DateTime::new(record.get(5).unwrap());
            let end_location = loc.get_location(LocationId::from(record.get(6).unwrap()));
            let end_side = StationSide::from(record.get(7).unwrap());
            let length = Distance::from_km(record.get(8).unwrap().parse().unwrap());
            let demand_amount: u8 = record.get(9).unwrap().parse().unwrap();
            let id = NodeId::from(&format!("ST:{}", record.get(10).unwrap()));
            let name = format!("{}-{}:{}", start_location, end_location, i);

            let service_trip = Node::create_service_node(
                id,
                start_location,
                end_location,
                start_time,
                end_time,
                start_side,
                end_side,
                length,
                Demand::new(demand_amount),
                name,
            );
            nodes.insert(id, service_trip);
            service_nodes.push(id);
        }

        let mut maintenance_nodes: Vec<NodeId> = Vec::new();
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_path(path_maintenance)
            .expect("csv-file for loading maintenance_slots not found");
        for result in reader.records() {
            let record =
                result.expect("Some recond cannot be read while reading maintenance_slots");
            let location = loc.get_location(LocationId::from(record.get(0).unwrap()));
            let start_time = DateTime::new(record.get(1).unwrap());
            let end_time = DateTime::new(record.get(2).unwrap());
            let id = NodeId::from(&format!("MS:{}", record.get(3).unwrap()));
            let name = format!("!{}:{}!", location, record.get(3).unwrap());

            let maintenance_slot =
                Node::create_maintenance_node(id, location, start_time, end_time, name);
            nodes.insert(id, maintenance_slot);
            maintenance_nodes.push(id);
        }

        let mut start_nodes: HashMap<VehicleId, NodeId> = HashMap::new();
        for vehicle_id in vehicles.iter() {
            let vehicle = vehicles.get_vehicle(vehicle_id);
            let node_id = NodeId::from(&format!("SN:{}", vehicle_id));
            let name = format!("|{}@{}", vehicle_id, vehicle.start_location());
            let start_node = Node::create_start_node(
                node_id,
                vehicle_id,
                vehicle.start_location(),
                vehicle.start_time(),
                name,
            );
            nodes.insert(node_id, start_node);
            start_nodes.insert(vehicle_id, node_id);
        }

        let mut end_nodes: Vec<NodeId> = Vec::new();
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_path(path_endpoints)
            .expect("csv-file for loading end_points not found");
        for result in reader.records() {
            let record = result.expect("Some recond cannot be read while reading end_points");
            let id = NodeId::from(&format!("EN:{}", record.get(0).unwrap()));
            let vehicle_type = VehicleType::Standard;
            let time = DateTime::new(record.get(1).unwrap());
            let location = loc.get_location(LocationId::from(record.get(2).unwrap()));
            let duration_till_maintenance = Duration::from_iso(record.get(3).unwrap());
            let dist_till_maintenance = Distance::from_km(record.get(4).unwrap().parse().unwrap());
            let name = format!("{}:{}|", location, record.get(0).unwrap());

            let end_point = Node::create_end_node(
                id,
                vehicle_type,
                location,
                time,
                duration_till_maintenance,
                dist_till_maintenance,
                name,
            );
            nodes.insert(id, end_point);
            end_nodes.push(id);
        }

        let mut nodes_sorted_by_start: Vec<NodeId> = nodes.keys().copied().collect();
        nodes_sorted_by_start.sort_by(|n1, n2| {
            nodes
                .get(n1)
                .unwrap()
                .cmp_start_time(nodes.get(n2).unwrap())
        });
        let mut nodes_sorted_by_end: Vec<NodeId> = nodes.keys().copied().collect();
        nodes_sorted_by_end
            .sort_by(|n1, n2| nodes.get(n1).unwrap().cmp_end_time(nodes.get(n2).unwrap()));

        // sort all indices by the start_time
        service_nodes.sort_by(|n1, n2| {
            nodes
                .get(n1)
                .unwrap()
                .cmp_start_time(nodes.get(n2).unwrap())
        });
        maintenance_nodes.sort_by(|n1, n2| {
            nodes
                .get(n1)
                .unwrap()
                .cmp_start_time(nodes.get(n2).unwrap())
        });
        end_nodes.sort_by(|n1, n2| {
            nodes
                .get(n1)
                .unwrap()
                .cmp_start_time(nodes.get(n2).unwrap())
        });

        Network {
            nodes,
            service_nodes,
            maintenance_nodes,
            start_nodes,
            end_nodes,
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
