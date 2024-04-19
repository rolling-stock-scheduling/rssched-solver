use time::{DateTime, Duration};

use crate::base_types::{
    DepotIdx, Distance, Idx, Location, NodeIdx, PassengerCount, VehicleCount, VehicleTypeIdx,
};

use core::cmp::Ordering;

use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    StartDepot((NodeIdx, DepotNode)),
    Service((NodeIdx, ServiceTrip)),
    Maintenance((NodeIdx, MaintenanceSlot)),
    EndDepot((NodeIdx, DepotNode)),
}

#[derive(Debug, PartialEq, Eq)]
pub struct DepotNode {
    depot_idx: DepotIdx,
    location: Location,
    id: String,
}

impl DepotNode {
    pub fn depot_idx(&self) -> DepotIdx {
        self.depot_idx
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ServiceTrip {
    id: String,
    vehicle_type: VehicleTypeIdx,
    origin: Location,
    destination: Location,
    departure: DateTime,
    arrival: DateTime,
    distance: Distance,
    passengers: PassengerCount,
    seated: PassengerCount,
    maximal_formation_count: Option<VehicleCount>,
}

impl ServiceTrip {
    pub fn vehicle_type(&self) -> VehicleTypeIdx {
        self.vehicle_type
    }

    pub fn passengers(&self) -> PassengerCount {
        self.passengers
    }

    pub fn seated(&self) -> PassengerCount {
        self.seated
    }

    pub fn maximal_formation_count(&self) -> Option<VehicleCount> {
        self.maximal_formation_count
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MaintenanceSlot {
    id: String,
    location: Location,
    start: DateTime,
    end: DateTime,
    track_count: VehicleCount,
}

impl MaintenanceSlot {
    pub fn track_count(&self) -> VehicleCount {
        self.track_count
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

    pub fn idx(&self) -> NodeIdx {
        match self {
            Node::Service((idx, _)) => *idx,
            Node::Maintenance((idx, _)) => *idx,
            Node::StartDepot((idx, _)) => *idx,
            Node::EndDepot((idx, _)) => *idx,
        }
    }

    pub fn start_time(&self) -> DateTime {
        match self {
            Node::Service((_, s)) => s.departure,
            Node::Maintenance((_, m)) => m.start,
            Node::StartDepot(_) => DateTime::Earliest, // start depots can not be reached by any nodes
            Node::EndDepot(_) => DateTime::Latest,     // end depots can be reached by all nodes
        }
    }

    pub fn end_time(&self) -> DateTime {
        match self {
            Node::Service((_, s)) => s.arrival,
            Node::Maintenance((_, m)) => m.end,
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
            Node::Service((_, s)) => s.origin,
            Node::Maintenance((_, m)) => m.location,
            Node::StartDepot((_, d)) => d.location,
            Node::EndDepot((_, d)) => d.location,
        }
    }

    pub fn end_location(&self) -> Location {
        match self {
            Node::Service((_, s)) => s.destination,
            Node::Maintenance((_, m)) => m.location,
            Node::StartDepot((_, d)) => d.location,
            Node::EndDepot((_, d)) => d.location,
        }
    }

    pub fn travel_distance(&self) -> Distance {
        match self {
            Node::Service((_, s)) => s.distance,
            _ => Distance::ZERO,
        }
    }

    pub(crate) fn as_service_trip(&self) -> &ServiceTrip {
        match self {
            Node::Service((_, s)) => s,
            _ => panic!("Node is not a service trip"),
        }
    }

    pub(crate) fn as_maintenance(&self) -> &MaintenanceSlot {
        match self {
            Node::Maintenance((_, m)) => m,
            _ => panic!("Node is not a maintenance slot"),
        }
    }

    pub(crate) fn as_depot(&self) -> &DepotNode {
        match self {
            Node::StartDepot((_, d)) => d,
            Node::EndDepot((_, d)) => d,
            _ => panic!("Node is not a depot"),
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Node::Service((_, s)) => &s.id,
            Node::Maintenance((_, m)) => &m.id,
            Node::StartDepot((_, d)) => &d.id,
            Node::EndDepot((_, d)) => &d.id,
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
                "{} (idx: {}) from {} ({}) to {} ({}), {}",
                self.id(),
                self.idx(),
                self.start_location(),
                self.start_time(),
                self.end_location(),
                self.end_time(),
                self.travel_distance()
            ),
            Node::Maintenance((_, m)) => println!(
                "{} (idx: {}) at {} (from {} to {} with {} tracks)",
                self.id(),
                self.idx(),
                self.start_location(),
                self.start_time(),
                self.end_time(),
                m.track_count()
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
        id: String,
        vehicle_type: VehicleTypeIdx,
        origin: Location,
        destination: Location,
        departure: DateTime,
        arrival: DateTime,
        distance: Distance,
        passengers: PassengerCount,
        seated: PassengerCount,
        maximal_formation_count: Option<VehicleCount>,
    ) -> ServiceTrip {
        ServiceTrip {
            id,
            vehicle_type,
            origin,
            destination,
            departure,
            arrival,
            distance,
            passengers,
            seated,
            maximal_formation_count,
        }
    }

    pub fn create_service_trip_node(idx: Idx, service_trip: ServiceTrip) -> Node {
        Node::Service((NodeIdx::service_from(idx), service_trip))
    }

    pub(crate) fn create_maintenance(
        id: String,
        location: Location,
        start: DateTime,
        end: DateTime,
        track_count: VehicleCount,
    ) -> MaintenanceSlot {
        MaintenanceSlot {
            id,
            location,
            start,
            end,
            track_count,
        }
    }

    pub fn create_maintenance_node(idx: Idx, maintenance_slot: MaintenanceSlot) -> Node {
        Node::Maintenance((NodeIdx::maintenance_from(idx), maintenance_slot))
    }

    pub(crate) fn create_start_depot_node(
        idx: Idx,
        id: String,
        depot_idx: DepotIdx,
        location: Location,
    ) -> Node {
        Node::StartDepot((
            NodeIdx::start_depot_from(idx),
            DepotNode {
                id,
                depot_idx,
                location,
            },
        ))
    }

    pub(crate) fn create_end_depot_node(
        idx: Idx,
        id: String,
        depot_idx: DepotIdx,
        location: Location,
    ) -> Node {
        Node::EndDepot((
            NodeIdx::end_depot_from(idx),
            DepotNode {
                id,
                depot_idx,
                location,
            },
        ))
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id())
    }
}
