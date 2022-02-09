pub(crate) mod service_trip;
use service_trip::ServiceTrip;

pub(crate) mod maintenance_slot;
use maintenance_slot::MaintenanceSlot;

mod terminal_nodes;
use terminal_nodes::{StartNode, EndNode};

use crate::time::{Time,Duration};
use crate::locations::Location;
use crate::units::Unit;
use crate::distance::Distance;


use std::fmt;


pub(crate) enum Node {
    Service(ServiceTrip),
    Maintenance(MaintenanceSlot),
    Start(StartNode),
    End(EndNode)
}


// methods
impl Node {
    pub(crate) fn start_time(&self) -> Time {
        match self {
            Node::Service(s) => s.departure(),
            Node::Maintenance(m) => m.start(),
            Node::Start(_) => Time::Earliest,
            Node::End(n) => n.time()
        }
    }

    pub(crate) fn end_time(&self) -> Time {
        match self {
            Node::Service(s) => s.arrival(),
            Node::Maintenance(m) => m.end(),
            Node::Start(n) => n.time(),
            Node::End(n) => Time::Latest
        }
    }

    pub(crate) fn start_location(&self) -> Location {
        match self {
            Node::Service(s) => s.origin(),
            Node::Maintenance(m) => m.location(),
            Node::Start(_) => Location::Infinity,
            Node::End(n) => n.location()
        }
    }

    pub(crate) fn end_location(&self) -> Location {
        match self {
            Node::Service(s) => s.destination(),
            Node::Maintenance(m) => m.location(),
            Node::Start(n) => n.location(),
            Node::End(_) => Location::Infinity
        }
    }

    pub(crate) fn length(&self) -> Distance {
        match self {
            Node::Service(s) => s.length(),
            _ => Distance::zero()
        }
    }

    pub(crate) fn travel_time(&self) -> Duration {
        match self {
            Node::Service(s) => s.travel_time(),
            _ => Duration::zero()
        }
    }

}



// static functions:
impl Node {

    // factory for creating a service trip
    pub(super) fn create_service_node(start_station: Location, end_station: Location, departure_time: Time, arrival_time: Time, length: Distance) -> Node {
        Node::Service(ServiceTrip::new(
            start_station,
            end_station,
            departure_time,
            arrival_time,
            length
        ))
    }

    // factory for creating a node for a maintenance slot
    pub(super) fn create_maintenance_node(location: Location, start_time: Time, end_time: Time) -> Node {
        Node::Maintenance(MaintenanceSlot::new(
            location,
            start_time,
            end_time
        ))
    }


    // factory for creating start and end node of a unit
    pub(super) fn create_unit_nodes(unit: &Unit, start_location: Location, start_time: Time, end_location: Location, end_time: Time) -> (Node, Node) {
        (Node::Start(StartNode::new(
            unit.get_id(),
            start_location,
            start_time
        )),
        Node::End(EndNode::new(
            unit.get_type(),
            end_location,
            end_time
        )))
    }

}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Node::Service(service_trip) => service_trip.fmt(f),
            Node::Maintenance(maintenance_slot) => maintenance_slot.fmt(f),
            Node::Start(start_node) => start_node.fmt(f),
            Node::End(end_node) => end_node.fmt(f)
        }
    }
}

