use time::{DateTime, Duration};

use crate::base_types::{Distance, Location, NodeId, PassengerCount, StationSide, VehicleTypeId};

use core::cmp::Ordering;

use std::{collections::HashMap, fmt};

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Service(ServiceTrip),
    Maintenance(MaintenanceSlot),
    StartDepot(Depot),
    EndDepot(Depot),
}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub struct MaintenanceSlot {
    id: NodeId,
    location: Location,
    start: DateTime,
    end: DateTime,
    name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Depot {
    id: NodeId,
    location: Location,
    capacities: HashMap<VehicleTypeId, Option<PassengerCount>>, // number of vehicles that can be spawned. None means no limit.
    name: String,
}

impl Depot {
    /// None means no limit
    pub fn capacitiy_for(&self, vehicle_type_id: VehicleTypeId) -> Option<PassengerCount> {
        *self.capacities.get(&vehicle_type_id).unwrap_or(&Some(0))
    }
}

impl ServiceTrip {
    pub fn departure_side(&self) -> StationSide {
        self.departure_side
    }

    pub fn arrival_side(&self) -> StationSide {
        self.arrival_side
    }

    pub fn demand(&self) -> PassengerCount {
        self.demand
    }
}

// methods
impl Node {
    pub fn is_service(&self) -> bool {
        match self {
            Node::Service(_) => true,
            _ => false,
        }
    }

    pub fn is_maintenance(&self) -> bool {
        match self {
            Node::Maintenance(_) => true,
            _ => false,
        }
    }

    pub fn is_depot(&self) -> bool {
        match self {
            Node::StartDepot(_) => true,
            Node::EndDepot(_) => true,
            _ => false,
        }
    }

    pub fn is_start_depot(&self) -> bool {
        match self {
            Node::StartDepot(_) => true,
            _ => false,
        }
    }

    pub fn is_end_depot(&self) -> bool {
        match self {
            Node::EndDepot(_) => true,
            _ => false,
        }
    }

    pub fn id(&self) -> NodeId {
        match self {
            Node::Service(s) => s.id,
            Node::Maintenance(m) => m.id,
            Node::StartDepot(d) => d.id,
            Node::EndDepot(d) => d.id,
        }
    }

    pub fn start_time(&self) -> DateTime {
        match self {
            Node::Service(s) => s.departure,
            Node::Maintenance(m) => m.start,
            Node::StartDepot(_) => DateTime::Earliest, // start depots can not be reached by any nodes
            Node::EndDepot(_) => DateTime::Latest,     // end depots can be reached by all nodes
        }
    }

    pub fn end_time(&self) -> DateTime {
        match self {
            Node::Service(s) => s.arrival,
            Node::Maintenance(m) => m.end,
            Node::StartDepot(_) => DateTime::Earliest, // start depots can reach all nodes
            Node::EndDepot(_) => DateTime::Latest,     // end depots cannot reach any nodes
        }
    }

    pub fn duration(&self) -> Duration {
        match self {
            Node::StartDepot(_) => Duration::zero(),
            Node::EndDepot(_) => Duration::zero(),
            _ => self.end_time() - self.start_time(),
        }
    }

    pub fn start_location(&self) -> Location {
        match self {
            Node::Service(s) => s.origin,
            Node::Maintenance(m) => m.location,
            Node::StartDepot(d) => d.location,
            Node::EndDepot(d) => d.location,
        }
    }

    pub fn end_location(&self) -> Location {
        match self {
            Node::Service(s) => s.destination,
            Node::Maintenance(m) => m.location,
            Node::StartDepot(d) => d.location,
            Node::EndDepot(d) => d.location,
        }
    }

    pub fn travel_distance(&self) -> Distance {
        match self {
            Node::Service(s) => s.travel_distance,
            _ => Distance::zero(),
        }
    }

    pub fn as_depot(&self) -> &Depot {
        match self {
            Node::StartDepot(d) => d,
            Node::EndDepot(d) => d,
            _ => panic!("Node is not a depot"),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Node::Service(s) => &s.name,
            Node::Maintenance(m) => &m.name,
            Node::StartDepot(d) => &d.name,
            Node::EndDepot(d) => &d.name,
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
            Node::Service(_) => println!(
                "{} (id: {}) from {} ({}) to {} ({}), {}",
                self.name(),
                self.id(),
                self.start_location(),
                self.start_time(),
                self.end_location(),
                self.end_time(),
                self.travel_distance()
            ),
            Node::Maintenance(_) => println!(
                "{} (id: {}) at {} (from {} to {})",
                self.name(),
                self.id(),
                self.start_location(),
                self.start_time(),
                self.end_time()
            ),
            Node::StartDepot(_) => println!("{} at {}", self.name(), self.start_location()),
            Node::EndDepot(_) => println!("{} at {}", self.name(), self.start_location()),
        }
    }
}

// static functions:
impl Node {
    // factory for creating a service trip
    pub(crate) fn create_service_node(
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
    pub(crate) fn create_maintenance_node(
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

    pub(crate) fn create_start_depot_node(
        id: NodeId,
        location: Location,
        capacities: HashMap<VehicleTypeId, Option<PassengerCount>>,
        name: String,
    ) -> Node {
        Node::StartDepot(Depot {
            id,
            location,
            capacities,
            name,
        })
    }

    pub(crate) fn create_end_depot_node(
        id: NodeId,
        location: Location,
        capacities: HashMap<VehicleTypeId, Option<PassengerCount>>,
        name: String,
    ) -> Node {
        Node::EndDepot(Depot {
            id,
            location,
            capacities,
            name,
        })
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
