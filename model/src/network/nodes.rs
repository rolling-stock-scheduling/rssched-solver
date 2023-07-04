use super::demand::Demand;
use crate::base_types::{DateTime, Distance, Duration, Location, NodeId, StationSide, VehicleId};
use crate::vehicles::VehicleType;

use core::cmp::Ordering;

use std::fmt;

pub enum Node {
    Service(ServiceTrip),
    Maintenance(MaintenanceSlot),
    Start(StartPoint),
    End(EndPoint),
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
    demand: Demand,
    name: String,
}

pub struct MaintenanceSlot {
    id: NodeId,
    location: Location,
    start: DateTime,
    end: DateTime,
    name: String,
}

pub struct StartPoint {
    node_id: NodeId,
    vehicle_id: VehicleId,
    location: Location,
    time: DateTime,
    name: String,
}

pub struct EndPoint {
    id: NodeId,
    vehicle_type: VehicleType,
    location: Location,
    time: DateTime,
    duration_till_maintenance: Duration,
    dist_till_maintenance: Distance,
    name: String,
}

impl EndPoint {
    pub fn dist_till_maintenance(&self) -> Distance {
        self.dist_till_maintenance
    }

    pub fn duration_till_maintenance(&self) -> Duration {
        self.duration_till_maintenance
    }
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
            Node::Start(s) => s.node_id,
            Node::End(n) => n.id,
        }
    }

    pub fn start_time(&self) -> DateTime {
        match self {
            Node::Service(s) => s.departure,
            Node::Maintenance(m) => m.start,
            Node::Start(_) => DateTime::Earliest,
            Node::End(e) => e.time,
        }
    }

    pub fn end_time(&self) -> DateTime {
        match self {
            Node::Service(s) => s.arrival,
            Node::Maintenance(m) => m.end,
            Node::Start(s) => s.time,
            Node::End(_) => DateTime::Latest,
        }
    }

    pub fn duration(&self) -> Duration {
        self.end_time() - self.start_time()
    }

    pub fn useful_duration(&self) -> Duration {
        if matches!(self, Node::Service(_)) || matches!(self, Node::Maintenance(_)) {
            self.duration()
        } else {
            Duration::zero()
        }
    }

    pub fn start_location(&self) -> Location {
        match self {
            Node::Service(s) => s.origin,
            Node::Maintenance(m) => m.location,
            Node::Start(_) => Location::Nowhere,
            Node::End(e) => e.location,
        }
    }

    pub fn end_location(&self) -> Location {
        match self {
            Node::Service(s) => s.destination,
            Node::Maintenance(m) => m.location,
            Node::Start(s) => s.location,
            Node::End(_) => Location::Nowhere,
        }
    }

    pub fn travel_distance(&self) -> Distance {
        match self {
            Node::Service(s) => s.travel_distance,
            _ => Distance::zero(),
        }
    }

    pub fn vehicle_type(&self) -> VehicleType {
        match self {
            Node::End(e) => e.vehicle_type,
            _ => panic!("Node is not an EndPoint."),
        }
    }

    pub fn demand(&self) -> Demand {
        match self {
            Node::Service(s) => s.demand,
            Node::Maintenance(_) => Demand::new(1),
            Node::Start(_) => Demand::new(1),
            Node::End(_) => Demand::new(1),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Node::Service(s) => &s.name,
            Node::Maintenance(m) => &m.name,
            Node::Start(s) => &s.name,
            Node::End(e) => &e.name,
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
                s.name, s.id, s.origin, s.departure, s.destination, s.arrival, s.travel_distance
            ),
            Node::Maintenance(m) => println!(
                "{} (id: {}) at {} (from {} to {})",
                m.name, m.id, m.location, m.start, m.end
            ),
            Node::Start(s) => println!(
                "{} (id: {}) of {} at {} ({})",
                s.name, s.node_id, s.vehicle_id, s.location, s.time
            ),
            Node::End(e) => println!(
                "{} (id: {}) for {:?} at {} ({})",
                e.name, e.id, e.vehicle_type, e.location, e.time
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
        demand: Demand,
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

    // factory for creating start and end node of a vehicle
    pub(super) fn create_start_node(
        node_id: NodeId,
        vehicle_id: VehicleId,
        location: Location,
        time: DateTime,
        name: String,
    ) -> Node {
        Node::Start(StartPoint {
            node_id,
            vehicle_id,
            location,
            time,
            name,
        })
    }

    pub(super) fn create_end_node(
        id: NodeId,
        vehicle_type: VehicleType,
        location: Location,
        time: DateTime,
        duration_till_maintenance: Duration,
        dist_till_maintenance: Distance,
        name: String,
    ) -> Node {
        Node::End(EndPoint {
            id,
            vehicle_type,
            location,
            time,
            duration_till_maintenance,
            dist_till_maintenance,
            name,
        })
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
