use time::{DateTime, Duration};

use crate::base_types::{DepotId, Distance, Location, NodeId, PassengerCount, VehicleTypeId};

use core::cmp::Ordering;

use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    // TODO store them as tuple (idx, DepotNode) etc.
    StartDepot(DepotNode),
    Service(ServiceTrip),
    Maintenance(MaintenanceSlot),
    EndDepot(DepotNode),
}

#[derive(Debug, PartialEq, Eq)]
pub struct DepotNode {
    idx: NodeId,
    depot_idx: DepotId,
    location: Location,
    id: String,
}

impl DepotNode {
    pub fn depot_idx(&self) -> DepotId {
        self.depot_idx
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ServiceTrip {
    idx: NodeId,
    id: String,
    vehicle_type: VehicleTypeId,
    origin: Location,
    destination: Location,
    departure: DateTime,
    arrival: DateTime,
    distance: Distance,
    passengers: PassengerCount,
    seated: PassengerCount,
}

impl ServiceTrip {
    pub fn idx(&self) -> NodeId {
        self.idx
    }

    pub fn vehicle_type(&self) -> VehicleTypeId {
        self.vehicle_type
    }

    pub fn passengers(&self) -> PassengerCount {
        self.passengers
    }

    pub fn seated(&self) -> PassengerCount {
        self.seated
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MaintenanceSlot {
    idx: NodeId,
    id: String,
    location: Location,
    start: DateTime,
    end: DateTime, // TODO add capacity, should be used for TraiFormation check
}

impl MaintenanceSlot {
    pub fn idx(&self) -> NodeId {
        self.idx
    }
}

// methods
impl Node {
    pub fn is_service(&self) -> bool {
        matches!(self, Node::Service(_))
    }

    pub fn is_maintenance(&self) -> bool {
        matches!(self, Node::Maintenance(_))
    }

    pub fn is_depot(&self) -> bool {
        matches!(self, Node::StartDepot(_) | Node::EndDepot(_))
    }

    pub fn is_start_depot(&self) -> bool {
        matches!(self, Node::StartDepot(_))
    }

    pub fn is_end_depot(&self) -> bool {
        matches!(self, Node::EndDepot(_))
    }

    pub fn idx(&self) -> NodeId {
        match self {
            Node::Service(s) => s.idx,
            Node::Maintenance(m) => m.idx,
            Node::StartDepot(d) => d.idx,
            Node::EndDepot(d) => d.idx,
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
            Node::StartDepot(_) => Duration::ZERO,
            Node::EndDepot(_) => Duration::ZERO,
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
            Node::Service(s) => s.distance,
            _ => Distance::ZERO,
        }
    }

    pub(crate) fn as_service_trip(&self) -> &ServiceTrip {
        match self {
            Node::Service(s) => s,
            _ => panic!("Node is not a service trip"),
        }
    }

    pub(crate) fn as_depot(&self) -> &DepotNode {
        match self {
            Node::StartDepot(d) => d,
            Node::EndDepot(d) => d,
            _ => panic!("Node is not a depot"),
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Node::Service(s) => &s.id,
            Node::Maintenance(m) => &m.id,
            Node::StartDepot(d) => &d.id,
            Node::EndDepot(d) => &d.id,
        }
    }

    /// compare to nodes according to the start_time (ties are broken by end_time and then id)
    pub fn cmp_start_time(&self, other: &Node) -> Ordering {
        self.start_time()
            .cmp(&other.start_time())
            .then(self.end_time().cmp(&other.end_time()))
            .then(self.idx().cmp(&other.idx()))
    }

    /// compare to nodes according to the end_time (ties are broken by start_time and then id)
    pub fn cmp_end_time(&self, other: &Node) -> Ordering {
        self.end_time()
            .cmp(&other.end_time())
            .then(self.start_time().cmp(&other.start_time()))
            .then(self.idx().cmp(&other.idx()))
    }

    pub fn print(&self) {
        match self {
            Node::Service(_) => println!(
                "{} (id: {}) from {} ({}) to {} ({}), {}",
                self.id(),
                self.idx(),
                self.start_location(),
                self.start_time(),
                self.end_location(),
                self.end_time(),
                self.travel_distance()
            ),
            Node::Maintenance(_) => println!(
                "{} (id: {}) at {} (from {} to {})",
                self.id(),
                self.idx(),
                self.start_location(),
                self.start_time(),
                self.end_time()
            ),
            Node::StartDepot(_) => println!("{} at {}", self.id(), self.start_location()),
            Node::EndDepot(_) => println!("{} at {}", self.id(), self.start_location()),
        }
    }
}

// static functions:
impl Node {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_service_trip(
        idx: NodeId,
        id: String,
        vehicle_type: VehicleTypeId,
        origin: Location,
        destination: Location,
        departure: DateTime,
        arrival: DateTime,
        distance: Distance,
        passengers: PassengerCount,
        seated: PassengerCount,
    ) -> ServiceTrip {
        ServiceTrip {
            idx,
            id,
            vehicle_type,
            origin,
            destination,
            departure,
            arrival,
            distance,
            passengers,
            seated,
        }
    }

    pub fn create_service_trip_node(service_trip: ServiceTrip) -> Node {
        Node::Service(service_trip)
    }

    pub(crate) fn create_maintenance(
        idx: NodeId,
        id: String,
        location: Location,
        start: DateTime,
        end: DateTime,
    ) -> MaintenanceSlot {
        MaintenanceSlot {
            idx,
            id,
            location,
            start,
            end,
        }
    }

    pub fn create_maintenance_node(maintenance_slot: MaintenanceSlot) -> Node {
        Node::Maintenance(maintenance_slot)
    }

    pub(crate) fn create_start_depot_node(
        idx: NodeId,
        id: String,
        depot_idx: DepotId,
        location: Location,
    ) -> Node {
        Node::StartDepot(DepotNode {
            idx,
            id,
            depot_idx,
            location,
        })
    }

    pub(crate) fn create_end_depot_node(
        idx: NodeId,
        id: String,
        depot_idx: DepotId,
        location: Location,
    ) -> Node {
        Node::EndDepot(DepotNode {
            idx,
            id,
            depot_idx,
            location,
        })
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id())
    }
}
