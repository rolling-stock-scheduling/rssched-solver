pub(crate) mod nodes;
use nodes::Node;


use crate::time::Time;
use crate::distance::Distance;
use crate::locations::Locations;
use crate::units::Units;
use std::fmt;

use std::iter::Iterator;

pub(crate) struct Network<'a> {
    service_nodes: Vec<Node>,
    start_nodes: Vec<Node>,
    end_nodes: Vec<Node>,
    maintenance_nodes: Vec<Node>,
    locations: &'a Locations
}

// static functions
impl<'a> Network<'a> {
    pub(crate) fn initialize(locations: &'a Locations, units: &Units) -> Network<'a> {
        // TODO: replace by reading in some files
        let location = locations.get_all_locations();
        let mut service_nodes: Vec<nodes::Node> = Vec::new();
        service_nodes.push(Node::create_service_node(
                location[0],
                location[1],
                Time::new("2021-12-23T21:56"),
                Time::new("2021-12-23T22:56"),
                Distance::from_km(200.0)));

        service_nodes.push(Node::create_service_node(
                location[1],
                location[0],
                Time::new("2021-12-24T21:56"),
                Time::new("2021-12-24T22:56"),
                Distance::from_km(200.0)));

        let mut maintenance_nodes: Vec<nodes::Node> = Vec::new();
        maintenance_nodes.push(Node::create_maintenance_node(
                location[2],
                Time::new("2021-12-23T21:56"),
                Time::new("2021-12-23T23:56") ));

        let mut maintenance_nodes: Vec<nodes::Node> = Vec::new();
        maintenance_nodes.push(Node::create_maintenance_node(
                location[2],
                Time::new("2021-12-23T11:56"),
                Time::new("2021-12-23T13:56") ));

        let mut start_nodes: Vec<nodes::Node> = Vec::with_capacity(units.len());
        let mut end_nodes: Vec<nodes::Node> = Vec::with_capacity(units.len());
        for (i, unit) in units.iter().enumerate() {

            let (start, end) = Node::create_unit_nodes(
                unit,
                unit.get_start_location(),
                unit.get_start_time(),
                location[1],
                Time::new("2021-12-26 00:00"));
            start_nodes.push(start);
            end_nodes.push(end);
        }

        Network{service_nodes,maintenance_nodes,start_nodes,end_nodes,locations}
    }
}

// methods
impl<'a> Network<'a> {
    pub(crate) fn all_nodes_iter(&self) -> impl Iterator<Item=&Node> + '_ {
        self.service_nodes.iter()
            .chain(self.start_nodes.iter())
            .chain(self.end_nodes.iter())
            .chain(self.maintenance_nodes.iter())
    }

    /// return the number of nodes in the network.
    pub(crate) fn size(&self) -> usize {
        self.service_nodes.len() + self.start_nodes.len() + self.end_nodes.len() + self.maintenance_nodes.len()
    }

    pub(crate) fn terminal_nodes(&self) -> (&Vec<Node>,&Vec<Node>) {
        (&self.start_nodes, &self.end_nodes)
    }

    /// returns True iff node1 can reach node2
    pub(crate) fn can_reach(&self, node1: &Node, node2: &Node) -> bool {
        node1.end_time() + self.locations.travel_time(node1.end_location(), node2.start_location()) < node2.start_time()
    }

    pub(crate) fn all_successors(&self, node: &'a Node) -> impl Iterator<Item=&Node> + '_ {
        self.all_nodes_iter().filter(|other| self.can_reach(node, other))
    }

    pub(crate) fn all_predecessors(&self, node: &'a Node) -> impl Iterator<Item=&Node> + '_ {
        self.all_nodes_iter().filter(|other| self.can_reach(other, node))
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
