use crate::time::{Time,Duration};
use crate::locations::Location;
use crate::units::{Unit,UnitType};
use crate::distance::Distance;
use crate::base_types::{NodeId,UnitId,Penalty};
use super::demand::Demand;
use crate::schedule::train_formation::TrainFormation;

use core::cmp::Ordering;


use std::fmt;

use crate::base_types::{PENALTY_ZERO, PENALTY_UNUSED_MAINTENANCE,PENALTY_INF};

pub(crate) enum Node {
    Service(ServiceTrip),
    Maintenance(MaintenanceSlot),
    Start(StartPoint),
    End(EndPoint)
}


pub(crate) struct ServiceTrip {
    id: NodeId,
    origin: Location,
    destination: Location,
    departure: Time,
    arrival: Time,
    travel_distance: Distance,
    demand: Demand,
}

pub(crate) struct MaintenanceSlot {
    id: NodeId,
    location: Location,
    start: Time,
    end: Time,
}

pub(crate) struct StartPoint {
    node_id: NodeId,
    unit_id: UnitId,
    location: Location,
    time: Time,
}

pub(crate) struct EndPoint {
    id: NodeId,
    unit_type: UnitType,
    location: Location,
    time: Time,
    duration_till_maintenance: Duration,
    dist_till_maintenance: Distance,
}

// methods
impl Node {
    pub(crate) fn id(&self) -> NodeId {
        match self {
            Node::Service(s) => s.id,
            Node::Maintenance(m) => m.id,
            Node::Start(s) => s.node_id,
            Node::End(n) => n.id
        }
    }

    pub(crate) fn start_time(&self) -> Time {
        match self {
            Node::Service(s) => s.departure,
            Node::Maintenance(m) => m.start,
            Node::Start(_) => Time::Earliest,
            Node::End(e) => e.time
        }
    }

    pub(crate) fn end_time(&self) -> Time {
        match self {
            Node::Service(s) => s.arrival,
            Node::Maintenance(m) => m.end,
            Node::Start(s) => s.time,
            Node::End(_) => Time::Latest
        }
    }

    pub(crate) fn duration(&self) -> Duration {
        self.end_time() - self.start_time()
    }

    pub(crate) fn start_location(&self) -> Location {
        match self {
            Node::Service(s) => s.origin,
            Node::Maintenance(m) => m.location,
            Node::Start(_) => Location::Nowhere,
            Node::End(e) => e.location
        }
    }

    pub(crate) fn end_location(&self) -> Location {
        match self {
            Node::Service(s) => s.destination,
            Node::Maintenance(m) => m.location,
            Node::Start(s) => s.location,
            Node::End(_) => Location::Nowhere
        }
    }

    pub(crate) fn travel_distance(&self) -> Distance {
        match self {
            Node::Service(s) => s.travel_distance,
            _ => Distance::zero()
        }
    }

    pub(crate) fn unit_type(&self) -> UnitType {
        match self {
            Node::End(e) => e.unit_type,
            _ => panic!("Node is not an EndPoint.")
        }
    }

    pub(crate) fn travel_time(&self) -> Duration {
        match self {
            Node::Service(s) => s.arrival - s.departure,
            _ => Duration::zero()
        }
    }

    pub(crate) fn demand(&self) -> Demand {
        match self {
            Node::Service(s) => s.demand,
            Node::Maintenance(_) => Demand::new(1),
            Node::Start(_) => Demand::new(1),
            Node::End(_) => Demand::new(1)
        }
    }

    pub(crate) fn cover_penalty(&self, train: &TrainFormation) -> Penalty {
        match self {
            Node::Service(s) => s.demand.compute_penalty(train),
            Node::Maintenance(m) => if train.len() == 1 {PENALTY_ZERO} else {PENALTY_UNUSED_MAINTENANCE},
            Node::Start(s) => if train.len() == 1 && train.iter().next().unwrap() == s.unit_id {PENALTY_ZERO} else {PENALTY_INF},
            Node::End(e) => if train.len() == 1 && train.get_as_units().first().unwrap().unit_type() == e.unit_type {PENALTY_ZERO} else {PENALTY_INF}
        }
    }

    /// compare to nodes according to the start_time (ties are broken by end_time and then id)
    pub(crate) fn cmp_start_time(&self, other: &Node)  -> Ordering {
        match self.start_time().partial_cmp(&other.start_time()) {
            Some(Ordering::Equal) =>
                match self.end_time().partial_cmp(&other.end_time()) {
                    Some(Ordering::Equal) => self.id().partial_cmp(&other.id()),
                    other => other
                }
            other => other
        }.unwrap()
    }

    /// compare to nodes according to the end_time (ties are broken by start_time and then id)
    pub(crate) fn cmp_end_time(&self, other: &Node)  -> Ordering {
        match self.end_time().partial_cmp(&other.end_time()) {
            Some(Ordering::Equal) =>
                match self.start_time().partial_cmp(&other.start_time()) {
                    Some(Ordering::Equal) => self.id().partial_cmp(&other.id()),
                    other => other
                }
            other => other
        }.unwrap()
    }

}





// static functions:
impl Node {

    // factory for creating a service trip
    pub(super) fn create_service_node(id: NodeId, origin: Location, destination: Location, departure: Time, arrival: Time, travel_distance: Distance, demand: Demand) -> Node {
        Node::Service(ServiceTrip{
            id,
            origin,
            destination,
            departure,
            arrival,
            travel_distance,
            demand}
        )
    }

    // factory for creating a node for a maintenance slot
    pub(super) fn create_maintenance_node(id: NodeId, location: Location, start: Time, end: Time) -> Node {
        Node::Maintenance(MaintenanceSlot{
            id,
            location,
            start,
            end
        })
    }


    // factory for creating start and end node of a unit
    pub(super) fn create_start_node(node_id: NodeId, unit_id: UnitId, location: Location, time: Time) -> Node {
        Node::Start(StartPoint{
            node_id,
            unit_id,
            location,
            time
        })
    }

    pub(super) fn create_end_node(id: NodeId, unit_type: UnitType, location: Location, time: Time, duration_till_maintenance: Duration, dist_till_maintenance: Distance) -> Node {
        Node::End(EndPoint{
            id,
            unit_type,
            location,
            time,
            duration_till_maintenance,
            dist_till_maintenance,
        })
    }

}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Node::Service(s) =>
                write!(f,"{} from {} ({}) to {} ({}), {}", s.id, s.origin, s.departure, s.destination, s.arrival, s.travel_distance),
            Node::Maintenance(m) =>
                write!(f,"{} at {} (from {} to {})", m.id, m.location, m.start, m.end),
            Node::Start(s) =>
                write!(f,"{} of {} at {} ({})", s.node_id, s.unit_id, s.location, s.time),
            Node::End(e) =>
                write!(f,"{} for {:?} at {} ({})", e.id, e.unit_type, e.location, e.time)
        }
    }
}

