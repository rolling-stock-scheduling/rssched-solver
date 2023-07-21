use crate::base_types::{
    DateTime, Distance, Duration, Location, NodeId, PassengerCount, StationSide, VehicleTypeId,
};
use crate::depots::Depot;

use core::cmp::Ordering;

use std::fmt;

pub enum Node {
    Service(ServiceTrip),
    Maintenance(MaintenanceSlot),
    Depot(DepotNode),
}

pub struct ServiceTrip {
    id: NodeId,
    origin: Location,
    destination: Location,
    departure: DateTime,
    arrival: DateTime,
    departure_side: StationSide,
    arrival_side: StationSide,
    travel_distance: Distance,
    demand: PassengerCount,
    name: String,
}

pub struct MaintenanceSlot {
    id: NodeId,
    location: Location,
    start: DateTime,
    end: DateTime,
    name: String,
}

pub struct DepotNode {
    node_id: NodeId,
    depot: Depot,
}

impl ServiceTrip {
    pub fn departure_side(&self) -> StationSide {
        self.departure_side
    }

    pub fn arrival_side(&self) -> StationSide {
        self.arrival_side
    }
}

// methods
impl Node {
    pub fn id(&self) -> NodeId {
        match self {
            Node::Service(s) => s.id,
            Node::Maintenance(m) => m.id,
            Node::Depot(d) => d.node_id,
        }
    }

    pub fn start_time(&self) -> DateTime {
        match self {
            Node::Service(s) => s.departure,
            Node::Maintenance(m) => m.start,
            Node::Depot(_) => DateTime::Latest, // Depots can reach all nodes
        }
    }

    pub fn end_time(&self) -> DateTime {
        match self {
            Node::Service(s) => s.arrival,
            Node::Maintenance(m) => m.end,
            Node::Depot(_) => DateTime::Earliest, // Depots can be reached from all nodes
        }
    }

    pub fn duration(&self) -> Duration {
        match self {
            Node::Depot(_) => Duration::zero(),
            _ => self.end_time() - self.start_time(),
        }
    }

    pub fn start_location(&self) -> Location {
        match self {
            Node::Service(s) => s.origin,
            Node::Maintenance(m) => m.location,
            Node::Depot(d) => d.depot.location(),
        }
    }

    pub fn end_location(&self) -> Location {
        match self {
            Node::Service(s) => s.destination,
            Node::Maintenance(m) => m.location,
            Node::Depot(d) => d.depot.location(),
        }
    }

    pub fn travel_distance(&self) -> Distance {
        match self {
            Node::Service(s) => s.travel_distance,
            _ => Distance::zero(),
        }
    }

    pub fn vehicle_type(&self) -> VehicleTypeId {
        match self {
            Node::Depot(d) => d.depot.vehicle_type(),
            _ => panic!("Node is not an Depot."),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Node::Service(s) => &s.name,
            Node::Maintenance(m) => &m.name,
            Node::Depot(d) => &d.depot.id().to_string(),
        }
    }

    /// compare to nodes according to the start_time (ties are broken by end_time and then id)
    pub fn cmp_start_time(&self, other: &Node) -> Ordering {
        self.start_time()
            .cmp(&other.start_time())
            .then(self.end_time().cmp(&other.end_time()))
            .then(self.id().cmp(&other.id()))
    }

    /// compare to nodes according to the end_time (ties are broken by start_time and then id)
    pub fn cmp_end_time(&self, other: &Node) -> Ordering {
        self.end_time()
            .cmp(&other.end_time())
            .then(self.start_time().cmp(&other.start_time()))
            .then(self.id().cmp(&other.id()))
    }

    pub fn print(&self) {
        match self {
            Node::Service(s) => println!(
                "{} (id: {}) from {} ({}) to {} ({}), {}",
                self.name(),
                self.id(),
                self.start_location(),
                self.start_time(),
                self.end_location(),
                self.end_time(),
                self.travel_distance()
            ),
            Node::Maintenance(m) => println!(
                "{} (id: {}) at {} (from {} to {})",
                self.name(),
                self.id(),
                self.start_location(),
                self.start_time(),
                self.end_time()
            ),
            Node::Depot(d) => println!(
                "{} of {} at {}",
                self.name(),
                self.vehicle_type(),
                self.start_location()
            ),
        }
    }
}

// static functions:
impl Node {
    // factory for creating a service trip
    pub(super) fn create_service_node(
        id: NodeId,
        origin: Location,
        destination: Location,
        departure: DateTime,
        arrival: DateTime,
        departure_side: StationSide,
        arrival_side: StationSide,
        travel_distance: Distance,
        demand: PassengerCount,
        name: String,
    ) -> Node {
        Node::Service(ServiceTrip {
            id,
            origin,
            destination,
            departure,
            arrival,
            departure_side,
            arrival_side,
            travel_distance,
            demand,
            name,
        })
    }

    // factory for creating a node for a maintenance slot
    pub(super) fn create_maintenance_node(
        id: NodeId,
        location: Location,
        start: DateTime,
        end: DateTime,
        name: String,
    ) -> Node {
        Node::Maintenance(MaintenanceSlot {
            id,
            location,
            start,
            end,
            name,
        })
    }

    pub(super) fn create_depot_node(node_id: NodeId, depot: Depot) -> Node {
        Node::Depot(DepotNode { node_id, depot })
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
