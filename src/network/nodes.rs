use crate::time::{Time,Duration};
use crate::locations::Location;
use crate::units::{Unit,UnitType};
use crate::distance::Distance;
use crate::base_types::{NodeId,UnitId};


use std::fmt;


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
    length: Distance,
    // covered_by: TrainComposition (ordered list of unit indices)
}

pub(crate) struct MaintenanceSlot {
    id: NodeId,
    location: Location,
    start: Time,
    end: Time,
    // used_by: UnitIdx
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
    // covered_by: Option<UnitId>
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

    pub(crate) fn start_location(&self) -> Location {
        match self {
            Node::Service(s) => s.origin,
            Node::Maintenance(m) => m.location,
            Node::Start(_) => Location::Infinity,
            Node::End(e) => e.location
        }
    }

    pub(crate) fn end_location(&self) -> Location {
        match self {
            Node::Service(s) => s.destination,
            Node::Maintenance(m) => m.location,
            Node::Start(s) => s.location,
            Node::End(_) => Location::Infinity
        }
    }

    pub(crate) fn length(&self) -> Distance {
        match self {
            Node::Service(s) => s.length,
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

}



// static functions:
impl Node {

    // factory for creating a service trip
    pub(super) fn create_service_node(id: NodeId, origin: Location, destination: Location, departure: Time, arrival: Time, length: Distance) -> Node {
        Node::Service(ServiceTrip{
            id,
            origin,
            destination,
            departure,
            arrival,
            length}
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
                write!(f,"{} from {} ({}) to {} ({}), {}", s.id, s.origin, s.departure, s.destination, s.arrival, s.length),
            Node::Maintenance(m) =>
                write!(f,"{} at {} (from {} to {})", m.id, m.location, m.start, m.end),
            Node::Start(s) =>
                write!(f,"{} of {} at {} ({})", s.node_id, s.unit_id, s.location, s.time),
            Node::End(e) =>
                write!(f,"{} for {:?} at {} ({})", e.id, e.unit_type, e.location, e.time)
        }
    }
}

