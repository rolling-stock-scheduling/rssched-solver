mod service_trip;
use service_trip::ServiceTrip;

mod maintenance_slot;
use maintenance_slot::MaintenanceSlot;

mod vehicle_nodes;
use vehicle_nodes::{StartNode, EndNode};

use crate::time::Time;
use crate::placeholder::{Location, Distance, VehicleId};


use std::fmt;


pub(super) enum Node {
    Service(ServiceTrip),
    Maintenance(MaintenanceSlot),
    Start(StartNode),
    End(EndNode)
}






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


    // factory for creating start and end node of a vehicle
    pub(super) fn create_vehicle_nodes(vehicle_id: VehicleId, start_location: Location, start_time: Time, end_location: Location, end_time: Time) -> (Node, Node) {
        (Node::Start(StartNode::new(
            vehicle_id,
            start_location,
            start_time
        )),
        Node::End(EndNode::new(
            vehicle_id,
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

