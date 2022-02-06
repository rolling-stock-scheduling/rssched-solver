pub(crate) mod nodes;
use nodes::Node;


use crate::time::Time;
use crate::distance::Distance;
use crate::locations::Locations;
use crate::vehicle::Vehicle;
use std::fmt;

use std::iter::Iterator;

pub(crate) struct Network<'a> {
    service_nodes: Vec<Node<'a>>,
    start_nodes: Vec<Node<'a>>,
    end_nodes: Vec<Node<'a>>,
    maintenance_nodes: Vec<Node<'a>>,
    locations: &'a Locations
}

// static functions
impl<'a> Network<'a> {
    pub(crate) fn initialize(locations: &'a Locations, vehicles: &'a Vec<Vehicle>) -> Network<'a> {
        // TODO: replace by reading in some files
        let station = locations.get_stations();
        let mut service_nodes: Vec<nodes::Node> = Vec::new();
        service_nodes.push(Node::create_service_node(
                &station[0],
                &station[1],
                Time::new("2021-12-23T21:56"),
                Time::new("2021-12-23T22:56"),
                Distance::from_km(200)));

        service_nodes.push(Node::create_service_node(
                &station[1],
                &station[0],
                Time::new("2021-12-24T21:56"),
                Time::new("2021-12-24T22:56"),
                Distance::from_km(200)));

        let mut maintenance_nodes: Vec<nodes::Node> = Vec::new();
        maintenance_nodes.push(Node::create_maintenance_node(
                &station[2],
                Time::new("2021-12-23T21:56"),
                Time::new("2021-12-23T23:56") ));

        let mut maintenance_nodes: Vec<nodes::Node> = Vec::new();
        maintenance_nodes.push(Node::create_maintenance_node(
                &station[2],
                Time::new("2021-12-23T11:56"),
                Time::new("2021-12-23T13:56") ));

        let mut start_nodes: Vec<nodes::Node> = Vec::with_capacity(vehicles.len());
        let mut end_nodes: Vec<nodes::Node> = Vec::with_capacity(vehicles.len());
        for (i, vehicle) in vehicles.iter().enumerate() {

            let (start, end) = Node::create_vehicle_nodes(
                vehicle,
                &station[i % vehicles.len()],
                Time::new("2021-12-10 08:00"),
                &station[1],
                Time::new("2021-12-26 00:00"));
            start_nodes.push(start);
            end_nodes.push(end);
        }

        Network{service_nodes,maintenance_nodes,start_nodes,end_nodes,locations}
    }
}

// methods
impl<'a> Network<'a> {
    pub(crate) fn all_nodes_iter(&self) -> impl Iterator<Item=&Node<'_>> + '_ {
        self.service_nodes.iter()
            .chain(self.start_nodes.iter())
            .chain(self.end_nodes.iter())
            .chain(self.maintenance_nodes.iter())
    }

    /// return the number of nodes in the network.
    pub(crate) fn size(&self) -> usize {
        self.service_nodes.len() + self.start_nodes.len() + self.end_nodes.len() + self.maintenance_nodes.len()
    }

    pub(crate) fn terminal_nodes(&self) -> (&Vec<Node<'a>>,&Vec<Node<'a>>) {
        (&self.start_nodes, &self.end_nodes)
    }

    pub(crate) fn all_successors(&self, node: &'a Node) -> impl Iterator<Item=&Node<'_>> + '_ {
        self.all_nodes_iter().filter(|other|
            node.end_time() + self.locations.travel_time(node.end_location(), other.start_location()) < other.start_time())
    }

    pub(crate) fn all_predecessors(&self, node: &'a Node) -> impl Iterator<Item=&Node<'_>> + '_ {
        self.all_nodes_iter().filter(|other|
            other.end_time() + self.locations.travel_time(other.end_location(), node.start_location()) < node.start_time())
    }
}

impl<'a> fmt::Display for Network<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"** network with {} nodes:\n", self.size())?;
        for (i,v) in self.all_nodes_iter().enumerate() {
            write!(f,"\t{}: {}\n", i, v)?;
        }
        Ok(())
    }
}
